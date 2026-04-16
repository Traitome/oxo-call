use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
#[cfg(not(target_arch = "wasm32"))]
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
#[cfg(not(target_arch = "wasm32"))]
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
    /// [Ablation] When true, do not load the skill file for the tool.
    no_skill: bool,
    /// [Ablation] When true, do not load tool documentation (--help output).
    no_doc: bool,
    /// [Ablation] When true, do not use the oxo-call system prompt.
    no_prompt: bool,
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
            no_skill: false,
            no_doc: false,
            no_prompt: false,
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

    /// [Ablation] Do not load the skill file for the tool.
    pub fn with_no_skill(mut self, no_skill: bool) -> Self {
        self.no_skill = no_skill;
        self
    }

    /// [Ablation] Do not load tool documentation (--help output).
    pub fn with_no_doc(mut self, no_doc: bool) -> Self {
        self.no_doc = no_doc;
        self
    }

    /// [Ablation] Do not use the oxo-call system prompt.
    pub fn with_no_prompt(mut self, no_prompt: bool) -> Self {
        self.no_prompt = no_prompt;
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

    /// Resolve documentation for the tool, showing a spinner while fetching.
    /// Also attempts to fetch help for the specific subcommand matching the user's task.
    async fn resolve_docs(&self, tool: &str, task: &str) -> Result<String> {
        let mut docs = if self.no_cache {
            self.fetcher.fetch_no_cache(tool).await?
        } else {
            self.fetcher.fetch(tool).await?
        };

        // Attempt subcommand-directed fetching: if the tool has subcommands and
        // the task mentions one, fetch that subcommand's detailed help.
        if docs.subcommand_help.is_none()
            && let Some(help_output) = &docs.help_output
        {
            docs.subcommand_help = self.fetcher.fetch_subcommand_help(tool, help_output, task);
        }

        Ok(docs.combined())
    }

    /// Core logic: fetch docs + load skill (in parallel) → (optionally optimize task) → call LLM.
    ///
    /// Documentation fetching and skill loading are independent operations that
    /// can run concurrently via `tokio::join!`, reducing latency by ~200-500ms
    /// compared to the previous serial approach.
    async fn prepare(&self, tool: &str, task: &str) -> Result<PrepareResult> {
        // ── Parallel fetch: docs + skill ──────────────────────────────────────
        //
        // Both are independent I/O operations: docs requires a subprocess call
        // (tool --help) while skill loading checks built-in → user → community →
        // MCP sources.  Running them concurrently saves the latency of whichever
        // finishes second.
        let spinner = if !self.no_doc {
            make_spinner(&format!("Fetching documentation for '{tool}'..."))
        } else {
            make_spinner("Loading skill...")
        };

        let docs_future = async {
            if self.no_doc {
                if self.verbose {
                    eprintln!(
                        "{} [Ablation] Skipping documentation (--no-doc)",
                        "[verbose]".dimmed()
                    );
                }
                Ok(String::new())
            } else {
                self.resolve_docs(tool, task).await
            }
        };

        let skill_future = async {
            if self.no_skill {
                if self.verbose {
                    eprintln!(
                        "{} [Ablation] Skipping skill (--no-skill)",
                        "[verbose]".dimmed()
                    );
                }
                None
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.skill_manager.load_async(tool).await
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.skill_manager.load(tool)
                }
            }
        };

        // Run both concurrently — skill loading never fails (returns None on miss).
        let (docs_result, skill) = tokio::join!(docs_future, skill_future);
        spinner.finish_and_clear();

        let docs = docs_result?;

        if self.verbose && !self.no_doc {
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
            } else if !self.no_skill {
                eprintln!("{} No skill found for '{}'", "[verbose]".dimmed(), tool);
            }
            let ctx_window = self.config.effective_context_window();
            let tier = self.config.effective_prompt_tier();
            let model_name = self.config.effective_model();
            let profile = crate::config::get_model_profile(&model_name);
            eprintln!(
                "{} LLM: provider={}, model={}, max_tokens={}, temperature={}, context_window={}, prompt_tier={:?}",
                "[verbose]".dimmed(),
                self.config.effective_provider(),
                model_name,
                self.config.effective_max_tokens().unwrap_or(2048),
                self.config.effective_temperature().unwrap_or(0.0),
                ctx_window,
                tier
            );
            eprintln!(
                "{} Model profile: instruction={:.1}, code={:.1}, bio={:.1}, style={:?}",
                "[verbose]".dimmed(),
                profile.instruction_following,
                profile.code_generation,
                profile.bio_knowledge,
                profile.preferred_prompt_style
            );
        }

        // ── Experiment context inference ──────────────────────────────────────
        // Enrich the LLM prompt with inferred context (assay type, organism, etc.)
        let context = crate::context::ExperimentContext::infer(&effective_task, &[]);
        let context_hint = context.to_prompt_hint();

        // ── User preference learning ─────────────────────────────────────────
        let preferences_hint = {
            let history = crate::history::HistoryStore::load_all().unwrap_or_default();
            let prefs = crate::history::learn_user_preferences(tool, &history);
            prefs.to_prompt_hint()
        };

        // Build enriched task with context and preference hints
        let enriched_task = if !context_hint.is_empty() || !preferences_hint.is_empty() {
            let mut parts = vec![effective_task.clone()];
            if !context_hint.is_empty() {
                parts.push(context_hint);
            }
            if !preferences_hint.is_empty() {
                parts.push(preferences_hint);
            }
            parts.join("\n")
        } else {
            effective_task.clone()
        };

        if self.verbose && enriched_task != effective_task {
            eprintln!(
                "{} Enriched task with context/preferences",
                "[verbose]".dimmed()
            );
        }

        let spinner = make_spinner(&format!(
            "Asking LLM to generate command arguments{skill_label}..."
        ));
        let suggestion = match self
            .llm
            .suggest_command(tool, &docs, &enriched_task, skill.as_ref(), self.no_prompt)
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
    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
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
                "inference_ms": result.suggestion.inference_ms,
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

        // ── Risk assessment ───────────────────────────────────────────────────
        let risk = assess_command_risk(&result.suggestion.args);

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

            // Display risk warnings
            if let Some(warning) = risk_warning_message(risk) {
                match risk {
                    RiskLevel::Dangerous => eprintln!("  {}", warning.red().bold()),
                    RiskLevel::Warning => eprintln!("  {}", warning.yellow()),
                    RiskLevel::Safe => {}
                }
                println!();
            }

            // Display format compatibility warnings
            let format_warnings =
                crate::format::validate_format_compatibility(&result.suggestion.args);
            for fw in &format_warnings {
                let msg = format!("  ⚠ Format: {}", fw.message);
                match fw.severity {
                    crate::format::WarningSeverity::Warning => eprintln!("{}", msg.yellow()),
                    crate::format::WarningSeverity::Info => eprintln!("{}", msg.dimmed()),
                }
            }
            if !format_warnings.is_empty() {
                println!();
            }

            println!("  {}", "Generated command:".bold().green());
            println!("  {}", full_cmd.green().bold());
            println!();
            if !result.suggestion.explanation.is_empty() {
                println!("  {}", "Explanation:".bold());
                println!("  {}", result.suggestion.explanation);
                println!();
            }

            // Validate input files exist before execution
            #[cfg(not(target_arch = "wasm32"))]
            {
                let missing = validate_input_files(&result.suggestion.args);
                if !missing.is_empty() {
                    eprintln!(
                        "  {} Input file(s) not found: {}",
                        "warning:".yellow().bold(),
                        missing.join(", ").yellow()
                    );
                    println!();
                }
            }
        }

        // Force --ask mode for dangerous commands even if not explicitly requested
        let effective_ask = ask || risk == RiskLevel::Dangerous;

        if effective_ask {
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
            // Resolve companion binary (e.g., "bowtie2-build" when tool is "bowtie2")
            let (eff_tool, eff_args) = effective_command(tool, &result.suggestion.args);

            // When the args contain shell operators (&&, ||, ;, |, >, …) the command
            // must be dispatched through a shell so those operators are interpreted as
            // shell syntax rather than being passed as literal strings to the tool.
            // `full_cmd` already has shell operators unquoted and all other args
            // properly single-quoted, so it is safe to pass directly to `sh -c`.
            let use_shell = args_require_shell(&result.suggestion.args);

            // When verification is enabled, capture stderr for analysis.
            // stdout is still streamed to the terminal via inheritance.
            let (exit_code, success, captured_stderr) = if self.verify {
                let output = if use_shell {
                    Command::new("sh")
                        .args(["-c", &full_cmd])
                        .output()
                        .map_err(|e| OxoError::ExecutionError(format!("sh: {e}")))?
                } else {
                    Command::new(eff_tool)
                        .args(eff_args)
                        .output()
                        .map_err(|e| OxoError::ToolNotFound(format!("{eff_tool}: {e}")))?
                };

                // Stream captured output to terminal so the user still sees it.
                use std::io::Write;
                let _ = std::io::stdout().write_all(&output.stdout);
                let _ = std::io::stderr().write_all(&output.stderr);

                let code = output.status.code().unwrap_or(-1);
                let ok = output.status.success();
                let stderr_text = String::from_utf8_lossy(&output.stderr).into_owned();
                (code, ok, stderr_text)
            } else {
                let status = if use_shell {
                    Command::new("sh")
                        .args(["-c", &full_cmd])
                        .status()
                        .map_err(|e| OxoError::ExecutionError(format!("sh: {e}")))?
                } else {
                    Command::new(eff_tool)
                        .args(eff_args)
                        .status()
                        .map_err(|e| OxoError::ToolNotFound(format!("{eff_tool}: {e}")))?
                };
                let code = status.code().unwrap_or(-1);
                let ok = status.success();
                (code, ok, String::new())
            };

            // Detect tool version for provenance (use effective tool binary)
            let tool_version = detect_tool_version(eff_tool);

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
    let (eff_tool, eff_args) = effective_command(tool, args);
    if eff_args.is_empty() {
        return eff_tool.to_string();
    }
    let args_str: Vec<String> = eff_args
        .iter()
        .map(|a| {
            // Shell operators (&&, ||, ;, |, >, …) are shell syntax, not values —
            // they must never be quoted, otherwise the shell would treat them as
            // literal string arguments to the tool.
            if is_shell_operator(a) {
                a.clone()
            } else if needs_quoting(a) {
                // Quote arguments that contain spaces or shell metacharacters
                format!("'{}'", a.replace('\'', "'\\''"))
            } else {
                a.clone()
            }
        })
        .collect();
    format!("{eff_tool} {}", args_str.join(" "))
}

