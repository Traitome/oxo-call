/// Workflow registry and LLM-based generator for oxo-call.
///
/// ## Architecture
/// 1. **Native engine** (`src/engine.rs`) — the primary executor.  Workflows are
///    described in `.oxo.toml` files and run directly by oxo-call with no
///    external workflow manager needed.
/// 2. **Built-in templates** — curated, production-tested `.oxo.toml` workflows
///    compiled into the binary for the most common omics assay types.
/// 3. **LLM-generated workflows** — the LLM produces native `.oxo.toml` output;
///    users can then export to Snakemake / Nextflow via `workflow export`.
/// 4. **Compatibility export** — existing Snakemake / Nextflow templates are
///    available for HPC environments that require those formats.
#[cfg(not(target_arch = "wasm32"))]
use crate::config::Config;
use crate::error::{OxoError, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

// ─── Built-in template registry ───────────────────────────────────────────────

/// A single built-in workflow template (all three formats compiled in).
#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub assay: &'static str,
    /// Native `.oxo.toml` format — the primary format used by the built-in engine.
    pub native: &'static str,
    /// Snakemake compatibility export.
    pub snakemake: &'static str,
    /// Nextflow DSL2 compatibility export.
    pub nextflow: &'static str,
}

/// All built-in templates compiled into the binary.
pub static BUILTIN_TEMPLATES: &[WorkflowTemplate] = &[
    WorkflowTemplate {
        name: "rnaseq",
        description: "Bulk RNA-seq: fastp QC → STAR alignment → featureCounts → MultiQC",
        assay: "transcriptomics",
        native: include_str!("../workflows/native/rnaseq.toml"),
        snakemake: include_str!("../workflows/snakemake/rnaseq.smk"),
        nextflow: include_str!("../workflows/nextflow/rnaseq.nf"),
    },
    WorkflowTemplate {
        name: "wgs",
        description: "Whole-genome sequencing: fastp → BWA-MEM2 → GATK BQSR → HaplotypeCaller",
        assay: "genomics",
        native: include_str!("../workflows/native/wgs.toml"),
        snakemake: include_str!("../workflows/snakemake/wgs.smk"),
        nextflow: include_str!("../workflows/nextflow/wgs.nf"),
    },
    WorkflowTemplate {
        name: "atacseq",
        description: "ATAC-seq: fastp → Bowtie2 → Picard dedup → blacklist filter → MACS3",
        assay: "epigenomics",
        native: include_str!("../workflows/native/atacseq.toml"),
        snakemake: include_str!("../workflows/snakemake/atacseq.smk"),
        nextflow: include_str!("../workflows/nextflow/atacseq.nf"),
    },
    WorkflowTemplate {
        name: "metagenomics",
        description: "Shotgun metagenomics: fastp → host removal → Kraken2 → Bracken",
        assay: "metagenomics",
        native: include_str!("../workflows/native/metagenomics.toml"),
        snakemake: include_str!("../workflows/snakemake/metagenomics.smk"),
        nextflow: include_str!("../workflows/nextflow/metagenomics.nf"),
    },
    WorkflowTemplate {
        name: "chipseq",
        description: "ChIP-seq: fastp → Bowtie2 → Picard MarkDup → blacklist filter → MACS3 → bigWig",
        assay: "epigenomics",
        native: include_str!("../workflows/native/chipseq.toml"),
        snakemake: include_str!("../workflows/snakemake/chipseq.smk"),
        nextflow: include_str!("../workflows/nextflow/chipseq.nf"),
    },
    WorkflowTemplate {
        name: "methylseq",
        description: "WGBS methylation: Trim Galore → Bismark alignment → dedup → methylation extraction",
        assay: "epigenomics",
        native: include_str!("../workflows/native/methylseq.toml"),
        snakemake: include_str!("../workflows/snakemake/methylseq.smk"),
        nextflow: include_str!("../workflows/nextflow/methylseq.nf"),
    },
    WorkflowTemplate {
        name: "scrnaseq",
        description: "scRNA-seq: fastp QC → STARsolo (10x v3) → EmptyDrops cell filtering → QC metrics",
        assay: "single-cell",
        native: include_str!("../workflows/native/scrnaseq.toml"),
        snakemake: include_str!("../workflows/snakemake/scrnaseq.smk"),
        nextflow: include_str!("../workflows/nextflow/scrnaseq.nf"),
    },
    WorkflowTemplate {
        name: "amplicon16s",
        description: "16S amplicon: cutadapt primer trim → fastp QC → DADA2 ASV denoising → SILVA taxonomy",
        assay: "metagenomics",
        native: include_str!("../workflows/native/amplicon16s.toml"),
        snakemake: include_str!("../workflows/snakemake/amplicon16s.smk"),
        nextflow: include_str!("../workflows/nextflow/amplicon16s.nf"),
    },
    WorkflowTemplate {
        name: "longreads",
        description: "Long-read assembly: NanoQ QC → Flye assembly → Medaka polishing → QUAST evaluation",
        assay: "genomics",
        native: include_str!("../workflows/native/longreads.toml"),
        snakemake: include_str!("../workflows/snakemake/longreads.smk"),
        nextflow: include_str!("../workflows/nextflow/longreads.nf"),
    },
];

/// Find a built-in template by name (case-insensitive).
pub fn find_template(name: &str) -> Option<&'static WorkflowTemplate> {
    BUILTIN_TEMPLATES
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

// ─── LLM-based generator ──────────────────────────────────────────────────────

