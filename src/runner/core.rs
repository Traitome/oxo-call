//! Core runner implementation.
//!
//! Contains the `Runner` struct and its primary methods for command generation
//! and execution.
//!
//! # Core flow
//!
//! ```text
//! Natural-language task
//!   ──▶ resolve_docs   (fetch --help, subcommand help, build StructuredDoc)
//!   ──▶ load_skill     (built-in / community / MCP; None if not found)
//!   ──▶ llm.suggest_command  (single LLM call, enriched prompt)
//!   ──▶ execute / dry_run
//! ```

use crate::config::Config;
use crate::doc_processor::{DocProcessor, StructuredDoc};
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
use crate::job;
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::skill::SkillManager;
use chrono::Utc;
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;
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
    /// The task description actually used.
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
    /// The task description that was actually used.
    pub(crate) effective_task: String,
    /// Structured documentation used for flag/subcommand validation, if available.
    pub(crate) structured_doc: Option<StructuredDoc>,
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
    /// [Ablation] When true, do not load the skill file for the tool.
    pub(crate) no_skill: bool,
    /// [Ablation] When true, do not load tool documentation (--help output).
    pub(crate) no_doc: bool,
    /// [Ablation] When true, do not use the oxo-call system prompt.
    pub(crate) no_prompt: bool,
    /// Named variables substituted into the task description before the LLM call.
    pub(crate) vars: HashMap<String, String>,
    /// Input items for batch/parallel execution (empty = single run).
    pub(crate) input_items: Vec<String>,
    /// Maximum number of parallel jobs when `input_items` is non-empty.
    pub(crate) jobs: usize,
    /// When true, stop the batch after the first failed item.
    pub(crate) stop_on_error: bool,
    /// When true, automatically retry failed commands with LLM-corrected arguments.
    pub(crate) auto_retry: bool,
    /// When true, disable SSE streaming for LLM responses.
    pub(crate) no_stream: bool,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        let fetcher = DocsFetcher::new(config.clone());
        let llm = LlmClient::new(config.clone());
        let skill_manager = SkillManager::new(config.clone());
        Runner {
            fetcher,
            llm,
            skill_manager,
            config,
            verbose: false,
            no_cache: false,
            verify: false,
            no_skill: false,
            no_doc: false,
            no_prompt: false,
            vars: HashMap::new(),
            input_items: Vec::new(),
            jobs: 1,
            stop_on_error: false,
            auto_retry: false,
            no_stream: false,
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

    /// Enable automatic retry with LLM-corrected commands on failure.
    pub fn with_auto_retry(mut self, auto_retry: bool) -> Self {
        self.auto_retry = auto_retry;
        self
    }

    /// Disable SSE streaming for LLM responses.
    pub fn with_no_stream(mut self, no_stream: bool) -> Self {
        self.no_stream = no_stream;
        if no_stream {
            self.llm.set_no_stream(true);
        }
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

    /// Set named variables that will be substituted into the task description.
    pub fn with_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.vars = vars;
        self
    }

    /// Set input items for batch / parallel execution.
    pub fn with_input_items(mut self, items: Vec<String>) -> Self {
        self.input_items = items;
        self
    }

    /// Set the maximum number of parallel jobs (default: 1 = sequential).
    pub fn with_jobs(mut self, jobs: usize) -> Self {
        self.jobs = jobs.max(1);
        self
    }

    /// When enabled, abort the batch after the first failed item.
    pub fn with_stop_on_error(mut self, stop_on_error: bool) -> Self {
        self.stop_on_error = stop_on_error;
        self
    }

    /// Resolve documentation for the tool, showing a spinner while fetching.
    /// Also attempts to fetch help for the specific subcommand matching the user's task.
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

    /// Core pipeline:
    ///
    /// 1. Fetch tool documentation (always — docs ground the flag catalog).
    /// 2. Load skill (optional enhancement; does not replace docs).
    /// 3. Extract `StructuredDoc` (flag catalog + examples for anti-hallucination).
    /// 4. Single LLM call with enriched, structured prompt.
    pub(crate) async fn prepare(&self, tool: &str, task: &str) -> Result<PrepareResult> {
        let spinner = if !self.no_doc {
            make_spinner(&format!("Fetching documentation for '{tool}'..."))
        } else {
            make_spinner("Loading skill...")
        };

        // ── Step 1: Fetch docs (always, regardless of skill availability) ───────
        let docs = if self.no_doc {
            if self.verbose {
                eprintln!(
                    "{} [Ablation] Skipping documentation (--no-doc)",
                    "[verbose]".dimmed()
                );
            }
            String::new()
        } else {
            self.resolve_docs(tool, task).await?
        };

        spinner.finish_and_clear();

        // ── Step 2: Load skill (enhancement, not replacement) ────────────────
        let skill = if self.no_skill {
            if self.verbose {
                eprintln!(
                    "{} [Ablation] Skipping skill (--no-skill)",
                    "[verbose]".dimmed()
                );
            }
            None
        } else {
            self.skill_manager.load_async(tool).await
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
        }

        // ── Step 3: Build StructuredDoc (flag catalog + anti-hallucination) ───
        let structured_doc = if !docs.is_empty() {
            let processor = DocProcessor::new();
            let sdoc = processor.clean_and_structure(&docs);
            if self.verbose {
                eprintln!(
                    "{} Doc analysis: quality={:.2}, {} flags, {} examples extracted",
                    "[verbose]".dimmed(),
                    sdoc.quality_score,
                    sdoc.flag_catalog.len(),
                    sdoc.extracted_examples.len(),
                );
            }
            Some(sdoc)
        } else {
            None
        };

        if self.verbose {
            let model_name = self.config.effective_model();
            let ctx_window = self.config.effective_context_window();
            let tier = self.config.effective_prompt_tier();
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

        // ── Version compatibility check ───────────────────────────────────────
        if let Some(s) = &skill
            && (s.meta.min_version.is_some() || s.meta.max_version.is_some())
            && let Some(detected) = detect_tool_version(tool)
        {
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
        }

        let docs_hash = sha256_hex(&docs);
        let skill_name = skill.as_ref().map(|s| s.meta.name.clone());

        // ── User preference hints from history ───────────────────────────────
        let preferences_hint = {
            let history = match crate::history::HistoryStore::load_all() {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("Failed to load command history for preference learning: {e}");
                    Vec::new()
                }
            };
            let prefs = crate::history::learn_user_preferences(tool, &history);
            prefs.to_prompt_hint()
        };

        // Build enriched task with preferences hint when available.
        // XML special characters in user-supplied text are escaped.
        let enriched_task = {
            fn xml_escape(s: &str) -> String {
                s.replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
            }
            let safe_task = xml_escape(task);
            if preferences_hint.is_empty() {
                task.to_string()
            } else {
                format!("<task>\n{safe_task}\n</task>\n<hints>\n{preferences_hint}\n</hints>")
            }
        };

        if self.verbose && enriched_task != task {
            eprintln!(
                "{} Enriched task with user preference hints",
                "[verbose]".dimmed()
            );
        }

        let spinner = make_spinner(&format!(
            "Asking LLM to generate command arguments{}...",
            skill_name
                .as_deref()
                .map(|n| format!(" (skill: {n})"))
                .unwrap_or_default()
        ));

        // ── Step 4: Single LLM call ──────────────────────────────────────────
        let suggestion = self
            .llm
            .suggest_command(
                tool,
                &docs,
                &enriched_task,
                skill.as_ref(),
                self.no_prompt,
                structured_doc.as_ref(),
            )
            .await?;

        spinner.finish_and_clear();

        if self.verbose {
            eprintln!(
                "{} LLM call complete: {:.1}ms",
                "[verbose]".dimmed(),
                suggestion.inference_ms,
            );
        }

        Ok(PrepareResult {
            suggestion,
            docs_hash,
            skill_name,
            effective_task: task.to_string(),
            structured_doc,
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
        // ── Apply vars + batch dispatch ──────────────────────────
        let _task_buf;
        let task: &str = if self.vars.is_empty() {
            task
        } else {
            _task_buf = job::interpolate_command(task, "", 0, &self.vars);
            &_task_buf
        };
        if !self.input_items.is_empty() {
            return self.dry_run_batch(tool, task, json).await;
        }

        let result = self.prepare(tool, task).await?;
        let full_cmd = build_command_string(tool, &result.suggestion.args);

        // Record in history before displaying, so the entry is always saved.
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
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Command (dry-run):".bold().yellow());
        println!("  {}", full_cmd.green().bold());
        println!();

        // ── Flag & subcommand validation ──────────────────────────────
        if let Some(ref sdoc) = result.structured_doc {
            let vr = super::validation::validate_args(&result.suggestion.args, sdoc);
            for w in &vr.warnings {
                eprintln!("  {} {}", "⚠ Validation:".yellow(), w.yellow());
            }
            if !vr.warnings.is_empty() {
                println!();
            }
        }

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
        // ── Apply vars + batch dispatch ──────────────────────────
        let _task_buf;
        let task: &str = if self.vars.is_empty() {
            task
        } else {
            _task_buf = job::interpolate_command(task, "", 0, &self.vars);
            &_task_buf
        };
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

            // ── Flag & subcommand validation ──────────────────────────────
            if let Some(ref sdoc) = result.structured_doc {
                let vr = super::validation::validate_args(&result.suggestion.args, sdoc);
                for w in &vr.warnings {
                    eprintln!("  {} {}", "⚠ Validation:".yellow(), w.yellow());
                }
                if !vr.warnings.is_empty() {
                    println!();
                }
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