/// Resolve the effective (executable, args) pair.
///
/// When the LLM generates a companion binary as the first argument
/// (e.g., `bowtie2-build` when the tool is `bowtie2`), the companion binary
/// is extracted and used as the actual executable with the remaining slice as
/// its arguments.  This lets a single skill cover a tool and its related
/// binaries without requiring a separate skill file for each.
///
/// Detection rule: `args[0]` is treated as a companion binary when all of:
/// 1. It does **not** start with `-` (flags start with a dash).
/// 2. It starts with `{tool}-` or `{tool}_` (it is derived from the tool name).
/// 3. It contains only `[A-Za-z0-9_-]` characters (looks like a binary name,
///    not a file path or an argument value).
///
/// Additionally, if `args[0]` is a script-style executable (ends with `.sh`,
/// `.py`, `.pl`, `.R`, `.rb`, or `.jl`), it is used as the command directly.
/// This handles tool packages where the actual executables are standalone
/// scripts with different names (e.g., BBtools → `bbduk.sh`, RSeQC →
/// `infer_experiment.py`, Strelka2 → `configureStrelkaGermlineWorkflow.py`).
pub(crate) fn effective_command<'a>(tool: &'a str, args: &'a [String]) -> (&'a str, &'a [String]) {
    if let Some(first) = args.first() {
        if is_companion_binary(tool, first) {
            return (first.as_str(), &args[1..]);
        }
        // Standalone script executables: if the first arg ends with a known
        // script extension and the stem looks like a binary name (no slashes,
        // no whitespace), treat it as the actual command.
        if is_script_executable(first) {
            return (first.as_str(), &args[1..]);
        }
    }
    (tool, args)
}