/// A parsed LLM-generated workflow response.
#[derive(Debug, Clone)]
pub struct GeneratedWorkflow {
    /// The format that was produced ("native", "snakemake", or "nextflow").
    pub engine: String,
    pub content: String,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

fn native_system_prompt() -> &'static str {
    r#"You are an expert bioinformatics workflow engineer.
Generate workflows in the oxo-call native TOML format (.oxo.toml).

Format rules:
- [workflow] block with name, description, version fields.
- [wildcards] block: keys are wildcard names (e.g. sample), values are example lists.
- [params] block: string key-value pairs for configurable paths and settings.
- [[step]] entries: name, cmd (shell command), depends_on (list of step names),
  inputs (list of file patterns), outputs (list of file patterns), gather (bool),
  env (optional string — shell preamble for conda/venv/PATH activation).
- Use {wildcard} for wildcard substitution in cmd/inputs/outputs.
- Use {params.KEY} for parameter substitution in cmd.
- A step with gather = true runs ONCE after all wildcard instances of its deps complete.
- Always include fastp as the first QC step.
- Always include a multiqc step with gather=true placed RIGHT AFTER fastp.
  MultiQC depends only on fastp (or equivalent QC step) and scans only the QC
  output directory.  It runs in parallel with downstream analysis steps.
- Use realistic default param values in [params].
- If different steps need different Python versions or conda environments,
  use the env field (e.g. env = "conda activate py2_env &&").

Respond with EXACTLY this format (nothing before or after):
WORKFLOW:
<complete .oxo.toml content here>
END_WORKFLOW
EXPLANATION:
<one-paragraph explanation of the pipeline steps and design choices>
"#
}

fn snakemake_system_prompt() -> &'static str {
    r#"You are an expert bioinformatics workflow engineer.
Generate Snakemake workflow files (Snakefile, version ≥7.0).

Rules:
- Define a `rule all` listing all final outputs.
- Each rule must have input:, output:, threads:, log:, and shell: blocks.
- Use configfile: "config.yaml" and a config.yaml comment at the bottom.
- Use expand() for sample wildcards.
- Always include fastp as the first QC step.

Respond with EXACTLY this format (nothing before or after):
WORKFLOW:
<complete Snakefile content here>
END_WORKFLOW
EXPLANATION:
<one-paragraph explanation>
"#
}

fn nextflow_system_prompt() -> &'static str {
    r#"You are an expert bioinformatics workflow engineer.
Generate Nextflow DSL2 workflow files (.nf).

Rules:
- Use nextflow.enable.dsl = 2.
- Define each step as a process with input:, output:, and script: blocks.
- Compose processes in a workflow block using channels.
- Use params for all configurable values.
- Always include fastp as the first QC step.

Respond with EXACTLY this format (nothing before or after):
WORKFLOW:
<complete .nf file content here>
END_WORKFLOW
EXPLANATION:
<one-paragraph explanation>
"#
}

fn parse_workflow_response(raw: &str, engine: &str) -> Option<GeneratedWorkflow> {
    let workflow_start = raw.find("WORKFLOW:")?;
    let workflow_end = raw.find("END_WORKFLOW")?;
    let explanation_start = raw.find("EXPLANATION:")?;

    if workflow_end <= workflow_start || explanation_start <= workflow_end {
        return None;
    }

    let content = raw
        .get(workflow_start + "WORKFLOW:".len()..workflow_end)?
        .trim()
        .to_string();
    let explanation = raw
        .get(explanation_start + "EXPLANATION:".len()..)?
        .trim()
        .to_string();

    if content.is_empty() {
        return None;
    }

    Some(GeneratedWorkflow {
        engine: engine.to_string(),
        content,
        explanation,
    })
}

