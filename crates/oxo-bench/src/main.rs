//! oxo-bench — systematic benchmark CLI for oxo-call.
//!
//! Usage:
//!   oxo-bench workflow         # Benchmark workflow parsing/expansion
//!   oxo-bench simulate         # Simulate omics data for testing
//!   oxo-bench eval-models      # Evaluate LLM models (requires API token)
//!   oxo-bench report <json>    # Render a saved JSON report to Markdown

use clap::{Parser, Subcommand};
use colored::Colorize;
use oxo_bench::{
    bench::{
        llm::{canonical_eval_tasks, ModelBenchConfig},
        workflow::bench_workflow_expand,
    },
    report::{
        print_workflow_report, summarise_by_model, print_model_summary,
        write_workflow_csv, write_scenarios_csv, write_eval_tasks_csv,
    },
    sim::omics::{canonical_scenarios, simulate_scenario},
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "oxo-bench",
    version,
    about = "Systematic benchmark suite for oxo-call — measures accuracy, reproducibility, and LLM model performance",
    long_about = r#"oxo-bench provides a rigorous, reproducible evaluation framework for oxo-call.

Quick start:
  # Benchmark workflow parsing and expansion performance
  oxo-bench workflow

  # Generate synthetic omics data for all canonical scenarios
  oxo-bench simulate --output /tmp/oxo_bench_data

  # Show the evaluation task suite (used for LLM model benchmarks)
  oxo-bench eval-models --list

  # Run LLM model evaluation (requires OXO_CALL_API_TOKEN env var)
  oxo-bench eval-models --model gpt-4o-mini --repeats 5
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Benchmark workflow TOML parsing and DAG expansion for all built-in templates
    Workflow {
        /// Number of benchmark runs to average (default: 100)
        #[arg(short, long, default_value = "100")]
        runs: usize,
        /// Write results to a JSON file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Simulate synthetic omics datasets for all canonical benchmark scenarios
    Simulate {
        /// Output directory for generated data
        #[arg(short, long, default_value = "/tmp/oxo_bench_data")]
        output: PathBuf,
        /// Only simulate a specific scenario by ID (e.g. "rnaseq_3s_pe150")
        #[arg(long)]
        scenario: Option<String>,
        /// List available scenarios and exit
        #[arg(long)]
        list: bool,
    },

    /// Evaluate LLM model accuracy and consistency on bioinformatics tasks
    #[command(name = "eval-models")]
    EvalModels {
        /// LLM model to evaluate (default: gpt-4o-mini)
        #[arg(long, default_value = "gpt-4o-mini")]
        model: String,
        /// Number of repeat calls per task for consistency measurement (default: 3)
        #[arg(long, default_value = "3")]
        repeats: usize,
        /// List evaluation tasks and exit (no API calls made)
        #[arg(long)]
        list: bool,
        /// Write results to a JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render a saved JSON benchmark report to Markdown
    Report {
        /// Path to a JSON report file produced by oxo-bench
        json: PathBuf,
    },

    /// Generate all benchmark CSV files (workflow stats, simulation scenarios, eval tasks)
    ///
    /// Writes three CSV files into the specified output directory:
    ///   bench_workflow.csv   — workflow template parse/expand timings and task counts
    ///   bench_scenarios.csv  — canonical omics simulation scenario metadata
    ///   bench_eval_tasks.csv — LLM evaluation task catalog
    #[command(name = "export-csv")]
    ExportCsv {
        /// Output directory for CSV files (default: docs/)
        #[arg(short, long, default_value = "docs")]
        output: PathBuf,
        /// Number of benchmark runs to average for workflow timings (default: 200)
        #[arg(short, long, default_value = "200")]
        runs: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("{} {e}", "error:".bold().red());
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Workflow { runs, output } => {
            println!(
                "{} Benchmarking workflow parsing & expansion ({runs} runs per template)...",
                "→".cyan().bold()
            );

            let results = bench_workflow_expand(runs);

            println!();
            let mut buf = Vec::new();
            print_workflow_report(&mut buf, &results)?;
            print!("{}", String::from_utf8(buf)?);

            if let Some(path) = output {
                let json = serde_json::to_string_pretty(&results)?;
                std::fs::write(&path, json)?;
                println!(
                    "\n{} Results written to {}",
                    "✓".green().bold(),
                    path.display().to_string().cyan()
                );
            }
        }

        Commands::Simulate { output, scenario, list } => {
            let scenarios = canonical_scenarios();

            if list {
                println!(
                    "{:<30} {:<14} {:<6} {}",
                    "Scenario ID".bold(),
                    "Assay".bold(),
                    "Samples".bold(),
                    "Description".bold()
                );
                println!("{}", "─".repeat(80).dimmed());
                for s in &scenarios {
                    println!(
                        "{:<30} {:<14} {:<6} {}",
                        s.id.cyan(),
                        s.assay,
                        s.samples.len(),
                        s.description
                    );
                }
                return Ok(());
            }

            let to_run: Vec<_> = if let Some(ref id) = scenario {
                let found: Vec<_> = scenarios.iter().filter(|s| &s.id == id).collect();
                if found.is_empty() {
                    anyhow::bail!("Scenario '{}' not found. Use --list to see available scenarios.", id);
                }
                found
            } else {
                scenarios.iter().collect()
            };

            std::fs::create_dir_all(&output)?;
            let total = to_run.len();

            for (i, s) in to_run.iter().enumerate() {
                let scenario_dir = output.join(&s.id);
                println!(
                    "[{}/{}] {} {} ({} samples, {} reads/sample)...",
                    i + 1,
                    total,
                    "→".cyan().bold(),
                    s.id.cyan(),
                    s.samples.len(),
                    s.reads_per_sample,
                );
                let files = simulate_scenario(s, &scenario_dir)?;
                println!(
                    "      {} {} FASTQ pairs → {}",
                    "✓".green().bold(),
                    files.len(),
                    scenario_dir.display().to_string().dimmed()
                );
            }

            println!(
                "\n{} Simulated {} scenario(s) in {}",
                "✓".green().bold(),
                to_run.len(),
                output.display().to_string().cyan()
            );
        }

        Commands::EvalModels { model, repeats, list, output } => {
            let tasks = canonical_eval_tasks();

            if list {
                println!(
                    "{:<16} {:<14} {}",
                    "Tool".bold(),
                    "Category".bold(),
                    "Task description".bold()
                );
                println!("{}", "─".repeat(80).dimmed());
                for t in &tasks {
                    let task_preview: String = t.task.chars().take(56).collect();
                    println!(
                        "{:<16} {:<14} {}",
                        t.tool.cyan(),
                        t.category,
                        task_preview
                    );
                }
                println!("\n{} {} evaluation tasks total", tasks.len(), "→".dimmed());
                return Ok(());
            }

            // Live LLM evaluation is gated behind the API token env var.
            let token = std::env::var("OXO_CALL_API_TOKEN").ok()
                .or_else(|| std::env::var("GITHUB_TOKEN").ok());

            if token.is_none() {
                eprintln!(
                    "{} OXO_CALL_API_TOKEN (or GITHUB_TOKEN) environment variable not set.\n\
                     Set it to run live LLM evaluations:\n\
                     export OXO_CALL_API_TOKEN=<your-token>",
                    "error:".bold().red()
                );
                std::process::exit(1);
            }

            println!(
                "{} LLM model evaluation: {} ({} repeats × {} tasks)",
                "→".cyan().bold(),
                model.cyan(),
                repeats,
                tasks.len()
            );
            println!(
                "{}",
                "Live API evaluation requires the main oxo-call binary.".dimmed()
            );
            println!(
                "{}",
                "Run: cargo run --bin oxo-call -- <commands> for live evaluation.".dimmed()
            );
            println!();

            // Print the evaluation config for reference.
            let config = ModelBenchConfig {
                model: model.clone(),
                n_repeats: repeats,
                temperature: 0.0,
                max_tokens: 512,
            };

            let config_json = serde_json::to_string_pretty(&config)?;
            println!("Evaluation configuration:");
            println!("{config_json}");
            println!();
            println!("Use --list to see all {} evaluation tasks.", tasks.len());

            if let Some(path) = output {
                let plan = serde_json::json!({
                    "config": config,
                    "tasks": tasks.iter().map(|t| serde_json::json!({
                        "tool": t.tool,
                        "task": t.task,
                        "category": t.category,
                        "required_patterns": t.required_patterns,
                    })).collect::<Vec<_>>()
                });
                std::fs::write(&path, serde_json::to_string_pretty(&plan)?)?;
                println!(
                    "{} Evaluation plan written to {}",
                    "✓".green().bold(),
                    path.display().to_string().cyan()
                );
            }
        }

        Commands::Report { json } => {
            let content = std::fs::read_to_string(&json)?;

            // Try to parse as workflow results first.
            if let Ok(wf_results) = serde_json::from_str::<Vec<oxo_bench::bench::workflow::BenchWorkflowResult>>(&content) {
                print_workflow_report(&mut std::io::stdout(), &wf_results)?;
                return Ok(());
            }

            // Try as model results.
            if let Ok(model_results) = serde_json::from_str::<Vec<oxo_bench::bench::llm::ModelBenchResult>>(&content) {
                let mut stdout = std::io::stdout();
                let summaries = summarise_by_model(&model_results);
                print_model_summary(&mut stdout, &summaries)?;
                return Ok(());
            }

            anyhow::bail!(
                "Could not parse '{}' as a workflow or model benchmark report.",
                json.display()
            );
        }

        Commands::ExportCsv { output, runs } => {
            std::fs::create_dir_all(&output)?;

            // ── 1. Workflow benchmark CSV ─────────────────────────────────
            let wf_csv_path = output.join("bench_workflow.csv");
            println!(
                "{} Benchmarking {} workflow templates ({runs} runs each)...",
                "→".cyan().bold(),
                oxo_bench::bench::workflow::ALL_BENCH_WORKFLOWS.len()
            );
            let wf_results = bench_workflow_expand(runs);
            {
                let mut f = std::fs::File::create(&wf_csv_path)?;
                write_workflow_csv(&mut f, &wf_results)?;
            }
            println!(
                "{} {}",
                "✓".green().bold(),
                wf_csv_path.display().to_string().cyan()
            );

            // Print a quick summary to stdout.
            println!();
            let mut buf = Vec::new();
            print_workflow_report(&mut buf, &wf_results)?;
            print!("{}", String::from_utf8(buf)?);
            println!();

            // ── 2. Simulation scenarios CSV ───────────────────────────────
            let sc_csv_path = output.join("bench_scenarios.csv");
            let scenarios = canonical_scenarios();
            {
                let mut f = std::fs::File::create(&sc_csv_path)?;
                write_scenarios_csv(&mut f, &scenarios)?;
            }
            println!(
                "{} {}  ({} scenarios)",
                "✓".green().bold(),
                sc_csv_path.display().to_string().cyan(),
                scenarios.len()
            );

            // ── 3. Eval tasks CSV ─────────────────────────────────────────
            let tasks_csv_path = output.join("bench_eval_tasks.csv");
            let tasks = canonical_eval_tasks();
            {
                let mut f = std::fs::File::create(&tasks_csv_path)?;
                write_eval_tasks_csv(&mut f, &tasks)?;
            }
            println!(
                "{} {}  ({} tasks)",
                "✓".green().bold(),
                tasks_csv_path.display().to_string().cyan(),
                tasks.len()
            );

            println!(
                "\n{} All CSVs written to {}/",
                "✓".green().bold(),
                output.display().to_string().cyan()
            );
        }
    }

    Ok(())
}