/// Script extensions recognised as standalone executables.
const SCRIPT_EXTENSIONS: &[&str] = &[".sh", ".py", ".pl", ".R", ".rb", ".jl"];

/// Returns `true` if `candidate` looks like a standalone script executable.
///
/// The candidate must:
/// 1. End with a known script extension (`.sh`, `.py`, `.pl`, `.R`, `.rb`, `.jl`).
/// 2. Not contain path separators (`/`, `\`) — to avoid treating file paths as commands.
/// 3. Have a stem (before extension) that contains only `[A-Za-z0-9_-]` characters.
///
/// Examples:
/// - `is_script_executable("bbduk.sh")` → `true`
/// - `is_script_executable("infer_experiment.py")` → `true`
/// - `is_script_executable("script.py")` → `true`
/// - `is_script_executable("input.fastq.gz")` → `false` (not a script extension)
/// - `is_script_executable("/path/to/script.py")` → `false` (contains path separator)
pub(crate) fn is_script_executable(candidate: &str) -> bool {
    // Must not contain path separators.
    if candidate.contains('/') || candidate.contains('\\') {
        return false;
    }
    // Must not start with a dash (flag).
    if candidate.starts_with('-') {
        return false;
    }
    // Check for a known script extension.
    for ext in SCRIPT_EXTENSIONS {
        if let Some(stem) = candidate.strip_suffix(ext) {
            // Stem must be non-empty and look like a binary name.
            return !stem.is_empty()
                && stem
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        }
    }
    false
}

