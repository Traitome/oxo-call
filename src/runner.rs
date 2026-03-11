use crate::config::Config;
use crate::docs::DocsFetcher;
use crate::error::{OxoError, Result};
use crate::history::{HistoryEntry, HistoryStore};
use crate::llm::{LlmClient, LlmCommandSuggestion};
use chrono::Utc;
use colored::Colorize;
use std::process::Command;
use uuid::Uuid;

pub struct Runner {
    fetcher: DocsFetcher,
    llm: LlmClient,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Runner {
            fetcher: DocsFetcher::new(config.clone()),
            llm: LlmClient::new(config),
        }
    }

    /// Resolve documentation for the tool
    async fn resolve_docs(&self, tool: &str) -> Result<String> {
        let docs = self.fetcher.fetch(tool).await?;
        Ok(docs.combined())
    }

    /// Core logic: fetch docs → call LLM → return suggestion
    async fn prepare(
        &self,
        tool: &str,
        task: &str,
    ) -> Result<LlmCommandSuggestion> {
        let docs = self.resolve_docs(tool).await?;
        let suggestion = self.llm.suggest_command(tool, &docs, task).await?;
        Ok(suggestion)
    }

    /// dry-run: show the command that would be executed without running it
    pub async fn dry_run(&self, tool: &str, task: &str) -> Result<()> {
        println!("{}", "Fetching documentation...".dimmed());
        let suggestion = self.prepare(tool, task).await?;

        let full_cmd = build_command_string(tool, &suggestion.args);

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Tool:".bold(), tool.cyan());
        println!("  {} {}", "Task:".bold(), task);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Command (dry-run):".bold().yellow());
        println!("  {}", full_cmd.green().bold());
        println!();
        println!("  {}", "Explanation:".bold());
        println!("  {}", suggestion.explanation);
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {}", "Use 'oxo-call run' to execute this command.".dimmed());

        Ok(())
    }

    /// run: execute the command for real
    pub async fn run(
        &self,
        tool: &str,
        task: &str,
        yes: bool,
    ) -> Result<()> {
        println!("{}", "Fetching documentation...".dimmed());
        let suggestion = self.prepare(tool, task).await?;

        let full_cmd = build_command_string(tool, &suggestion.args);

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Tool:".bold(), tool.cyan());
        println!("  {} {}", "Task:".bold(), task);
        println!("{}", "─".repeat(60).dimmed());
        println!();
        println!("  {}", "Generated command:".bold().green());
        println!("  {}", full_cmd.green().bold());
        println!();
        println!("  {}", "Explanation:".bold());
        println!("  {}", suggestion.explanation);
        println!();

        if !yes {
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

        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("  {} {}", "Running:".bold(), full_cmd.cyan());
        println!("{}", "─".repeat(60).dimmed());
        println!();

        let status = Command::new(tool)
            .args(&suggestion.args)
            .status()
            .map_err(|e| OxoError::ToolNotFound(format!("{tool}: {e}")))?;

        let exit_code = status.code().unwrap_or(-1);
        let success = status.success();

        // Record in history
        let entry = HistoryEntry {
            id: Uuid::new_v4().to_string(),
            tool: tool.to_string(),
            task: task.to_string(),
            command: full_cmd.clone(),
            exit_code,
            executed_at: Utc::now(),
            dry_run: false,
        };
        let _ = HistoryStore::append(entry);

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
            // Quote arguments that contain spaces
            if a.contains(' ') {
                format!("'{a}'")
            } else {
                a.clone()
            }
        })
        .collect();
    format!("{tool} {}", args_str.join(" "))
}
