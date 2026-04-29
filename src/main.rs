//! **oxo-call** — Model-intelligent orchestration for CLI bioinformatics.
//!
//! This binary provides the `oxo-call` command, which uses LLM intelligence to
//! generate, verify, and execute bioinformatics tool invocations.
//!
//! ## Architecture overview
//!
//! The application follows a **docs-first grounding** pattern:
//!
//! 1. **Documentation fetch** — Tool docs are fetched/cached before any LLM call
//!    (`docs` module).
//! 2. **Skill selection** — The best matching skill (built-in, community, or MCP)
//!    is loaded to provide domain-specific guidance (`skill` module).
//! 3. **LLM command generation** — The prompt, enriched with structured docs and
//!    skill context, is sent to a local or remote LLM (`llm` module).
//! 4. **Execution & verification** — The generated command is executed (optionally
//!    with retry logic) and its output is verified (`runner` module).
//!
//! ## Command routing
//!
//! All CLI commands are defined in [`cli`] and dispatched in [`main`].  The
//! primary user-facing commands are:
//!
//! - `run` / `dry-run` — Generate and (optionally) execute tool commands
//! - `docs` — Fetch and manage tool documentation
//! - `skill` — Manage built-in, community, and MCP skills
//! - `workflow` — Manage multi-step DAG workflows (`.oxo.toml`)
//! - `job` — Define and run batch job templates
//! - `history` — Browse command history with provenance
//! - `chat` — Interactive LLM chat mode
//! - `config` — Manage configuration
//! - `server` — Remote server management
//! - `license` — License activation and verification

mod auto_fixer;
mod bench;
mod cache;
mod chat;
mod cli;
mod cli_pattern;
mod command_corrector;
mod command_validator;
mod config;
mod constraint_graph;
mod context;
mod copilot_auth;
mod doc_enhancer;
mod doc_processor;
mod doc_summarizer;
mod docs;
mod engine;
mod error;
mod execution;
mod format;
mod generator;
mod handlers;
mod history;
mod index;
mod job;
mod knowledge;
mod license;
mod llm;
mod llm_workflow;
mod markdown;
mod mcp;
mod mini_skill_cache;
mod orchestrator;
mod rag;
mod reflection_engine;
mod runner;
mod sanitize;
mod server;
mod skill;
mod streaming_display;
mod subcommand_detector;
mod task_complexity;
mod task_normalizer;
mod validation_loop;
mod workflow;
mod workflow_graph;

/// A single crate-wide mutex that **all** test modules must acquire before
/// reading or writing `OXO_CALL_DATA_DIR` (or any other process-global
/// environment variable used by tests).
#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use clap::{CommandFactory, Parser};
use cli::{
    Cli, Commands, ConfigCommands, DocsCommands, HistoryCommands, IndexCommands, JobCommands,
    LicenseCommands, ModelCommands, RunScenario, ServerCommands, ShellType, SkillCommands,
    SkillMcpCommands, WorkflowCommands,
};
use colored::Colorize;
use handlers::{config_verify_suggestions, print_index_table, with_source};

#[tokio::main]
async fn main() {
    // Install color-eyre for enhanced error reporting (backtraces, color output)
    if let Err(e) = error::install_error_handler() {
        eprintln!("warning: failed to install color-eyre handler: {e}");
    }

    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "error:".bold().red(), e);
        std::process::exit(1);
    }
}

/// GitHub Copilot models available for selection during `config login`.
/// Each entry is (model_id, display_description, is_free_tier).
/// Sourced from: https://docs.github.com/en/copilot/reference/ai-models/supported-models
/// Only GA models are listed; models in "Closing down" or "Public preview" status are omitted.
const COPILOT_MODELS: &[(&str, &str, bool)] = &[
    // ── OpenAI ──────────────────────────────────────────────────────────────────
    (
        "gpt-5-mini",
        "GPT-5 Mini         · OpenAI, fast lightweight",
        true,
    ),
    (
        "gpt-4.1",
        "GPT-4.1            · OpenAI, general-purpose",
        false,
    ),
    (
        "gpt-5.2",
        "GPT-5.2            · OpenAI, deep reasoning",
        false,
    ),
    (
        "gpt-5.2-codex",
        "GPT-5.2-Codex      · OpenAI, agentic coding",
        false,
    ),
    (
        "gpt-5.3-codex",
        "GPT-5.3-Codex      · OpenAI, agentic coding",
        false,
    ),
    (
        "gpt-5.4",
        "GPT-5.4            · OpenAI, frontier reasoning",
        false,
    ),
    (
        "gpt-5.4-mini",
        "GPT-5.4 Mini       · OpenAI, agentic lightweight",
        false,
    ),
    // ── Anthropic ───────────────────────────────────────────────────────────────
    (
        "claude-haiku-4.5",
        "Claude Haiku 4.5   · Anthropic, fast",
        false,
    ),
    ("claude-sonnet-4", "Claude Sonnet 4    · Anthropic", false),
    (
        "claude-sonnet-4.5",
        "Claude Sonnet 4.5  · Anthropic, agent tasks",
        false,
    ),
    (
        "claude-sonnet-4.6",
        "Claude Sonnet 4.6  · Anthropic, newest sonnet",
        false,
    ),
    (
        "claude-opus-4.5",
        "Claude Opus 4.5    · Anthropic, most capable",
        false,
    ),
    (
        "claude-opus-4.6",
        "Claude Opus 4.6    · Anthropic, newest opus",
        false,
    ),
    // ── Google ──────────────────────────────────────────────────────────────────
    (
        "gemini-2.5-pro",
        "Gemini 2.5 Pro     · Google, deep reasoning",
        false,
    ),
];