/// Returns `true` if `candidate` looks like a companion binary of `tool`.
///
/// A companion binary either:
/// 1. Starts with `{tool}-` or `{tool}_` (forward prefix convention):
///    e.g., `bowtie2-build` (tool: `bowtie2`), `hisat2-build` (tool: `hisat2`),
///    `bismark_genome_preparation` (tool: `bismark`).
/// 2. Ends with `_{tool}` (reverse suffix convention):
///    e.g., `deduplicate_bismark` (tool: `bismark`), `rsem-calculate-expression`
///    already covered by prefix.
///
/// The candidate may have a recognized script extension (`.sh`, `.py`, `.pl`,
/// `.R`, `.rb`, `.jl`) which is stripped before the prefix/suffix check. This
/// handles tools like BBtools (`bbduk.sh`), Manta (`configureManta.py`),
/// HOMER (`annotatePeaks.pl`), and Arriba (`draw_fusions.R`).
///
/// In both cases the *stem* (without extension) must contain only
/// `[A-Za-z0-9_-]` characters (looks like a binary name, not a file path)
/// and must not start with `-` (which would indicate a CLI flag instead).
///
/// Examples:
/// - `is_companion_binary("bowtie2", "bowtie2-build")` → `true`  (prefix)
/// - `is_companion_binary("hisat2", "hisat2-build")` → `true`   (prefix)
/// - `is_companion_binary("bismark", "bismark_genome_preparation")` → `true` (prefix)
/// - `is_companion_binary("bismark", "bismark_methylation_extractor")` → `true`
/// - `is_companion_binary("bismark", "deduplicate_bismark")` → `true` (suffix)
/// - `is_companion_binary("bbtools", "bbduk.sh")` → `true` (script companion)
/// - `is_companion_binary("manta", "configureManta.py")` → `true` (script companion)
/// - `is_companion_binary("bowtie2", "-x")` → `false`  (flag)
/// - `is_companion_binary("bowtie2", "bowtie2-input.fq")` → `false` (data file)
/// - `is_companion_binary("samtools", "sort")` → `false`  (no tool prefix/suffix)
pub(crate) fn is_companion_binary(tool: &str, candidate: &str) -> bool {
    if candidate.starts_with('-') {
        return false; // CLI flag, not a binary
    }
    // Recognised script extensions that companion binaries may carry.
    const SCRIPT_EXTS: &[&str] = &[".sh", ".py", ".pl", ".R", ".rb", ".jl"];

    // Strip a trailing script extension (if any) to obtain the binary stem.
    let stem = SCRIPT_EXTS
        .iter()
        .find_map(|ext| candidate.strip_suffix(ext))
        .unwrap_or(candidate);

    // The stem must look like a binary name: only alphanumeric, hyphen, underscore chars.
    if !stem
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return false;
    }

    // For script companions, any file-extension companion that contains the
    // tool name anywhere in its stem (case-insensitive) is accepted.
    // This covers "configureManta.py" → stem "configureManta" contains "manta" (ci),
    // "bbduk.sh" → stem "bbduk" starts with "bb" but tool is "bbtools" — handled below,
    // "annotatePeaks.pl" → HOMER, etc.
    if candidate != stem {
        let stem_lower = stem.to_ascii_lowercase();
        let tool_lower = tool.to_ascii_lowercase();
        if stem_lower.contains(&tool_lower) {
            return true;
        }
    }

    // Forward prefix: {tool}- or {tool}_
    let hyphen_prefix = format!("{tool}-");
    let underscore_prefix = format!("{tool}_");
    if stem.starts_with(&hyphen_prefix) || stem.starts_with(&underscore_prefix) {
        return true;
    }
    // Reverse suffix: _{tool} (covers deduplicate_bismark → bismark, etc.)
    // Require len > suffix len so that _{tool} alone (with no prefix) is not matched —
    // a candidate exactly equal to "_{tool}" would be a degenerate binary name.
    let underscore_suffix = format!("_{tool}");
    stem.ends_with(&underscore_suffix) && stem.len() > underscore_suffix.len()
}

/// Returns `true` if `arg` is a standalone shell control operator.
///
/// These tokens are shell syntax, not argument values, and must **never** be
/// quoted in the display string produced by `build_command_string`.  They also
/// signal that the full command string must be dispatched via `sh -c` rather
/// than being passed directly to `execve`.
///
/// Note: a bare `&` (background operator) is intentionally excluded because it
/// can legitimately appear as part of samtools/awk filter expressions when the
/// LLM tokenizes `flag & 0x4` as three separate tokens.  Only unambiguous
/// multi-character operators and the most common single-character I/O operators
/// are listed here.
fn is_shell_operator(arg: &str) -> bool {
    matches!(
        arg,
        "&&" | "||" | ";" | ";;" | "|" | ">" | ">>" | "<" | "<<" | "2>" | "2>>"
    )
}

