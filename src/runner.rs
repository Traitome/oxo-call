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

/// The LLM-generated command that will be executed (or sent over SSH).
pub struct GeneratedCommand {
    /// The full shell command string, ready to execute.
    pub full_cmd: String,
    /// Human-readable explanation from the LLM.
    pub explanation: String,
    /// The task description actually used (may differ from user input when
    /// `--optimize-task` is active).
    pub effective_task: String,
}

/// Intermediate result from the `prepare` step that carries provenance metadata
/// alongside the LLM suggestion.
struct PrepareResult {
    suggestion: LlmCommandSuggestion,
    /// SHA-256 hex digest of the documentation text used in the prompt.
    docs_hash: String,
    /// Name of the matched skill, if one was loaded.
    skill_name: Option<String>,
    /// The task description that was actually used (may differ from the user-supplied
    /// task when `--optimize-task` is enabled).
    effective_task: String,
}

pub struct Runner {
    config: Config,
    fetcher: DocsFetcher,
    llm: LlmClient,
    skill_manager: SkillManager,
    verbose: bool,
    no_cache: bool,
    /// When true, use LLM to verify the result after execution.
    verify: bool,
    /// When true, use LLM to optimize/expand the user's task description before
    /// building the command generation prompt.
    optimize_task: bool,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Runner {
            fetcher: DocsFetcher::new(config.clone()),
            llm: LlmClient::new(config.clone()),
            skill_manager: SkillManager::new(config.clone()),
            config,
            verbose: false,
            no_cache: false,
            verify: false,
            optimize_task: false,
        }
    }

    /// Enable verbose output for this runner.
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Skip cached documentation and fetch fresh --help output.
    pub fn with_no_cache(mut self, no_cache: bool) -> Self {
        self.no_cache = no_cache;
        self
    }

    /// Enable LLM-based result verification after execution.
    pub fn with_verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Enable LLM-based task description optimization before generating the command.
    pub fn with_optimize_task(mut self, optimize_task: bool) -> Self {
        self.optimize_task = optimize_task;
        self
    }

    /// Resolve documentation for the tool, showing a spinner while fetching
    async fn resolve_docs(&self, tool: &str) -> Result<String> {
        let docs = if self.no_cache {
            self.fetcher.fetch_no_cache(tool).await?
        } else {
            self.fetcher.fetch(tool).await?
        };
        Ok(docs.combined())
    }

    /// Core logic: fetch docs → (optionally optimize task) → load skill → call LLM → return suggestion + provenance.
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

        if self.verbose {
            eprintln!(
                "{} Documentation: {} chars{}",
                "[verbose]".dimmed(),
                docs.len(),
                if self.no_cache {
                    " (fresh, cache skipped)"
                } else {
                    ""
                }
            );
        }

        let docs_hash = sha256_hex(&docs);

        // Optionally optimize the task description with LLM before command generation.
        let effective_task = if self.optimize_task {
            let spinner = make_spinner("Optimizing task description...");
            let refined = match self.llm.optimize_task(tool, task).await {
                Ok(t) => {
                    spinner.finish_and_clear();
                    t
                }
                Err(e) => {
                    spinner.finish_and_clear();
                    if self.verbose {
                        eprintln!(
                            "{} Task optimization failed ({}), using original task",
                            "[verbose]".dimmed(),
                            e
                        );
                    }
                    task.to_string()
                }
            };
            if self.verbose && refined != task {
                eprintln!(
                    "{} Task optimized: {}",
                    "[verbose]".dimmed(),
                    refined.dimmed()
                );
            }
            refined
        } else {
            task.to_string()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let skill = self.skill_manager.load_async(tool).await;
        #[cfg(target_arch = "wasm32")]
        let skill = self.skill_manager.load(tool);
        let skill_name = skill.as_ref().map(|s| s.meta.name.clone());
        let skill_label = if skill.is_some() {
            format!(" (skill: {})", tool)
        } else {
            String::new()
        };

        if self.verbose {
            if let Some(ref s) = skill {
                eprintln!(
                    "{} Skill loaded: {} ({} concepts, {} pitfalls, {} examples)",
                    "[verbose]".dimmed(),
                    s.meta.name,
                    s.context.concepts.len(),
                    s.context.pitfalls.len(),
                    s.examples.len()
                );
            } else {
                eprintln!("{} No skill found for '{}'", "[verbose]".dimmed(), tool);
            }
            eprintln!(
                "{} LLM: provider={}, model={}, max_tokens={}, temperature={}",
                "[verbose]".dimmed(),
                self.config.effective_provider(),
                self.config.effective_model(),
                self.config.effective_max_tokens().unwrap_or(2048),
                self.config.effective_temperature().unwrap_or(0.0)
            );
        }

        let spinner = make_spinner(&format!(
            "Asking LLM to generate command arguments{skill_label}..."
        ));
        let suggestion = match self
            .llm
            .suggest_command(tool, &docs, &effective_task, skill.as_ref())
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
            effective_task,
        })
    }

    /// Generate the LLM-suggested command without printing or executing it.
    ///
    /// Used by the `server run` handler to obtain the command string that will
    /// be sent over SSH, while keeping display logic in the caller.
    pub async fn generate_command(&self, tool: &str, task: &str) -> Result<GeneratedCommand> {
        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);
        Ok(GeneratedCommand {
            full_cmd,
            explanation: result.suggestion.explanation.clone(),
            effective_task: result.effective_task.clone(),
        })
    }

    /// dry-run: show the command that would be executed without running it
    pub async fn dry_run(&self, tool: &str, task: &str, json: bool) -> Result<()> {
        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        if json {
            let output = serde_json::json!({
                "tool": tool,
                "task": task,
                "effective_task": result.effective_task,
                "command": full_cmd,
                "args": result.suggestion.args,
                "explanation": result.suggestion.explanation,
                "dry_run": true,
                "skill": result.skill_name,
                "model": self.config.effective_model(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            return Ok(());
        }

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Tool:".bold(), tool.cyan());
        println!("  {} {}", "Task:".bold(), task);
        if result.effective_task != task {
            println!(
                "  {} {}",
                "Optimized task:".bold().dimmed(),
                result.effective_task.dimmed()
            );
        }
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
    pub async fn run(&self, tool: &str, task: &str, ask: bool, json: bool) -> Result<()> {
        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        if !json {
            println!();
            println!("{}", "─".repeat(60).dimmed());
            println!("  {} {}", "Tool:".bold(), tool.cyan());
            println!("  {} {}", "Task:".bold(), task);
            if result.effective_task != task {
                println!(
                    "  {} {}",
                    "Optimized task:".bold().dimmed(),
                    result.effective_task.dimmed()
                );
            }
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

        if !json {
            println!();
            println!("{}", "─".repeat(60).dimmed());
            println!("  {} {}", "Running:".bold(), full_cmd.cyan());
            println!("{}", "─".repeat(60).dimmed());
            println!();
        }

        // Process execution is not supported in WebAssembly
        #[cfg(target_arch = "wasm32")]
        return Err(OxoError::ExecutionError(
            "Command execution is not supported in WebAssembly".to_string(),
        ));

        #[cfg(not(target_arch = "wasm32"))]
        {
            // When verification is enabled, capture stderr for analysis.
            // stdout is still streamed to the terminal via inheritance.
            let (exit_code, success, captured_stderr) = if self.verify {
                let output = Command::new(tool)
                    .args(&result.suggestion.args)
                    .output()
                    .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

                // Stream captured output to terminal so the user still sees it.
                use std::io::Write;
                let _ = std::io::stdout().write_all(&output.stdout);
                let _ = std::io::stderr().write_all(&output.stderr);

                let code = output.status.code().unwrap_or(-1);
                let ok = output.status.success();
                let stderr_text = String::from_utf8_lossy(&output.stderr).into_owned();
                (code, ok, stderr_text)
            } else {
                let status = Command::new(tool)
                    .args(&result.suggestion.args)
                    .status()
                    .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;
                let code = status.code().unwrap_or(-1);
                let ok = status.success();
                (code, ok, String::new())
            };

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
                    skill_name: result.skill_name.clone(),
                    model: Some(self.config.effective_model()),
                }),
            };
            let _ = HistoryStore::append(entry);

            if json {
                let output = serde_json::json!({
                    "tool": tool,
                    "task": task,
                    "effective_task": result.effective_task,
                    "command": full_cmd,
                    "args": result.suggestion.args,
                    "explanation": result.suggestion.explanation,
                    "dry_run": false,
                    "exit_code": exit_code,
                    "success": success,
                    "skill": result.skill_name,
                    "model": self.config.effective_model(),
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
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
            }

            // LLM-based result verification (when --verify is enabled).
            if self.verify {
                self.run_verification(VerifyParams {
                    tool,
                    task: &result.effective_task,
                    command: &full_cmd,
                    exit_code,
                    stderr: &captured_stderr,
                    args: &result.suggestion.args,
                    json,
                })
                .await;
            }

            Ok(())
        }
    }
}

/// Parameters for LLM-based run result verification.
#[cfg(not(target_arch = "wasm32"))]
struct VerifyParams<'a> {
    tool: &'a str,
    task: &'a str,
    command: &'a str,
    exit_code: i32,
    stderr: &'a str,
    args: &'a [String],
    json: bool,
}

impl Runner {
    /// Perform LLM verification of a completed command run and print/return results.
    #[cfg(not(target_arch = "wasm32"))]
    async fn run_verification(&self, params: VerifyParams<'_>) {
        let VerifyParams {
            tool,
            task,
            command,
            exit_code,
            stderr,
            args,
            json,
        } = params;
        // Collect expected output files from the args list.
        let output_files = detect_output_files(args);
        // Probe each file for existence and size (consume `output_files` directly).
        let file_info: Vec<(String, Option<u64>)> = output_files
            .into_iter()
            .map(|p| {
                let size = std::fs::metadata(&p).ok().map(|m| m.len());
                (p, size)
            })
            .collect();

        let spinner = make_spinner("Verifying result with LLM...");
        let verification = match self
            .llm
            .verify_run_result(tool, task, command, exit_code, stderr, &file_info)
            .await
        {
            Ok(v) => {
                spinner.finish_and_clear();
                v
            }
            Err(e) => {
                spinner.finish_and_clear();
                eprintln!(
                    "{} LLM verification failed: {}",
                    "warning:".yellow().bold(),
                    e
                );
                return;
            }
        };

        if json {
            // JSON mode: just print the verification block.
            let v = serde_json::json!({
                "verification": {
                    "success": verification.success,
                    "summary": verification.summary,
                    "issues": verification.issues,
                    "suggestions": verification.suggestions,
                }
            });
            println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            return;
        }

        println!();
        println!("{}", "─".repeat(60).dimmed());
        let label = if verification.success {
            "LLM Verification: OK".bold().green().to_string()
        } else {
            "LLM Verification: Issues detected".bold().red().to_string()
        };
        println!("  {}", label);
        if !verification.summary.is_empty() {
            println!("  {}", verification.summary);
        }
        if !verification.issues.is_empty() {
            println!();
            println!("  {}", "Issues:".bold().yellow());
            for issue in &verification.issues {
                println!("    {} {}", "•".yellow(), issue);
            }
        }
        if !verification.suggestions.is_empty() {
            println!();
            println!("  {}", "Suggestions:".bold().cyan());
            for sug in &verification.suggestions {
                println!("    {} {}", "→".cyan(), sug);
            }
        }
        println!("{}", "─".repeat(60).dimmed());
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

/// Scan command args to identify likely output file paths.
///
/// Heuristics applied (in order):
/// 1. The value after common output flags (`-o`, `--output`, `-O`, `--out`, `-b`).
/// 2. Any positional argument (not starting with `-`) that looks like a file path
///    — i.e. contains a dot and no shell metacharacters.
///
/// The returned list is deduplicated and never exceeds 20 entries to keep the
/// verification prompt size bounded.
#[cfg(not(target_arch = "wasm32"))]
fn detect_output_files(args: &[String]) -> Vec<String> {
    const OUTPUT_FLAGS: &[&str] = &["-o", "--output", "-O", "--out", "-b", "--bam"];
    let mut files: Vec<String> = Vec::new();
    let mut take_next = false;

    for arg in args {
        if take_next {
            files.push(arg.clone());
            take_next = false;
            continue;
        }
        // Check for --flag=value form
        let mut matched_flag = false;
        for &flag in OUTPUT_FLAGS {
            if arg.starts_with(&format!("{flag}=")) {
                let value = &arg[flag.len() + 1..];
                if !value.is_empty() {
                    files.push(value.to_string());
                }
                matched_flag = true;
                break;
            }
            if arg == flag {
                take_next = true;
                matched_flag = true;
                break;
            }
        }
        if matched_flag {
            continue;
        }
        // Heuristic: positional arg that looks like a file path
        if !arg.starts_with('-')
            && arg.contains('.')
            && !arg.contains(';')
            && !arg.contains('&')
            && !arg.contains('|')
        {
            files.push(arg.clone());
        }
    }

    // Deduplicate, preserving order.
    let mut seen = std::collections::HashSet::new();
    files.retain(|f| seen.insert(f.clone()));
    files.truncate(20);
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_output_files_short_flag() {
        let args: Vec<String> = vec![
            "-o".to_string(),
            "out.bam".to_string(),
            "input.bam".to_string(),
        ];
        let files = detect_output_files(&args);
        assert!(files.contains(&"out.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_long_flag() {
        let args: Vec<String> = vec!["--output".to_string(), "result.vcf".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"result.vcf".to_string()));
    }

    #[test]
    fn test_detect_output_files_equals_form() {
        let args: Vec<String> = vec!["--output=sorted.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"sorted.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_positional() {
        let args: Vec<String> = vec![
            "-t".to_string(),
            "8".to_string(),
            "input.fastq.gz".to_string(),
            "output.fastq.gz".to_string(),
        ];
        let files = detect_output_files(&args);
        // Positional file-like args are included
        assert!(files.contains(&"input.fastq.gz".to_string()));
        assert!(files.contains(&"output.fastq.gz".to_string()));
    }

    #[test]
    fn test_detect_output_files_deduplicates() {
        let args: Vec<String> = vec![
            "-o".to_string(),
            "out.bam".to_string(),
            "out.bam".to_string(),
        ];
        let files = detect_output_files(&args);
        assert_eq!(files.iter().filter(|f| *f == "out.bam").count(), 1);
    }

    #[test]
    fn test_detect_output_files_skips_flags() {
        let args: Vec<String> = vec![
            "--threads".to_string(),
            "8".to_string(),
            "--sort".to_string(),
        ];
        let files = detect_output_files(&args);
        // Pure flags and numeric values without dots should not be included
        assert!(!files.contains(&"--threads".to_string()));
        assert!(!files.contains(&"--sort".to_string()));
    }

    // ─── build_command_string ─────────────────────────────────────────────────

    #[test]
    fn test_build_command_string_no_args() {
        assert_eq!(build_command_string("echo", &[]), "echo");
    }

    #[test]
    fn test_build_command_string_simple_args() {
        let args: Vec<String> = vec!["-o".to_string(), "out.bam".to_string()];
        let cmd = build_command_string("samtools", &args);
        assert_eq!(cmd, "samtools -o out.bam");
    }

    #[test]
    fn test_build_command_string_quotes_args_with_spaces() {
        let args: Vec<String> = vec!["--output".to_string(), "my output file.bam".to_string()];
        let cmd = build_command_string("samtools", &args);
        assert!(
            cmd.contains("'my output file.bam'"),
            "args with spaces should be quoted"
        );
    }

    #[test]
    fn test_build_command_string_quotes_args_with_special_chars() {
        let args: Vec<String> = vec!["--filter".to_string(), "flag & 0x4".to_string()];
        let cmd = build_command_string("samtools", &args);
        assert!(cmd.contains("'flag"), "args with & should be quoted");
    }

    // ─── needs_quoting ────────────────────────────────────────────────────────

    #[test]
    fn test_needs_quoting_simple_arg_false() {
        assert!(!needs_quoting("-o"));
        assert!(!needs_quoting("out.bam"));
        assert!(!needs_quoting("--threads=8"));
    }

    #[test]
    fn test_needs_quoting_space_true() {
        assert!(needs_quoting("my file.bam"));
    }

    #[test]
    fn test_needs_quoting_special_chars_true() {
        assert!(needs_quoting("a;b"));
        assert!(needs_quoting("a&b"));
        assert!(needs_quoting("a|b"));
        assert!(needs_quoting("$HOME"));
        assert!(needs_quoting("`cmd`"));
        assert!(needs_quoting("(subshell)"));
        assert!(needs_quoting("a<b"));
        assert!(needs_quoting("a>b"));
        assert!(needs_quoting("a!b"));
        assert!(needs_quoting("a\\b"));
        assert!(needs_quoting("a\"b"));
        assert!(needs_quoting("a'b"));
    }

    #[test]
    fn test_needs_quoting_tab_true() {
        assert!(needs_quoting("a\tb"));
    }

    // ─── sha256_hex ───────────────────────────────────────────────────────────

    #[test]
    fn test_sha256_hex_empty_string() {
        let hash = sha256_hex("");
        // SHA256 of empty string is well-known
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_hello_world() {
        let hash = sha256_hex("hello world");
        assert_eq!(hash.len(), 64, "SHA256 hex should be 64 characters");
        // Only hex characters
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_sha256_hex_deterministic() {
        let hash1 = sha256_hex("test input");
        let hash2 = sha256_hex("test input");
        assert_eq!(hash1, hash2, "SHA256 should be deterministic");
    }

    #[test]
    fn test_sha256_hex_different_inputs_produce_different_hashes() {
        let hash1 = sha256_hex("input one");
        let hash2 = sha256_hex("input two");
        assert_ne!(hash1, hash2);
    }

    // ─── Runner::new and builder methods ─────────────────────────────────────

    #[test]
    fn test_runner_new() {
        use crate::config::Config;
        let cfg = Config::default();
        let runner = Runner::new(cfg);
        // Just verify construction doesn't panic
        let runner = runner.with_verbose(true);
        let runner = runner.with_no_cache(true);
        let runner = runner.with_verify(true);
        let _runner = runner.with_optimize_task(true);
    }
}
