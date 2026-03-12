mod cli;
mod config;
mod docs;
mod error;
mod history;
mod index;
mod license;
mod llm;
mod runner;
mod skill;

use clap::Parser;
use cli::{
    Cli, Commands, ConfigCommands, DocsCommands, HistoryCommands, IndexCommands, LicenseCommands,
    SkillCommands,
};
use colored::Colorize;

#[tokio::main]
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
        Commands::Run { tool, task, yes } => {
            let cfg = config::Config::load()?;
            let runner = runner::Runner::new(cfg);
            runner.run(&tool, &task, yes).await?;
        }

        Commands::DryRun { tool, task } => {
            let cfg = config::Config::load()?;
            let runner = runner::Runner::new(cfg);
            runner.dry_run(&tool, &task).await?;
        }

        Commands::Index { command } => match command {
            IndexCommands::Add { tool, url } => {
                let cfg = config::Config::load()?;
                let mgr = index::IndexManager::new(cfg);
                let entry = mgr.add(&tool, url.as_deref()).await?;
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
                        let entry = mgr.add(&t, url.as_deref()).await?;
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
                                "No tools in index. Use 'oxo-call index add <tool>' first."
                                    .yellow()
                            );
                            return Ok(());
                        }
                        let tools: Vec<String> =
                            entries.iter().map(|e| e.tool_name.clone()).collect();
                        println!("Updating {} tool(s)...", tools.len());
                        for t in &tools {
                            match mgr.add(t, None).await {
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
                        "No tools indexed yet. Use 'oxo-call index add <tool>' to index a tool."
                            .yellow()
                    );
                    return Ok(());
                }
                println!(
                    "{:<20} {:<15} {:<12} {}",
                    "Tool".bold(),
                    "Version".bold(),
                    "Size".bold(),
                    "Indexed At".bold()
                );
                println!("{}", "─".repeat(70).dimmed());
                for e in &entries {
                    println!(
                        "{:<20} {:<15} {:<12} {}",
                        e.tool_name.cyan(),
                        e.version.as_deref().unwrap_or("-"),
                        format!("{} B", e.doc_size_bytes),
                        e.indexed_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                }
            }
        },

        Commands::Docs { command } => match command {
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
                let fetcher = docs::DocsFetcher::new(cfg);
                let spinner =
                    runner::make_spinner(&format!("Fetching remote documentation for '{tool}'..."));
                let content = match fetcher.fetch_remote(&tool, &url).await {
                    Ok(c) => {
                        spinner.finish_and_clear();
                        c
                    }
                    Err(e) => {
                        spinner.finish_and_clear();
                        return Err(e);
                    }
                };
                fetcher.save_cache(&tool, &content)?;
                println!(
                    "{} Saved {} bytes of documentation for '{}'",
                    "✓".green().bold(),
                    content.len(),
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
                println!("  {} ", "Effective settings:".bold());
                let token_status = if cfg.effective_api_token().is_some() {
                    "set (from config or environment)".green().to_string()
                } else {
                    "not set".red().to_string()
                };
                println!("  {:<25} {}", "effective api_token", token_status);
                println!(
                    "  {:<25} {}",
                    "effective api_base",
                    cfg.effective_api_base()
                );
                println!("  {:<25} {}", "effective model", cfg.effective_model());
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
                                lic.payload.issued_at,
                                lic.payload.perpetual
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
    }

    Ok(())
}