/// Returns `true` if any argument is a standalone shell control operator.
///
/// When this returns `true` the generated command must be executed through a
/// shell (`sh -c`) so that operators such as `&&`, `|`, `>` are interpreted as
/// shell syntax rather than being passed as literal argument strings to the
/// first tool.
fn args_require_shell(args: &[String]) -> bool {
    args.iter().any(|a| is_shell_operator(a))
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

// ─── Command risk assessment ──────────────────────────────────────────────────

/// Risk level of a generated command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// No dangerous operations detected.
    Safe,
    /// Potentially risky operations (e.g., force overwrite flags).
    Warning,
    /// Highly dangerous operations (e.g., rm, sudo, dd).
    Dangerous,
}

/// Assess the risk level of a generated command by scanning its arguments
/// for dangerous operations such as `rm`, `sudo`, overwrite redirects, etc.
///
/// Returns `RiskLevel::Dangerous` for destructive operations that could cause
/// irreversible data loss, `RiskLevel::Warning` for operations that use force
/// flags or overwrite existing files, and `RiskLevel::Safe` otherwise.
pub fn assess_command_risk(args: &[String]) -> RiskLevel {
    if args.is_empty() {
        return RiskLevel::Safe;
    }

    // Dangerous commands that may appear in multi-step pipelines (after && or ||)
    const DANGEROUS_COMMANDS: &[&str] = &["rm", "rmdir", "sudo", "dd", "mkfs", "chmod", "chown"];

    // Force/overwrite flags that bypass safety checks
    const FORCE_FLAGS: &[&str] = &["-f", "--force", "--overwrite", "-y", "--yes"];

    let mut risk = RiskLevel::Safe;

    // Check each arg for dangerous patterns
    for (i, arg) in args.iter().enumerate() {
        let lower = arg.to_ascii_lowercase();

        // Direct dangerous command (first token or after && / ||)
        let is_cmd_position =
            i == 0 || (i > 0 && matches!(args[i - 1].as_str(), "&&" | "||" | ";" | "|"));

        if is_cmd_position {
            for &cmd in DANGEROUS_COMMANDS {
                if lower == cmd || lower.ends_with(&format!("/{cmd}")) {
                    return RiskLevel::Dangerous;
                }
            }
        }

        // Overwrite redirect (>) as a standalone operator
        if arg == ">" {
            risk = risk.max_level(RiskLevel::Warning);
        }

        // Force flags
        for &flag in FORCE_FLAGS {
            if lower == flag {
                risk = risk.max_level(RiskLevel::Warning);
            }
        }
    }

    // Check for input==output (file would be truncated before read)
    if has_same_input_output(args) {
        risk = risk.max_level(RiskLevel::Warning);
    }

    risk
}

impl RiskLevel {
    /// Returns the higher risk level of self and other.
    fn max_level(self, other: RiskLevel) -> RiskLevel {
        match (self, other) {
            (RiskLevel::Dangerous, _) | (_, RiskLevel::Dangerous) => RiskLevel::Dangerous,
            (RiskLevel::Warning, _) | (_, RiskLevel::Warning) => RiskLevel::Warning,
            _ => RiskLevel::Safe,
        }
    }
}

/// Check if the command appears to use the same file as both input and output.
fn has_same_input_output(args: &[String]) -> bool {
    const OUTPUT_FLAGS: &[&str] = &["-o", "--output", "-O", "--out"];
    let mut output_file: Option<&str> = None;

    for (i, arg) in args.iter().enumerate() {
        for &flag in OUTPUT_FLAGS {
            if arg == flag
                && let Some(val) = args.get(i + 1)
            {
                output_file = Some(val.as_str());
            }
            if let Some(rest) = arg.strip_prefix(&format!("{flag}="))
                && !rest.is_empty()
            {
                output_file = Some(rest);
            }
        }
    }

    if let Some(out) = output_file {
        // Check if any non-flag arg matches the output file
        for arg in args {
            if !arg.starts_with('-') && arg.as_str() == out && arg.contains('.') {
                // Same file appears as both positional (input) and output
                return true;
            }
        }
    }

    false
}

/// Return risk warning message for display.
pub fn risk_warning_message(risk: RiskLevel) -> Option<&'static str> {
    match risk {
        RiskLevel::Dangerous => Some(
            "⚠️  DANGEROUS: This command contains destructive operations (rm/sudo/dd). \
             Review carefully before executing!",
        ),
        RiskLevel::Warning => Some(
            "⚠  WARNING: This command uses force flags or may overwrite files. \
             Double-check the arguments.",
        ),
        RiskLevel::Safe => None,
    }
}

