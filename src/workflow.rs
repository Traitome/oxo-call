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
  inputs (list of file patterns), outputs (list of file patterns), gather (bool).
- Use {wildcard} for wildcard substitution in cmd/inputs/outputs.
- Use {params.KEY} for parameter substitution in cmd.
- A step with gather = true runs ONCE after all wildcard instances of its deps complete.
- Always include fastp as the first QC step.
- Use realistic default param values in [params].

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
    println!("{}", "─".repeat(78).dimmed());
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