async fn run(cli: Cli) -> error::Result<()> {
    // Commands that are permitted without a valid license file.
    // `--help` and `--version` are handled by clap before reaching this function.
    let license_exempt = matches!(
        cli.command,
        Commands::License { .. } | Commands::Completion { .. }
    );

    if !license_exempt {
        let license_path = cli.license.as_deref();
        if let Err(e) = license::load_and_verify(license_path) {
            eprintln!("{} {}", "license error:".bold().red(), e);
            std::process::exit(2);
        }
    }

    // Load config once at startup instead of per-command to avoid multiple disk reads.
    // License-exempt commands use default config (no file read needed).
    let base_cfg = if license_exempt {
        config::Config::default()
    } else {
        config::Config::load().await?
    };

    let verbose = cli.verbose;

    match cli.command {
        Commands::Chat {
            tool,
            question,
            interactive,
            model,
            no_cache,
            scenario,
            json,
            no_stream,
        } => {
            let mut cfg = base_cfg.clone();
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }
            if no_stream {
                cfg.llm.stream = false;
            }

            let mut chat_session = chat::ChatSession::new(cfg)
                .with_verbose(verbose)
                .with_no_cache(no_cache)
                .with_scenario(scenario);

            if interactive {
                chat_session.run_interactive(tool.as_deref()).await?;
            } else {
                match (tool, question) {
                    (Some(tool), Some(question)) => {
                        chat_session.run_single(&tool, &question, json).await?;
                    }
                    (Some(question), None) => {
                        // Single positional arg: treat as a general question (no tool context).
                        chat_session.run_single_general(&question, json).await?;
                    }
                    _ => {
                        return Err(error::OxoError::ConfigError(
                            "Non-interactive chat requires a question. \
                             Usage: oxo-call chat <question> or oxo-call chat <tool> <question>"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        Commands::Run {
            tool,
            task,
            ask,
            model,
            no_cache,
            json,
            verify,
            vars,
            input_list,
            input_items,
            jobs,
            stop_on_error,
            auto_retry,
            scenario,
            no_stream,
            no_skill: cli_no_skill,
            no_doc: cli_no_doc,
            no_prompt: cli_no_prompt,
        } => {
            let mut cfg = base_cfg.clone();
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }

            // Convert validated scenario enum to workflow scenario
            // and set ablation flags based on scenario OR CLI options
            // CLI options override scenario defaults when explicitly set
            let (force_scenario, no_skill, no_doc, no_prompt) = match scenario {
                Some(RunScenario::Bare) => (
                    Some(workflow_graph::WorkflowScenario::Bare),
                    true, // no_skill
                    true, // no_doc
                    true, // no_prompt
                ),
                Some(RunScenario::Prompt) => (
                    Some(workflow_graph::WorkflowScenario::Prompt),
                    true,  // no_skill
                    true,  // no_doc
                    false, // no_prompt
                ),
                Some(RunScenario::Doc) => (
                    Some(workflow_graph::WorkflowScenario::Doc),
                    true,  // no_skill
                    false, // no_doc
                    false, // no_prompt
                ),
                Some(RunScenario::Skill) => (
                    Some(workflow_graph::WorkflowScenario::Skill),
                    false, // no_skill
                    true,  // no_doc
                    false, // no_prompt
                ),
                Some(RunScenario::Full) => (
                    Some(workflow_graph::WorkflowScenario::Full),
                    false, // no_skill
                    false, // no_doc
                    false, // no_prompt
                ),
                None => (None, false, false, false),
            };

            // CLI ablation options override scenario-based settings
            let no_skill = no_skill || cli_no_skill;
            let no_doc = no_doc || cli_no_doc;
            let no_prompt = no_prompt || cli_no_prompt;

            // Collect input items from --input-list / --input-items.
            let all_items = {
                let mut v: Vec<String> = Vec::new();
                if let Some(ref path) = input_list {
                    v.extend(job::read_input_list(path)?);
                }
                if let Some(ref items_str) = input_items {
                    v.extend(
                        items_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()),
                    );
                }
                v
            };

            // Parse --var KEY=VALUE pairs.
            let var_map = {
                let mut m = std::collections::HashMap::new();
                for v in &vars {
                    let (k, val) = job::parse_var(v)?;
                    m.insert(k, val);
                }
                m
            };

            let mut runner = runner::Runner::new(cfg);
            runner
                .with_verbose(verbose)
                .with_no_cache(no_cache)
                .with_no_skill(no_skill)
                .with_no_doc(no_doc)
                .with_no_prompt(no_prompt)
                .with_verify(verify)
                .with_auto_retry(auto_retry)
                .with_no_stream(no_stream);
            if let Some(sc) = force_scenario {
                runner.with_scenario(sc);
            }
            runner
                .with_vars(var_map)
                .with_input_items(all_items)
                .with_jobs(jobs)
                .with_stop_on_error(stop_on_error);
            runner.run(&tool, &task, ask, json).await?;
        }

        Commands::DryRun {
            tool,
            task,
            model,
            no_cache,
            json,
            no_skill,
            no_doc,
            no_prompt,
            vars,
            input_list,
            input_items,
            jobs,
            stop_on_error,
            scenario,
            no_stream,
        } => {
            let mut cfg = base_cfg.clone();
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }

            // Convert validated scenario enum to workflow scenario
            // and set ablation flags based on scenario
            let (force_scenario, no_skill, no_doc, no_prompt) = match scenario {
                Some(RunScenario::Bare) => (
                    Some(workflow_graph::WorkflowScenario::Bare),
                    true,
                    true,
                    true,
                ),
                Some(RunScenario::Prompt) => (
                    Some(workflow_graph::WorkflowScenario::Prompt),
                    true,
                    true,
                    false,
                ),
                Some(RunScenario::Doc) => (
                    Some(workflow_graph::WorkflowScenario::Doc),
                    true,
                    false,
                    false,
                ),
                Some(RunScenario::Skill) => (
                    Some(workflow_graph::WorkflowScenario::Skill),
                    false,
                    true,
                    false,
                ),
                Some(RunScenario::Full) => (
                    Some(workflow_graph::WorkflowScenario::Full),
                    false,
                    false,
                    false,
                ),
                None => (None, no_skill, no_doc, no_prompt),
            };

            let all_items = {
                let mut v: Vec<String> = Vec::new();
                if let Some(ref path) = input_list {
                    v.extend(job::read_input_list(path)?);
                }
                if let Some(ref items_str) = input_items {
                    v.extend(
                        items_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()),
                    );
                }
                v
            };

            let var_map = {
                let mut m = std::collections::HashMap::new();
                for v in &vars {
                    let (k, val) = job::parse_var(v)?;
                    m.insert(k, val);
                }
                m
            };

            let mut runner = runner::Runner::new(cfg);
            runner
                .with_verbose(verbose)
                .with_no_cache(no_cache)
                .with_no_skill(no_skill)
                .with_no_doc(no_doc)
                .with_no_prompt(no_prompt)
                .with_no_stream(no_stream)
                .with_jobs(jobs)
                .with_stop_on_error(stop_on_error);
            if let Some(sc) = force_scenario {
                runner.with_scenario(sc);
            }
            let runner = runner.with_vars(var_map).with_input_items(all_items);
            runner.dry_run(&tool, &task, json, None).await?;
        }

        Commands::Index { command } => match command {
            IndexCommands::Add {
                tool,
                url,
                file,
                dir,
            } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                let entry = mgr
                    .add(&tool, url.as_deref(), file.as_deref(), dir.as_deref())
                    .await?;
                println!(
                    "{} Indexed '{}' ({} bytes from: {})",
                    "✓".green().bold(),
                    entry.tool_name.cyan(),
                    entry.doc_size_bytes,
                    entry.sources.join(", ")
                );
                if let Some(version) = entry.version {
                    println!("  {} {}", "Version:".bold(), version);
                }
            }
            IndexCommands::Remove { tool } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                mgr.remove(&tool)?;
                println!(
                    "{} Removed '{}' from index",
                    "✓".green().bold(),
                    tool.cyan()
                );
            }
            IndexCommands::Update { tool, url } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                match tool {
                    Some(t) => {
                        let entry = mgr.add(&t, url.as_deref(), None, None).await?;
                        println!(
                            "{} Updated '{}' ({} bytes)",
                            "✓".green().bold(),
                            entry.tool_name.cyan(),
                            entry.doc_size_bytes
                        );
                    }
                    None => {
                        // Update all indexed tools
                        let entries = mgr.list()?;
                        if entries.is_empty() {
                            println!(
                                "{}",
                                "No tools in index. Use 'oxo-call docs add <tool>' first.".yellow()
                            );
                            return Ok(());
                        }
                        let tools: Vec<String> =
                            entries.iter().map(|e| e.tool_name.clone()).collect();
                        println!("Updating {} tool(s)...", tools.len());
                        for t in &tools {
                            match mgr.add(t, None, None, None).await {
                                Ok(e) => {
                                    println!("  {} '{}'", "✓".green().bold(), e.tool_name.cyan())
                                }
                                Err(e) => println!("  {} '{}': {}", "✗".red().bold(), t.cyan(), e),
                            }
                        }
                    }
                }
            }
            IndexCommands::List => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                let entries = mgr.list()?;
                if entries.is_empty() {
                    println!(
                        "{}",
                        "No tools indexed yet. Use 'oxo-call docs add <tool>' to index a tool."
                            .yellow()
                    );
                    return Ok(());
                }
                print_index_table(&entries);
            }
        },

        Commands::Docs { command } => match command {
            DocsCommands::Add {
                tool,
                url,
                file,
                dir,
            } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                let entry = mgr
                    .add(&tool, url.as_deref(), file.as_deref(), dir.as_deref())
                    .await?;
                println!(
                    "{} Indexed '{}' ({} bytes from: {})",
                    "✓".green().bold(),
                    entry.tool_name.cyan(),
                    entry.doc_size_bytes,
                    entry.sources.join(", ")
                );
                if let Some(version) = entry.version {
                    println!("  {} {}", "Version:".bold(), version);
                }
            }
            DocsCommands::Remove { tool } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                mgr.remove(&tool)?;
                println!(
                    "{} Removed '{}' from documentation index",
                    "✓".green().bold(),
                    tool.cyan()
                );
            }
            DocsCommands::Update { tool, url } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                match tool {
                    Some(t) => {
                        let entry = mgr.add(&t, url.as_deref(), None, None).await?;
                        println!(
                            "{} Updated '{}' ({} bytes)",
                            "✓".green().bold(),
                            entry.tool_name.cyan(),
                            entry.doc_size_bytes
                        );
                    }
                    None => {
                        let entries = mgr.list()?;
                        if entries.is_empty() {
                            println!(
                                "{}",
                                "No tools indexed yet. Use 'oxo-call docs add <tool>' first."
                                    .yellow()
                            );
                            return Ok(());
                        }
                        let tools: Vec<String> =
                            entries.iter().map(|e| e.tool_name.clone()).collect();
                        println!("Updating {} tool(s)...", tools.len());
                        for t in &tools {
                            match mgr.add(t, None, None, None).await {
                                Ok(e) => {
                                    println!("  {} '{}'", "✓".green().bold(), e.tool_name.cyan())
                                }
                                Err(e) => println!("  {} '{}': {}", "✗".red().bold(), t.cyan(), e),
                            }
                        }
                    }
                }
            }
            DocsCommands::List => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                let entries = mgr.list()?;
                if entries.is_empty() {
                    println!(
                        "{}",
                        "No tools indexed yet. Use 'oxo-call docs add <tool>' to index a tool."
                            .yellow()
                    );
                    return Ok(());
                }
                print_index_table(&entries);
            }
            DocsCommands::Show { tool } => {
                let cfg = base_cfg.clone();
                let fetcher = docs::DocsFetcher::new(cfg);
                let cache_path = fetcher.cache_path(&tool)?;
                if cache_path.exists() {
                    let content = std::fs::read_to_string(&cache_path)?;
                    println!("{}", content);
                } else {
                    // Fall back to live --help
                    let docs = fetcher.fetch(&tool).await?;
                    if let Some(help) = docs.help_output {
                        println!("{}", help);
                    }
                }
            }
            DocsCommands::Fetch { tool, url } => {
                let cfg = base_cfg.clone();
                let mgr = index::IndexManager::new(cfg);
                let spinner =
                    runner::make_spinner(&format!("Fetching remote documentation for '{tool}'..."));
                let entry = match mgr.add(&tool, Some(&url), None, None).await {
                    Ok(e) => {
                        spinner.finish_and_clear();
                        e
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
                        return Err(e);
                    }
                };
                println!(
                    "{} Saved {} bytes of documentation for '{}'",
                    "✓".green().bold(),
                    entry.doc_size_bytes,
                    tool.cyan()
                );
            }
            DocsCommands::Path { tool } => {
                let cfg = base_cfg.clone();
                let fetcher = docs::DocsFetcher::new(cfg);
                let path = fetcher.cache_path(&tool)?;
                println!("{}", path.display());
            }
        },

        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => {
                let mut cfg = base_cfg.clone();
                // Capture old provider before applying the change so we can
                // give context-aware hints when the user switches providers.
                let old_provider = cfg.effective_provider();
                cfg.set(&key, &value)?;
                cfg.save()?;
                println!("{} Set '{}' = '{}'", "✓".green().bold(), key.cyan(), value);

                // Provider-switch guidance
                if key == "llm.provider" && value != old_provider {
                    let has_token = cfg.llm.api_token.as_ref().is_some_and(|t| !t.is_empty());
                    if value == "ollama" && has_token {
                        eprintln!(
                            "\n{} Switched to Ollama (local inference, no API token needed).\n  \
                             Your existing API token is still stored in the config but will be ignored.\n  \
                             To clean it up: {}",
                            "hint:".bold().cyan(),
                            "oxo-call config set llm.api_token \"\"".dimmed()
                        );
                    } else if value != "ollama" && !has_token {
                        eprintln!(
                            "\n{} Provider '{}' requires an API token.\n  \
                             Set one with: {}",
                            "hint:".bold().cyan(),
                            value,
                            "oxo-call config set llm.api_token <your-token>".dimmed()
                        );
                    }
                }
            }
            ConfigCommands::Get { key } => {
                let cfg = base_cfg.clone();
                let value = cfg.get(&key)?;
                println!("{}", value);
            }
            ConfigCommands::Show => {
                let cfg = base_cfg.clone();
                let path = config::Config::config_path()?;

                println!("{}", "oxo-call configuration".bold());
                println!("{}", "─".repeat(50).dimmed());
                println!(
                    "  {} {}",
                    "Config file:".bold(),
                    path.display().to_string().dimmed()
                );
                println!();
                println!(
                    "  {} ",
                    "Stored values (config.toml / built-in defaults):".bold()
                );
                println!("  {} ", "[llm]".bold().cyan());
                println!("  {:<25} {}", "provider", cfg.llm.provider);
                println!(
                    "  {:<25} {}",
                    "api_token",
                    cfg.llm
                        .api_token
                        .as_deref()
                        .map(|t| if t.is_empty() { "(not set)" } else { "***" })
                        .unwrap_or("(not set)")
                );
                println!(
                    "  {:<25} {}",
                    "api_base",
                    cfg.llm.api_base.as_deref().unwrap_or("(default)")
                );
                println!(
                    "  {:<25} {}",
                    "model",
                    cfg.llm.model.as_deref().unwrap_or("(default)")
                );
                println!("  {:<25} {}", "max_tokens", cfg.llm.max_tokens);
                println!("  {:<25} {}", "temperature", cfg.llm.temperature);
                println!("  {:<25} {}", "stream", cfg.llm.stream);
                println!(
                    "  {:<25} {}",
                    "models",
                    if cfg.llm.models.is_empty() {
                        "(none)".to_string()
                    } else {
                        cfg.llm.models.join(", ")
                    }
                );
                println!();
                println!("  {} ", "[docs]".bold().cyan());
                println!("  {:<25} {}", "auto_update", cfg.docs.auto_update);
                println!(
                    "  {:<25} {}",
                    "local_paths",
                    if cfg.docs.local_paths.is_empty() {
                        "(none)".to_string()
                    } else {
                        cfg.docs
                            .local_paths
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                );
                println!();
                println!(
                    "  {} ",
                    "Effective values (after env overrides / provider defaults):".bold()
                );
                let token_status = if cfg.effective_api_token().is_some() {
                    format!(
                        "{} [{}]",
                        "set".green(),
                        cfg.effective_source("llm.api_token")?.dimmed()
                    )
                } else if cfg.provider_requires_token() {
                    format!(
                        "{} [{}]",
                        "not set".red(),
                        cfg.effective_source("llm.api_token")?.dimmed()
                    )
                } else {
                    format!(
                        "{} [{}]",
                        "not required".dimmed(),
                        cfg.effective_source("llm.api_token")?.dimmed()
                    )
                };
                println!(
                    "  {:<25} {}",
                    "provider",
                    with_source(
                        &cfg.effective_provider(),
                        &cfg.effective_source("llm.provider")?
                    )
                );
                println!("  {:<25} {}", "api_token", token_status);
                println!(
                    "  {:<25} {}",
                    "api_base",
                    with_source(
                        &cfg.effective_api_base(),
                        &cfg.effective_source("llm.api_base")?
                    )
                );
                println!(
                    "  {:<25} {}",
                    "model",
                    with_source(&cfg.effective_model(), &cfg.effective_source("llm.model")?)
                );
                println!(
                    "  {:<25} {}",
                    "max_tokens",
                    with_source(
                        &cfg.effective_max_tokens()?.to_string(),
                        &cfg.effective_source("llm.max_tokens")?
                    )
                );
                println!(
                    "  {:<25} {}",
                    "temperature",
                    with_source(
                        &cfg.effective_temperature()?.to_string(),
                        &cfg.effective_source("llm.temperature")?
                    )
                );
                println!(
                    "  {:<25} {}",
                    "auto_update",
                    with_source(
                        &cfg.effective_docs_auto_update()?.to_string(),
                        &cfg.effective_source("docs.auto_update")?
                    )
                );
                println!(
                    "  {:<25} {}",
                    "stream",
                    with_source(
                        &cfg.llm.stream.to_string(),
                        &cfg.effective_source("llm.stream")?
                    )
                );
            }
            ConfigCommands::Verify { verbose } => {
                let cfg = base_cfg.clone();
                let client = llm::LlmClient::new(cfg.clone());

                println!("{}", "Verifying LLM configuration...".bold());
                println!("{}", "─".repeat(50).dimmed());
                println!(
                    "  {:<18} {}",
                    "provider",
                    with_source(
                        &cfg.effective_provider(),
                        &cfg.effective_source("llm.provider")?
                    )
                );
                println!(
                    "  {:<18} {}",
                    "api_base",
                    with_source(
                        &cfg.effective_api_base(),
                        &cfg.effective_source("llm.api_base")?
                    )
                );
                println!(
                    "  {:<18} {}",
                    "model",
                    with_source(&cfg.effective_model(), &cfg.effective_source("llm.model")?)
                );
                println!(
                    "  {:<18} {}",
                    "max_tokens",
                    with_source(
                        &cfg.effective_max_tokens()?.to_string(),
                        &cfg.effective_source("llm.max_tokens")?
                    )
                );
                println!(
                    "  {:<18} {}",
                    "temperature",
                    with_source(
                        &cfg.effective_temperature()?.to_string(),
                        &cfg.effective_source("llm.temperature")?
                    )
                );
                println!(
                    "  {:<18} {}",
                    "api_token",
                    if cfg.effective_api_token().is_some() {
                        format!(
                            "{} [{}]",
                            "set".green(),
                            cfg.effective_source("llm.api_token")?.dimmed()
                        )
                    } else if cfg.provider_requires_token() {
                        format!(
                            "{} [{}]",
                            "not set".red(),
                            cfg.effective_source("llm.api_token")?.dimmed()
                        )
                    } else {
                        format!(
                            "{} [{}]",
                            "not required".dimmed(),
                            cfg.effective_source("llm.api_token")?.dimmed()
                        )
                    }
                );
                println!();

                match client.verify_configuration().await {
                    Ok(result) => {
                        println!("{} LLM configuration is valid", "✓".green().bold());
                        println!("  {:<18} {}", "provider", result.provider);
                        println!("  {:<18} {}", "api_base", result.api_base);
                        println!("  {:<18} {}", "model", result.model);
                        println!(
                            "  {:<18} {}",
                            "response",
                            if result.response_preview.is_empty() {
                                "(empty response)".yellow().to_string()
                            } else {
                                result.response_preview
                            }
                        );
                    }
                    Err(error::OxoError::LlmError(message)) => {
                        eprintln!("{} {}", "configuration check failed:".bold().red(), message);
                        if verbose {
                            eprintln!();
                            eprintln!("{}", "Raw error detail:".bold());
                            eprintln!("  {message}");
                        }
                        for suggestion in config_verify_suggestions(&cfg, &message) {
                            eprintln!("  - {}", suggestion);
                        }
                        std::process::exit(1);
                    }
                    Err(e) => return Err(e),
                }
            }
            ConfigCommands::Path => {
                let path = config::Config::config_path()?;
                println!("{}", path.display());
            }
            ConfigCommands::Login { provider } => {
                match provider.as_str() {
                    "github-copilot" => {
                        println!("{}", "Authenticating with GitHub Copilot...".bold());
                        println!();

                        // Use the built-in Copilot CLI GitHub App client ID
                        // This produces ghu_ tokens that work with the Copilot internal API
                        println!("  Starting GitHub OAuth device-authorization flow…");
                        println!("  (Using Copilot CLI's GitHub App for compatibility)");
                        println!();

                        let manager = copilot_auth::get_token_manager();
                        let github_token = manager.run_device_flow().await?;

                        // Verify the token works by exchanging for a Copilot session token
                        println!();
                        println!("  Verifying token...");
                        match manager.exchange_token(&github_token).await {
                            Ok(copilot_token) => {
                                println!("  {} Token verified successfully!", "✓".green());
                                println!(
                                    "  Session token expires in {} seconds.",
                                    copilot_token.refresh_in
                                );
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} Token verification failed: {e}",
                                    "error:".bold().red()
                                );
                                eprintln!();
                                eprintln!("  This usually means:");
                                eprintln!("    1. You don't have a GitHub Copilot subscription");
                                eprintln!(
                                    "    2. Your organization hasn't enabled Copilot for you"
                                );
                                eprintln!();
                                eprintln!(
                                    "  Visit https://github.com/settings/copilot to check your subscription."
                                );
                                std::process::exit(1);
                            }
                        }

                        // --- Interactive model selection ---
                        println!();
                        println!("  {}", "Select a GitHub Copilot model:".bold());
                        println!();
                        for (i, (id, desc, is_free)) in COPILOT_MODELS.iter().enumerate() {
                            let free_tag = if *is_free {
                                format!(" {}", "[free tier ⭐]".green())
                            } else {
                                String::new()
                            };
                            let default_tag = if i == 0 {
                                format!(" {}", "[default]".dimmed())
                            } else {
                                String::new()
                            };
                            println!(
                                "    {}. {}{}{}",
                                (i + 1).to_string().bold(),
                                desc,
                                free_tag,
                                default_tag
                            );
                            // Print the model id indented under the description
                            println!("       {}", id.dimmed());
                        }
                        println!();
                        println!(
                            "  💡 {}",
                            "Free-tier models (⭐) work on all GitHub Copilot plans.".dimmed()
                        );
                        println!();

                        use std::io::IsTerminal as _;
                        let selected_model = if std::io::stdin().is_terminal() {
                            use std::io::Write as _;
                            print!(
                                "  Enter number [1–{}], or press {} for default ({}): ",
                                COPILOT_MODELS.len(),
                                "Enter".bold(),
                                COPILOT_MODELS[0].0.green()
                            );
                            std::io::stdout().flush().ok();
                            let mut sel = String::new();
                            std::io::stdin().read_line(&mut sel).ok();
                            let sel = sel.trim();
                            if sel.is_empty() {
                                COPILOT_MODELS[0].0.to_string()
                            } else if let Ok(n) = sel.parse::<usize>() {
                                if n >= 1 && n <= COPILOT_MODELS.len() {
                                    COPILOT_MODELS[n - 1].0.to_string()
                                } else {
                                    println!(
                                        "  {} Invalid number, using default ({}).",
                                        "⚠".yellow(),
                                        COPILOT_MODELS[0].0
                                    );
                                    COPILOT_MODELS[0].0.to_string()
                                }
                            } else {
                                // User typed a raw model name
                                sel.to_string()
                            }
                        } else {
                            // Non-interactive (piped/script), use default silently
                            COPILOT_MODELS[0].0.to_string()
                        };

                        let mut cfg = base_cfg.clone();
                        cfg.llm.provider = "github-copilot".to_string();
                        cfg.llm.api_token = Some(github_token);
                        cfg.llm.model = Some(selected_model.clone());
                        // Pre-populate the model list with all available Copilot models
                        // so the user can switch quickly with `config model use`.
                        for (id, _, _) in COPILOT_MODELS {
                            if !cfg.llm.models.contains(&id.to_string()) {
                                cfg.llm.models.push(id.to_string());
                            }
                        }
                        cfg.save()?;

                        println!();
                        println!("{} Authenticated with GitHub Copilot.", "✓".green().bold());
                        println!("  provider  github-copilot");
                        println!(
                            "  model     {} (switch with `oxo-call config model use <model>`)",
                            selected_model
                        );
                        println!();
                        println!("  Run `oxo-call config verify` to confirm everything works.");
                        println!("  Run `oxo-call config model list` to see available models.");
                    }
                    other => {
                        eprintln!(
                            "{} Interactive login is not supported for provider `{other}`.",
                            "error:".bold().red()
                        );
                        eprintln!();
                        eprintln!("Set the token manually:");
                        eprintln!("  oxo-call config set llm.provider {other}");
                        eprintln!("  oxo-call config set llm.api_token <your-token>");
                        std::process::exit(1);
                    }
                }
            }
            ConfigCommands::Model { command } => {
                let mut cfg = base_cfg.clone();
                match command {
                    ModelCommands::List => {
                        let active = cfg.effective_model();
                        if cfg.llm.models.is_empty() {
                            println!("{}", "No models configured.".yellow());
                            println!();
                            println!("  Add models with `oxo-call config model add <model-id>`");
                            println!(
                                "  or run `oxo-call config login` to populate the list automatically."
                            );
                        } else {
                            println!("{}", "Configured models:".bold());
                            println!();
                            for m in &cfg.llm.models {
                                if m == &active {
                                    println!("  {} {}", "★".yellow().bold(), m.green().bold());
                                } else {
                                    println!("    {}", m);
                                }
                            }
                            println!();
                            println!(
                                "  {} {} (active model — switch with `oxo-call config model use <model>`)",
                                "★".yellow(),
                                active.green()
                            );
                        }
                    }
                    ModelCommands::Add { model } => {
                        if cfg.llm.models.contains(&model) {
                            println!("{} Model '{}' is already in the list.", "⚠".yellow(), model);
                        } else {
                            cfg.llm.models.push(model.clone());
                            cfg.save()?;
                            println!("{} Added '{}' to model list.", "✓".green().bold(), model);
                        }
                    }
                    ModelCommands::Remove { model } => {
                        let before = cfg.llm.models.len();
                        cfg.llm.models.retain(|m| m != &model);
                        if cfg.llm.models.len() == before {
                            eprintln!(
                                "{} Model '{}' not found in list.",
                                "error:".bold().red(),
                                model
                            );
                            std::process::exit(1);
                        }
                        cfg.save()?;
                        println!(
                            "{} Removed '{}' from model list.",
                            "✓".green().bold(),
                            model
                        );
                    }
                    ModelCommands::Use { model } => {
                        cfg.llm.model = Some(model.clone());
                        cfg.save()?;
                        println!(
                            "{} Active model set to '{}'.",
                            "✓".green().bold(),
                            model.green()
                        );
                        println!("  Run `oxo-call config verify` to confirm the model works.");
                    }
                }
            }
        },

        Commands::History { command } => match command {
            HistoryCommands::List { limit, tool } => {
                let mut entries = history::HistoryStore::load_all()?;
                if let Some(t) = &tool {
                    entries.retain(|e| e.tool == *t);
                }
                // Most recent first
                entries.reverse();
                entries.truncate(limit);

                if entries.is_empty() {
                    println!("{}", "No history found.".yellow());
                    return Ok(());
                }

                println!(
                    "{:<10} {:<14} {:<6} {:<18} {:<26} {}",
                    "Status".bold(),
                    "Tool".bold(),
                    "Exit".bold(),
                    "Server".bold(),
                    "Time".bold(),
                    "Command".bold()
                );
                println!("{}", "─".repeat(110).dimmed());
                for e in &entries {
                    let status = if e.dry_run {
                        "dry-run".yellow().to_string()
                    } else if e.exit_code == 0 {
                        "ok".green().to_string()
                    } else {
                        "failed".red().to_string()
                    };
                    let server_col = e
                        .server
                        .as_deref()
                        .map(|s| s.cyan().to_string())
                        .unwrap_or_else(|| "—".dimmed().to_string());
                    let cmd_short = if e.command.len() > 40 {
                        // Safe UTF-8 truncation
                        let safe_end = e
                            .command
                            .char_indices()
                            .nth(40)
                            .map(|(i, _)| i)
                            .unwrap_or(e.command.len());
                        format!("{}...", &e.command[..safe_end])
                    } else {
                        e.command.clone()
                    };
                    println!(
                        "{:<10} {:<14} {:<6} {:<18} {:<26} {}",
                        status,
                        e.tool.cyan().to_string(),
                        e.exit_code,
                        server_col,
                        e.executed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                        cmd_short
                    );
                    // Show provenance info when available
                    if let Some(ref prov) = e.provenance {
                        let mut parts = Vec::new();
                        if let Some(ref v) = prov.tool_version {
                            parts.push(format!("ver={v}"));
                        }
                        if let Some(ref m) = prov.model {
                            parts.push(format!("model={m}"));
                        }
                        if let Some(ref s) = prov.skill_name {
                            parts.push(format!("skill={s}"));
                        }
                        if let Some(ref h) = prov.docs_hash {
                            parts.push(format!("docs={}", &h[..8.min(h.len())]));
                        }
                        if !parts.is_empty() {
                            println!("           {}", format!("[{}]", parts.join(", ")).dimmed());
                        }
                    }
                }
            }
            HistoryCommands::Clear { yes } => {
                if !yes {
                    print!("{} [y/N] ", "Clear all history?".bold().yellow());
                    use std::io::{self, Write};
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
                    if input.trim().to_lowercase() != "y" {
                        println!("{}", "Aborted.".red());
                        return Ok(());
                    }
                }
                history::HistoryStore::clear()?;
                println!("{} History cleared.", "✓".green().bold());
            }
        },

        Commands::Skill { command } => {
            let mut cfg = base_cfg.clone();
            let mgr = skill::SkillManager::new(cfg.clone());

            match command {
                SkillCommands::List => {
                    let skills = mgr.list_all_async().await;
                    if skills.is_empty() {
                        println!("{}", "No skills found.".yellow());
                        return Ok(());
                    }
                    println!(
                        "{:<20} {:<16} {}",
                        "Tool".bold(),
                        "Source".bold(),
                        "Description".bold()
                    );
                    println!("{}", "─".repeat(75).dimmed());
                    for (name, source) in &skills {
                        // For local/built-in skills, load description synchronously.
                        // For MCP skills (source starts with "mcp:"), skip the extra
                        // HTTP round-trip and show the server label instead.
                        let desc = if source.starts_with("mcp:") {
                            format!("[{}]", source.trim_start_matches("mcp:"))
                        } else {
                            mgr.load(name)
                                .map(|s| s.meta.description)
                                .unwrap_or_default()
                        };
                        let source_colored = match source.as_str() {
                            "built-in" => source.dimmed().to_string(),
                            "community" => source.cyan().to_string(),
                            "user" => source.green().bold().to_string(),
                            s if s.starts_with("mcp:") => source.yellow().to_string(),
                            _ => source.clone(),
                        };
                        println!("{:<20} {:<16} {}", name.cyan(), source_colored, desc);
                    }
                    println!(
                        "\n{} skills available. Use 'oxo-call skill show <tool>' to inspect one.",
                        skills.len()
                    );
                }

                SkillCommands::Show { tool } => {
                    if let Some(skill) = mgr.load_async(&tool).await {
                        println!("{}", format!("# Skill: {}", skill.meta.name).bold());
                        println!("Category: {}", skill.meta.category.cyan());
                        println!("Description: {}", skill.meta.description);
                        if !skill.meta.tags.is_empty() {
                            println!("Tags: {}", skill.meta.tags.join(", ").dimmed());
                        }
                        if let Some(url) = &skill.meta.source_url {
                            println!("Source: {}", url.dimmed());
                        }
                        println!();
                        print!("{}", skill.to_prompt_section());
                    } else {
                        println!(
                            "{} No skill found for '{}'. Use 'oxo-call skill install {}' to install one.",
                            "ℹ".blue().bold(),
                            tool.cyan(),
                            tool
                        );
                    }
                }

                SkillCommands::Install { tool, url } => {
                    let spinner =
                        runner::make_spinner(&format!("Installing skill for '{tool}'..."));
                    let result = match url {
                        Some(u) => mgr.install_from_url(&tool, &u).await,
                        None => mgr.install_from_registry(&tool).await,
                    };
                    spinner.finish_and_clear();
                    match result {
                        Ok(skill) => println!(
                            "{} Installed skill for '{}' ({})",
                            "✓".green().bold(),
                            tool.cyan(),
                            skill.meta.category
                        ),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }

                SkillCommands::Remove { tool } => {
                    mgr.remove(&tool)?;
                    println!("{} Removed skill for '{}'", "✓".green().bold(), tool.cyan());
                }

                SkillCommands::Create { tool, output, llm } => {
                    let template = if llm {
                        let llm_client = llm::LlmClient::new(cfg.clone());
                        // Only show spinner for non-streaming mode.
                        let spinner = if !cfg.llm.stream {
                            Some(runner::make_spinner(&format!(
                                "Generating skill for '{tool}' using skill-generator workflow..."
                            )))
                        } else {
                            None
                        };
                        // Use enhanced generation with skill-generator integration
                        match llm_client.generate_skill_template_enhanced(&tool).await {
                            Ok(generated) => {
                                if let Some(sp) = spinner {
                                    sp.finish_and_clear();
                                }
                                println!(
                                    "{} Generated skill for '{}' using skill-generator workflow",
                                    "✓".green().bold(),
                                    tool.cyan()
                                );
                                generated
                            }
                            Err(e) => {
                                if let Some(sp) = spinner {
                                    sp.finish_and_clear();
                                }
                                eprintln!(
                                    "{} Enhanced generation failed ({}), trying basic template",
                                    "!".yellow().bold(),
                                    e
                                );
                                // Fallback to basic template
                                match llm_client.generate_skill_template(&tool).await {
                                    Ok(basic) => {
                                        println!(
                                            "{} Generated basic skill template for '{}'",
                                            "✓".green().bold(),
                                            tool.cyan()
                                        );
                                        basic
                                    }
                                    Err(_) => {
                                        eprintln!(
                                            "{} LLM generation failed (network/API unavailable), using skeleton template",
                                            "!".yellow().bold()
                                        );
                                        skill::SkillManager::create_template(&tool)
                                    }
                                }
                            }
                        }
                    } else {
                        skill::SkillManager::create_template(&tool)
                    };
                    match output {
                        Some(path) => {
                            std::fs::write(&path, &template)?;
                            println!(
                                "{} Template written to '{}'",
                                "✓".green().bold(),
                                path.display()
                            );
                            // Add reference hint for skill registry
                            println!(
                                "\n{} Reference: Browse existing skills at https://github.com/Traitome/oxo-call/tree/main/skills",
                                "💡".cyan()
                            );
                        }
                        None => {
                            print!("{template}");
                            // Add reference hint for skill registry (only for --llm or when no output)
                            if llm {
                                eprintln!(
                                    "\n{} Reference: Browse existing skills at https://github.com/Traitome/oxo-call/tree/main/skills",
                                    "💡".cyan()
                                );
                            }
                        }
                    }
                }

                SkillCommands::Verify { tool, no_llm } => {
                    // ── Structural validation ──────────────────────────────
                    let skill_opt = mgr.load_async(&tool).await;
                    match skill_opt {
                        None => {
                            println!(
                                "{} No skill found for '{}'. Use 'oxo-call skill install {}' or 'oxo-call skill create {}' first.",
                                "✗".red().bold(),
                                tool.cyan(),
                                tool,
                                tool
                            );
                        }
                        Some(ref s) => {
                            println!(
                                "{} Skill: {}  ({})",
                                "→".blue().bold(),
                                s.meta.name.cyan().bold(),
                                s.meta.category.dimmed()
                            );
                            println!();

                            // Structural depth check
                            let depth_issues = skill::validate_skill_depth(s);
                            if depth_issues.is_empty() {
                                println!(
                                    "{}  Structural check: {} ({} concepts, {} pitfalls, {} examples)",
                                    "✓".green().bold(),
                                    "PASS".green(),
                                    s.context.concepts.len(),
                                    s.context.pitfalls.len(),
                                    s.examples.len()
                                );
                            } else {
                                println!(
                                    "{}  Structural check: {}",
                                    "✗".red().bold(),
                                    "FAIL".red()
                                );
                                for issue in &depth_issues {
                                    println!("    {} {}", "·".red(), issue);
                                }
                            }

                            // Optional LLM review
                            if !no_llm {
                                let raw_md = s.to_prompt_section();
                                let llm_client = llm::LlmClient::new(cfg.clone());
                                // Only show spinner for non-streaming mode.
                                let spinner = if !cfg.llm.stream {
                                    Some(runner::make_spinner("Running LLM quality review..."))
                                } else {
                                    None
                                };
                                match llm_client.verify_skill(&tool, &raw_md).await {
                                    Ok(report) => {
                                        if let Some(sp) = spinner {
                                            sp.finish_and_clear();
                                        }
                                        let verdict_label = if report.passed {
                                            "PASS".green().to_string()
                                        } else {
                                            "FAIL".red().to_string()
                                        };
                                        println!(
                                            "{}  LLM review: {}",
                                            if report.passed {
                                                "✓".green().bold()
                                            } else {
                                                "✗".red().bold()
                                            },
                                            verdict_label
                                        );
                                        if !report.summary.is_empty() {
                                            println!("   {}", report.summary.dimmed());
                                        }
                                        if !report.issues.is_empty() {
                                            println!("\n   {}", "Issues:".bold());
                                            for issue in &report.issues {
                                                println!("    {} {}", "·".red(), issue);
                                            }
                                        }
                                        if !report.suggestions.is_empty() {
                                            println!("\n   {}", "Suggestions:".bold());
                                            for sug in &report.suggestions {
                                                println!("    {} {}", "·".cyan(), sug);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if let Some(sp) = spinner {
                                            sp.finish_and_clear();
                                        }
                                        println!(
                                            "{}  LLM review skipped: {}",
                                            "!".yellow().bold(),
                                            e
                                        );
                                        println!(
                                            "   (Run 'oxo-call config verify' to check your LLM setup)"
                                        );
                                    }
                                }
                            }

                            println!();
                            if depth_issues.is_empty() {
                                println!(
                                    "{} Use 'oxo-call skill polish {}' to have the LLM enhance this skill.",
                                    "ℹ".blue().bold(),
                                    tool
                                );
                            } else {
                                println!(
                                    "{} Use 'oxo-call skill polish {}' to automatically fix these issues.",
                                    "ℹ".blue().bold(),
                                    tool
                                );
                            }
                        }
                    }
                }

                SkillCommands::Polish { tool, output } => {
                    // Resolve the skill source file path
                    let skill_path = mgr.find_user_or_community_skill_path(&tool)?;
                    let skill_content = std::fs::read_to_string(&skill_path)?;

                    let llm_client = llm::LlmClient::new(cfg.clone());
                    // Only show spinner for non-streaming mode.
                    let spinner = if !cfg.llm.stream {
                        Some(runner::make_spinner(&format!(
                            "Polishing skill '{tool}' with LLM..."
                        )))
                    } else {
                        None
                    };
                    match llm_client.polish_skill(&tool, &skill_content).await {
                        Ok(improved) => {
                            if let Some(sp) = spinner {
                                sp.finish_and_clear();
                            }
                            let dest = output.unwrap_or_else(|| skill_path.clone());
                            std::fs::write(&dest, &improved)?;
                            println!(
                                "{} Polished skill for '{}' saved to '{}'",
                                "✓".green().bold(),
                                tool.cyan(),
                                dest.display()
                            );
                            println!(
                                "   Use 'oxo-call skill verify {}' to review the result.",
                                tool
                            );
                        }
                        Err(e) => {
                            if let Some(sp) = spinner {
                                sp.finish_and_clear();
                            }
                            return Err(e);
                        }
                    }
                }

                SkillCommands::Path => {
                    let path = mgr.user_skill_dir()?;
                    println!("{}", path.display());
                }

                // ── MCP skill provider management ─────────────────────────
                SkillCommands::McpServer { command } => match command {
                    SkillMcpCommands::Add { url, name, api_key } => {
                        let label = name.clone().unwrap_or_else(|| url.clone());
                        // Prevent duplicates by URL
                        if cfg.mcp.servers.iter().any(|s| s.url == url) {
                            println!(
                                "{} MCP server '{}' is already registered.",
                                "ℹ".blue().bold(),
                                url.cyan()
                            );
                            return Ok(());
                        }
                        cfg.mcp.servers.push(config::McpServerConfig {
                            url: url.clone(),
                            name: label.clone(),
                            api_key,
                        });
                        cfg.save()?;
                        println!(
                            "{} Registered MCP server '{}' ({})",
                            "✓".green().bold(),
                            label.cyan(),
                            url
                        );
                    }

                    SkillMcpCommands::Remove { url_or_name } => {
                        let before = cfg.mcp.servers.len();
                        cfg.mcp
                            .servers
                            .retain(|s| s.url != url_or_name && s.name != url_or_name);
                        if cfg.mcp.servers.len() == before {
                            return Err(crate::error::OxoError::IndexError(format!(
                                "No MCP server found matching '{url_or_name}'"
                            )));
                        }
                        cfg.save()?;
                        println!(
                            "{} Removed MCP server '{}'",
                            "✓".green().bold(),
                            url_or_name.cyan()
                        );
                    }

                    SkillMcpCommands::List => {
                        if cfg.mcp.servers.is_empty() {
                            println!("{}", "No MCP skill servers registered.".yellow());
                            println!("Add one with: {}", "oxo-call skill mcp add <url>".bold());
                        } else {
                            println!("{:<24} {}", "Name / Label".bold(), "URL".bold());
                            println!("{}", "─".repeat(60).dimmed());
                            for s in &cfg.mcp.servers {
                                println!("{:<24} {}", s.name().cyan(), s.url);
                            }
                            println!("\n{} server(s) registered.", cfg.mcp.servers.len());
                        }
                    }

                    SkillMcpCommands::Ping => {
                        if cfg.mcp.servers.is_empty() {
                            println!("{}", "No MCP skill servers registered.".yellow());
                            return Ok(());
                        }
                        for server in &cfg.mcp.servers {
                            let client = mcp::McpClient::new(server.clone());
                            let spinner =
                                runner::make_spinner(&format!("Pinging '{}'...", server.name()));
                            match client.initialize().await {
                                Ok((server_name, version)) => {
                                    spinner.finish_and_clear();
                                    // Count available skills
                                    let skill_count = client
                                        .list_skill_resources()
                                        .await
                                        .map(|r| r.len())
                                        .unwrap_or(0);
                                    println!(
                                        "{} {} — {} v{} ({} skills)",
                                        "✓".green().bold(),
                                        server.name().cyan(),
                                        server_name,
                                        version,
                                        skill_count
                                    );
                                }
                                Err(e) => {
                                    spinner.finish_and_clear();
                                    println!(
                                        "{} {} — {}",
                                        "✗".red().bold(),
                                        server.name().cyan(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                },
            }
        }

        Commands::License { command } => {
            match command {
                None => {
                    // Show static license information
                    print!("{}", license::LICENSE_INFO);
                    // Show current license status if a file is present
                    let license_path = cli.license.as_deref();
                    match license::load_and_verify(license_path) {
                        Ok(lic) => {
                            println!(
                                "  Current license: {} — issued to: {}{}",
                                lic.payload.license_type.to_string().green().bold(),
                                lic.payload.issued_to_org.cyan(),
                                lic.payload
                                    .contact_email
                                    .as_deref()
                                    .map(|e| format!(" <{e}>"))
                                    .unwrap_or_default()
                            );
                            println!(
                                "  Issued at: {}  |  Perpetual: {}",
                                lic.payload.issued_at, lic.payload.perpetual
                            );
                        }
                        Err(license::LicenseError::NotFound) => {
                            println!("  Current license: {}", "none found".yellow());
                        }
                        Err(e) => {
                            println!("  Current license: {} — {e}", "invalid".red().bold());
                        }
                    }
                }
                Some(LicenseCommands::Verify) => {
                    let license_path = cli.license.as_deref();
                    match license::load_and_verify(license_path) {
                        Ok(lic) => {
                            println!("{} License is valid", "✓".green().bold());
                            println!(
                                "  Type       : {}",
                                lic.payload.license_type.to_string().green()
                            );
                            println!("  Issued to  : {}", lic.payload.issued_to_org.cyan());
                            if let Some(email) = &lic.payload.contact_email {
                                println!("  Contact    : {}", email);
                            }
                            println!("  License ID : {}", lic.payload.license_id.dimmed());
                            println!("  Issued at  : {}", lic.payload.issued_at);
                            println!(
                                "  Perpetual  : {}",
                                if lic.payload.perpetual { "yes" } else { "no" }
                            );
                        }
                        Err(e) => {
                            eprintln!("{} {}", "✗ License error:".red().bold(), e);
                            std::process::exit(2);
                        }
                    }
                }
            }
        }

        Commands::Workflow { command } => match command {
            WorkflowCommands::List => {
                workflow::print_template_list();
            }

            WorkflowCommands::Show { name, engine } => {
                // Check if `name` is a file path first, then fall back to built-in name.
                let content = if std::path::Path::new(&name).exists() {
                    let def = engine::WorkflowDef::from_file(std::path::Path::new(&name))?;
                    match engine.as_str() {
                        "snakemake" => engine::to_snakemake(&def),
                        "nextflow" => engine::to_nextflow(&def),
                        _ => std::fs::read_to_string(&name)?,
                    }
                } else {
                    match workflow::find_template(&name) {
                        Some(tpl) => match engine.as_str() {
                            "snakemake" => tpl.snakemake.to_string(),
                            "nextflow" => tpl.nextflow.to_string(),
                            _ => tpl.native.to_string(),
                        },
                        None => {
                            eprintln!(
                                "{} Unknown workflow template '{}'. Run 'oxo-call workflow list'.",
                                "error:".red().bold(),
                                name
                            );
                            std::process::exit(1);
                        }
                    }
                };
                println!("{content}");
            }

            WorkflowCommands::RunWorkflow { file, verify } => {
                let path = std::path::Path::new(&file);
                let source = if path.exists() {
                    std::fs::read_to_string(path)?
                } else {
                    // Try as built-in template name.
                    match workflow::find_template(&file) {
                        Some(tpl) => tpl.native.to_string(),
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                let def = engine::WorkflowDef::from_str_content(&source)?;
                let tasks = engine::expand(&def)?;
                engine::execute(tasks, false).await?;
                if verify {
                    let cfg = base_cfg.clone();
                    engine::verify_workflow_results(&def, &cfg).await;
                }
            }

            WorkflowCommands::DryRunWorkflow { file } => {
                let path = std::path::Path::new(&file);
                let source = if path.exists() {
                    std::fs::read_to_string(path)?
                } else {
                    match workflow::find_template(&file) {
                        Some(tpl) => tpl.native.to_string(),
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                let def = engine::WorkflowDef::from_str_content(&source)?;
                let tasks = engine::expand(&def)?;
                engine::execute(tasks, true).await?;
            }

            WorkflowCommands::Export { file, to, output } => {
                let path = std::path::Path::new(&file);
                let def = if path.exists() {
                    engine::WorkflowDef::from_file(path)?
                } else {
                    match workflow::find_template(&file) {
                        Some(tpl) => engine::WorkflowDef::from_str_content(tpl.native)?,
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                let content = match to.as_str() {
                    "nextflow" => engine::to_nextflow(&def),
                    _ => engine::to_snakemake(&def),
                };
                match output {
                    Some(out_path) => {
                        std::fs::write(&out_path, &content)?;
                        println!(
                            "{} Exported to {}",
                            "✓".green().bold(),
                            out_path.display().to_string().cyan()
                        );
                    }
                    None => println!("{content}"),
                }
            }

            WorkflowCommands::Generate {
                task,
                engine: engine_name,
                output,
                no_stream,
            } => {
                let mut cfg = base_cfg.clone();
                if no_stream {
                    cfg.llm.stream = false;
                }
                let label = match engine_name.as_str() {
                    "snakemake" => "Snakemake",
                    "nextflow" => "Nextflow DSL2",
                    _ => "native (.oxo.toml)",
                };

                // Only show spinner for non-streaming mode.
                // Streaming mode uses StreamingDisplay internally.
                let spinner = if !cfg.llm.stream {
                    Some(runner::make_spinner(&format!(
                        "Generating {label} workflow with LLM..."
                    )))
                } else {
                    None
                };

                let wf = match workflow::generate_workflow(&cfg, &task, &engine_name).await {
                    Ok(w) => {
                        if let Some(sp) = spinner {
                            sp.finish_and_clear();
                        }
                        w
                    }
                    Err(e) => {
                        if let Some(sp) = spinner {
                            sp.finish_and_clear();
                        }
                        return Err(e);
                    }
                };

                match output {
                    Some(path) => {
                        std::fs::write(&path, &wf.content)?;
                        println!(
                            "{} Workflow written to {}",
                            "✓".green().bold(),
                            path.display().to_string().cyan()
                        );
                        if !wf.explanation.is_empty() {
                            println!();
                            println!("  {}", "Pipeline explanation:".bold());
                            println!();
                            markdown::render_markdown(&wf.explanation);
                        }
                    }
                    None => {
                        workflow::print_generated_workflow(&wf);
                    }
                }
            }

            WorkflowCommands::Infer {
                task,
                data,
                engine: engine_name,
                output,
                run,
                no_stream,
            } => {
                let mut cfg = base_cfg.clone();
                if no_stream {
                    cfg.llm.stream = false;
                }

                if !data.exists() || !data.is_dir() {
                    eprintln!(
                        "{} Data directory '{}' does not exist or is not a directory.",
                        "error:".red().bold(),
                        data.display()
                    );
                    std::process::exit(1);
                }

                // Print discovered data context to the user.
                let ctx = workflow::scan_data_directory(&data);
                println!(
                    "{} Scanning data directory: {}",
                    "→".cyan().bold(),
                    data.display().to_string().cyan()
                );
                println!("  {} {}", "Data type:".bold(), ctx.data_type_hint);
                if ctx.samples.is_empty() {
                    println!(
                        "  {} {}",
                        "Samples:".bold(),
                        "(none detected — LLM will use placeholder names)".dimmed()
                    );
                } else {
                    println!(
                        "  {} {} detected: {}",
                        "Samples:".bold(),
                        ctx.samples.len(),
                        ctx.samples.join(", ").cyan()
                    );
                }
                println!();

                let label = match engine_name.as_str() {
                    "snakemake" => "Snakemake",
                    "nextflow" => "Nextflow DSL2",
                    _ => "native (.oxo.toml)",
                };

                // Only show spinner for non-streaming mode.
                // Streaming mode uses StreamingDisplay internally.
                let spinner = if !cfg.llm.stream {
                    Some(runner::make_spinner(&format!(
                        "Generating {label} workflow from data context with LLM..."
                    )))
                } else {
                    None
                };

                let wf = match workflow::infer_workflow(&cfg, &task, &data, &engine_name).await {
                    Ok(w) => {
                        if let Some(sp) = spinner {
                            sp.finish_and_clear();
                        }
                        w
                    }
                    Err(e) => {
                        if let Some(sp) = spinner {
                            sp.finish_and_clear();
                        }
                        return Err(e);
                    }
                };

                match output {
                    Some(ref path) => {
                        std::fs::write(path, &wf.content)?;
                        println!(
                            "{} Workflow written to {}",
                            "✓".green().bold(),
                            path.display().to_string().cyan()
                        );
                        if !wf.explanation.is_empty() {
                            println!();
                            println!("  {}", "Pipeline explanation:".bold());
                            println!();
                            markdown::render_markdown(&wf.explanation);
                        }

                        if run {
                            if engine_name != "native" {
                                eprintln!(
                                    "{} --run is only supported for native (.oxo.toml) workflows.",
                                    "error:".red().bold()
                                );
                                std::process::exit(1);
                            }
                            println!();
                            println!(
                                "{} Running generated workflow: {}",
                                "→".cyan().bold(),
                                path.display().to_string().cyan()
                            );
                            let def = engine::WorkflowDef::from_file(path)?;
                            let tasks = engine::expand(&def)?;
                            engine::execute(tasks, false).await?;
                        }
                    }
                    None => {
                        if run {
                            eprintln!(
                                "{} --run requires --output <file> to save the workflow first.",
                                "error:".red().bold()
                            );
                            std::process::exit(1);
                        }
                        workflow::print_generated_workflow(&wf);
                    }
                }
            }

            WorkflowCommands::Verify { file } => {
                let path = std::path::Path::new(&file);
                let def = if path.exists() {
                    engine::WorkflowDef::from_file(path)?
                } else {
                    match workflow::find_template(&file) {
                        Some(tpl) => engine::WorkflowDef::from_str_content(tpl.native)?,
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                let diags = engine::verify(&def);
                let has_errors = engine::print_verify_report(&def, &diags);
                if has_errors {
                    std::process::exit(1);
                }
            }

            WorkflowCommands::Format { file, stdout } => {
                let path = std::path::Path::new(&file);
                let def = if path.exists() {
                    engine::WorkflowDef::from_file(path)?
                } else {
                    match workflow::find_template(&file) {
                        Some(tpl) => engine::WorkflowDef::from_str_content(tpl.native)?,
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                let formatted = engine::format_toml(&def);
                if stdout || !path.exists() {
                    println!("{formatted}");
                } else {
                    std::fs::write(path, &formatted)?;
                    println!(
                        "{} Formatted {}",
                        "✓".green().bold(),
                        path.display().to_string().cyan()
                    );
                }
            }

            WorkflowCommands::Vis { file } => {
                let path = std::path::Path::new(&file);
                let def = if path.exists() {
                    engine::WorkflowDef::from_file(path)?
                } else {
                    match workflow::find_template(&file) {
                        Some(tpl) => engine::WorkflowDef::from_str_content(tpl.native)?,
                        None => {
                            eprintln!(
                                "{} '{}' is not a file or a known built-in template.",
                                "error:".red().bold(),
                                file
                            );
                            std::process::exit(1);
                        }
                    }
                };
                engine::visualize_workflow(&def)?;
            }
        },

        Commands::Server { command } => {
            match command {
                ServerCommands::Add {
                    name,
                    host,
                    user,
                    port,
                    identity_file,
                    server_type,
                    scheduler,
                    work_dir,
                } => {
                    let cfg = base_cfg.clone();
                    let st: server::ServerType = server_type
                        .parse()
                        .map_err(|e: String| error::OxoError::ConfigError(e))?;
                    let new_host = server::ServerHost {
                        name: name.clone(),
                        host,
                        user,
                        port,
                        identity_file,
                        server_type: st.clone(),
                        scheduler,
                        work_dir,
                    };
                    let mut mgr = server::ServerManager::new(cfg);
                    mgr.add(new_host)?;

                    println!(
                        "{} Registered server '{}' ({})",
                        "✓".green().bold(),
                        name.cyan(),
                        st
                    );

                    // For HPC nodes, try to detect the scheduler
                    if st == server::ServerType::Hpc
                        && let Some(server_host) = mgr.find(&name)
                    {
                        let server_host = server_host.clone();
                        if let Some(detected) = mgr.detect_scheduler(&server_host) {
                            println!(
                                "  {} Detected scheduler: {}",
                                "→".cyan().bold(),
                                detected.green()
                            );
                        } else {
                            println!(
                                "  {} Could not auto-detect scheduler. Use --scheduler to set it.",
                                "ℹ".blue().bold()
                            );
                        }
                    }
                }

                ServerCommands::Remove { name } => {
                    let cfg = base_cfg.clone();
                    let mut mgr = server::ServerManager::new(cfg);
                    mgr.remove(&name)?;
                    println!("{} Removed server '{}'", "✓".green().bold(), name.cyan());
                }

                ServerCommands::List => {
                    let cfg = base_cfg.clone();
                    let mgr = server::ServerManager::new(cfg);
                    let hosts = mgr.list();
                    if hosts.is_empty() {
                        println!(
                            "{}",
                            "No servers registered. Use 'oxo-call server add' to register one."
                                .yellow()
                        );
                    } else {
                        let active_name = mgr.get_active().map(|h| h.name.as_str()).unwrap_or("");
                        println!(
                            "{:<3} {:<16} {:<24} {:<14} {:<12} {}",
                            " ".bold(),
                            "Name".bold(),
                            "Host".bold(),
                            "Type".bold(),
                            "Scheduler".bold(),
                            "User".bold()
                        );
                        println!("{}", "─".repeat(84).dimmed());
                        for h in hosts {
                            let marker = if h.name == active_name { "✦" } else { " " };
                            println!(
                                "{:<3} {:<16} {:<24} {:<14} {:<12} {}",
                                marker.green().bold(),
                                h.name.cyan(),
                                h.ssh_dest(),
                                h.server_type.to_string(),
                                h.scheduler.as_deref().unwrap_or("—"),
                                h.user.as_deref().unwrap_or("(current)")
                            );
                        }
                        println!("\n{} server(s) registered.", hosts.len());
                        if !active_name.is_empty() {
                            println!(
                                "Active server: {} (use 'server run <tool> <task>' without --server)",
                                active_name.cyan()
                            );
                        }
                    }
                }

                ServerCommands::Status { name } => {
                    let cfg = base_cfg.clone();
                    let mgr = server::ServerManager::new(cfg);
                    let host = mgr.find(&name).ok_or_else(|| {
                        error::OxoError::ConfigError(format!("No server found with name '{name}'"))
                    })?;
                    let spinner =
                        runner::make_spinner(&format!("Checking connection to '{name}'..."));
                    let connected = mgr.check_connection(host)?;
                    spinner.finish_and_clear();
                    if connected {
                        println!(
                            "{} Server '{}' is reachable ({})",
                            "✓".green().bold(),
                            name.cyan(),
                            host.ssh_dest()
                        );
                        if host.server_type == server::ServerType::Hpc
                            && let Some(ref sched) = host.scheduler
                        {
                            println!("  {} Scheduler: {}", "→".cyan().bold(), sched.green());
                        }
                    } else {
                        eprintln!(
                            "{} Cannot connect to '{}' ({})",
                            "✗".red().bold(),
                            name.cyan(),
                            host.ssh_dest()
                        );
                        eprintln!("  Check SSH configuration, keys, and network connectivity.");
                        std::process::exit(1);
                    }
                }

                ServerCommands::SshConfig { yes, server_type } => {
                    let entries = server::parse_ssh_config();
                    if entries.is_empty() {
                        println!(
                            "{}",
                            "No hosts found in ~/.ssh/config (or file does not exist).".yellow()
                        );
                        return Ok(());
                    }

                    let cfg = base_cfg.clone();
                    let mut mgr = server::ServerManager::new(cfg);
                    let already_registered: std::collections::HashSet<String> =
                        mgr.list().iter().map(|h| h.name.clone()).collect();

                    println!(
                        "{} Found {} host(s) in ~/.ssh/config:\n",
                        "→".cyan().bold(),
                        entries.len()
                    );
                    println!(
                        "{:<5} {:<20} {:<28} {:<12} {}",
                        "#".bold(),
                        "Alias".bold(),
                        "HostName".bold(),
                        "User".bold(),
                        "Port".bold()
                    );
                    println!("{}", "─".repeat(75).dimmed());
                    for (i, e) in entries.iter().enumerate() {
                        let tag = if already_registered.contains(&e.alias) {
                            format!(" {}", "[registered]".dimmed())
                        } else {
                            String::new()
                        };
                        println!(
                            "{:<5} {:<20} {:<28} {:<12} {}{}",
                            (i + 1).to_string().yellow(),
                            e.alias.cyan(),
                            e.hostname.as_deref().unwrap_or("—"),
                            e.user.as_deref().unwrap_or("—"),
                            e.port
                                .map(|p| p.to_string())
                                .unwrap_or_else(|| "22".to_string()),
                            tag
                        );
                    }
                    println!();

                    // Determine which indices to import.
                    let selected: Vec<usize> = if yes {
                        (0..entries.len()).collect()
                    } else {
                        use std::io::Write;
                        print!(
                            "Select hosts to import ({}, {} or {}):\n> ",
                            "e.g. 1,3,5-7".dimmed(),
                            "'all'".bold(),
                            "Enter to cancel".dimmed()
                        );
                        std::io::stdout().flush().ok();
                        let mut input = String::new();
                        if let Err(e) = std::io::stdin().read_line(&mut input) {
                            eprintln!("{} Failed to read input: {e}", "✗".red().bold());
                            return Ok(());
                        }
                        let input = input.trim();
                        if input.is_empty() {
                            println!("{}", "Cancelled.".dimmed());
                            return Ok(());
                        }
                        server::parse_selection(input, entries.len())
                    };

                    if selected.is_empty() {
                        println!("{}", "No valid selection — nothing imported.".yellow());
                        return Ok(());
                    }

                    // In batch mode (--yes) use the global --type for every host.
                    // In interactive mode, allow the user to set a type per host.
                    let default_st: server::ServerType = server_type
                        .parse()
                        .map_err(|e: String| error::OxoError::ConfigError(e))?;

                    // Build a map: selected index → ServerType
                    let per_host_types: Vec<server::ServerType> = if yes {
                        // Batch: apply default to all.
                        selected.iter().map(|_| default_st.clone()).collect()
                    } else {
                        // Interactive: offer per-host type override.
                        use std::io::Write;
                        print!(
                            "Set server type per host? ({}/{}, Enter for all-'{}'):\n> ",
                            "y".bold(),
                            "N".dimmed(),
                            server_type.yellow()
                        );
                        std::io::stdout().flush().ok();
                        let mut ans = String::new();
                        std::io::stdin().read_line(&mut ans).ok();
                        if ans.trim().eq_ignore_ascii_case("y") {
                            let mut types = Vec::new();
                            for idx in &selected {
                                let e = &entries[*idx];
                                loop {
                                    print!(
                                        "  Type for '{}' ([{}]/{}): ",
                                        e.alias.cyan(),
                                        "workstation".green(),
                                        "hpc".yellow()
                                    );
                                    std::io::stdout().flush().ok();
                                    let mut t_input = String::new();
                                    std::io::stdin().read_line(&mut t_input).ok();
                                    let t_str = t_input.trim();
                                    if t_str.is_empty() {
                                        types.push(default_st.clone());
                                        break;
                                    }
                                    match t_str.parse::<server::ServerType>() {
                                        Ok(st) => {
                                            types.push(st);
                                            break;
                                        }
                                        Err(_) => {
                                            eprintln!(
                                                "  {} Invalid type '{}'. Use 'workstation' or 'hpc'.",
                                                "✗".red(),
                                                t_str
                                            );
                                        }
                                    }
                                }
                            }
                            types
                        } else {
                            selected.iter().map(|_| default_st.clone()).collect()
                        }
                    };

                    let mut imported = 0usize;
                    let mut skipped = 0usize;
                    for (i, idx) in selected.iter().enumerate() {
                        let e = &entries[*idx];
                        if already_registered.contains(&e.alias) {
                            println!(
                                "  {} '{}' already registered — skipping.",
                                "·".dimmed(),
                                e.alias.cyan()
                            );
                            skipped += 1;
                            continue;
                        }
                        let host_type = per_host_types
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| default_st.clone());
                        let new_host = server::ServerHost {
                            name: e.alias.clone(),
                            host: e.hostname.clone().unwrap_or_else(|| e.alias.clone()),
                            user: e.user.clone(),
                            port: e.port,
                            identity_file: e.identity_file.clone(),
                            server_type: host_type,
                            scheduler: None,
                            work_dir: None,
                        };
                        mgr.add(new_host)?;
                        println!("  {} Registered '{}'", "✓".green().bold(), e.alias.cyan());
                        imported += 1;
                    }

                    println!();
                    println!(
                        "{} Imported {}, skipped {}.",
                        "→".cyan().bold(),
                        format!("{imported} host(s)").green(),
                        format!("{skipped} already registered").yellow()
                    );
                }

                ServerCommands::Use { name } => {
                    let cfg = base_cfg.clone();
                    let mut mgr = server::ServerManager::new(cfg);
                    mgr.set_active(&name)?;
                    println!(
                        "{} Active server set to '{}'",
                        "✓".green().bold(),
                        name.cyan()
                    );
                    println!(
                        "  You can now run: {}",
                        "oxo-call server run <tool> <task>".bold()
                    );
                }

                ServerCommands::Unuse => {
                    let cfg = base_cfg.clone();
                    let mut mgr = server::ServerManager::new(cfg);
                    mgr.clear_active()?;
                    println!("{} Active server cleared.", "✓".green().bold());
                }

                ServerCommands::Run {
                    tool,
                    task,
                    server: server_flag,
                    model,
                    no_cache,
                    json,
                    verify,
                    no_stream,
                } => {
                    let cfg = base_cfg.clone();
                    let mgr = server::ServerManager::new(cfg.clone());

                    // Resolve server: explicit flag → active server → error
                    let host = match &server_flag {
                        Some(name) => mgr.find(name).ok_or_else(|| {
                            error::OxoError::ConfigError(format!(
                                "No server found with name '{name}'. Run 'oxo-call server list'."
                            ))
                        })?,
                        None => mgr.get_active().ok_or_else(|| {
                            error::OxoError::ConfigError(
                                "No server specified and no active server set. \
                                 Use --server <name> or run 'oxo-call server use <name>'"
                                    .to_string(),
                            )
                        })?,
                    }
                    .clone();
                    let server_name = host.name.clone();

                    // Warn about login node compute execution
                    if host.server_type == server::ServerType::Hpc
                        && server::ServerManager::is_compute_command(&tool)
                    {
                        eprintln!(
                            "{} This appears to be a compute-intensive command.",
                            "⚠ WARNING:".bold().yellow()
                        );
                        eprintln!(
                            "  Server '{}' is an HPC login node. Running compute jobs directly on",
                            server_name.cyan()
                        );
                        eprintln!(
                            "  login nodes is discouraged and may be prohibited by your site policy."
                        );
                        if let Some(ref sched) = host.scheduler {
                            eprintln!(
                                "  Consider submitting through {} instead (e.g., sbatch, qsub).",
                                sched.green()
                            );
                        }
                        eprintln!();
                    }

                    // Generate the command via LLM
                    let mut run_cfg = cfg;
                    if let Some(ref m) = model {
                        run_cfg.llm.model = Some(m.clone());
                    }
                    let mut runner_inst = runner::Runner::new(run_cfg);
                    runner_inst
                        .with_verbose(verbose)
                        .with_no_cache(no_cache)
                        .with_verify(verify)
                        .with_no_stream(no_stream);
                    let generated = runner_inst.generate_command(&tool, &task).await?;

                    // Show preview
                    if json {
                        let preview = serde_json::json!({
                            "tool": tool,
                            "task": task,
                            "effective_task": generated.effective_task,
                            "command": generated.full_cmd,
                            "explanation": generated.explanation,
                            "server": server_name,
                            "ssh_dest": host.ssh_dest(),
                        });
                        println!("{}", serde_json::to_string_pretty(&preview)?);
                    } else {
                        println!();
                        println!("{}", "─".repeat(60).dimmed());
                        println!("  {} {}", "Tool:".bold(), tool.cyan());
                        println!("  {} {}", "Task:".bold(), task);
                        if generated.effective_task != task {
                            println!(
                                "  {} {}",
                                "Optimized task:".bold().dimmed(),
                                generated.effective_task.dimmed()
                            );
                        }
                        println!("{}", "─".repeat(60).dimmed());
                        println!();
                        println!("  {}", "Generated command:".bold().green());
                        println!("  {}", generated.full_cmd.green().bold());
                        println!();
                        if !generated.explanation.is_empty() {
                            println!("  {}", "Explanation:".bold());
                            println!();
                            markdown::render_markdown(&generated.explanation);
                            println!();
                        }
                        println!("{}", "─".repeat(60).dimmed());
                    }

                    // Ask for confirmation before SSH execution
                    use std::io::Write as _;
                    print!(
                        "\n  {} [y/N] ",
                        format!("Execute on '{server_name}'?").bold().yellow()
                    );
                    std::io::stdout().flush().ok();
                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm).ok();
                    if confirm.trim().to_lowercase() != "y"
                        && confirm.trim().to_lowercase() != "yes"
                    {
                        println!("{}", "Aborted.".red());
                        return Ok(());
                    }

                    // Execute via SSH
                    println!();
                    println!("{}", "─".repeat(60).dimmed());
                    println!(
                        "  {} ssh {} '{}'",
                        "Running:".bold(),
                        host.ssh_dest().cyan(),
                        generated.full_cmd
                    );
                    println!("{}", "─".repeat(60).dimmed());
                    println!();

                    let mut ssh_cmd = std::process::Command::new("ssh");
                    for arg in &host.ssh_args() {
                        ssh_cmd.arg(arg);
                    }
                    ssh_cmd.arg(&generated.full_cmd);

                    let status = ssh_cmd.status()?;
                    let exit_code = status.code().unwrap_or(-1);

                    // Record the server run in history.
                    let _ = history::HistoryStore::append(history::HistoryEntry {
                        id: uuid::Uuid::new_v4().to_string(),
                        tool: tool.clone(),
                        task: task.clone(),
                        command: generated.full_cmd.clone(),
                        exit_code,
                        executed_at: chrono::Utc::now(),
                        dry_run: false,
                        server: Some(server_name.clone()),
                        provenance: None,
                    });

                    println!();
                    if status.success() {
                        println!(
                            "{} Command completed on '{}'.",
                            "✓".green().bold(),
                            server_name.cyan()
                        );
                    } else {
                        eprintln!(
                            "{} Command exited with code {exit_code} on '{}'.",
                            "✗".red().bold(),
                            server_name.cyan()
                        );
                        return Err(error::OxoError::ExecutionError(format!(
                            "SSH command on '{server_name}' exited with code {exit_code}"
                        )));
                    }
                }

                ServerCommands::DryRun {
                    tool,
                    task,
                    server: server_flag,
                    model,
                    no_cache,
                    json,
                    no_stream,
                } => {
                    let cfg = base_cfg.clone();
                    let mgr = server::ServerManager::new(cfg.clone());

                    // Resolve server: explicit flag → active server → error
                    let host = match &server_flag {
                        Some(name) => mgr.find(name).ok_or_else(|| {
                            error::OxoError::ConfigError(format!(
                                "No server found with name '{name}'. Run 'oxo-call server list'."
                            ))
                        })?,
                        None => mgr.get_active().ok_or_else(|| {
                            error::OxoError::ConfigError(
                                "No server specified and no active server set. \
                                 Use --server <name> or run 'oxo-call server use <name>'"
                                    .to_string(),
                            )
                        })?,
                    }
                    .clone();
                    let server_name = host.name.clone();

                    let mut run_cfg = cfg;
                    if let Some(ref m) = model {
                        run_cfg.llm.model = Some(m.clone());
                    }
                    let mut runner_inst = runner::Runner::new(run_cfg);
                    runner_inst
                        .with_verbose(verbose)
                        .with_no_cache(no_cache)
                        .with_no_stream(no_stream);
                    runner_inst
                        .dry_run(&tool, &task, json, Some(&server_name))
                        .await?;

                    println!(
                        "\n{} Target server: '{}' ({}{})",
                        "→".cyan().bold(),
                        server_name.cyan(),
                        host.ssh_dest(),
                        if host.server_type == server::ServerType::Hpc {
                            format!(
                                ", HPC login node{}",
                                host.scheduler
                                    .as_ref()
                                    .map(|s| format!(", scheduler: {s}"))
                                    .unwrap_or_default()
                            )
                        } else {
                            String::new()
                        }
                    );

                    if host.server_type == server::ServerType::Hpc {
                        if let Some(ref sched) = host.scheduler {
                            println!(
                                "  {} Wrap compute commands with {} for submission (e.g., sbatch --wrap=\"...\")",
                                "ℹ".blue().bold(),
                                sched.green()
                            );
                        } else {
                            eprintln!(
                                "  {} This is an HPC login node. Consider submitting via a scheduler.",
                                "⚠".yellow().bold()
                            );
                        }
                    }
                }
            }
        }

        Commands::Job { command } => {
            match command {
                JobCommands::Add {
                    name,
                    command,
                    description,
                    tags,
                    schedule,
                } => {
                    let now = chrono::Utc::now();
                    let entry = job::JobEntry {
                        name: name.clone(),
                        command,
                        description,
                        tags,
                        schedule,
                        run_count: 0,
                        last_run: None,
                        last_exit_code: None,
                        created_at: now,
                        updated_at: now,
                    };
                    job::JobManager::add(entry)?;
                    println!(
                        "{} Job '{}' added to your library.",
                        "✓".green().bold(),
                        name.cyan()
                    );
                    println!("  Run with: {}", format!("oxo-call job run {name}").bold());
                }

                JobCommands::Remove { name } => {
                    job::JobManager::remove(&name)?;
                    println!("{} Job '{}' removed.", "✓".green().bold(), name.cyan());
                }

                JobCommands::List { tag, builtin } => {
                    if builtin {
                        let entries = job::builtin_jobs(tag.as_deref());
                        if entries.is_empty() {
                            if let Some(ref t) = tag {
                                println!("No built-in jobs found with tag '{t}'.");
                            } else {
                                println!("No built-in jobs available.");
                            }
                        } else {
                            println!();
                            println!("  {} Built-in job templates:", "●".cyan());
                            println!();
                            for j in &entries {
                                let tags_str = if j.tags.is_empty() {
                                    String::new()
                                } else {
                                    format!("  [{}]", j.tags.join(", ").dimmed())
                                };
                                println!(
                                    "  {}{}  {}",
                                    j.name.cyan().bold(),
                                    tags_str,
                                    j.description.dimmed()
                                );
                                println!("    {}", j.command.green());
                            }
                            println!();
                            println!(
                                "  {} built-in template(s). Import one with: {}",
                                entries.len(),
                                "oxo-call job import <name>".bold()
                            );
                        }
                    } else {
                        let entries = job::JobManager::list(tag.as_deref())?;
                        if entries.is_empty() {
                            if let Some(ref t) = tag {
                                println!("No jobs found with tag '{t}'.");
                            } else {
                                println!("No jobs saved yet.");
                                println!(
                                    "  Add one with: {}",
                                    "oxo-call job add <name> '<command>'".bold()
                                );
                                println!(
                                    "  Or browse built-ins: {}",
                                    "oxo-call job list --builtin".bold()
                                );
                            }
                        } else {
                            println!();
                            for entry in &entries {
                                let tags_str = if entry.tags.is_empty() {
                                    String::new()
                                } else {
                                    format!("  [{}]", entry.tags.join(", ").dimmed())
                                };
                                let status = match entry.last_exit_code {
                                    None => "–".dimmed().to_string(),
                                    Some(0) => "✓".green().to_string(),
                                    Some(_) => "✗".red().to_string(),
                                };
                                println!(
                                    "  {} {}{}  {}",
                                    status,
                                    entry.name.cyan().bold(),
                                    tags_str,
                                    entry.description.as_deref().unwrap_or("").dimmed()
                                );
                                println!("    {}", entry.command.green());
                            }
                            println!();
                            println!(
                                "  {} job(s). Run: {}",
                                entries.len(),
                                "oxo-call job run <name>".bold()
                            );
                        }
                    }
                }

                JobCommands::Show { name } => {
                    let entry = job::JobManager::find(&name)?.ok_or_else(|| {
                        error::OxoError::ConfigError(format!("No job found with name '{name}'"))
                    })?;
                    println!();
                    println!("  {}  {}", "Name:".bold(), entry.name.cyan().bold());
                    println!("  {}  {}", "Command:".bold(), entry.command.green());
                    if let Some(ref desc) = entry.description {
                        println!("  {}  {}", "Description:".bold(), desc);
                    }
                    if !entry.tags.is_empty() {
                        println!("  {}  {}", "Tags:".bold(), entry.tags.join(", "));
                    }
                    if let Some(ref sched) = entry.schedule {
                        println!("  {}  {}", "Schedule:".bold(), sched.yellow());
                    }
                    println!(
                        "  {}  {}",
                        "Status:".bold(),
                        match entry.last_exit_code {
                            None => "never run".dimmed().to_string(),
                            Some(0) => "ok (last run succeeded)".green().to_string(),
                            Some(c) => format!("failed (exit code {c})").red().to_string(),
                        }
                    );
                    println!("  {}  {}", "Run count:".bold(), entry.run_count);
                    if let Some(ref lr) = entry.last_run {
                        println!(
                            "  {}  {}",
                            "Last run:".bold(),
                            lr.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                    println!(
                        "  {}  {}",
                        "Created:".bold(),
                        entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    if entry.updated_at != entry.created_at {
                        println!(
                            "  {}  {}",
                            "Updated:".bold(),
                            entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
                        );
                    }
                    println!();
                }

                JobCommands::Run {
                    name,
                    server: server_flag,
                    dry_run,
                    vars,
                    input_list,
                    input_items,
                    jobs,
                    keep_order,
                    stop_on_error,
                } => {
                    let entry = job::JobManager::find(&name)?.ok_or_else(|| {
                        error::OxoError::ConfigError(format!("No job found with name '{name}'"))
                    })?;

                    // Parse --var KEY=VALUE pairs.
                    let mut var_map = std::collections::HashMap::new();
                    for v in &vars {
                        let (k, val) = job::parse_var(v)?;
                        var_map.insert(k, val);
                    }

                    // Collect input items.
                    let mut all_items: Vec<String> = Vec::new();
                    if let Some(ref path) = input_list {
                        all_items.extend(job::read_input_list(path)?);
                    }
                    if let Some(ref items_str) = input_items {
                        all_items.extend(
                            items_str
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty()),
                        );
                    }
                    // If neither --input-list nor --input-items was given but stdin is not
                    // a terminal, read items from stdin (one per line, skip blank/#-lines).
                    if all_items.is_empty() && input_list.is_none() && input_items.is_none() {
                        use std::io::IsTerminal;
                        if !std::io::stdin().is_terminal() {
                            use std::io::BufRead;
                            let stdin = std::io::stdin();
                            all_items = stdin
                                .lock()
                                .lines()
                                .map_while(|l| l.ok())
                                .filter(|l| {
                                    !l.trim().is_empty() && !l.trim_start().starts_with('#')
                                })
                                .collect();
                        }
                    }

                    // Apply var substitution to the base command (no item yet).
                    let base_cmd = if var_map.is_empty() {
                        entry.command.clone()
                    } else {
                        job::interpolate_command(&entry.command, "", 0, &var_map)
                    };

                    if all_items.is_empty() {
                        // ── Single run (original behaviour + var substitution) ──────────

                        println!();
                        println!("  {} {}", "Command:".bold(), base_cmd.green().bold());
                        if let Some(ref desc) = entry.description {
                            println!("  {} {}", "Description:".bold(), desc.dimmed());
                        }

                        if dry_run {
                            println!();
                            println!("{}", "  (dry-run — not executed)".yellow());
                            return Ok(());
                        }

                        let started = chrono::Utc::now();
                        let start_inst = std::time::Instant::now();

                        if let Some(ref srv_name) = server_flag {
                            let cfg = base_cfg.clone();
                            let mgr = server::ServerManager::new(cfg);
                            let host = mgr.find(srv_name).ok_or_else(|| {
                                error::OxoError::ConfigError(format!(
                                    "No server found with name '{srv_name}'. \
                                     Run 'oxo-call server list'."
                                ))
                            })?;

                            println!(
                                "  {} {} ({})",
                                "Server:".bold(),
                                srv_name.cyan(),
                                host.ssh_dest()
                            );
                            println!();
                            println!(
                                "  {} ssh {} '{}'",
                                "Running:".bold(),
                                host.ssh_dest().cyan(),
                                base_cmd
                            );
                            println!("{}", "─".repeat(60).dimmed());

                            let mut ssh_cmd = std::process::Command::new("ssh");
                            for arg in &host.ssh_args() {
                                ssh_cmd.arg(arg);
                            }
                            ssh_cmd.arg(&base_cmd);
                            let status = ssh_cmd.status()?;
                            let dur = start_inst.elapsed().as_secs_f64();
                            let code = status.code().unwrap_or(1);

                            let _ = job::JobManager::record_run(
                                &name,
                                &base_cmd,
                                Some(srv_name.clone()),
                                code,
                                started,
                                dur,
                            );

                            println!();
                            if status.success() {
                                println!(
                                    "{} Job completed on '{}'.",
                                    "✓".green().bold(),
                                    srv_name.cyan()
                                );
                            } else {
                                eprintln!(
                                    "{} Job exited with code {code} on '{}'.",
                                    "✗".red().bold(),
                                    srv_name.cyan()
                                );
                                return Err(error::OxoError::ExecutionError(format!(
                                    "SSH job on '{srv_name}' exited with code {code}"
                                )));
                            }
                        } else {
                            // Local execution
                            println!("{}", "─".repeat(60).dimmed());
                            let status = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&base_cmd)
                                .status()?;
                            let dur = start_inst.elapsed().as_secs_f64();
                            let code = status.code().unwrap_or(1);

                            let _ = job::JobManager::record_run(
                                &name, &base_cmd, None, code, started, dur,
                            );

                            println!();
                            if status.success() {
                                println!("{} Done.", "✓".green().bold());
                            } else {
                                eprintln!("{} Job exited with code {code}.", "✗".red().bold());
                                return Err(error::OxoError::ExecutionError(format!(
                                    "Job '{name}' exited with code {code}"
                                )));
                            }
                        }
                    } else {
                        // ── Batch run (one item per invocation) ──────────────────────────

                        let n = all_items.len();
                        let jobs = jobs.max(1);
                        // `keep_order` controls whether we join handles in submission order
                        // (always true with our current Vec-based approach) or fire-and-forget.
                        // Both modes are order-preserving for correctness; the flag is reserved
                        // for future output-buffering support.
                        let _ = keep_order;

                        if dry_run {
                            println!();
                            println!(
                                "  {} {}",
                                "Command template:".bold(),
                                entry.command.green().bold()
                            );
                            println!(
                                "  {} {} items would be processed:",
                                "Batch:".bold(),
                                n.to_string().cyan()
                            );
                            println!("{}", "─".repeat(60).dimmed());
                            for (i, item) in all_items.iter().enumerate() {
                                let cmd =
                                    job::interpolate_command(&entry.command, item, i + 1, &var_map);
                                println!("  [{:>3}] {}", i + 1, cmd.green());
                            }
                            println!("{}", "─".repeat(60).dimmed());
                            println!("{}", "  (dry-run — not executed)".yellow());
                            return Ok(());
                        }

                        println!();
                        println!(
                            "  {} {}",
                            "Command template:".bold(),
                            entry.command.green().bold()
                        );
                        println!(
                            "  {} {} items, {} parallel{}",
                            "Batch:".bold(),
                            n.to_string().cyan(),
                            jobs.to_string().cyan(),
                            if stop_on_error {
                                " (stop-on-error)".yellow().to_string()
                            } else {
                                String::new()
                            }
                        );
                        println!("{}", "─".repeat(60).dimmed());
                        println!();

                        use std::sync::Arc;
                        // Record wall-clock start before tasks are spawned.
                        let started_all = chrono::Utc::now();
                        let start_inst_all = std::time::Instant::now();

                        let sem = Arc::new(tokio::sync::Semaphore::new(jobs));
                        let mut handles: Vec<(
                            String,
                            tokio::task::JoinHandle<error::Result<i32>>,
                        )> = Vec::with_capacity(n);

                        for (i, item) in all_items.iter().enumerate() {
                            let cmd =
                                job::interpolate_command(&entry.command, item, i + 1, &var_map);
                            let sem_clone = Arc::clone(&sem);
                            let item_label = item.clone();
                            let handle: tokio::task::JoinHandle<error::Result<i32>> =
                                tokio::spawn(async move {
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
                                                error::OxoError::ExecutionError(format!(
                                                    "failed to run '{item_label}': {e}"
                                                ))
                                            })
                                    })
                                    .await
                                    .map_err(|e| {
                                        error::OxoError::ExecutionError(format!(
                                            "task join error: {e}"
                                        ))
                                    })?
                                });
                            handles.push((item.clone(), handle));
                        }

                        let mut failed = 0usize;
                        let mut done = 0usize;
                        for (item, handle) in handles {
                            let code = match handle.await {
                                Ok(Ok(c)) => c,
                                Ok(Err(e)) => {
                                    failed += 1;
                                    eprintln!("  {} {}: {}", "✗".red().bold(), item, e);
                                    -1
                                }
                                Err(e) => {
                                    failed += 1;
                                    eprintln!("  {} {}: join error: {}", "✗".red().bold(), item, e);
                                    -1
                                }
                            };
                            // Count non-zero exit codes as failures.
                            // Sentinel -1 already incremented `failed` above.
                            if code != 0 && code != -1 {
                                failed += 1;
                            }
                            done += 1;
                            match code {
                                0 => println!("  {} [{}/{}] {}", "✓".green().bold(), done, n, item),
                                -1 => {} // error already printed
                                c => eprintln!(
                                    "  {} [{}/{}] {} (exit {})",
                                    "✗".red().bold(),
                                    done,
                                    n,
                                    item,
                                    c.to_string().red()
                                ),
                            }
                            if stop_on_error && failed > 0 {
                                eprintln!(
                                    "  {} stop-on-error: aborting after first failure ({}/{} done)",
                                    "⚡".yellow().bold(),
                                    done,
                                    n
                                );
                                break;
                            }
                        }

                        let dur = start_inst_all.elapsed().as_secs_f64();
                        // Record the batch run under the job's name.
                        let summary_cmd =
                            format!("{}  # batch:{n} vars:{}", entry.command, var_map.len());
                        let _ = job::JobManager::record_run(
                            &name,
                            &summary_cmd,
                            None,
                            if failed == 0 { 0 } else { 1 },
                            started_all,
                            dur,
                        );

                        println!();
                        println!("{}", "─".repeat(60).dimmed());
                        if failed == 0 {
                            println!(
                                "  {} All {} items completed successfully.",
                                "✓".green().bold(),
                                done.to_string().green()
                            );
                        } else {
                            eprintln!(
                                "  {} {}/{} items failed.",
                                "✗".red().bold(),
                                failed.to_string().red(),
                                done
                            );
                            return Err(error::OxoError::ExecutionError(format!(
                                "{failed}/{done} batch items failed"
                            )));
                        }
                        println!("{}", "─".repeat(60).dimmed());
                    }
                }

                JobCommands::Edit {
                    name,
                    command: new_command,
                    description: new_description,
                    tags,
                    schedule: new_schedule,
                    clear_schedule,
                } => {
                    let new_tags = if tags.is_empty() { None } else { Some(tags) };
                    job::JobManager::edit(
                        &name,
                        new_command.as_deref(),
                        new_description.as_deref(),
                        false,
                        new_tags,
                        new_schedule.as_deref(),
                        clear_schedule,
                    )?;
                    println!("{} Job '{}' updated.", "✓".green().bold(), name.cyan());
                }

                JobCommands::Rename { from, to } => {
                    job::JobManager::rename(&from, &to)?;
                    println!(
                        "{} Job '{}' renamed to '{}'.",
                        "✓".green().bold(),
                        from.cyan(),
                        to.cyan()
                    );
                }

                JobCommands::Status { name } => {
                    if let Some(ref job_name) = name {
                        // Single job status with recent run history
                        let entry = job::JobManager::find(job_name)?.ok_or_else(|| {
                            error::OxoError::ConfigError(format!(
                                "No job found with name '{job_name}'"
                            ))
                        })?;
                        println!();
                        println!("  {}  {}", "Job:".bold(), entry.name.cyan().bold());
                        println!("  {}  {}", "Command:".bold(), entry.command.green());
                        println!(
                            "  {}  {}",
                            "Status:".bold(),
                            match entry.last_exit_code {
                                None => "never run".dimmed().to_string(),
                                Some(0) => "ok".green().to_string(),
                                Some(c) => format!("failed (exit {c})").red().to_string(),
                            }
                        );
                        println!("  {}  {}", "Run count:".bold(), entry.run_count);
                        if let Some(ref lr) = entry.last_run {
                            println!(
                                "  {}  {}",
                                "Last run:".bold(),
                                lr.format("%Y-%m-%d %H:%M:%S UTC")
                            );
                        }
                        if let Some(ref sched) = entry.schedule {
                            println!("  {}  {}", "Schedule:".bold(), sched.yellow());
                        }

                        // Show last 5 runs from history
                        let runs = job::JobRunStore::load(Some(job_name))?;
                        if !runs.is_empty() {
                            println!();
                            println!("  {} Recent runs:", "◆".cyan());
                            for run in runs.iter().rev().take(5) {
                                let status_icon = if run.exit_code == 0 {
                                    "✓".green().to_string()
                                } else {
                                    "✗".red().to_string()
                                };
                                println!(
                                    "    {} {}  exit={}  {:.1}s{}",
                                    status_icon,
                                    run.started_at.format("%Y-%m-%d %H:%M:%S"),
                                    run.exit_code,
                                    run.duration_secs,
                                    run.server
                                        .as_ref()
                                        .map(|s| format!("  server={s}"))
                                        .unwrap_or_default()
                                );
                            }
                        }
                        println!();
                    } else {
                        // All jobs status summary
                        let entries = job::JobManager::list(None)?;
                        if entries.is_empty() {
                            println!("No jobs saved yet.");
                        } else {
                            println!();
                            println!(
                                "  {:<20} {:<10} {:<6} {}",
                                "JOB".bold(),
                                "STATUS".bold(),
                                "RUNS".bold(),
                                "LAST RUN".bold()
                            );
                            println!("  {}", "─".repeat(60).dimmed());
                            for entry in &entries {
                                let status = match entry.last_exit_code {
                                    None => "never".dimmed().to_string(),
                                    Some(0) => "ok".green().to_string(),
                                    Some(c) => format!("fail({c})").red().to_string(),
                                };
                                let last = entry
                                    .last_run
                                    .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                                    .unwrap_or_else(|| "–".to_string());
                                println!(
                                    "  {:<20} {:<10} {:<6} {}",
                                    entry.name.cyan().to_string(),
                                    status,
                                    entry.run_count,
                                    last.dimmed()
                                );
                            }
                            println!();
                        }
                    }
                }

                JobCommands::History { name, limit } => {
                    let runs = job::JobRunStore::load(name.as_deref())?;
                    if runs.is_empty() {
                        if let Some(ref n) = name {
                            println!("No run history for job '{n}'.");
                        } else {
                            println!("No job run history found.");
                        }
                    } else {
                        println!();
                        if let Some(ref n) = name {
                            println!(
                                "  {} Run history for '{}' (last {}):",
                                "◆".cyan(),
                                n.cyan().bold(),
                                limit
                            );
                        } else {
                            println!(
                                "  {} All job run history (last {} per job):",
                                "◆".cyan(),
                                limit
                            );
                        }
                        println!();
                        // When showing all jobs, group by job name.
                        if name.is_none() {
                            // Collect unique job names in the order they last ran.
                            let mut seen: Vec<String> = Vec::new();
                            let mut seen_set = std::collections::HashSet::new();
                            for run in runs.iter().rev() {
                                if seen_set.insert(run.job_name.clone()) {
                                    seen.push(run.job_name.clone());
                                }
                            }
                            for job_name in seen {
                                let job_runs: Vec<&job::JobRun> =
                                    runs.iter().filter(|r| r.job_name == job_name).collect();
                                println!("  {} '{}'", "◇".cyan(), job_name.cyan().bold());
                                for run in job_runs.iter().rev().take(limit) {
                                    let status_icon = if run.exit_code == 0 {
                                        "✓".green().to_string()
                                    } else {
                                        "✗".red().to_string()
                                    };
                                    println!(
                                        "    {} {}  exit={}  {:.2}s{}",
                                        status_icon,
                                        run.started_at.format("%Y-%m-%d %H:%M:%S UTC"),
                                        run.exit_code,
                                        run.duration_secs,
                                        run.server
                                            .as_ref()
                                            .map(|s| format!("  [{}]", s.cyan()))
                                            .unwrap_or_default()
                                    );
                                    println!("      {}", run.command.dimmed());
                                }
                                println!();
                            }
                        } else {
                            for run in runs.iter().rev().take(limit) {
                                let status_icon = if run.exit_code == 0 {
                                    "✓".green().to_string()
                                } else {
                                    "✗".red().to_string()
                                };
                                println!(
                                    "  {} {}  exit={}  {:.2}s{}",
                                    status_icon,
                                    run.started_at.format("%Y-%m-%d %H:%M:%S UTC"),
                                    run.exit_code,
                                    run.duration_secs,
                                    run.server
                                        .as_ref()
                                        .map(|s| format!("  [{}]", s.cyan()))
                                        .unwrap_or_default()
                                );
                                println!("    {}", run.command.dimmed());
                            }
                            println!();
                        }
                    }
                }

                JobCommands::Schedule { name, cron } => {
                    job::JobManager::set_schedule(&name, cron.as_deref())?;
                    if let Some(ref expr) = cron {
                        println!(
                            "{} Schedule for '{}' set to: {}",
                            "✓".green().bold(),
                            name.cyan(),
                            expr.yellow()
                        );
                        println!(
                            "  Use 'oxo-call job run {}' in a cron job to execute it.",
                            name.cyan()
                        );
                    } else {
                        println!(
                            "{} Schedule for '{}' cleared.",
                            "✓".green().bold(),
                            name.cyan()
                        );
                    }
                }

                JobCommands::Generate {
                    description,
                    name,
                    tags,
                    dry_run,
                } => {
                    use crate::llm::LlmClient;

                    let cfg = base_cfg.clone();
                    let llm = LlmClient::new(cfg);

                    println!("  {} Generating job from description …", "⚙".cyan());

                    let (generated_cmd, explanation) =
                        llm.generate_shell_command(&description).await?;

                    println!();
                    println!("  {}  {}", "Command:".bold(), generated_cmd.green().bold());
                    if !explanation.is_empty() {
                        println!("  {}", "Explanation:".bold());
                        println!();
                        markdown::render_markdown(&explanation);
                    }

                    if dry_run {
                        println!();
                        println!("{}", "  (dry-run — not saved)".yellow());
                        return Ok(());
                    }

                    // Derive a name if not provided
                    let job_name = name.unwrap_or_else(|| {
                        // Slugify: take first 5 words, lowercase, join with '-'
                        let slug = description
                            .split_whitespace()
                            .take(5)
                            .map(|w| {
                                w.chars()
                                    .filter(|c| c.is_alphanumeric() || *c == '-')
                                    .collect::<String>()
                                    .to_lowercase()
                            })
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join("-");
                        // Fallback to a timestamped name if slug is empty
                        if slug.is_empty() {
                            format!("generated-{}", chrono::Utc::now().format("%Y%m%d%H%M%S"))
                        } else {
                            slug
                        }
                    });

                    let now = chrono::Utc::now();
                    let entry = job::JobEntry {
                        name: job_name.clone(),
                        command: generated_cmd.clone(),
                        description: if explanation.is_empty() {
                            Some(description.clone())
                        } else {
                            Some(explanation.clone())
                        },
                        tags,
                        schedule: None,
                        run_count: 0,
                        last_run: None,
                        last_exit_code: None,
                        created_at: now,
                        updated_at: now,
                    };

                    job::JobManager::add(entry)?;
                    println!();
                    println!(
                        "{} Job '{}' saved to your library.",
                        "✓".green().bold(),
                        job_name.cyan()
                    );
                    println!(
                        "  Run with: {}",
                        format!("oxo-call job run {job_name}").bold()
                    );
                }

                JobCommands::Import { name, as_name, all } => {
                    if all {
                        // Collect existing job names to skip duplicates.
                        let existing: std::collections::HashSet<String> =
                            job::JobManager::list(None)?
                                .into_iter()
                                .map(|e| e.name)
                                .collect();
                        let mut imported = 0usize;
                        let mut skipped = 0usize;
                        let now = chrono::Utc::now();
                        for template in job::BUILTIN_JOBS {
                            if existing.contains(template.name) {
                                println!(
                                    "  {} '{}' already exists — skipping.",
                                    "↷".yellow(),
                                    template.name.cyan()
                                );
                                skipped += 1;
                                continue;
                            }
                            let entry = job::JobEntry {
                                name: template.name.to_string(),
                                command: template.command.to_string(),
                                description: Some(template.description.to_string()),
                                tags: template.tags.iter().map(|s| s.to_string()).collect(),
                                schedule: None,
                                run_count: 0,
                                last_run: None,
                                last_exit_code: None,
                                created_at: now,
                                updated_at: now,
                            };
                            job::JobManager::add(entry)?;
                            println!(
                                "  {} Imported '{}'",
                                "✓".green().bold(),
                                template.name.cyan()
                            );
                            imported += 1;
                        }
                        println!();
                        println!(
                            "{} Imported {}, skipped {}.",
                            "→".cyan().bold(),
                            format!("{imported} job(s)").green(),
                            format!("{skipped} already existed").yellow()
                        );
                    } else {
                        let name = name.ok_or_else(|| {
                            error::OxoError::ConfigError(
                                "Provide a template name or use --all to import every template."
                                    .to_string(),
                            )
                        })?;
                        let template = job::BUILTIN_JOBS
                            .iter()
                            .find(|j| j.name == name)
                            .ok_or_else(|| {
                                error::OxoError::ConfigError(format!(
                                    "No built-in job template named '{name}'. \
                                     Run 'oxo-call job list --builtin' to see all templates."
                                ))
                            })?;

                        let final_name = as_name.unwrap_or_else(|| name.clone());
                        let now = chrono::Utc::now();
                        let entry = job::JobEntry {
                            name: final_name.clone(),
                            command: template.command.to_string(),
                            description: Some(template.description.to_string()),
                            tags: template.tags.iter().map(|s| s.to_string()).collect(),
                            schedule: None,
                            run_count: 0,
                            last_run: None,
                            last_exit_code: None,
                            created_at: now,
                            updated_at: now,
                        };
                        job::JobManager::add(entry)?;
                        println!(
                            "{} Built-in job '{}' imported as '{}'.",
                            "✓".green().bold(),
                            name.cyan(),
                            final_name.cyan()
                        );
                        println!(
                            "  Run with: {}",
                            format!("oxo-call job run {final_name}").bold()
                        );
                    }
                }
            }
        }

        Commands::Completion { shell } => {
            let mut cli_app = Cli::command();
            let shell = match shell {
                ShellType::Bash => clap_complete::Shell::Bash,
                ShellType::Zsh => clap_complete::Shell::Zsh,
                ShellType::Fish => clap_complete::Shell::Fish,
                ShellType::Powershell => clap_complete::Shell::PowerShell,
                ShellType::Elvish => clap_complete::Shell::Elvish,
            };
            // Generate into a buffer first so that a broken pipe (e.g. `| head`)
            // does not cause a panic from within clap_complete.
            let mut buf: Vec<u8> = Vec::new();
            clap_complete::generate(shell, &mut cli_app, "oxo-call", &mut buf);
            use std::io::Write as _;
            let _ = std::io::stdout().write_all(&buf);
        }
    }

    Ok(())
}