// ─── Input file validation ────────────────────────────────────────────────────

/// Scan command args for tokens that look like input file paths and check
/// whether they exist on disk.  Returns a list of file paths that were not
/// found.  This helps catch typos and missing files *before* the tool runs,
/// avoiding wasted API calls and confusing tool error messages.
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_input_files(args: &[String]) -> Vec<String> {
    const INPUT_FLAGS: &[&str] = &[
        "-i",
        "--input",
        "-I",
        "--in",
        "-1",
        "-2",
        "--in1",
        "--in2",
        "-x",
        "-U",
        "--ref",
        "--reference",
        "--genome",
        "--genome-dir",
        "--genomeDir",
        "--sjdbGTFfile",
        "--gtf",
        "--bed",
    ];
    const OUTPUT_FLAGS: &[&str] = &["-o", "--output", "-O", "--out", "-b", "--bam", "-S"];

    let mut missing = Vec::new();
    let mut skip_next = false;
    let mut known_output_indices = std::collections::HashSet::new();

    // First pass: mark indices of values following output flags
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        for &flag in OUTPUT_FLAGS {
            if arg == flag {
                known_output_indices.insert(i + 1);
                skip_next = true;
                break;
            }
        }
    }

    // Second pass: check input-like tokens
    skip_next = false;
    let mut expect_input = false;
    for (i, arg) in args.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        // Skip output values
        if known_output_indices.contains(&i) {
            continue;
        }

        // If previous token was an input flag, this token is an input file
        if expect_input {
            expect_input = false;
            if !arg.starts_with('-')
                && looks_like_file_path(arg)
                && !std::path::Path::new(arg).exists()
            {
                missing.push(arg.clone());
            }
            continue;
        }

        // Check if current token is an input flag
        for &flag in INPUT_FLAGS {
            if arg == flag {
                expect_input = true;
                break;
            }
        }
        if expect_input {
            continue;
        }

        // Shell operators reset context
        if is_shell_operator(arg) {
            continue;
        }

        // Positional args that look like file paths
        if !arg.starts_with('-')
            && looks_like_file_path(arg)
            && !known_output_indices.contains(&i)
            // Only check files that look like bioinformatics data files
            && has_bio_extension(arg)
            && !std::path::Path::new(arg).exists()
        {
            missing.push(arg.clone());
        }
    }

    missing
}

/// Heuristic: does this token look like a file path?
fn looks_like_file_path(arg: &str) -> bool {
    arg.contains('.')
        && !arg.contains(';')
        && !arg.contains('&')
        && !arg.contains('|')
        && !arg.contains('>')
        && !arg.contains('<')
        && !arg.starts_with("http://")
        && !arg.starts_with("https://")
}

