//! Batch execution implementation.
//!
//! Contains methods for running commands across multiple input items
//! in parallel with configurable concurrency.

use crate::error::{OxoError, Result};
use crate::history::{CommandProvenance, HistoryEntry, HistoryStore};
use crate::job;
use crate::markdown;
use chrono::Utc;
use colored::Colorize;
use std::sync::Arc;
use uuid::Uuid;

use super::core::Runner;
use super::utils::{build_command_string, detect_tool_version};

/// Trait for batch execution methods.
pub(crate) trait BatchRunner {
    async fn run_batch(&self, tool: &str, task: &str, json: bool) -> Result<()>;
    async fn dry_run_batch(&self, tool: &str, task: &str, json: bool) -> Result<()>;
}

impl BatchRunner for Runner {
    /// Execute the LLM-generated command template for every input item.
    ///
    /// The command template is obtained with a single LLM call; `{item}` (and
    /// other placeholders) are then substituted for each item before execution.
    /// Up to `self.jobs` items are run concurrently.
    ///
    /// When `self.stop_on_error` is true, remaining handles are aborted after
    /// the first failure, and the batch exits immediately with an error.
    async fn run_batch(&self, tool: &str, task: &str, json: bool) -> Result<()> {
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
                println!();
                markdown::render_markdown(&result.suggestion.explanation);
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
                let status = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .status()
                    .await
                    .map_err(|e| {
                        OxoError::ExecutionError(format!("failed to run '{item_label}': {e}"))
                    })?;
                Ok(status.code().unwrap_or(-1))
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
            // Count non-zero exit codes as failures
            if code != 0 && code != -1 {
                failed += 1;
            }
            done += 1;
            if !json {
                match code {
                    0 => println!("  {} [{}/{}] {}", "✓".green().bold(), done, n, item),
                    -1 => {}
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
                    cache_hit: None,
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
            println!();
            markdown::render_markdown(&result.suggestion.explanation);
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

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use super::super::core::Runner;

    /// Verify that Runner can be constructed and has the expected defaults for batch-related fields.
    #[test]
    fn test_runner_batch_field_defaults() {
        let r = Runner::new(Config::default());
        assert!(r.input_items.is_empty(), "input_items should start empty");
        assert_eq!(r.jobs, 1, "default jobs should be 1");
        assert!(!r.stop_on_error, "stop_on_error should default to false");
    }

    #[test]
    fn test_runner_batch_with_items_and_jobs() {
        let mut r = Runner::new(Config::default());
        let items = vec!["a.bam".to_string(), "b.bam".to_string()];
        r.with_input_items(items.clone()).with_jobs(4).with_stop_on_error(true);
        assert_eq!(r.input_items, items);
        assert_eq!(r.jobs, 4);
        assert!(r.stop_on_error);
    }

    #[test]
    fn test_runner_with_jobs_many_values() {
        let cases = vec![(0usize, 1usize), (1, 1), (8, 8), (64, 64)];
        for (input, expected) in cases {
            let mut r = Runner::new(Config::default());
            r.with_jobs(input);
            assert_eq!(r.jobs, expected);
        }
    }
}
