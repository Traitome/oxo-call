use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
use crate::job;
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::skill::SkillManager;
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;
use colored::Colorize;
#[cfg(not(target_arch = "wasm32"))]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
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
    /// Named variables substituted into the task description before the LLM call.
    #[cfg(not(target_arch = "wasm32"))]
    vars: HashMap<String, String>,
    /// Input items for batch/parallel execution (empty = single run).
    #[cfg(not(target_arch = "wasm32"))]
    input_items: Vec<String>,
    /// Maximum number of parallel jobs when `input_items` is non-empty.
    #[cfg(not(target_arch = "wasm32"))]
    jobs: usize,
    /// When true, stop the batch after the first failed item.
    #[cfg(not(target_arch = "wasm32"))]
    stop_on_error: bool,
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
            #[cfg(not(target_arch = "wasm32"))]
            vars: HashMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            input_items: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            jobs: 1,
            #[cfg(not(target_arch = "wasm32"))]
            stop_on_error: false,
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

    /// Set named variables that will be substituted into the task description
    /// (and, when an input list is present, into the generated command) before
    /// the LLM call.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.vars = vars;
        self
    }

    /// Set input items for batch / parallel execution.
    ///
    /// When non-empty, the LLM is called once and the generated command
    /// template (which may contain `{item}`) is executed for every item.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_input_items(mut self, items: Vec<String>) -> Self {
        self.input_items = items;
        self
    }

    /// Set the maximum number of parallel jobs (default: 1 = sequential).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs.max(1);
        self
    }

    /// When enabled, abort the batch after the first failed item.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_stop_on_error(mut self, stop_on_error: bool) -> Self {
        self.stop_on_error = stop_on_error;
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

    /// dry-run: show the command that would be executed without running it.
    /// Records the generated command in history with `dry_run = true`.
    /// Pass `server` to tag the history entry with the remote server name.
    pub async fn dry_run(
        &self,
        tool: &str,
        task: &str,
        json: bool,
        server: Option<&str>,
    ) -> Result<()> {
        // ── Native-only: apply vars + batch dispatch ──────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        let _task_buf;
        #[cfg(not(target_arch = "wasm32"))]
        let task: &str = if self.vars.is_empty() {
            task
        } else {
            _task_buf = job::interpolate_command(task, "", 0, &self.vars);
            &_task_buf
        };
        #[cfg(not(target_arch = "wasm32"))]
        if !self.input_items.is_empty() {
            return self.dry_run_batch(tool, task, json).await;
        }
        // ─────────────────────────────────────────────────────────────────────

        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        // Record in history before displaying, so the entry is always saved.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tool_version = detect_tool_version(tool);
            let entry = HistoryEntry {
                id: Uuid::new_v4().to_string(),
                tool: tool.to_string(),
                task: task.to_string(),
                command: full_cmd.clone(),
                exit_code: 0,
                executed_at: Utc::now(),
                dry_run: true,
                server: server.map(str::to_string),
                provenance: Some(CommandProvenance {
                    tool_version,
                    docs_hash: Some(result.docs_hash.clone()),
                    skill_name: result.skill_name.clone(),
                    model: Some(self.config.effective_model()),
                }),
            };
            let _ = HistoryStore::append(entry);
        }

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
        // ── Native-only: apply vars + batch dispatch ──────────────────────────
        #[cfg(not(target_arch = "wasm32"))]
        let _task_buf;
        #[cfg(not(target_arch = "wasm32"))]
        let task: &str = if self.vars.is_empty() {
            task
        } else {
            _task_buf = job::interpolate_command(task, "", 0, &self.vars);
            &_task_buf
        };
        #[cfg(not(target_arch = "wasm32"))]
        if !self.input_items.is_empty() {
            return self.run_batch(tool, task, json).await;
        }
        // ─────────────────────────────────────────────────────────────────────

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
                server: None,
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

