/// Workflow generation for oxo-call.
///
/// The `workflow` subcommand bridges individual `oxo-call run` invocations into
/// complete, runnable bioinformatics pipelines expressed as Snakemake or Nextflow
/// workflow files.
///
/// ## Architecture
/// 1. **Built-in templates** — curated, production-tested Snakemake/Nextflow
///    workflows for the most common omics assay types (RNA-seq, WGS, ATAC-seq,
///    metagenomics, …).  Users can list and inspect these directly.
/// 2. **LLM-generated workflows** — for tasks without a matching built-in
///    template the LLM is prompted with the task description and a rich system
///    prompt that includes workflow-engine syntax rules and common bioinformatics
///    conventions.  The structured response is a complete, ready-to-run file.
use crate::config::Config;
use crate::error::{OxoError, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

// ─── Built-in template registry ───────────────────────────────────────────────

/// A single built-in workflow template.
#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub name: &'static str,
    pub description: &'static str,
    pub assay: &'static str,
    pub snakemake: &'static str,
    pub nextflow: &'static str,
}

/// All built-in templates compiled into the binary.
pub static BUILTIN_TEMPLATES: &[WorkflowTemplate] = &[
    WorkflowTemplate {
        name: "rnaseq",
        description: "Bulk RNA-seq: QC → trimming → STAR alignment → featureCounts quantification",
        assay: "transcriptomics",
        snakemake: include_str!("../workflows/snakemake/rnaseq.smk"),
        nextflow: include_str!("../workflows/nextflow/rnaseq.nf"),
    },
    WorkflowTemplate {
        name: "wgs",
        description: "Whole-genome sequencing: QC → BWA-MEM2 alignment → GATK HaplotypeCaller variant calling",
        assay: "genomics",
        snakemake: include_str!("../workflows/snakemake/wgs.smk"),
        nextflow: include_str!("../workflows/nextflow/wgs.nf"),
    },
    WorkflowTemplate {
        name: "atacseq",
        description: "ATAC-seq: QC → Bowtie2 alignment → peak calling with MACS3 → annotation",
        assay: "epigenomics",
        snakemake: include_str!("../workflows/snakemake/atacseq.smk"),
        nextflow: include_str!("../workflows/nextflow/atacseq.nf"),
    },
    WorkflowTemplate {
        name: "metagenomics",
        description: "Shotgun metagenomics: QC → host removal → Kraken2 classification → Bracken abundance estimation",
        assay: "metagenomics",
        snakemake: include_str!("../workflows/snakemake/metagenomics.smk"),
        nextflow: include_str!("../workflows/nextflow/metagenomics.nf"),
    },
];

/// Find a built-in template by name (case-insensitive).
pub fn find_template(name: &str) -> Option<&'static WorkflowTemplate> {
    BUILTIN_TEMPLATES
        .iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
}

// ─── LLM-based workflow generator ────────────────────────────────────────────

/// A parsed LLM-generated workflow response.
#[derive(Debug, Clone)]
pub struct GeneratedWorkflow {
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

fn workflow_system_prompt(engine: &str) -> String {
    let engine_rules = match engine {
        "nextflow" => {
            "You generate Nextflow (DSL2) workflow files (.nf).  Rules:\n\
             - Use DSL2 syntax: `nextflow.enable.dsl=2`\n\
             - Define each step as a `process` with `input:`, `output:`, and `script:` blocks.\n\
             - Compose processes in a `workflow` block using channels.\n\
             - Use `params` for all configurable values (threads, paths, references).\n\
             - Output actual file paths in `publishDir`.\n\
             - Include a `nextflow.config` snippet at the bottom as a comment block."
        }
        _ => {
            // Default: snakemake
            "You generate Snakemake workflow files (Snakefile).  Rules:\n\
             - Use Snakemake ≥7.0 syntax.\n\
             - Define a `rule all` that lists all final output files.\n\
             - Each rule must have `input:`, `output:`, `threads:`, `shell:` or `run:` blocks.\n\
             - Use `config` dict and a `config.yaml` comment block at the bottom for all parameters.\n\
             - Use `expand()` for sample wildcards.\n\
             - Include `log:` directives for every rule.\n\
             - Prefer `shell:` with actual tool commands over `run:` blocks."
        }
    };

    format!(
        "You are an expert bioinformatics workflow engineer.\n\
         {engine_rules}\n\n\
         General rules for all workflows:\n\
         - Always include QC (FastQC/fastp) as the first step.\n\
         - Handle both paired-end and single-end reads unless the task specifies otherwise.\n\
         - Use production best practices: read groups for GATK, sorted BAMs, etc.\n\
         - Include reasonable default thread counts (e.g., 8 threads for alignment steps).\n\
         - Never hallucinate tool flags not present in common documentation.\n\
         - The workflow must be complete and directly runnable with minimal user edits.\n\n\
         Respond with EXACTLY this format (nothing before or after):\n\
         WORKFLOW:\n\
         <complete workflow file content here>\n\
         END_WORKFLOW\n\
         EXPLANATION:\n\
         <one-paragraph explanation of the pipeline steps and design choices>\n"
    )
}

fn parse_workflow_response(raw: &str, engine: &str) -> Option<GeneratedWorkflow> {
    let workflow_start = raw.find("WORKFLOW:")?;
    let workflow_end = raw.find("END_WORKFLOW")?;
    let explanation_start = raw.find("EXPLANATION:")?;

    if workflow_end <= workflow_start || explanation_start <= workflow_end {
        return None;
    }

    let content = raw[workflow_start + "WORKFLOW:".len()..workflow_end]
        .trim()
        .to_string();
    let explanation = raw[explanation_start + "EXPLANATION:".len()..]
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

/// Calls the configured LLM to generate a workflow for the given task.
#[cfg(not(target_arch = "wasm32"))]
pub async fn generate_workflow(
    config: &Config,
    task: &str,
    engine: &str,
) -> Result<GeneratedWorkflow> {
    use reqwest::Client;

    let system = workflow_system_prompt(engine);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: format!(
                "Generate a complete, production-ready {engine} workflow for the following task:\n\n{task}"
            ),
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

    // Retry once with a stricter reminder if parsing fails
    if let Some(wf) = parse_workflow_response(&raw, engine) {
        return Ok(wf);
    }

    // Second attempt: re-prompt with format reminder
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
    println!("{}", "─".repeat(78).dimmed());
    for t in BUILTIN_TEMPLATES {
        println!("{:<16} {:<18} {}", t.name.cyan(), t.assay, t.description);
    }
    println!();
    println!(
        "{}",
        "Use 'oxo-call workflow show <name>' to inspect a template.".dimmed()
    );
    println!(
        "{}",
        "Use 'oxo-call workflow generate \"<task>\"' to generate a custom workflow.".dimmed()
    );
}

/// Display a generated workflow result (to stdout).
pub fn print_generated_workflow(wf: &GeneratedWorkflow) {
    println!();
    println!("{}", "─".repeat(70).dimmed());
    println!("  {} {}", "Engine:".bold(), wf.engine.cyan());
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