/// Call the configured LLM to generate a workflow for the given task.
///
/// The `engine` parameter controls the output format:
///   - `"native"` (default) → `.oxo.toml` for the built-in oxo engine
///   - `"snakemake"` → Snakefile
///   - `"nextflow"` → Nextflow DSL2 `.nf`
#[cfg(not(target_arch = "wasm32"))]
pub async fn generate_workflow(
    config: &Config,
    task: &str,
    engine: &str,
) -> Result<GeneratedWorkflow> {
    use reqwest::Client;

    let system = match engine {
        "snakemake" => snakemake_system_prompt().to_string(),
        "nextflow" => nextflow_system_prompt().to_string(),
        _ => native_system_prompt().to_string(),
    };

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!("Generate a complete, production-ready workflow for:\n\n{task}"),
        },
    ];

    let token = config.effective_api_token().ok_or_else(|| {
        OxoError::LlmError(
            "No API token configured. Set it with:\n  oxo-call config set llm.api_token <token>"
                .to_string(),
        )
    })?;

    let api_base = config.effective_api_base();
    if !api_base.starts_with("https://")
        && !api_base.starts_with("http://localhost")
        && !api_base.starts_with("http://127.0.0.1")
        && !api_base.starts_with("http://[::1]")
    {
        return Err(OxoError::LlmError(format!(
            "API base URL must use HTTPS for remote endpoints: {api_base}"
        )));
    }

    let model = config.effective_model();
    let max_tokens = config.effective_max_tokens()?;
    let temperature = config.effective_temperature()?;

    let request = ChatRequest {
        model: model.clone(),
        messages,
        max_tokens,
        temperature,
    };

    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| OxoError::LlmError(e.to_string()))?;

    let url = format!("{api_base}/chat/completions");

    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await
        .map_err(|e| OxoError::LlmError(format!("HTTP error: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_string());
        return Err(OxoError::LlmError(format!(
            "LLM API returned {status}: {body}"
        )));
    }

    let chat: ChatResponse = resp
        .json()
        .await
        .map_err(|e| OxoError::LlmError(format!("Failed to parse LLM response: {e}")))?;

    let raw = chat
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    if let Some(wf) = parse_workflow_response(&raw, engine) {
        return Ok(wf);
    }

    // Second attempt: re-prompt with format reminder.
    let mut messages2 = request.messages.clone();
    messages2.push(ChatMessage {
        role: "assistant".to_string(),
        content: raw.clone(),
    });
    messages2.push(ChatMessage {
        role: "user".to_string(),
        content: "Your response was not in the required format.  \
                  Please rewrite it starting with WORKFLOW: and ending with END_WORKFLOW, \
                  then EXPLANATION:"
            .to_string(),
    });

    let request2 = ChatRequest {
        model: model.clone(),
        messages: messages2,
        max_tokens,
        temperature,
    };

    let resp2 = client
        .post(&url)
        .bearer_auth(&token)
        .json(&request2)
        .send()
        .await
        .map_err(|e| OxoError::LlmError(format!("HTTP error on retry: {e}")))?;

    let raw2 = resp2
        .json::<ChatResponse>()
        .await
        .ok()
        .and_then(|c| c.choices.into_iter().next())
        .map(|c| c.message.content)
        .unwrap_or_default();

    parse_workflow_response(&raw2, engine).ok_or_else(|| {
        OxoError::LlmError("LLM did not return a parseable workflow after two attempts".to_string())
    })
}

// ─── Display helpers ──────────────────────────────────────────────────────────

/// Print the list of built-in templates.
pub fn print_template_list() {
    println!(
        "{:<16} {:<18} {}",
        "Name".bold(),
        "Assay".bold(),
        "Description".bold()
    );
    println!("{}", "─".repeat(90).dimmed());
    for t in BUILTIN_TEMPLATES {
        println!("{:<16} {:<18} {}", t.name.cyan(), t.assay, t.description);
    }
    println!();
    println!(
        "{}",
        "Use 'oxo-call workflow show <name>' to view the native template.".dimmed()
    );
    println!(
        "{}",
        "Use 'oxo-call workflow run <file.toml>' to execute a workflow.".dimmed()
    );
    println!(
        "{}",
        "Use 'oxo-call workflow generate \"<task>\"' to generate a custom workflow.".dimmed()
    );
    println!(
        "{}",
        "Use 'oxo-call workflow infer --data <dir> \"<task>\"' to auto-generate from data."
            .dimmed()
    );
}

/// Display a generated or shown workflow (to stdout or file).
pub fn print_generated_workflow(wf: &GeneratedWorkflow) {
    println!();
    println!("{}", "─".repeat(70).dimmed());
    println!("  {} {}", "Format:".bold(), wf.engine.cyan());
    println!("{}", "─".repeat(70).dimmed());
    println!();
    println!("{}", wf.content);
    println!();
    if !wf.explanation.is_empty() {
        println!("{}", "─".repeat(70).dimmed());
        println!("  {}", "Pipeline explanation:".bold());
        println!("  {}", wf.explanation);
        println!("{}", "─".repeat(70).dimmed());
    }
}

// ─── Data context discovery ────────────────────────────────────────────────

/// Summary of files discovered in a data directory for LLM prompt enrichment.
#[derive(Debug, Clone, Default)]
pub struct DataContext {
    /// Inferred sample names (without read-pair suffix and extension).
    pub samples: Vec<String>,
    /// Representative file paths observed.
    pub files: Vec<String>,
    /// Detected data type hint based on file patterns.
    pub data_type_hint: String,
}

/// Scan a directory for bioinformatics input files (FASTQ, BAM, etc.) and
/// infer sample names and data type from the file naming patterns.
///
/// Returns a [`DataContext`] that can be injected into the LLM prompt to
/// produce a workflow with real sample names and paths already filled in.
pub fn scan_data_directory(dir: &std::path::Path) -> DataContext {
    use std::collections::BTreeSet;

    let mut ctx = DataContext::default();
    let mut sample_set: BTreeSet<String> = BTreeSet::new();
    let mut file_list: Vec<String> = Vec::new();
    let mut has_fastq = false;
    let mut has_bam = false;
    let mut has_fast5 = false;
    let mut has_pod5 = false;
    let mut looks_single_end = false;

    let read_dir = match std::fs::read_dir(dir) {
        Ok(d) => d,
        Err(_) => return ctx,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let fname = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let path_str = path.to_string_lossy().to_string();

        // Collect representative files (cap at 20 for prompt brevity).
        if file_list.len() < 20 {
            file_list.push(path_str.clone());
        }

        // Detect file types.
        let lower = fname.to_lowercase();
        if lower.ends_with(".fastq.gz")
            || lower.ends_with(".fastq")
            || lower.ends_with(".fq.gz")
            || lower.ends_with(".fq")
        {
            has_fastq = true;
            // Extract sample name by stripping common read-pair suffixes:
            // _R1.fastq.gz, _R2.fastq.gz, _1.fastq.gz, _2.fastq.gz,
            // _R1_001.fastq.gz, etc.
            let stem = lower
                .trim_end_matches(".gz")
                .trim_end_matches(".fastq")
                .trim_end_matches(".fq");
            let sample = strip_read_pair_suffix(stem);
            if !sample.is_empty() {
                sample_set.insert(sample);
            }
            // Check for single-end pattern (no R1/R2).
            if !lower.contains("_r1")
                && !lower.contains("_r2")
                && !lower.contains("_1.")
                && !lower.contains("_2.")
            {
                looks_single_end = true;
            }
        } else if lower.ends_with(".bam") {
            has_bam = true;
            let stem = fname.trim_end_matches(".bam");
            let sample = strip_bam_suffix(stem);
            if !sample.is_empty() {
                sample_set.insert(sample);
            }
        } else if lower.ends_with(".fast5") || lower.ends_with(".fast5.gz") {
            has_fast5 = true;
        } else if lower.ends_with(".pod5") {
            has_pod5 = true;
        }
    }

    ctx.samples = sample_set.into_iter().collect();
    ctx.files = file_list;

    // Determine data type hint.
    ctx.data_type_hint = if has_fast5 || has_pod5 {
        "Oxford Nanopore long-read (raw signal files detected)".to_string()
    } else if has_fastq && !has_bam {
        if looks_single_end {
            "Illumina short-read FASTQ (single-end)".to_string()
        } else {
            "Illumina short-read FASTQ (paired-end)".to_string()
        }
    } else if has_bam {
        "Pre-aligned BAM files".to_string()
    } else {
        "Unknown — no recognised bioinformatics files detected".to_string()
    };

    ctx
}

fn strip_read_pair_suffix(stem: &str) -> String {
    // Remove common trailing patterns: _R1, _R2, _1, _2, _R1_001, _R2_001.
    let patterns = ["_r1_001", "_r2_001", "_r1", "_r2", "_1", "_2"];
    let lower = stem.to_lowercase();
    for pat in &patterns {
        if let Some(pos) = lower.rfind(pat) {
            let candidate = &stem[..pos];
            if !candidate.is_empty() {
                return candidate.to_string();
            }
        }
    }
    stem.to_string()
}

fn strip_bam_suffix(stem: &str) -> String {
    // Strip common BAM suffixes: .sorted, .markdup, .recal, etc.
    let patterns = [".sorted", ".markdup", ".recal", ".dedup"];
    let mut s = stem.to_string();
    for pat in &patterns {
        if let Some(pos) = s.rfind(pat) {
            s = s[..pos].to_string();
        }
    }
    s
}

/// Build the enriched user prompt for `workflow infer`, incorporating discovered
/// data context (sample names, file paths, data type).
pub fn build_infer_prompt(task: &str, ctx: &DataContext, data_dir: &str) -> String {
    let sample_list = if ctx.samples.is_empty() {
        "  (no samples auto-detected — please specify in [wildcards])".to_string()
    } else {
        ctx.samples
            .iter()
            .map(|s| format!("  - {s}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let file_preview = if ctx.files.is_empty() {
        "  (directory is empty or contains no recognised files)".to_string()
    } else {
        ctx.files
            .iter()
            .map(|f| format!("  {f}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "Generate a complete, production-ready workflow for the following task:\n\n\
         TASK:\n{task}\n\n\
         DATA DIRECTORY: {data_dir}\n\
         DATA TYPE: {dtype}\n\n\
         AUTO-DETECTED SAMPLES ({n}):\n{sample_list}\n\n\
         REPRESENTATIVE FILES (first {nf}):\n{file_preview}\n\n\
         INSTRUCTIONS:\n\
         - Use the exact data directory path '{data_dir}' as the input data location.\n\
         - Use the auto-detected sample names in the [wildcards] section.\n\
         - Substitute realistic paths for all reference files (mark with # REQUIRED comments).\n\
         - Ensure the workflow is runnable with the oxo-call native engine.\n\
         - Include all necessary steps for a complete, publication-ready analysis.",
        task = task,
        data_dir = data_dir,
        dtype = ctx.data_type_hint,
        n = ctx.samples.len(),
        sample_list = sample_list,
        nf = ctx.files.len(),
        file_preview = file_preview,
    )
}

/// Call the LLM to infer and generate a workflow from a task description and
/// a scanned data directory context.
///
/// Unlike [`generate_workflow`], this function:
/// 1. Scans the data directory to discover sample names and file patterns.
/// 2. Builds an enriched prompt that includes real paths and sample names.
/// 3. Uses the native TOML system prompt so the output is immediately runnable.
#[cfg(not(target_arch = "wasm32"))]
pub async fn infer_workflow(
    config: &Config,
    task: &str,
    data_dir: &std::path::Path,
    engine: &str,
) -> Result<GeneratedWorkflow> {
    use reqwest::Client;

    let ctx = scan_data_directory(data_dir);
    let data_dir_str = data_dir.to_string_lossy();
    let user_prompt = build_infer_prompt(task, &ctx, &data_dir_str);

    let system = match engine {
        "snakemake" => snakemake_system_prompt().to_string(),
        "nextflow" => nextflow_system_prompt().to_string(),
        _ => native_system_prompt().to_string(),
    };

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
        },
    ];

    let token = config.effective_api_token().ok_or_else(|| {
        OxoError::LlmError(
            "No API token configured. Set it with:\n  oxo-call config set llm.api_token <token>"
                .to_string(),
        )
    })?;

    let api_base = config.effective_api_base();
    if !api_base.starts_with("https://")
        && !api_base.starts_with("http://localhost")
        && !api_base.starts_with("http://127.0.0.1")
        && !api_base.starts_with("http://[::1]")
    {
        return Err(OxoError::LlmError(format!(
            "API base URL must use HTTPS for remote endpoints: {api_base}"
        )));
    }

    let model = config.effective_model();
    let max_tokens = config.effective_max_tokens()?;
    let temperature = config.effective_temperature()?;

    let request = ChatRequest {
        model: model.clone(),
        messages,
        max_tokens,
        temperature,
    };

    let client = Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| OxoError::LlmError(e.to_string()))?;

    let url = format!("{api_base}/chat/completions");

    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(&request)
        .send()
        .await
        .map_err(|e| OxoError::LlmError(format!("HTTP error: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<no body>".to_string());
        return Err(OxoError::LlmError(format!(
            "LLM API returned {status}: {body}"
        )));
    }

    let chat: ChatResponse = resp
        .json()
        .await
        .map_err(|e| OxoError::LlmError(format!("Failed to parse LLM response: {e}")))?;

    let raw = chat
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    if let Some(wf) = parse_workflow_response(&raw, engine) {
        return Ok(wf);
    }

    // Retry with format correction.
    let mut messages2 = request.messages.clone();
    messages2.push(ChatMessage {
        role: "assistant".to_string(),
        content: raw.clone(),
    });
    messages2.push(ChatMessage {
        role: "user".to_string(),
        content: "Your response was not in the required format.  \
                  Please rewrite it starting with WORKFLOW: and ending with END_WORKFLOW, \
                  then EXPLANATION:"
            .to_string(),
    });

    let request2 = ChatRequest {
        model: model.clone(),
        messages: messages2,
        max_tokens,
        temperature,
    };

    let resp2 = client
        .post(&url)
        .bearer_auth(&token)
        .json(&request2)
        .send()
        .await
        .map_err(|e| OxoError::LlmError(format!("HTTP error on retry: {e}")))?;

    let raw2 = resp2
        .json::<ChatResponse>()
        .await
        .ok()
        .and_then(|c| c.choices.into_iter().next())
        .map(|c| c.message.content)
        .unwrap_or_default();

    parse_workflow_response(&raw2, engine).ok_or_else(|| {
        OxoError::LlmError("LLM did not return a parseable workflow after two attempts".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // ─── find_template ────────────────────────────────────────────────────────

    #[test]
    fn test_find_template_rnaseq() {
        let t = find_template("rnaseq").expect("rnaseq template should exist");
        assert_eq!(t.name, "rnaseq");
        assert!(!t.native.is_empty());
        assert!(!t.snakemake.is_empty());
        assert!(!t.nextflow.is_empty());
    }

    #[test]
    fn test_find_template_case_insensitive() {
        assert!(find_template("RNAseq").is_some());
        assert!(find_template("RNASEQ").is_some());
        assert!(find_template("WGS").is_some());
    }

    #[test]
    fn test_find_template_unknown_returns_none() {
        assert!(find_template("unknown-workflow-xyz").is_none());
        assert!(find_template("").is_none());
    }

    #[test]
    fn test_all_builtin_templates_exist() {
        let expected = [
            "rnaseq",
            "wgs",
            "atacseq",
            "metagenomics",
            "chipseq",
            "methylseq",
            "scrnaseq",
            "amplicon16s",
            "longreads",
        ];
        for name in &expected {
            assert!(
                find_template(name).is_some(),
                "Expected built-in template '{name}' to exist"
            );
        }
    }

    #[test]
    fn test_template_has_description_and_assay() {
        let t = find_template("wgs").unwrap();
        assert!(!t.description.is_empty());
        assert!(!t.assay.is_empty());
    }

    // ─── parse_workflow_response ──────────────────────────────────────────────

    #[test]
    fn test_parse_workflow_response_valid() {
        let raw = "WORKFLOW:\n[workflow]\nname = \"test\"\nEND_WORKFLOW\nEXPLANATION:\nThis is a test workflow.";
        let wf = parse_workflow_response(raw, "native");
        assert!(wf.is_some());
        let wf = wf.unwrap();
        assert_eq!(wf.engine, "native");
        assert!(wf.content.contains("[workflow]"));
        assert_eq!(wf.explanation, "This is a test workflow.");
    }

    #[test]
    fn test_parse_workflow_response_missing_markers_returns_none() {
        let raw = "This has no markers at all.";
        assert!(parse_workflow_response(raw, "native").is_none());
    }

    #[test]
    fn test_parse_workflow_response_empty_content_returns_none() {
        let raw = "WORKFLOW:\nEND_WORKFLOW\nEXPLANATION:\nSome explanation.";
        assert!(parse_workflow_response(raw, "native").is_none());
    }

    // ─── scan_data_directory ──────────────────────────────────────────────────

    #[test]
    fn test_scan_data_directory_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.samples.is_empty());
        assert!(ctx.files.is_empty());
        assert!(ctx.data_type_hint.contains("Unknown") || ctx.data_type_hint.contains("no"));
    }

    #[test]
    fn test_scan_data_directory_nonexistent_returns_default() {
        let ctx = scan_data_directory(Path::new("/nonexistent/dir/xyz"));
        assert!(ctx.samples.is_empty());
        assert!(ctx.files.is_empty());
    }

    #[test]
    fn test_scan_data_directory_paired_end_fastq() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sample1_R1.fastq.gz"), b"").unwrap();
        std::fs::write(tmp.path().join("sample1_R2.fastq.gz"), b"").unwrap();
        std::fs::write(tmp.path().join("sample2_R1.fastq.gz"), b"").unwrap();
        std::fs::write(tmp.path().join("sample2_R2.fastq.gz"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.samples.contains(&"sample1".to_string()));
        assert!(ctx.samples.contains(&"sample2".to_string()));
        assert!(ctx.data_type_hint.contains("paired-end") || ctx.data_type_hint.contains("FASTQ"));
    }

    #[test]
    fn test_scan_data_directory_bam_files() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sample_A.sorted.bam"), b"").unwrap();
        std::fs::write(tmp.path().join("sample_B.markdup.bam"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.data_type_hint.contains("BAM") || ctx.data_type_hint.contains("aligned"));
    }

    #[test]
    fn test_scan_data_directory_fast5_detected() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("read1.fast5"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(
            ctx.data_type_hint.contains("Nanopore") || ctx.data_type_hint.contains("long-read")
        );
    }

    #[test]
    fn test_scan_data_directory_pod5_detected() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("read1.pod5"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(
            ctx.data_type_hint.contains("Nanopore") || ctx.data_type_hint.contains("long-read")
        );
    }

    #[test]
    fn test_scan_data_directory_single_end_fastq() {
        let tmp = tempfile::tempdir().unwrap();
        // Only files without R1/R2 pattern -> single-end hint
        std::fs::write(tmp.path().join("sample.fastq.gz"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.data_type_hint.contains("single-end") || ctx.data_type_hint.contains("FASTQ"));
    }

    // ─── build_infer_prompt ───────────────────────────────────────────────────

    #[test]
    fn test_build_infer_prompt_contains_task_and_dir() {
        let ctx = DataContext {
            samples: vec!["sample1".to_string(), "sample2".to_string()],
            files: vec!["/data/sample1_R1.fastq.gz".to_string()],
            data_type_hint: "Illumina short-read FASTQ (paired-end)".to_string(),
        };
        let prompt = build_infer_prompt("align paired-end reads to human genome", &ctx, "/data");
        assert!(prompt.contains("align paired-end reads"));
        assert!(prompt.contains("/data"));
        assert!(prompt.contains("sample1"));
        assert!(prompt.contains("sample2"));
    }

    #[test]
    fn test_build_infer_prompt_no_samples_message() {
        let ctx = DataContext {
            samples: Vec::new(),
            files: Vec::new(),
            data_type_hint: "Unknown".to_string(),
        };
        let prompt = build_infer_prompt("do something", &ctx, "/tmp/data");
        assert!(prompt.contains("no samples") || prompt.contains("auto-detected"));
    }

    #[test]
    fn test_build_infer_prompt_contains_data_type() {
        let ctx = DataContext {
            samples: vec!["s1".to_string()],
            files: vec!["/data/s1.bam".to_string()],
            data_type_hint: "Pre-aligned BAM files".to_string(),
        };
        let prompt = build_infer_prompt("variant calling", &ctx, "/data");
        assert!(prompt.contains("Pre-aligned BAM"));
    }

    // ─── strip_read_pair_suffix (indirectly via scan) ─────────────────────────

    #[test]
    fn test_strip_read_pair_suffix_r1_001_pattern() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("ctrl_R1_001.fastq.gz"), b"").unwrap();
        std::fs::write(tmp.path().join("ctrl_R2_001.fastq.gz"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.samples.contains(&"ctrl".to_string()));
    }

    // ─── strip_read_pair_suffix (directly) ───────────────────────────────────

    #[test]
    fn test_strip_read_pair_suffix_r1() {
        assert_eq!(strip_read_pair_suffix("sample_r1"), "sample");
    }

    #[test]
    fn test_strip_read_pair_suffix_r2() {
        assert_eq!(strip_read_pair_suffix("sample_r2"), "sample");
    }

    #[test]
    fn test_strip_read_pair_suffix_1() {
        assert_eq!(strip_read_pair_suffix("ctrl_1"), "ctrl");
    }

    #[test]
    fn test_strip_read_pair_suffix_2() {
        assert_eq!(strip_read_pair_suffix("ctrl_2"), "ctrl");
    }

    #[test]
    fn test_strip_read_pair_suffix_no_suffix() {
        assert_eq!(strip_read_pair_suffix("sample_only"), "sample_only");
    }

    // ─── strip_bam_suffix (via scan_data_directory) ───────────────────────────

    #[test]
    fn test_scan_data_directory_bam_sorted_suffix_stripped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sampleA.sorted.bam"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(
            ctx.samples.contains(&"sampleA".to_string()),
            "'.sorted' suffix should be stripped from BAM sample name"
        );
    }

    #[test]
    fn test_scan_data_directory_bam_markdup_suffix_stripped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("sampleB.markdup.bam"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(
            ctx.samples.contains(&"sampleB".to_string()),
            "'.markdup' suffix should be stripped"
        );
    }

    // ─── scan_data_directory: file cap ───────────────────────────────────────

    #[test]
    fn test_scan_data_directory_caps_file_list_at_20() {
        let tmp = tempfile::tempdir().unwrap();
        for i in 0..30 {
            std::fs::write(tmp.path().join(format!("sample{i}_R1.fastq.gz")), b"").unwrap();
        }
        let ctx = scan_data_directory(tmp.path());
        assert!(ctx.files.len() <= 20, "file list should be capped at 20");
    }

    // ─── build_infer_prompt: file preview ────────────────────────────────────

    #[test]
    fn test_build_infer_prompt_contains_file_preview() {
        let ctx = DataContext {
            samples: vec!["s1".to_string()],
            files: vec![
                "/data/s1_R1.fastq.gz".to_string(),
                "/data/s1_R2.fastq.gz".to_string(),
            ],
            data_type_hint: "Illumina short-read FASTQ (paired-end)".to_string(),
        };
        let prompt = build_infer_prompt("align reads", &ctx, "/data");
        assert!(prompt.contains("s1_R1.fastq.gz"));
        assert!(prompt.contains("s1_R2.fastq.gz"));
    }

    // ─── find_template ───────────────────────────────────────────────────────

    #[test]
    fn test_find_template_all_have_native_content() {
        for t in BUILTIN_TEMPLATES {
            assert!(!t.native.is_empty(), "{}: native template is empty", t.name);
            assert!(
                !t.snakemake.is_empty(),
                "{}: snakemake template is empty",
                t.name
            );
            assert!(
                !t.nextflow.is_empty(),
                "{}: nextflow template is empty",
                t.name
            );
        }
    }

    // ─── GeneratedWorkflow display helpers ────────────────────────────────────

    #[test]
    fn test_print_generated_workflow_no_explanation() {
        let wf = GeneratedWorkflow {
            engine: "native".to_string(),
            content: "[workflow]\nname = \"test\"\n".to_string(),
            explanation: String::new(),
        };
        // Should not panic even with empty explanation
        print_generated_workflow(&wf);
    }

    #[test]
    fn test_print_generated_workflow_with_explanation() {
        let wf = GeneratedWorkflow {
            engine: "snakemake".to_string(),
            content: "rule all:\n  input:\n    \"output.txt\"\n".to_string(),
            explanation: "This workflow runs STAR alignment.".to_string(),
        };
        // Should not panic
        print_generated_workflow(&wf);
    }

    // ─── parse_workflow_response edge cases ───────────────────────────────────

    #[test]
    fn test_parse_workflow_response_reordered_markers_none() {
        // END_WORKFLOW before WORKFLOW: — should return None
        let raw = "END_WORKFLOW\nWORKFLOW:\ncontent\nEXPLANATION:\nexplanation";
        assert!(parse_workflow_response(raw, "native").is_none());
    }

    #[test]
    fn test_parse_workflow_response_explanation_before_end_none() {
        // EXPLANATION: before END_WORKFLOW — should return None
        let raw = "WORKFLOW:\ncontent\nEXPLANATION:\nexplanation\nEND_WORKFLOW";
        assert!(parse_workflow_response(raw, "native").is_none());
    }

    #[test]
    fn test_parse_workflow_response_all_engines() {
        for engine in &["native", "snakemake", "nextflow"] {
            let raw = format!(
                "WORKFLOW:\n[workflow]\nname = \"test\"\nEND_WORKFLOW\nEXPLANATION:\nA test.\n"
            );
            let wf = parse_workflow_response(&raw, engine);
            assert!(wf.is_some(), "should parse for engine {engine}");
            assert_eq!(wf.unwrap().engine, *engine);
        }
    }

    // ─── DataContext default ──────────────────────────────────────────────────

    #[test]
    fn test_data_context_default() {
        let ctx = DataContext::default();
        assert!(ctx.samples.is_empty());
        assert!(ctx.files.is_empty());
        assert!(ctx.data_type_hint.is_empty());
    }

    // ─── System prompt content tests ──────────────────────────────────────────

    #[test]
    fn test_native_system_prompt_content() {
        let prompt = native_system_prompt();
        assert!(prompt.contains("oxo-call"), "should mention oxo-call");
        assert!(prompt.contains(".oxo.toml"), "should mention .oxo.toml");
        assert!(prompt.contains("[workflow]"), "should show format example");
        assert!(prompt.contains("WORKFLOW:"), "should have output format");
        assert!(prompt.contains("END_WORKFLOW"), "should have end marker");
        assert!(
            prompt.contains("EXPLANATION:"),
            "should have explanation marker"
        );
    }

    #[test]
    fn test_snakemake_system_prompt_content() {
        let prompt = snakemake_system_prompt();
        assert!(prompt.contains("Snakemake"), "should mention Snakemake");
        assert!(prompt.contains("WORKFLOW:"), "should have output format");
        assert!(prompt.contains("END_WORKFLOW"), "should have end marker");
        assert!(prompt.contains("rule all"), "should mention rule all");
    }

    #[test]
    fn test_nextflow_system_prompt_content() {
        let prompt = nextflow_system_prompt();
        assert!(prompt.contains("Nextflow"), "should mention Nextflow");
        assert!(
            prompt.contains("DSL2") || prompt.contains("dsl"),
            "should mention DSL2"
        );
        assert!(prompt.contains("WORKFLOW:"), "should have output format");
        assert!(prompt.contains("END_WORKFLOW"), "should have end marker");
    }

    // ─── print_template_list ─────────────────────────────────────────────────

    #[test]
    fn test_print_template_list_no_panic() {
        // Should not panic when called
        print_template_list();
    }

    // ─── strip_bam_suffix direct tests ───────────────────────────────────────

    #[test]
    fn test_strip_bam_suffix_recal() {
        assert_eq!(strip_bam_suffix("sampleC.recal"), "sampleC");
    }

    #[test]
    fn test_strip_bam_suffix_dedup() {
        assert_eq!(strip_bam_suffix("sampleD.dedup"), "sampleD");
    }

    #[test]
    fn test_strip_bam_suffix_multiple_suffixes() {
        // sorted.markdup should strip both
        assert_eq!(strip_bam_suffix("sample.sorted.markdup"), "sample");
    }

    #[test]
    fn test_strip_bam_suffix_no_suffix() {
        assert_eq!(strip_bam_suffix("sample"), "sample");
    }

    // ─── strip_read_pair_suffix 001 variants ─────────────────────────────────

    #[test]
    fn test_strip_read_pair_suffix_r1_001() {
        assert_eq!(strip_read_pair_suffix("ctrl_r1_001"), "ctrl");
    }

    #[test]
    fn test_strip_read_pair_suffix_r2_001() {
        assert_eq!(strip_read_pair_suffix("ctrl_r2_001"), "ctrl");
    }

    // ─── scan_data_directory: single-end FASTQ ───────────────────────────────

    #[test]
    fn test_scan_data_directory_single_end_fastq_hint() {
        let tmp = tempfile::tempdir().unwrap();
        // Single-end file: no _R1/_R2 pattern
        std::fs::write(tmp.path().join("sample_SE.fastq.gz"), b"").unwrap();

        let ctx = scan_data_directory(tmp.path());
        assert!(
            ctx.data_type_hint.contains("single-end") || ctx.data_type_hint.contains("FASTQ"),
            "should detect single-end FASTQ: {}",
            ctx.data_type_hint
        );
    }

    // ─── ChatMessage / ChatRequest / ChatResponse in workflow.rs ─────────────

    #[test]
    fn test_workflow_chat_message_clone_and_debug() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "generate rnaseq workflow".to_string(),
        };
        let cloned = msg.clone();
        assert_eq!(cloned.role, msg.role);
        assert_eq!(cloned.content, msg.content);
        let dbg = format!("{msg:?}");
        assert!(dbg.contains("user"));
    }

    #[test]
    fn test_workflow_chat_request_serialization() {
        let req = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![ChatMessage {
                role: "system".to_string(),
                content: "You are an expert.".to_string(),
            }],
            max_tokens: 4096,
            temperature: 0.2,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"gpt-4o\""));
        assert!(json.contains("\"max_tokens\":4096"));
        assert!(json.contains("You are an expert."));
    }

    #[test]
    fn test_workflow_chat_request_debug() {
        let req = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![],
            max_tokens: 100,
            temperature: 0.0,
        };
        let s = format!("{req:?}");
        assert!(s.contains("test-model"));
    }

    #[test]
    fn test_workflow_chat_response_deserialization() {
        let json = r#"{"choices": [{"message": {"role": "assistant", "content": "WORKFLOW:\n[workflow]\nEND_WORKFLOW\nEXPLANATION:\ntest"}}]}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices.len(), 1);
        assert!(resp.choices[0].message.content.contains("WORKFLOW:"));
    }

    #[test]
    fn test_workflow_chat_response_empty_choices() {
        let json = r#"{"choices": []}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(resp.choices.is_empty());
    }

    // ─── Mock HTTP tests for generate_workflow / infer_workflow ───────────────

    #[cfg(not(target_arch = "wasm32"))]
    mod mock_tests {
        use super::*;
        use crate::config::Config;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        fn completion_body(content: &str) -> serde_json::Value {
            serde_json::json!({
                "choices": [{"message": {"role": "assistant", "content": content}}]
            })
        }

        fn mock_cfg(base_url: &str) -> Config {
            let mut cfg = Config::default();
            cfg.llm.api_token = Some("test-token".to_string());
            cfg.llm.api_base = Some(base_url.to_string());
            cfg.llm.model = Some("gpt-4o-mini".to_string());
            cfg
        }

        fn valid_workflow_response(engine: &str) -> String {
            let content = if engine == "snakemake" {
                "rule all:\n  input: expand(\"qc/{sample}.html\")\n"
            } else if engine == "nextflow" {
                "nextflow.enable.dsl = 2\nworkflow {\n}\n"
            } else {
                "[workflow]\nname = \"rnaseq\"\n[[step]]\nname = \"fastp\"\ncmd = \"fastp\"\n"
            };
            format!("WORKFLOW:\n{content}\nEND_WORKFLOW\nEXPLANATION:\nA complete workflow.")
        }

        // ── generate_workflow ─────────────────────────────────────────────────

        #[tokio::test]
        async fn test_generate_workflow_native_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("native"))),
                )
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "native").await;

            assert!(result.is_ok(), "should succeed: {:?}", result.err());
            let wf = result.unwrap();
            assert_eq!(wf.engine, "native");
            assert!(!wf.content.is_empty());
            assert!(!wf.explanation.is_empty());
        }

        #[tokio::test]
        async fn test_generate_workflow_snakemake_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("snakemake"))),
                )
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "snakemake").await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap().engine, "snakemake");
        }

        #[tokio::test]
        async fn test_generate_workflow_nextflow_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("nextflow"))),
                )
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "nextflow").await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap().engine, "nextflow");
        }

        #[tokio::test]
        async fn test_generate_workflow_retries_on_bad_format() {
            let server = MockServer::start().await;
            // First call returns invalid format
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("This is not valid format at all")),
                )
                .up_to_n_times(1)
                .mount(&server)
                .await;
            // Second call returns valid format
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("native"))),
                )
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "native").await;

            assert!(
                result.is_ok(),
                "should succeed after retry: {:?}",
                result.err()
            );
        }

        #[tokio::test]
        async fn test_generate_workflow_both_attempts_fail_returns_error() {
            let server = MockServer::start().await;
            // Both calls return invalid format
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("Invalid format - no WORKFLOW: markers")),
                )
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "native").await;

            assert!(
                result.is_err(),
                "should fail when both attempts return bad format"
            );
        }

        #[tokio::test]
        async fn test_generate_workflow_http_error() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
                .mount(&server)
                .await;

            let result =
                generate_workflow(&mock_cfg(&server.uri()), "rnaseq pipeline", "native").await;

            assert!(result.is_err());
        }

        // ── infer_workflow ────────────────────────────────────────────────────

        #[tokio::test]
        async fn test_infer_workflow_native_success() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("native"))),
                )
                .mount(&server)
                .await;

            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(tmp.path().join("sample1_R1.fastq.gz"), b"").unwrap();
            std::fs::write(tmp.path().join("sample1_R2.fastq.gz"), b"").unwrap();

            let result = infer_workflow(
                &mock_cfg(&server.uri()),
                "rnaseq analysis",
                tmp.path(),
                "native",
            )
            .await;

            assert!(
                result.is_ok(),
                "infer_workflow should succeed: {:?}",
                result.err()
            );
            let wf = result.unwrap();
            assert_eq!(wf.engine, "native");
        }

        #[tokio::test]
        async fn test_infer_workflow_retries_on_bad_format() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body("Not valid workflow format")),
                )
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("native"))),
                )
                .mount(&server)
                .await;

            let tmp = tempfile::tempdir().unwrap();
            let result = infer_workflow(
                &mock_cfg(&server.uri()),
                "rnaseq analysis",
                tmp.path(),
                "native",
            )
            .await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_infer_workflow_empty_dir() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/chat/completions"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(completion_body(&valid_workflow_response("native"))),
                )
                .mount(&server)
                .await;

            let tmp = tempfile::tempdir().unwrap();
            // Empty directory — scan_data_directory returns empty context
            let result = infer_workflow(
                &mock_cfg(&server.uri()),
                "rnaseq analysis",
                tmp.path(),
                "native",
            )
            .await;

            assert!(result.is_ok());
        }
    }
}