impl Runner {
    /// Execute the LLM-generated command template for every input item.
    ///
    /// The command template is obtained with a single LLM call; `{item}` (and
    /// other placeholders) are then substituted for each item before execution.
    /// Up to `self.jobs` items are run concurrently.
    ///
    /// When `self.stop_on_error` is true, remaining handles are aborted after
    /// the first failure, and the batch exits immediately with an error.
    #[cfg(not(target_arch = "wasm32"))]
    async fn run_batch(&self, tool: &str, task: &str, json: bool) -> Result<()> {
        use std::sync::Arc;

        let result = self.prepare(tool, task).await?;
        let cmd_template = build_command_string(tool, &result.suggestion.args);
        // Clone provenance fields before result is consumed.
        let docs_hash = result.docs_hash.clone();
        let skill_name = result.skill_name.clone();

        let items = &self.input_items;
        let n = items.len();
        let jobs = self.jobs.max(1);

        if !json {
            println!();
            println!("{}", "─".repeat(60).dimmed());
            println!("  {} {}", "Tool:".bold(), tool.cyan());
            println!("  {} {}", "Task template:".bold(), task);
            println!("{}", "─".repeat(60).dimmed());
            println!();
            println!("  {}", "Command template:".bold().green());
            println!("  {}", cmd_template.green().bold());
            println!();
            if !result.suggestion.explanation.is_empty() {
                println!("  {}", "Explanation:".bold());
                println!("  {}", result.suggestion.explanation);
                println!();
            }
            println!(
                "  {} {} items, {} parallel{}",
                "Batch:".bold(),
                n.to_string().cyan(),
                jobs.to_string().cyan(),
                if self.stop_on_error {
                    " (stop-on-error)".yellow().to_string()
                } else {
                    String::new()
                },
            );
            println!("{}", "─".repeat(60).dimmed());
            println!();
        }

        // Start timing before spawning so wall-time includes queue wait.
        let batch_started = Utc::now();
        let sem = Arc::new(tokio::sync::Semaphore::new(jobs));
        let mut handles: Vec<(String, tokio::task::JoinHandle<Result<i32>>)> =
            Vec::with_capacity(n);

        for (i, item) in items.iter().enumerate() {
            let cmd = job::interpolate_command(&cmd_template, item, i + 1, &self.vars);
            let sem_clone = Arc::clone(&sem);
            let item_label = item.clone();
            let handle: tokio::task::JoinHandle<Result<i32>> = tokio::spawn(async move {
                let _permit = sem_clone
                    .acquire_owned()
                    .await
                    .expect("semaphore closed unexpectedly");
                tokio::task::spawn_blocking(move || {
                    std::process::Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .status()
                        .map(|s| s.code().unwrap_or(-1))
                        .map_err(|e| {
                            OxoError::ExecutionError(format!("failed to run '{item_label}': {e}"))
                        })
                })
                .await
                .map_err(|e| OxoError::ExecutionError(format!("task join error: {e}")))?
            });
            handles.push((item.clone(), handle));
        }

        let mut failed = 0usize;
        let mut done = 0usize;
        let mut results: Vec<(String, i32)> = Vec::with_capacity(n);
        for (item, handle) in handles {
            let code = match handle.await {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    failed += 1;
                    if !json {
                        eprintln!("  {} {}: {}", "✗".red().bold(), item, e);
                    }
                    // Use -1 as sentinel for "spawn/IO error" so we can distinguish
                    // it from a real process exit code of -1 in the output section.
                    -1
                }
                Err(e) => {
                    failed += 1;
                    if !json {
                        eprintln!("  {} {}: join error: {}", "✗".red().bold(), item, e);
                    }
                    -1
                }
            };
            // Count non-zero exit codes as failures (applies in both json and text mode).
            // Sentinel value -1 already incremented failed above; skip to avoid double-count.
            if code != 0 && code != -1 {
                failed += 1;
            }
            done += 1;
            if !json {
                match code {
                    0 => println!("  {} [{}/{}] {}", "✓".green().bold(), done, n, item),
                    -1 => {} // error already printed in match arm above
                    c => eprintln!(
                        "  {} [{}/{}] {} (exit {})",
                        "✗".red().bold(),
                        done,
                        n,
                        item,
                        c.to_string().red()
                    ),
                }
            }
            // Move item into results (no clone needed — item is no longer used after this).
            results.push((item, code));

            // Stop-on-error: abort remaining handles after the first failure.
            if self.stop_on_error && failed > 0 {
                if !json {
                    eprintln!(
                        "  {} stop-on-error: aborting after first failure ({}/{} done)",
                        "⚡".yellow().bold(),
                        done,
                        n
                    );
                }
                break;
            }
        }

