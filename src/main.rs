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
mod mcp;
mod runner;
mod sanitize;
mod server;
mod skill;
mod workflow;

use clap::{CommandFactory, Parser};
use cli::{
    Cli, Commands, ConfigCommands, DocsCommands, HistoryCommands, IndexCommands, LicenseCommands,
    ServerCommands, ShellType, SkillCommands, SkillMcpCommands, WorkflowCommands,
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

    let verbose = cli.verbose;

    match cli.command {
        Commands::Run {
            tool,
            task,
            ask,
            model,
            no_cache,
            json,
            verify,
            optimize_task,
        } => {
            let mut cfg = config::Config::load()?;
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }
            let runner = runner::Runner::new(cfg)
                .with_verbose(verbose)
                .with_no_cache(no_cache)
                .with_verify(verify)
                .with_optimize_task(optimize_task);
            runner.run(&tool, &task, ask, json).await?;
        }

        Commands::DryRun {
            tool,
            task,
            model,
            no_cache,
            json,
            optimize_task,
        } => {
            let mut cfg = config::Config::load()?;
            if let Some(ref m) = model {
                cfg.llm.model = Some(m.clone());
            }
            let runner = runner::Runner::new(cfg)
                .with_verbose(verbose)
                .with_no_cache(no_cache)
                .with_optimize_task(optimize_task);
            runner.dry_run(&tool, &task, json).await?;
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
            let mut cfg = config::Config::load()?;
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

            #[cfg(not(target_arch = "wasm32"))]
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
                    let cfg = config::Config::load()?;
                    engine::verify_workflow_results(&def, &cfg).await;
                }
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

        #[cfg(not(target_arch = "wasm32"))]
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
                    let cfg = config::Config::load()?;
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
                    let cfg = config::Config::load()?;
                    let mut mgr = server::ServerManager::new(cfg);
                    mgr.remove(&name)?;
                    println!("{} Removed server '{}'", "✓".green().bold(), name.cyan());
                }

                ServerCommands::List => {
                    let cfg = config::Config::load()?;
                    let mgr = server::ServerManager::new(cfg);
                    let hosts = mgr.list();
                    if hosts.is_empty() {
                        println!(
                            "{}",
                            "No servers registered. Use 'oxo-call server add' to register one."
                                .yellow()
                        );
                    } else {
                        println!(
                            "{:<16} {:<24} {:<14} {:<12} {}",
                            "Name".bold(),
                            "Host".bold(),
                            "Type".bold(),
                            "Scheduler".bold(),
                            "User".bold()
                        );
                        println!("{}", "─".repeat(80).dimmed());
                        for h in hosts {
                            println!(
                                "{:<16} {:<24} {:<14} {:<12} {}",
                                h.name.cyan(),
                                h.ssh_dest(),
                                h.server_type.to_string(),
                                h.scheduler.as_deref().unwrap_or("—"),
                                h.user.as_deref().unwrap_or("(current)")
                            );
                        }
                        println!("\n{} server(s) registered.", hosts.len());
                    }
                }

                ServerCommands::Status { name } => {
                    let cfg = config::Config::load()?;
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

                ServerCommands::SshConfig => {
                    let entries = server::parse_ssh_config();
                    if entries.is_empty() {
                        println!(
                            "{}",
                            "No hosts found in ~/.ssh/config (or file does not exist).".yellow()
                        );
                        return Ok(());
                    }
                    println!(
                        "{} Found {} host(s) in ~/.ssh/config:",
                        "→".cyan().bold(),
                        entries.len()
                    );
                    println!();
                    println!(
                        "{:<20} {:<28} {:<12} {}",
                        "Alias".bold(),
                        "HostName".bold(),
                        "User".bold(),
                        "Port".bold()
                    );
                    println!("{}", "─".repeat(70).dimmed());
                    for e in &entries {
                        println!(
                            "{:<20} {:<28} {:<12} {}",
                            e.alias.cyan(),
                            e.hostname.as_deref().unwrap_or("—"),
                            e.user.as_deref().unwrap_or("—"),
                            e.port
                                .map(|p| p.to_string())
                                .unwrap_or_else(|| "22".to_string())
                        );
                    }
                    println!();
                    println!(
                        "To register a host: {}",
                        "oxo-call server add <name> --host <hostname> --type <workstation|hpc>"
                            .bold()
                    );
                }

                ServerCommands::Run {
                    server: server_name,
                    tool,
                    task,
                    model,
                } => {
                    let cfg = config::Config::load()?;
                    let mgr = server::ServerManager::new(cfg.clone());
                    let host = mgr.find(&server_name).ok_or_else(|| {
                    error::OxoError::ConfigError(format!(
                        "No server found with name '{server_name}'. Run 'oxo-call server list'."
                    ))
                })?.clone();

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

                    // Generate command with LLM (dry-run locally to preview)
                    println!(
                        "{} Generating command for '{}' on server '{}'...",
                        "→".cyan().bold(),
                        tool.cyan(),
                        server_name.cyan()
                    );
                    let mut run_cfg = cfg;
                    if let Some(ref m) = model {
                        run_cfg.llm.model = Some(m.clone());
                    }
                    let runner_inst = runner::Runner::new(run_cfg).with_verbose(verbose);
                    runner_inst.dry_run(&tool, &task, false).await?;

                    println!(
                        "\n{} To execute on '{}': ssh {} '<generated-command>'",
                        "→".cyan().bold(),
                        server_name.cyan(),
                        host.ssh_dest()
                    );
                }

                ServerCommands::DryRun {
                    server: server_name,
                    tool,
                    task,
                    model,
                } => {
                    let cfg = config::Config::load()?;
                    let mgr = server::ServerManager::new(cfg.clone());
                    let host = mgr.find(&server_name).ok_or_else(|| {
                    error::OxoError::ConfigError(format!(
                        "No server found with name '{server_name}'. Run 'oxo-call server list'."
                    ))
                })?.clone();

                    let mut run_cfg = cfg;
                    if let Some(ref m) = model {
                        run_cfg.llm.model = Some(m.clone());
                    }
                    let runner_inst = runner::Runner::new(run_cfg).with_verbose(verbose);
                    runner_inst.dry_run(&tool, &task, false).await?;

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

        #[cfg(target_arch = "wasm32")]
        Commands::Server { .. } => {
            eprintln!(
                "{} 'server' commands are not supported on WebAssembly.",
                "error:".red().bold()
            );
            std::process::exit(1);
        }

        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            let shell = match shell {
                ShellType::Bash => clap_complete::Shell::Bash,
                ShellType::Zsh => clap_complete::Shell::Zsh,
                ShellType::Fish => clap_complete::Shell::Fish,
                ShellType::Powershell => clap_complete::Shell::PowerShell,
                ShellType::Elvish => clap_complete::Shell::Elvish,
            };
            clap_complete::generate(shell, &mut cmd, "oxo-call", &mut std::io::stdout());
        }
    }

    Ok(())
}
