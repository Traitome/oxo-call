use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::skill::SkillManager;
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
use colored::Colorize;
#[cfg(not(target_arch = "wasm32"))]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use uuid::Uuid;

/// Intermediate result from the `prepare` step that carries provenance metadata
/// alongside the LLM suggestion.
struct PrepareResult {
    suggestion: LlmCommandSuggestion,
    /// SHA-256 hex digest of the documentation text used in the prompt.
    docs_hash: String,
    /// Name of the matched skill, if one was loaded.
    skill_name: Option<String>,
}

pub struct Runner {
    config: Config,
    fetcher: DocsFetcher,
    llm: LlmClient,
    skill_manager: SkillManager,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Runner {
            fetcher: DocsFetcher::new(config.clone()),
            llm: LlmClient::new(config.clone()),
            skill_manager: SkillManager::new(config.clone()),
            config,
        }
    }

    /// Resolve documentation for the tool, showing a spinner while fetching
    async fn resolve_docs(&self, tool: &str) -> Result<String> {
        let docs = self.fetcher.fetch(tool).await?;
        Ok(docs.combined())
    }

    /// Core logic: fetch docs → load skill → call LLM → return suggestion + provenance.
    async fn prepare(&self, tool: &str, task: &str) -> Result<PrepareResult> {
        let spinner = make_spinner(&format!("Fetching documentation for '{tool}'..."));
        let docs = match self.resolve_docs(tool).await {
            Ok(d) => {
                spinner.finish_and_clear();
                d
            }
            Err(e) => {
                spinner.finish_and_clear();
                return Err(e);
            }
        };

        let docs_hash = sha256_hex(&docs);

        let skill = self.skill_manager.load(tool);
        let skill_name = skill.as_ref().map(|s| s.meta.name.clone());
        let skill_label = if skill.is_some() {
            format!(" (skill: {})", tool)
        } else {
            String::new()
        };

        let spinner = make_spinner(&format!(
            "Asking LLM to generate command arguments{skill_label}..."
        ));
        let suggestion = match self
            .llm
            .suggest_command(tool, &docs, task, skill.as_ref())
            .await
        {
            Ok(s) => {
                spinner.finish_and_clear();
                s
            }
            Err(e) => {
                spinner.finish_and_clear();
                return Err(e);
            }
        };

        Ok(PrepareResult {
            suggestion,
            docs_hash,
            skill_name,
        })
    }

    /// dry-run: show the command that would be executed without running it
    pub async fn dry_run(&self, tool: &str, task: &str) -> Result<()> {
        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Tool:".bold(), tool.cyan());
        println!("  {} {}", "Task:".bold(), task);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Command (dry-run):".bold().yellow());
        println!("  {}", full_cmd.green().bold());
        println!();
        if !result.suggestion.explanation.is_empty() {
            println!("  {}", "Explanation:".bold());
            println!("  {}", result.suggestion.explanation);
            println!();
        }
        println!("{}", "─".repeat(60).dimmed());
        println!(
            "  {}",
            "Use 'oxo-call run' to execute this command.".dimmed()
        );

        Ok(())
    }

    /// run: execute the command for real
    pub async fn run(&self, tool: &str, task: &str, ask: bool) -> Result<()> {
        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Tool:".bold(), tool.cyan());
        println!("  {} {}", "Task:".bold(), task);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Generated command:".bold().green());
        println!("  {}", full_cmd.green().bold());
        println!();
        if !result.suggestion.explanation.is_empty() {
            println!("  {}", "Explanation:".bold());
            println!("  {}", result.suggestion.explanation);
            println!();
        }

        if ask {
            print!("  {} [y/N] ", "Execute this command?".bold().yellow());
            use std::io::{self, Write};
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                println!("{}", "Aborted.".red());
                return Ok(());
            }
        }

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Running:".bold(), full_cmd.cyan());
        println!("{}", "─".repeat(60).dimmed());
        println!();

        // Process execution is not supported in WebAssembly
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::ExecutionError(
            "Command execution is not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            let status = Command::new(tool)
                .args(&result.suggestion.args)
                .status()
                .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

            let exit_code = status.code().unwrap_or(-1);
            let success = status.success();

            // Detect tool version for provenance
            let tool_version = detect_tool_version(tool);

            // Record in history with provenance
            let entry = HistoryEntry {
                id: Uuid::new_v4().to_string(),
                tool: tool.to_string(),
                task: task.to_string(),
                command: full_cmd.clone(),
                exit_code,
                executed_at: Utc::now(),
                dry_run: false,
                provenance: Some(CommandProvenance {
                    tool_version,
                    docs_hash: Some(result.docs_hash),
                    skill_name: result.skill_name,
                    model: Some(self.config.effective_model()),
                }),
            };
            let _ = HistoryStore::append(entry);

            println!();
            println!("{}", "─".repeat(60).dimmed());
            if success {
                println!(
                    "  {} exit code {}",
                    "Completed successfully,".bold().green(),
                    exit_code.to_string().green()
                );
            } else {
                println!(
                    "  {} exit code {}",
                    "Command failed,".bold().red(),
                    exit_code.to_string().red()
                );
            }
            println!("{}", "─".repeat(60).dimmed());

            Ok(())
        }
    }
}