        // Record a single batch summary in command history.
        {
            let tool_version = detect_tool_version(tool);
            let history_entry = HistoryEntry {
                id: Uuid::new_v4().to_string(),
                tool: tool.to_string(),
                task: task.to_string(),
                command: format!("{cmd_template}  # batch:{n} vars:{}", self.vars.len()),
                exit_code: if failed == 0 { 0 } else { 1 },
                executed_at: batch_started,
                dry_run: false,
                server: None,
                provenance: Some(CommandProvenance {
                    tool_version,
                    docs_hash: Some(docs_hash),
                    skill_name,
                    model: Some(self.config.effective_model()),
                }),
            };
            let _ = HistoryStore::append(history_entry);
        }

        if json {
            let entries: Vec<serde_json::Value> = results
                .iter()
                .enumerate()
                .map(|(i, (item, code))| {
                    let cmd = job::interpolate_command(&cmd_template, item, i + 1, &self.vars);
                    serde_json::json!({
                        "item": item,
                        "command": cmd,
                        "exit_code": code,
                        "success": *code == 0,
                    })
                })
                .collect();
            let output = serde_json::json!({
                "tool": tool,
                "task_template": task,
                "command_template": cmd_template,
                "total": n,
                "processed": done,
                "failed": failed,
                "success": failed == 0,
                "stopped_early": self.stop_on_error && done < n,
                "results": entries,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!();
            println!("{}", "─".repeat(60).dimmed());
            if failed == 0 {
                println!(
                    "  {} All {} items completed successfully.",
                    "✓".green().bold(),
                    n.to_string().green()
                );
            } else {
                eprintln!(
                    "  {} {}/{} items failed.",
                    "✗".red().bold(),
                    failed.to_string().red(),
                    done
                );
            }
            println!("{}", "─".repeat(60).dimmed());
        }

        if failed > 0 {
            return Err(OxoError::ExecutionError(format!(
                "{failed}/{done} batch items failed"
            )));
        }
        Ok(())
    }

    /// Show the interpolated command for every input item without executing.
    #[cfg(not(target_arch = "wasm32"))]
    async fn dry_run_batch(&self, tool: &str, task: &str, json: bool) -> Result<()> {
        let result = self.prepare(tool, task).await?;
        let cmd_template = build_command_string(tool, &result.suggestion.args);

        let items = &self.input_items;
        let n = items.len();

        let commands: Vec<String> = items
            .iter()
            .enumerate()
            .map(|(i, item)| job::interpolate_command(&cmd_template, item, i + 1, &self.vars))
            .collect();

        if json {
            let output = serde_json::json!({
                "tool": tool,
                "task_template": task,
                "command_template": cmd_template,
                "commands": commands,
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
        println!("  {} {}", "Task template:".bold(), task);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Command template (dry-run):".bold().yellow());
        println!("  {}", cmd_template.green().bold());
        println!();
        if !result.suggestion.explanation.is_empty() {
            println!("  {}", "Explanation:".bold());
            println!("  {}", result.suggestion.explanation);
            println!();
        }
        println!(
            "  {} {} items would be processed:",
            "Batch:".bold(),
            n.to_string().cyan()
        );
        println!("{}", "─".repeat(60).dimmed());
        for (i, cmd) in commands.iter().enumerate() {
            println!("  [{:>3}] {}", i + 1, cmd.as_str().green());
        }
        println!("{}", "─".repeat(60).dimmed());
        println!(
            "  {}",
            "Use 'oxo-call run' to execute these commands.".dimmed()
        );

        Ok(())
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

    // ─── detect_tool_version ─────────────────────────────────────────────────

    #[test]
    fn test_detect_tool_version_existing_tool() {
        // 'ls' always exists on Linux — result may be Some or None
        // depending on whether it prints version info; just verify no panic.
        let result = detect_tool_version("ls");
        // Can be Some("ls (GNU coreutils) 8.32") or None (macOS) — both OK.
        let _ = result;
    }

    #[test]
    fn test_detect_tool_version_nonexistent_tool_returns_none() {
        let result = detect_tool_version("__nonexistent_binary_oxo_call_test__");
        assert!(result.is_none(), "nonexistent tool should return None");
    }

    #[test]
    fn test_detect_tool_version_echo_command() {
        // `echo --version` on Linux prints to stdout (GNU coreutils) or does nothing.
        // Either Some or None is valid — just verify no panic.
        let _result = detect_tool_version("echo");
    }

    // ─── make_spinner ─────────────────────────────────────────────────────────

    #[test]
    fn test_make_spinner_creates_without_panic() {
        let pb = make_spinner("Test message");
        // Verify the spinner can be finished without panicking.
        pb.finish_and_clear();
    }

    #[test]
    fn test_make_spinner_with_empty_message() {
        let pb = make_spinner("");
        pb.finish_and_clear();
    }

    // ─── detect_output_files extra edge cases ────────────────────────────────

    #[test]
    fn test_detect_output_files_bam_flag() {
        let args: Vec<String> = vec!["--bam".to_string(), "output.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(
            files.contains(&"output.bam".to_string()),
            "--bam flag should capture next arg"
        );
    }

    #[test]
    fn test_detect_output_files_short_b_flag() {
        let args: Vec<String> = vec!["-b".to_string(), "output.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(
            files.contains(&"output.bam".to_string()),
            "-b flag should capture next arg"
        );
    }

    #[test]
    fn test_detect_output_files_equals_form_bam() {
        let args: Vec<String> = vec!["-b=output.bam".to_string()];
        let files = detect_output_files(&args);
        assert!(files.contains(&"output.bam".to_string()));
    }

    #[test]
    fn test_detect_output_files_equals_form_empty_value_ignored() {
        // --output= with nothing after = should not add empty string
        let args: Vec<String> = vec!["--output=".to_string()];
        let files = detect_output_files(&args);
        assert!(
            !files.contains(&String::new()),
            "empty value after = should not be collected"
        );
    }

    #[test]
    fn test_detect_output_files_positional_with_semicolon_excluded() {
        // Args containing shell metacharacters should not be collected as files
        let args: Vec<String> = vec!["input;rm -rf /".to_string()];
        let files = detect_output_files(&args);
        assert!(files.is_empty(), "args with ; should be excluded");
    }

    #[test]
    fn test_detect_output_files_positional_with_pipe_excluded() {
        let args: Vec<String> = vec!["input|cat".to_string()];
        let files = detect_output_files(&args);
        assert!(files.is_empty(), "args with | should be excluded");
    }

    #[test]
    fn test_detect_output_files_positional_with_ampersand_excluded() {
        let args: Vec<String> = vec!["input&output".to_string()];
        let files = detect_output_files(&args);
        assert!(files.is_empty(), "args with & should be excluded");
    }

    #[test]
    fn test_detect_output_files_truncates_at_20() {
        // Create 25 unique output flags with different file names
        let mut args: Vec<String> = Vec::new();
        for i in 0..25 {
            args.push(format!("positional_{i}.bam"));
        }
        let files = detect_output_files(&args);
        assert!(
            files.len() <= 20,
            "detect_output_files should cap at 20 entries"
        );
    }

    #[test]
    fn test_detect_output_files_no_dot_excluded() {
        // Positional args without a dot are NOT file-like; should not be collected
        let args: Vec<String> = vec!["nodot".to_string(), "anotherword".to_string()];
        let files = detect_output_files(&args);
        assert!(
            !files.contains(&"nodot".to_string()),
            "arg without dot should not be collected"
        );
    }

    // ─── build_command_string: single-quote escaping ─────────────────────────

    #[test]
    fn test_build_command_string_escapes_single_quotes_in_args() {
        // An arg containing a single quote must be escaped as '\''
        let args: Vec<String> = vec!["it's".to_string()];
        let cmd = build_command_string("echo", &args);
        assert!(
            cmd.contains("'\\'"),
            "single quote should be escaped as '\\'"
        );
    }
}