/// Check if a path has a bioinformatics-relevant file extension.
fn has_bio_extension(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    const EXTENSIONS: &[&str] = &[
        ".bam",
        ".sam",
        ".cram",
        ".fastq",
        ".fq",
        ".fasta",
        ".fa",
        ".fna",
        ".vcf",
        ".bcf",
        ".bed",
        ".gff",
        ".gtf",
        ".bw",
        ".bigwig",
        ".fastq.gz",
        ".fq.gz",
        ".vcf.gz",
        ".bed.gz",
        ".fa.gz",
        ".bai",
        ".csi",
        ".tbi",
        ".idx",
    ];
    EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
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

    #[test]
    fn test_build_command_string_does_not_quote_shell_and_and() {
        // && is a shell operator — must appear unquoted so sh -c can interpret it
        let args: Vec<String> = vec![
            "sort".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "input.bam".to_string(),
            "&&".to_string(),
            "samtools".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
        ];
        let cmd = build_command_string("samtools", &args);
        assert!(cmd.contains(" && "), "cmd should contain unquoted &&");
        assert!(!cmd.contains("'&&'"), "&& must not be single-quoted");
        assert!(
            cmd.contains("samtools index"),
            "second subcommand must be present"
        );
    }

    #[test]
    fn test_build_command_string_does_not_quote_pipe() {
        let args: Vec<String> = vec![
            "view".to_string(),
            "input.bam".to_string(),
            "|".to_string(),
            "grep".to_string(),
            "^SQ".to_string(),
        ];
        let cmd = build_command_string("samtools", &args);
        assert!(cmd.contains(" | "), "cmd should contain unquoted |");
        assert!(!cmd.contains("'|'"), "| must not be single-quoted");
    }

    // ─── is_shell_operator ────────────────────────────────────────────────────

    #[test]
    fn test_is_shell_operator_known_operators() {
        assert!(is_shell_operator("&&"));
        assert!(is_shell_operator("||"));
        assert!(is_shell_operator(";"));
        assert!(is_shell_operator("|"));
        assert!(is_shell_operator(">"));
        assert!(is_shell_operator(">>"));
        assert!(is_shell_operator("<"));
        assert!(is_shell_operator("2>"));
        assert!(is_shell_operator("2>>"));
    }

    #[test]
    fn test_is_shell_operator_rejects_non_operators() {
        assert!(!is_shell_operator("-o"));
        assert!(!is_shell_operator("out.bam"));
        // Bare & is intentionally excluded — it appears in filter expressions
        // like `flag & 0x4` which `parse_shell_args` may tokenize as ["flag", "&", "0x4"]
        assert!(!is_shell_operator("&"));
        assert!(!is_shell_operator("flag & 0x4"));
        assert!(!is_shell_operator("samtools"));
        assert!(!is_shell_operator(""));
    }

    // ─── args_require_shell ───────────────────────────────────────────────────

    #[test]
    fn test_args_require_shell_with_double_ampersand() {
        let args: Vec<String> = vec![
            "sort".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
            "&&".to_string(),
            "samtools".to_string(),
            "index".to_string(),
            "sorted.bam".to_string(),
        ];
        assert!(args_require_shell(&args));
    }

    #[test]
    fn test_args_require_shell_with_pipe() {
        let args: Vec<String> = vec!["view".to_string(), "|".to_string(), "grep".to_string()];
        assert!(args_require_shell(&args));
    }

    #[test]
    fn test_args_require_shell_without_operators() {
        let args: Vec<String> = vec![
            "sort".to_string(),
            "-o".to_string(),
            "out.bam".to_string(),
            "input.bam".to_string(),
        ];
        assert!(!args_require_shell(&args));
    }

    #[test]
    fn test_args_require_shell_empty() {
        assert!(!args_require_shell(&[]));
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

    // ─── companion binary detection ───────────────────────────────────────────

    #[test]
    fn test_is_companion_binary_bowtie2_build() {
        assert!(is_companion_binary("bowtie2", "bowtie2-build"));
    }

    #[test]
    fn test_is_companion_binary_hisat2_build() {
        assert!(is_companion_binary("hisat2", "hisat2-build"));
    }

    #[test]
    fn test_is_companion_binary_bismark_underscore_prefix() {
        assert!(is_companion_binary("bismark", "bismark_genome_preparation"));
        assert!(is_companion_binary(
            "bismark",
            "bismark_methylation_extractor"
        ));
    }

    #[test]
    fn test_is_companion_binary_reverse_suffix() {
        // deduplicate_bismark ends with _bismark — detected as companion of bismark
        assert!(is_companion_binary("bismark", "deduplicate_bismark"));
    }

    #[test]
    fn test_is_companion_binary_reverse_suffix_requires_prefix() {
        // The candidate must be longer than just "_{tool}" — tool name alone doesn't count
        assert!(!is_companion_binary("bismark", "_bismark"));
    }

    #[test]
    fn test_is_companion_binary_flag_is_not_companion() {
        assert!(!is_companion_binary("bowtie2", "-x"));
        assert!(!is_companion_binary("bowtie2", "--no-unal"));
    }

    #[test]
    fn test_is_companion_binary_filename_is_not_companion() {
        // Data files (.fq, .fastq, .bam) are not companions even with tool name prefix
        assert!(!is_companion_binary("bowtie2", "bowtie2-input.fq"));
        assert!(!is_companion_binary("samtools", "sorted.bam"));
    }

    #[test]
    fn test_is_companion_binary_script_extension() {
        // Script-style companions (.sh, .py, .pl, .R) whose stem contains the
        // tool name (case-insensitive) should be detected.
        assert!(is_companion_binary("manta", "configureManta.py"));
        assert!(is_companion_binary("strelka2", "configureStrelka2.py"));
        // HOMER scripts contain "homer" nowhere, but are used as first token.
        // These rely on user-side dispatching so we don't match them here:
        assert!(!is_companion_binary("homer", "annotatePeaks.pl"));
    }

    #[test]
    fn test_is_companion_binary_script_prefix() {
        // bbtools companions: bbduk.sh, bbmap.sh, bbmerge.sh — stem starts with "bb"
        // but tool is "bbtools". These don't contain the full tool name, so they
        // are not matched by the simple contains check.
        // They are documented as separate companion scripts in the bbtools package.
        assert!(!is_companion_binary("bbtools", "bbduk.sh"));
    }

    #[test]
    fn test_is_companion_binary_no_prefix_match() {
        // "sort" does not start with "samtools-"
        assert!(!is_companion_binary("samtools", "sort"));
        assert!(!is_companion_binary("samtools", "index"));
    }

    #[test]
    fn test_effective_command_companion_redirects_tool() {
        let args: Vec<String> = vec![
            "bowtie2-build".to_string(),
            "reference.fa".to_string(),
            "ref_idx".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("bowtie2", &args);
        assert_eq!(eff_tool, "bowtie2-build");
        assert_eq!(eff_args, &["reference.fa", "ref_idx"]);
    }

    #[test]
    fn test_effective_command_normal_args_unchanged() {
        let args: Vec<String> = vec![
            "-x".to_string(),
            "ref_idx".to_string(),
            "-1".to_string(),
            "R1.fq.gz".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("bowtie2", &args);
        assert_eq!(eff_tool, "bowtie2");
        assert_eq!(eff_args, args.as_slice());
    }

    #[test]
    fn test_effective_command_samtools_subcommand_unchanged() {
        // "sort" is a samtools subcommand, not a companion binary — must NOT redirect
        let args: Vec<String> = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "-o".to_string(),
            "sorted.bam".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("samtools", &args);
        assert_eq!(eff_tool, "samtools");
        assert_eq!(eff_args, args.as_slice());
    }

    #[test]
    fn test_build_command_string_companion_binary() {
        // When first arg is a companion binary, build_command_string should use
        // that binary as the executable, not prepend the base tool name.
        let args: Vec<String> = vec![
            "bowtie2-build".to_string(),
            "reference.fa".to_string(),
            "ref_idx".to_string(),
        ];
        let cmd = build_command_string("bowtie2", &args);
        assert_eq!(cmd, "bowtie2-build reference.fa ref_idx");
        assert!(
            !cmd.starts_with("bowtie2 bowtie2-build"),
            "must not double the tool name"
        );
    }

    #[test]
    fn test_effective_command_script_companion() {
        // configureManta.py is a script companion of manta → should redirect
        let args: Vec<String> = vec![
            "configureManta.py".to_string(),
            "--bam".to_string(),
            "input.bam".to_string(),
            "--referenceFasta".to_string(),
            "ref.fa".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("manta", &args);
        assert_eq!(eff_tool, "configureManta.py");
        assert_eq!(
            eff_args,
            &["--bam", "input.bam", "--referenceFasta", "ref.fa"]
        );
    }

    #[test]
    fn test_effective_command_standalone_script() {
        // bbduk.sh is a standalone script from the bbtools package
        let args: Vec<String> = vec![
            "bbduk.sh".to_string(),
            "in=reads.fq".to_string(),
            "out=clean.fq".to_string(),
            "ref=adapters.fa".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("bbtools", &args);
        assert_eq!(eff_tool, "bbduk.sh");
        assert_eq!(
            eff_args,
            &["in=reads.fq", "out=clean.fq", "ref=adapters.fa"]
        );
    }

    #[test]
    fn test_effective_command_rseqc_script() {
        // infer_experiment.py is a standalone RSeQC script
        let args: Vec<String> = vec![
            "infer_experiment.py".to_string(),
            "-i".to_string(),
            "aligned.bam".to_string(),
            "-r".to_string(),
            "ref.bed".to_string(),
        ];
        let (eff_tool, eff_args) = effective_command("rseqc", &args);
        assert_eq!(eff_tool, "infer_experiment.py");
        assert_eq!(eff_args, &["-i", "aligned.bam", "-r", "ref.bed"]);
    }

    #[test]
    fn test_is_script_executable() {
        assert!(is_script_executable("bbduk.sh"));
        assert!(is_script_executable("infer_experiment.py"));
        assert!(is_script_executable("annotatePeaks.pl"));
        assert!(is_script_executable("draw_fusions.R"));
        assert!(is_script_executable("configureStrelkaGermlineWorkflow.py"));
        // NOT script executables:
        assert!(!is_script_executable("reads.fastq.gz")); // data file extension
        assert!(!is_script_executable("-i")); // flag
        assert!(!is_script_executable("/usr/bin/script.py")); // path
        assert!(!is_script_executable("sort")); // no extension
        assert!(!is_script_executable("input.bam")); // data file
    }

    #[test]
    fn test_effective_command_data_file_not_script() {
        // A data file like input.bam should NOT be treated as a script executable
        let args: Vec<String> = vec!["input.bam".to_string(), "-o".to_string()];
        let (eff_tool, _) = effective_command("samtools", &args);
        assert_eq!(eff_tool, "samtools");
    }
}