fn build_command_string(tool: &str, args: &[String]) -> String {
    if args.is_empty() {
        return tool.to_string();
    }
    let args_str: Vec<String> = args
        .iter()
        .map(|a| {
            // Quote arguments that contain spaces or shell metacharacters
            if needs_quoting(a) {
                format!("'{}'", a.replace('\'', "'\\''"))
            } else {
                a.clone()
            }
        })
        .collect();
    format!("{tool} {}", args_str.join(" "))
}

/// Returns `true` if the argument contains characters that require quoting.
fn needs_quoting(arg: &str) -> bool {
    arg.contains(' ')
        || arg.contains('\t')
        || arg.contains(';')
        || arg.contains('&')
        || arg.contains('|')
        || arg.contains('$')
        || arg.contains('`')
        || arg.contains('(')
        || arg.contains(')')
        || arg.contains('<')
        || arg.contains('>')
        || arg.contains('!')
        || arg.contains('\\')
        || arg.contains('"')
        || arg.contains('\'')
}

/// Compute the SHA-256 hex digest of a string.
///
/// Used for the `docs_hash` field in command provenance so that identical
/// documentation inputs produce identical hashes across platforms and Rust
/// versions.
fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    result.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Try to detect the version string of an external tool by running `<tool> --version`.
#[cfg(not(target_arch = "wasm32"))]
fn detect_tool_version(tool: &str) -> Option<String> {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| {
            let text = String::from_utf8_lossy(&out.stdout);
            let line = text.lines().next().unwrap_or("").trim().to_string();
            if line.is_empty() {
                // Some tools print version info to stderr
                let stderr = String::from_utf8_lossy(&out.stderr);
                let sline = stderr.lines().next().unwrap_or("").trim().to_string();
                if sline.is_empty() { None } else { Some(sline) }
            } else {
                Some(line)
            }
        })
}

/// Create a styled progress spinner for long-running operations.
#[cfg(not(target_arch = "wasm32"))]
pub fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// No-op spinner used on WebAssembly targets where terminal progress bars are
/// unavailable.
#[cfg(target_arch = "wasm32")]
pub struct Spinner;

#[cfg(target_arch = "wasm32")]
impl Spinner {
    pub fn finish_and_clear(&self) {}
    pub fn set_message(&self, _msg: String) {}
}

/// Create a no-op spinner on WebAssembly targets.
#[cfg(target_arch = "wasm32")]
pub fn make_spinner(_msg: &str) -> Spinner {
    Spinner
}
