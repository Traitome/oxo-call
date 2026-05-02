//! Core runner implementation.
//!
//! Contains the `Runner` struct and its primary methods for command generation
//! and execution.

use crate::config::Config;
use crate::doc_processor::{DocProcessor, StructuredDoc};
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::execution::feedback::{FeedbackCollector, FeedbackEntry};
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
use crate::job;
use crate::knowledge::error_db::ErrorKnowledgeDb;
use crate::llm::{LlmClient, LlmCommandSuggestion};
use crate::markdown;
use crate::schema::{CliSchema, CliStyle, parse_help, schema_post_process};
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
    /// The task description actually used (may differ from user input when
    /// automatic normalization is active).
    pub effective_task: String,
}

/// Intermediate result from the `prepare` step that carries provenance metadata
/// alongside the LLM suggestion.
pub(crate) struct PrepareResult {
    pub(crate) suggestion: LlmCommandSuggestion,
    pub(crate) docs_hash: String,
    pub(crate) skill_name: Option<String>,
    pub(crate) effective_task: String,
    pub(crate) structured_doc: Option<StructuredDoc>,
    #[allow(dead_code)]
    pub(crate) parsed_schema: Option<CliSchema>,
}

pub struct Runner {
    pub(crate) config: Config,
    fetcher: DocsFetcher,
    pub(crate) llm: LlmClient,
    skill_manager: SkillManager,
    pub(crate) verbose: bool,
    pub(crate) no_cache: bool,
    pub(crate) verify: bool,
    pub(crate) vars: HashMap<String, String>,
    /// Input items for batch/parallel execution (empty = single run).
    pub(crate) input_items: Vec<String>,
    /// Maximum number of parallel jobs when `input_items` is non-empty.
    pub(crate) jobs: usize,
    /// When true, stop the batch after the first failed item.
    pub(crate) stop_on_error: bool,
    /// When true, automatically retry failed commands with LLM-corrected arguments.
    pub(crate) auto_retry: bool,
    /// Force a specific workflow scenario (auto-detected by default)
    pub(crate) force_scenario: Option<crate::workflow_graph::WorkflowScenario>,
    /// When true, disable skill loading.
    pub(crate) no_skill: bool,
    /// When true, disable documentation fetching.
    pub(crate) no_doc: bool,
    /// When true, disable the system prompt (bare LLM mode).
    pub(crate) no_prompt: bool,
    /// When true, disable SSE streaming for LLM responses.
    pub(crate) no_stream: bool,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        // Build sub-components before consuming `config`, using clones only
        // where the constructor takes ownership.  The final field assignment
        // consumes the original—one fewer clone than the naïve approach.
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
            vars: HashMap::new(),
            input_items: Vec::new(),
            jobs: 1,
            stop_on_error: false,
            auto_retry: false,
            force_scenario: None,
            no_skill: false,
            no_doc: false,
            no_prompt: false,
            no_stream: false,
        }
    }

    /// Enable verbose output for this runner.
    pub fn with_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    /// Skip cached documentation and fetch fresh --help output.
    pub fn with_no_cache(&mut self, no_cache: bool) -> &mut Self {
        self.no_cache = no_cache;
        self
    }

    /// Enable LLM-based result verification after execution.
    pub fn with_verify(&mut self, verify: bool) -> &mut Self {
        self.verify = verify;
        self
    }

    /// Enable automatic retry with LLM-corrected commands on failure.
    pub fn with_auto_retry(&mut self, auto_retry: bool) -> &mut Self {
        self.auto_retry = auto_retry;
        self
    }

    /// Force a specific workflow scenario.
    pub fn with_scenario(
        &mut self,
        scenario: crate::workflow_graph::WorkflowScenario,
    ) -> &mut Self {
        self.force_scenario = Some(scenario);
        self
    }

    /// Disable skill loading for this run.
    pub fn with_no_skill(&mut self, no_skill: bool) -> &mut Self {
        self.no_skill = no_skill;
        self
    }

    /// Disable documentation fetching for this run.
    pub fn with_no_doc(&mut self, no_doc: bool) -> &mut Self {
        self.no_doc = no_doc;
        self
    }

    /// Disable the system prompt for this run (bare LLM mode).
    pub fn with_no_prompt(&mut self, no_prompt: bool) -> &mut Self {
        self.no_prompt = no_prompt;
        self
    }

    /// Disable SSE streaming for LLM responses.
    pub fn with_no_stream(&mut self, no_stream: bool) -> &mut Self {
        self.no_stream = no_stream;
        if no_stream {
            self.llm.set_no_stream(true);
        }
        self
    }

    pub fn with_vars(&mut self, vars: HashMap<String, String>) -> &mut Self {
        self.vars = vars;
        self
    }

    /// Set input items for batch / parallel execution.
    ///
    /// When non-empty, the LLM is called once and the generated command
    /// template (which may contain `{item}`) is executed for every item.
    pub fn with_input_items(&mut self, items: Vec<String>) -> &mut Self {
        self.input_items = items;
        self
    }

    /// Set the maximum number of parallel jobs (default: 1 = sequential).
    pub fn with_jobs(&mut self, jobs: usize) -> &mut Self {
        self.jobs = jobs.max(1);
        self
    }

    /// When enabled, abort the batch after the first failed item.
    pub fn with_stop_on_error(&mut self, stop_on_error: bool) -> &mut Self {
        self.stop_on_error = stop_on_error;
        self
    }

    /// Resolve documentation for the tool, showing a spinner while fetching.
    /// Also attempts to fetch help for the specific subcommand matching the user's task.
    ///
    /// Returns the raw ToolDocs object for StructuredDoc processing,
    /// then callers can compress as needed.
    pub(crate) async fn resolve_docs_raw(
        &self,
        tool: &str,
        task: &str,
    ) -> Result<crate::docs::ToolDocs> {
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

        Ok(docs)
    }

    /// Summarize ToolDocs for LLM prompt, with model-specific length optimization.
    fn summarize_docs_for_model(&self, docs: &crate::docs::ToolDocs) -> String {
        let model_size = self.config.model_size_category();
        docs.combined_for_model(model_size)
    }

    fn max_doc_len_for_model(&self) -> usize {
        match self.config.model_size_category() {
            "small" => crate::doc_summarizer::MAX_DOC_LEN_SMALL_MODEL,
            "large" => crate::doc_summarizer::MAX_DOC_LEN_LARGE_MODEL,
            _ => crate::doc_summarizer::MAX_DOC_LEN_MEDIUM_MODEL,
        }
    }

    /// Core logic: fetch docs + load skill → select workflow mode → run LlmWorkflowExecutor.
    ///
    /// Mode selection rules (in priority order):
    /// 1. `--scenario` flag set → use scenario's default mode.
    /// 2. No static skill + docs available → Quality (generates mini-skill from doc, cached).
    /// 3. Static skill available or no docs → Fast (skill already provides grounding).
    ///
    /// In Quality mode the executor optionally normalizes the task (if vague/short/non-ASCII),
    /// generates a mini-skill from the documentation (result cached to disk), and uses it
    /// for command generation.
    pub(crate) async fn prepare(&self, tool: &str, task: &str) -> Result<PrepareResult> {
        let scenario = self
            .force_scenario
            .unwrap_or(crate::workflow_graph::WorkflowScenario::Full);

        // Ablation flags override scenario-based defaults.
        // This supports the 5 benchmark ablation scenarios:
        //   bare:   --no-skill --no-doc --no-prompt
        //   prompt: --no-skill --no-doc
        //   skill:  --no-doc
        //   doc:    --no-skill
        //   full:   (no flags)
        let should_load_skill =
            !self.no_skill && matches!(scenario, crate::workflow_graph::WorkflowScenario::Full);
        let should_fetch_doc = !self.no_doc
            && matches!(
                scenario,
                crate::workflow_graph::WorkflowScenario::Doc
                    | crate::workflow_graph::WorkflowScenario::Full
            );
        let no_prompt =
            self.no_prompt || matches!(scenario, crate::workflow_graph::WorkflowScenario::Bare);

        let skill = if should_load_skill {
            self.skill_manager.load_async(tool).await
        } else {
            if self.verbose && matches!(scenario, crate::workflow_graph::WorkflowScenario::Bare) {
                eprintln!(
                    "{} [bare] Skipping skill and documentation",
                    "[verbose]".dimmed()
                );
            }
            None
        };

        let tool_docs = if !should_fetch_doc {
            None
        } else {
            let spinner = make_spinner(&format!("Fetching documentation for '{tool}'..."));
            let result = self.resolve_docs_raw(tool, task).await;
            spinner.finish_and_clear();
            Some(result?)
        };

        // ── Build StructuredDoc from RAW docs BEFORE summarization ──
        // This is critical: we need the full documentation to extract flags and examples.
        // Summarization happens AFTER StructuredDoc creation, so flag_catalog remains accurate.
        let (structured_doc, parsed_schema) = if let Some(ref docs) = tool_docs {
            let raw_docs = docs.combined();
            let processor = DocProcessor::new();
            let sdoc = processor.clean_and_structure(&raw_docs);

            let schema = parse_help(tool, &raw_docs);

            if self.verbose {
                eprintln!(
                    "{} Doc analysis: quality={:.2}, flags={}/{} (catalog/quick), {} examples extracted, {} subcommands",
                    "[verbose]".dimmed(),
                    sdoc.quality_score,
                    sdoc.flag_catalog.len(),
                    sdoc.quick_flags.len(),
                    sdoc.extracted_examples.len(),
                    sdoc.commands.split(',').count().saturating_sub(1),
                );
                if !sdoc.commands.is_empty() && sdoc.commands.split(',').count() <= 10 {
                    eprintln!("{} Subcommands: {}", "[verbose]".dimmed(), sdoc.commands);
                }
                eprintln!(
                    "{} Schema: style={:?}, flags={}, subcommands={}, source={}",
                    "[verbose]".dimmed(),
                    schema.cli_style,
                    schema.flags.len() + schema.global_flags.len(),
                    schema.subcommands.len(),
                    schema.schema_source,
                );
                if let Some(subcmd) = schema.select_subcommand(task) {
                    eprintln!(
                        "{} Selected subcommand: {}",
                        "[verbose]".dimmed(),
                        subcmd.name
                    );
                }
            }
            (Some(sdoc), Some(schema))
        } else {
            (None, None)
        };

        // Now summarize docs for LLM prompt (model-specific length optimization)
        let docs = if let Some(ref docs_obj) = tool_docs {
            let summarized = self.summarize_docs_for_model(docs_obj);

            // Build structured header from raw docs (usage lines, valid flags, subcommands)
            let raw_docs = docs_obj.combined();
            let structured_header =
                crate::doc_summarizer::build_structured_summary(&raw_docs, tool);

            let enhanced_docs = if let Some(ref schema) = parsed_schema {
                let mut enhanced = summarized.clone();

                if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
                    enhanced.push_str("\n\n=== CRITICAL: SUBCOMMAND REQUIRED ===\n");
                    enhanced.push_str("This tool REQUIRES a subcommand as the FIRST argument.\n");
                    if let Some(subcmd) = schema.select_subcommand(task) {
                        enhanced.push_str(&format!(
                            "RECOMMENDED subcommand for this task: '{}'\n",
                            subcmd.name
                        ));
                    }
                    enhanced.push_str(&format!(
                        "Available subcommands: {}\n",
                        schema
                            .subcommands
                            .iter()
                            .map(|s| s.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }

                let all_flag_names =
                    schema.all_flag_names(schema.select_subcommand(task).map(|s| s.name.as_str()));
                if !all_flag_names.is_empty() && !structured_header.contains("VALID FLAGS") {
                    enhanced.push_str("\n\n=== VALID FLAGS (use ONLY these) ===\n");
                    let flags_preview: Vec<String> = all_flag_names
                        .iter()
                        .take(20)
                        .map(|s| s.to_string())
                        .collect();
                    enhanced.push_str(&flags_preview.join(", "));
                    enhanced.push('\n');
                }

                if !schema.constraints.is_empty() {
                    for constraint in &schema.constraints {
                        enhanced.push_str(&format!("- {}\n", constraint.message()));
                    }
                }

                enhanced
            } else {
                summarized
            };

            // Prepend structured header (usage lines, valid flags, subcommands)
            // This is the most critical information for the LLM.
            // Ensure total doesn't exceed model budget by trimming body if needed.
            let final_docs = if structured_header.is_empty() {
                enhanced_docs
            } else {
                let max_doc_len = self.max_doc_len_for_model();
                let header_len = structured_header.len() + 2; // +2 for "\n\n"
                if header_len + enhanced_docs.len() <= max_doc_len {
                    format!("{}\n\n{}", structured_header, enhanced_docs)
                } else {
                    // Trim body to fit header + body within budget
                    let body_budget = max_doc_len.saturating_sub(header_len);
                    let trimmed_body = if enhanced_docs.len() > body_budget {
                        enhanced_docs[..body_budget].to_string()
                    } else {
                        enhanced_docs
                    };
                    format!("{}\n\n{}", structured_header, trimmed_body)
                }
            };

            if self.verbose {
                let original_len = docs_obj.combined().len();
                let summarized_len = final_docs.len();
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
                    self.config.model_size_category()
                );
            }
            final_docs
        } else {
            String::new()
        };

        let docs_hash = sha256_hex(&docs);

        let _effective_task = task.to_string();

        let skill_name = skill.as_ref().map(|s| s.meta.name.clone());

        let skill_label = skill_name
            .as_ref()
            .map(|n| format!(" (skill: {n})"))
            .unwrap_or_default();

        if self.verbose {
            if let Some(ref s) = skill {
                eprintln!(
                    "{} Skill loaded: {} ({} examples)",
                    "[verbose]".dimmed(),
                    s.meta.name,
                    s.examples.len()
                );
            } else if should_load_skill {
                eprintln!("{} No skill found for '{}'", "[verbose]".dimmed(), tool);
            }
            let model_name = self.config.effective_model();
            eprintln!(
                "{} LLM: model={}, provider={}",
                "[verbose]".dimmed(),
                model_name,
                self.config.effective_provider(),
            );
        }

        let spinner = make_spinner(&format!(
            "Asking LLM to generate command arguments{skill_label}..."
        ));

        // Single LLM call — no task normalization, no multi-agent orchestration
        let mut suggestion = self
            .llm
            .suggest_command(
                tool,
                &docs,
                task, // Use original task, NOT normalized
                skill.as_ref(),
                no_prompt,
                structured_doc.as_ref(),
                parsed_schema.as_ref(),
            )
            .await?;

        spinner.finish_and_clear();

        // Deterministic schema post-process (no LLM involved)
        if let Some(ref sch) = parsed_schema {
            suggestion.args = schema_post_process(&suggestion.args, sch, task);
        }

        if self.verbose {
            eprintln!(
                "{} Generated: args={:?}, explanation={}",
                "[verbose]".dimmed(),
                suggestion.args,
                suggestion.explanation,
            );
        }

        Ok(PrepareResult {
            suggestion,
            docs_hash,
            skill_name,
            effective_task: task.to_string(),
            structured_doc,
            parsed_schema,
        })
    }

    /// Generate the LLM-suggested command without printing or executing it.
    ///
    /// Used by the `server run` handler to obtain the command string that will
    /// be sent over SSH, while keeping display logic in the caller.
    pub async fn generate_command(&self, tool: &str, task: &str) -> Result<GeneratedCommand> {
        let result = self.prepare(tool, task).await?;
        // Corrections are now applied in prepare()
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
        if result.effective_task != task {
            println!(
                "  {} {}",
                "Normalized task:".bold().dimmed(),
                result.effective_task.dimmed()
            );
        }
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Command (dry-run):".bold().yellow());
        println!("  {}", full_cmd.green().bold());
        println!();

        // ── Flag & subcommand validation ──────────────────────────────
        // Prefer schema-based validation (more accurate) over doc-based validation
        if let Some(ref schema) = result.parsed_schema {
            let vr = schema.validate_args(&result.suggestion.args, None);
            for err in &vr.errors {
                eprintln!(
                    "  {} {}",
                    "⚠ Validation:".yellow(),
                    format!("{:?}", err).yellow()
                );
            }
            if !vr.errors.is_empty() {
                println!();
            }
        } else if let Some(ref sdoc) = result.structured_doc {
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
            println!();
            markdown::render_markdown(&result.suggestion.explanation);
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
            if result.effective_task != task {
                println!(
                    "  {} {}",
                    "Normalized task:".bold().dimmed(),
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
                println!();
                markdown::render_markdown(&result.suggestion.explanation);
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

        // ── Execution: Feedback Collector ────────────────────────────────
        let _ = FeedbackCollector::record(FeedbackEntry {
            tool: tool.to_string(),
            task: task.to_string(),
            generated_command: full_cmd.clone(),
            was_modified: false,
            modified_command: None,
            exit_code,
            user_approved: success,
            model: self.config.effective_model(),
            recorded_at: Utc::now().to_rfc3339(),
        });

        // ── Execution: Error Knowledge DB learning ──────────────────────
        if !success {
            let _ = ErrorKnowledgeDb::record(crate::knowledge::error_db::ErrorRecord {
                tool: tool.to_string(),
                task: task.to_string(),
                failed_command: full_cmd.clone(),
                exit_code,
                stderr_snippet: captured_stderr.chars().take(2000).collect(),
                error_category: crate::knowledge::error_db::ErrorCategory::classify(
                    &captured_stderr,
                ),
                resolution: None,
                recorded_at: Utc::now().to_rfc3339(),
            });
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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_runner() -> Runner {
        Runner::new(Config::default())
    }

    #[test]
    fn test_runner_new_defaults() {
        let r = default_runner();
        assert!(!r.verbose);
        assert!(!r.no_cache);
        assert!(!r.verify);
        assert!(!r.auto_retry);
        assert!(!r.stop_on_error);
        assert!(!r.no_stream);
        assert_eq!(r.jobs, 1);
        assert!(r.vars.is_empty());
        assert!(r.input_items.is_empty());
        assert!(r.force_scenario.is_none());
    }

    #[test]
    fn test_runner_builder_flags() {
        let cases: Vec<(&str, Box<dyn Fn(&Runner) -> bool>)> = vec![
            ("verbose=true", Box::new(|r: &Runner| r.verbose)),
            ("no_cache=true", Box::new(|r: &Runner| r.no_cache)),
            ("verify=true", Box::new(|r: &Runner| r.verify)),
            ("auto_retry=true", Box::new(|r: &Runner| r.auto_retry)),
            ("stop_on_error=true", Box::new(|r: &Runner| r.stop_on_error)),
            ("no_skill=true", Box::new(|r: &Runner| r.no_skill)),
            ("no_doc=true", Box::new(|r: &Runner| r.no_doc)),
            ("no_prompt=true", Box::new(|r: &Runner| r.no_prompt)),
            ("no_stream=true", Box::new(|r: &Runner| r.no_stream)),
        ];

        for (label, checker) in &cases {
            let mut r = default_runner();
            match *label {
                "verbose=true" => {
                    r.with_verbose(true);
                }
                "no_cache=true" => {
                    r.with_no_cache(true);
                }
                "verify=true" => {
                    r.with_verify(true);
                }
                "auto_retry=true" => {
                    r.with_auto_retry(true);
                }
                "stop_on_error=true" => {
                    r.with_stop_on_error(true);
                }
                "no_skill=true" => {
                    r.with_no_skill(true);
                }
                "no_doc=true" => {
                    r.with_no_doc(true);
                }
                "no_prompt=true" => {
                    r.with_no_prompt(true);
                }
                "no_stream=true" => {
                    r.with_no_stream(true);
                }
                _ => {}
            }
            assert!(checker(&r), "flag {} should be true", label);
        }
    }

    #[test]
    fn test_runner_with_jobs_clamps_to_one() {
        let cases = vec![(0, 1), (1, 1), (4, 4), (16, 16)];
        for (input, expected) in cases {
            let mut r = default_runner();
            r.with_jobs(input);
            assert_eq!(r.jobs, expected, "jobs({}) should be {}", input, expected);
        }
    }

    #[test]
    fn test_runner_with_vars() {
        let mut r = default_runner();
        let mut vars = HashMap::new();
        vars.insert("key1".to_string(), "val1".to_string());
        vars.insert("key2".to_string(), "val2".to_string());
        r.with_vars(vars.clone());
        assert_eq!(r.vars, vars);
    }

    #[test]
    fn test_runner_with_input_items() {
        let items = vec![
            "a.bam".to_string(),
            "b.bam".to_string(),
            "c.bam".to_string(),
        ];
        let mut r = default_runner();
        r.with_input_items(items.clone());
        assert_eq!(r.input_items, items);
    }

    #[test]
    fn test_runner_with_scenario() {
        use crate::workflow_graph::WorkflowScenario;
        let cases = vec![
            WorkflowScenario::Bare,
            WorkflowScenario::Full,
            WorkflowScenario::Doc,
        ];
        for scenario in cases {
            let mut r = default_runner();
            r.with_scenario(scenario);
            assert!(r.force_scenario.is_some());
        }
    }

    #[test]
    fn test_generated_command_fields() {
        let cmd = GeneratedCommand {
            full_cmd: "samtools sort -o out.bam in.bam".to_string(),
            explanation: "Sort BAM by coordinate".to_string(),
            effective_task: "sort BAM file".to_string(),
        };
        assert!(cmd.full_cmd.contains("samtools"));
        assert!(!cmd.explanation.is_empty());
        assert!(!cmd.effective_task.is_empty());
    }
}
