//! Core runner implementation.
//!
//! Contains the `Runner` struct and its primary methods for command generation
//! and execution.

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
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;
#[cfg(not(target_arch = "wasm32"))]
use uuid::Uuid;

use super::batch::BatchRunner;
use super::retry::RetryRunner;
use super::utils::{
    RiskLevel, assess_command_risk, build_command_string, detect_tool_version, effective_command,
    make_spinner, risk_warning_message, sha256_hex, validate_input_files,
};

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
pub(crate) struct PrepareResult {
    pub(crate) suggestion: LlmCommandSuggestion,
    /// SHA-256 hex digest of the documentation text used in the prompt.
    pub(crate) docs_hash: String,
    /// Name of the matched skill, if one was loaded.
    pub(crate) skill_name: Option<String>,
    /// The task description that was actually used (may differ from the user-supplied
    /// task when `--optimize-task` is enabled).
    pub(crate) effective_task: String,
}

pub struct Runner {
    pub(crate) config: Config,
    fetcher: DocsFetcher,
    pub(crate) llm: LlmClient,
    skill_manager: SkillManager,
    pub(crate) verbose: bool,
    pub(crate) no_cache: bool,
    /// When true, use LLM to verify the result after execution.
    pub(crate) verify: bool,
    /// When true, use LLM to optimize/expand the user's task description before
    /// building the command generation prompt.
    pub(crate) optimize_task: bool,
    /// [Ablation] When true, do not load the skill file for the tool.
    pub(crate) no_skill: bool,
    /// [Ablation] When true, do not load tool documentation (--help output).
    pub(crate) no_doc: bool,
    /// [Ablation] When true, do not use the oxo-call system prompt.
    pub(crate) no_prompt: bool,
    /// Named variables substituted into the task description before the LLM call.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) vars: HashMap<String, String>,
    /// Input items for batch/parallel execution (empty = single run).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) input_items: Vec<String>,
    /// Maximum number of parallel jobs when `input_items` is non-empty.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) jobs: usize,
    /// When true, stop the batch after the first failed item.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) stop_on_error: bool,
    /// When true, automatically retry failed commands with LLM-corrected arguments.
    pub(crate) auto_retry: bool,
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
            auto_retry: false,
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

    /// Enable automatic retry with LLM-corrected commands on failure.
    pub fn with_auto_retry(mut self, auto_retry: bool) -> Self {
        self.auto_retry = auto_retry;
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
    ///
    /// Uses intelligent summarization based on model size to keep prompts concise
    /// while preserving critical information.
    pub(crate) async fn resolve_docs(&self, tool: &str, task: &str) -> Result<String> {
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

        // Use model-specific summarization for optimal prompt length
        let model_size = self.config.model_size_category();
        let summarized = docs.combined_for_model(model_size);

        if self.verbose {
            let original_len = docs.combined().len();
            let summarized_len = summarized.len();
            let reduction = if original_len > 0 {
                (1.0 - summarized_len as f64 / original_len as f64) * 100.0
            } else {
                0.0
            };
            eprintln!(
                "{} Documentation: {} chars → {} chars ({:.1}% reduction, model={})",
                "[verbose]".dimmed(),
                original_len,
                summarized_len,
                reduction,
                model_size
            );
        }

        Ok(summarized)
    }

    /// Core logic: fetch docs + load skill (in parallel) → (optionally optimize task) → call LLM.
    ///
    /// Documentation fetching and skill loading are independent operations that
    /// can run concurrently via `tokio::join!`, reducing latency by ~200-500ms
    /// compared to the previous serial approach.
    ///
    /// **Adaptive Doc Injection**: When both skill and doc are enabled (full scenario),
    /// documentation is only injected if:
    /// 1. Skill is missing, OR
    /// 2. Skill quality is low (few examples, incomplete coverage)
    ///
    /// This prevents low-quality documentation from degrading performance when
    /// high-quality skill knowledge is available.
    pub(crate) async fn prepare(&self, tool: &str, task: &str) -> Result<PrepareResult> {
        // ── Parallel fetch: docs + skill ──────────────────────────────────────
        let spinner = if !self.no_doc {
            make_spinner(&format!("Fetching documentation for '{tool}'..."))
        } else {
            make_spinner("Loading skill...")
        };

        // Load skill first to determine if doc is needed
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

        // Run skill loading first
        let skill = skill_future.await;

        // Determine if we need documentation based on skill quality
        let should_fetch_doc = if self.no_doc {
            false
        } else if skill.is_none() {
            // No skill available → need doc
            true
        } else {
            // Skill available → check quality
            if let Some(ref s) = skill {
                // Skill quality heuristics:
                // - Low quality: <3 examples OR <3 pitfalls
                // - Medium quality: 3-5 examples
                // - High quality: >5 examples
                let example_count = s.examples.len();
                let pitfall_count = s.context.pitfalls.len();

                // Only fetch doc if skill quality is low
                example_count < 3 || pitfall_count < 3
            } else {
                false
            }
        };

        let docs_future = async {
            if !should_fetch_doc {
                if self.verbose && !self.no_doc {
                    eprintln!(
                        "{} Skipping documentation (high-quality skill available)",
                        "[verbose]".dimmed()
                    );
                }
                Ok(String::new())
            } else {
                self.resolve_docs(tool, task).await
            }
        };

        // Run doc fetching if needed
        let docs_result = docs_future.await;
        spinner.finish_and_clear();

        let docs = docs_result?;

        if self.verbose && !docs.is_empty() {
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

        // ── Version compatibility check ───────────────────────────────────────────
        if let Some(s) = &skill
            && (s.meta.min_version.is_some() || s.meta.max_version.is_some())
        {
            if let Some(detected) = detect_tool_version(tool) {
                if let Err(e) = crate::runner::utils::check_version_compatibility(
                    &detected,
                    s.meta.min_version.as_deref(),
                    s.meta.max_version.as_deref(),
                ) {
                    eprintln!("{} {}", "warning:".bold().yellow(), e);
                    eprintln!(
                        "{} Skill '{}' may have outdated examples or flags.",
                        "warning:".bold().yellow(),
                        s.meta.name
                    );
                } else if self.verbose {
                    eprintln!(
                        "{} Tool version {} is compatible with skill requirements",
                        "[verbose]".dimmed(),
                        detected
                    );
                }
            } else if self.verbose {
                eprintln!(
                    "{} Could not detect tool version for compatibility check",
                    "[verbose]".dimmed(),
                );
            }
        }

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
                    cache_hit: None,
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
            let use_shell = super::utils::args_require_shell(&result.suggestion.args);

            // When verification is enabled, capture stderr for analysis.
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
                    cache_hit: None,
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
                self.run_verification(super::retry::VerifyParams {
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

            // ── Auto-retry on failure ─────────────────────────────────────────
            if self.auto_retry && !success {
                if !json {
                    println!();
                    println!(
                        "  {} Analyzing failure and generating corrected command...",
                        "⟳".cyan().bold()
                    );
                }

                let stderr_for_retry = if !captured_stderr.is_empty() {
                    captured_stderr.clone()
                } else {
                    format!("Command failed with exit code {exit_code}")
                };

                match self
                    .auto_retry_on_failure(
                        tool,
                        &result.effective_task,
                        &full_cmd,
                        exit_code,
                        &stderr_for_retry,
                        json,
                    )
                    .await
                {
                    Ok(()) => {}
                    Err(e) => {
                        if !json {
                            eprintln!("  {} Auto-retry failed: {}", "✗".red().bold(), e);
                        }
                    }
                }
            }

            Ok(())
        }
    }
}
