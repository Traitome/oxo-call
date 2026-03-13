mod cli;
mod config;
mod docs;
mod engine;
mod error;
mod handlers;
mod history;
mod index;
mod license;
mod llm;
mod runner;
mod sanitize;
mod skill;
mod workflow;

use clap::Parser;
use cli::{
    Cli, Commands, ConfigCommands, DocsCommands, HistoryCommands, IndexCommands, LicenseCommands,
    SkillCommands, WorkflowCommands,
};
use colored::Colorize;
use handlers::{config_verify_suggestions, print_index_table, with_source};

#[cfg_attr(not(target_arch = "wasm32"), tokio::main)]
#[cfg_attr(target_arch = "wasm32", tokio::main(flavor = "current_thread"))]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("{} {}", "error:".bold().red(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> error::Result<()> {
    // Commands that are permitted without a valid license file.
    // `--help` and `--version` are handled by clap before reaching this function.
    let license_exempt = matches!(cli.command, Commands::License { .. });

    if !license_exempt {
        let license_path = cli.license.as_deref();
        if let Err(e) = license::load_and_verify(license_path) {
            eprintln!("{} {}", "license error:".bold().red(), e);
            std::process::exit(2);
        }
    }

    match cli.command {
        Commands::Run { tool, task, ask } => {
            let cfg = config::Config::load()?;
            let runner = runner::Runner::new(cfg);
            runner.run(&tool, &task, ask).await?;
        }

        Commands::DryRun { tool, task } => {
            let cfg = config::Config::load()?;
            let runner = runner::Runner::new(cfg);
            runner.dry_run(&tool, &task).await?;
        }

        Commands::Index { command } => match command {
            IndexCommands::Add {
                tool,
                url,
                file,
                dir,
            } => {
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
                let mgr = index::IndexManager::new(cfg);
                mgr.remove(&tool)?;
                println!(
                    "{} Removed '{}' from index",
                    "✓".green().bold(),
                    tool.cyan()
                );
            }
            IndexCommands::Update { tool, url } => {
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
                let mgr = index::IndexManager::new(cfg);
                mgr.remove(&tool)?;
                println!(
                    "{} Removed '{}' from documentation index",
                    "✓".green().bold(),
                    tool.cyan()
                );
            }
            DocsCommands::Update { tool, url } => {
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
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
                let cfg = config::Config::load()?;
                let fetcher = docs::DocsFetcher::new(cfg);
                let path = fetcher.cache_path(&tool)?;
                println!("{}", path.display());
            }
        },

        Commands::Config { command } => match command {
            ConfigCommands::Set { key, value } => {
                let mut cfg = config::Config::load()?;
                cfg.set(&key, &value)?;
                cfg.save()?;
                println!("{} Set '{}' = '{}'", "✓".green().bold(), key.cyan(), value);
            }
            ConfigCommands::Get { key } => {
                let cfg = config::Config::load()?;
                let value = cfg.get(&key)?;
                println!("{}", value);
            }
            ConfigCommands::Show => {
                let cfg = config::Config::load()?;
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
                } else {
                    format!(
                        "{} [{}]",
                        "not set".red(),
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
            }
            ConfigCommands::Verify => {
                let cfg = config::Config::load()?;
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
                    } else {
                        format!(
                            "{} [{}]",
                            "not set".red(),
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
                    "{:<8} {:<12} {:<8} {:<28} {}",
                    "Status".bold(),
                    "Tool".bold(),
                    "Exit".bold(),
                    "Time".bold(),
                    "Command".bold()
                );
                println!("{}", "─".repeat(100).dimmed());
                for e in &entries {
                    let status = if e.dry_run {
                        "dry-run".yellow().to_string()
                    } else if e.exit_code == 0 {
                        "ok".green().to_string()
                    } else {
                        "failed".red().to_string()
                    };
                    let cmd_short = if e.command.len() > 45 {
                        format!("{}...", &e.command[..45])
                    } else {
                        e.command.clone()
                    };
                    println!(
                        "{:<8} {:<12} {:<8} {:<28} {}",
                        status,
                        e.tool.cyan().to_string(),
                        e.exit_code,
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
                            println!("         {}", format!("[{}]", parts.join(", ")).dimmed());
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
            let cfg = config::Config::load()?;
            let mgr = skill::SkillManager::new(cfg);

            match command {
                SkillCommands::List => {
                    let skills = mgr.list_all();
                    if skills.is_empty() {
                        println!("{}", "No skills found.".yellow());
                        return Ok(());
                    }
                    println!(
                        "{:<20} {:<12} {}",
                        "Tool".bold(),
                        "Source".bold(),
                        "Description".bold()
                    );
                    println!("{}", "─".repeat(70).dimmed());
                    for (name, source) in &skills {
                        let desc = mgr
                            .load(name)
                            .map(|s| s.meta.description)
                            .unwrap_or_default();
                        let source_colored = match source.as_str() {
                            "built-in" => source.dimmed().to_string(),
                            "community" => source.cyan().to_string(),
                            "user" => source.green().bold().to_string(),
                            _ => source.clone(),
                        };
                        println!("{:<20} {:<12} {}", name.cyan(), source_colored, desc);
                    }
                    println!(
                        "\n{} skills available. Use 'oxo-call skill show <tool>' to inspect one.",
                        skills.len()
                    );
                }

                SkillCommands::Show { tool } => {
                    if let Some(skill) = mgr.load(&tool) {
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

                SkillCommands::Create { tool, output } => {
                    let template = skill::SkillManager::create_template(&tool);
                    match output {
                        Some(path) => {
                            std::fs::write(&path, &template)?;
                            println!(
                                "{} Template written to '{}'",
                                "✓".green().bold(),
                                path.display()
                            );
                        }
                        None => print!("{template}"),
                    }
                }

                SkillCommands::Path => {
                    let path = mgr.user_skill_dir()?;
                    println!("{}", path.display());
                }
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

            #[cfg(not(target_arch = "wasm32"))]
            WorkflowCommands::RunWorkflow { file } => {
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
            }

            #[cfg(target_arch = "wasm32")]
            WorkflowCommands::RunWorkflow { .. } => {
                eprintln!(
                    "{} 'workflow run' is not supported on WebAssembly.",
                    "error:".red().bold()
                );
                std::process::exit(1);
            }

            #[cfg(not(target_arch = "wasm32"))]
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

            #[cfg(target_arch = "wasm32")]
            WorkflowCommands::DryRunWorkflow { .. } => {
                eprintln!(
                    "{} 'workflow dry-run' is not supported on WebAssembly.",
                    "error:".red().bold()
                );
                std::process::exit(1);
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

            #[cfg(not(target_arch = "wasm32"))]
            WorkflowCommands::Generate {
                task,
                engine: engine_name,
                output,
            } => {
                let cfg = config::Config::load()?;
                let label = match engine_name.as_str() {
                    "snakemake" => "Snakemake",
                    "nextflow" => "Nextflow DSL2",
                    _ => "native (.oxo.toml)",
                };
                let spinner =
                    runner::make_spinner(&format!("Generating {label} workflow with LLM..."));
                let wf = match workflow::generate_workflow(&cfg, &task, &engine_name).await {
                    Ok(w) => {
                        spinner.finish_and_clear();
                        w
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
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
                            println!("  {}", wf.explanation);
                        }
                    }
                    None => {
                        workflow::print_generated_workflow(&wf);
                    }
                }
            }

            #[cfg(target_arch = "wasm32")]
            WorkflowCommands::Generate { .. } => {
                eprintln!(
                    "{} 'workflow generate' is not supported on WebAssembly.",
                    "error:".red().bold()
                );
                std::process::exit(1);
            }

            #[cfg(not(target_arch = "wasm32"))]
            WorkflowCommands::Infer {
                task,
                data,
                engine: engine_name,
                output,
                run,
            } => {
                let cfg = config::Config::load()?;

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
                let spinner = runner::make_spinner(&format!(
                    "Generating {label} workflow from data context with LLM..."
                ));
                let wf = match workflow::infer_workflow(&cfg, &task, &data, &engine_name).await {
                    Ok(w) => {
                        spinner.finish_and_clear();
                        w
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
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
                            println!("  {}", wf.explanation);
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

            #[cfg(target_arch = "wasm32")]
            WorkflowCommands::Infer { .. } => {
                eprintln!(
                    "{} 'workflow infer' is not supported on WebAssembly.",
                    "error:".red().bold()
                );
                std::process::exit(1);
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
    }

    Ok(())
}
