//! oxo-bench — systematic benchmark CLI for oxo-call.
//!
//! Usage:
//!   oxo-bench workflow         # Benchmark workflow parsing/expansion
//!   oxo-bench simulate         # Simulate omics data for testing
//!   oxo-bench eval-models      # Evaluate LLM models (requires API token)
//!   oxo-bench report <json>    # Render a saved JSON report to Markdown
//!   oxo-bench generate         # Generate reference commands & usage descriptions
//!   oxo-bench eval             # Run full benchmark evaluation across models
//!   oxo-bench summary          # Print cross-model comparison summary

use clap::{Parser, Subcommand};
use colored::Colorize;
use oxo_bench::{
    bench::{
        llm::{ModelBenchConfig, canonical_eval_tasks},
        runner::{
            ModelAggResult, OxoCallGenerator, TrialResult, aggregate_results, run_benchmark,
            run_mock_benchmark, summarise_by_tool, write_model_agg_csv,
            write_tool_model_summary_csv, write_trials_csv,
        },
        scenario::{
            Scenario, UsageDescription, generate_descriptions, generate_scenarios,
            load_skills_from_dir, write_descriptions_csv, write_scenarios_csv,
        },
        workflow::bench_workflow_expand,
    },
    config::BenchConfig,
    report::{
        print_model_summary, print_workflow_report, summarise_by_model, write_eval_tasks_csv,
        write_scenarios_csv as write_simulation_scenarios_csv, write_workflow_csv,
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
  # Generate reference commands and usage descriptions from skill files
  oxo-bench generate --skills-dir skills/ --output bench_data/

  # Run full benchmark evaluation with multiple models
  oxo-bench eval --config bench_config.toml

  # View cross-model comparison summary
  oxo-bench summary --results-dir bench_results/

  # Benchmark workflow parsing and expansion performance
  oxo-bench workflow

  # Generate synthetic omics data for all canonical scenarios
  oxo-bench simulate --output /tmp/oxo_bench_data

  # Show the evaluation task suite (used for LLM model benchmarks)
  oxo-bench eval-models --list
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

    // ── New commands ─────────────────────────────────────────────────────────
    /// Generate reference command scenarios and usage descriptions from skill files
    ///
    /// Parses all built-in skill documentation (skills/*.md), extracts or
    /// synthesises 10 usage scenarios per tool, then generates 10 diverse
    /// English descriptions per scenario (simulating users of different
    /// experience levels).
    ///
    /// Outputs two CSV files:
    ///   reference_commands.csv  — 10 scenarios × N tools
    ///   usage_descriptions.csv  — 100 descriptions × N tools
    Generate {
        /// Path to the skills directory
        #[arg(long, default_value = "skills")]
        skills_dir: PathBuf,
        /// Output directory for CSVs
        #[arg(short, long, default_value = "bench_data")]
        output: PathBuf,
        /// Limit to specific tools (comma-separated, e.g. "samtools,bwa,fastp")
        #[arg(long)]
        tools: Option<String>,
    },

    /// Run full benchmark evaluation across one or more LLM models
    ///
    /// Reads reference_commands.csv and usage_descriptions.csv (from the
    /// `generate` step), calls `oxo-call dry-run --json` for each description,
    /// and compares the generated command with the reference.
    ///
    /// Supports multiple models via a TOML configuration file.  Each model
    /// can be evaluated in serial or parallel, with configurable repeat counts.
    Eval {
        /// Path to benchmark config TOML (generates default if missing)
        #[arg(short, long, default_value = "bench_config.toml")]
        config: PathBuf,
        /// Directory containing reference_commands.csv and usage_descriptions.csv
        #[arg(long, default_value = "bench_data")]
        data_dir: PathBuf,
        /// Output directory for result CSVs
        #[arg(short, long, default_value = "bench_results")]
        output: PathBuf,
        /// Path to the oxo-call binary (default: auto-detect)
        #[arg(long)]
        oxo_call: Option<String>,
        /// Limit to specific tools (comma-separated)
        #[arg(long)]
        tools: Option<String>,
        /// Override repeat count from config
        #[arg(long)]
        repeats: Option<usize>,
        /// Use mock generator instead of real oxo-call (for offline testing).
        /// Produces deterministic results with model-specific perturbation
        /// rates so that different models show different accuracy levels.
        #[arg(long)]
        mock: bool,
    },

    /// Print cross-model comparison summary from benchmark results
    Summary {
        /// Directory containing benchmark result CSVs
        #[arg(short, long, default_value = "bench_results")]
        results_dir: PathBuf,
    },

    /// Generate a default benchmark configuration file
    #[command(name = "init-config")]
    InitConfig {
        /// Path to write the config file
        #[arg(short, long, default_value = "bench_config.toml")]
        output: PathBuf,
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

        Commands::Simulate {
            output,
            scenario,
            list,
        } => {
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
                    anyhow::bail!(
                        "Scenario '{}' not found. Use --list to see available scenarios.",
                        id
                    );
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

        Commands::EvalModels {
            model,
            repeats,
            list,
            output,
        } => {
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
                    println!("{:<16} {:<14} {}", t.tool.cyan(), t.category, task_preview);
                }
                println!("\n{} {} evaluation tasks total", tasks.len(), "→".dimmed());
                return Ok(());
            }

            // Live LLM evaluation is gated behind the API token env var.
            let token = std::env::var("OXO_CALL_API_TOKEN")
                .ok()
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
            if let Ok(wf_results) = serde_json::from_str::<
                Vec<oxo_bench::bench::workflow::BenchWorkflowResult>,
            >(&content)
            {
                print_workflow_report(&mut std::io::stdout(), &wf_results)?;
                return Ok(());
            }

            // Try as model results.
            if let Ok(model_results) =
                serde_json::from_str::<Vec<oxo_bench::bench::llm::ModelBenchResult>>(&content)
            {
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
                write_simulation_scenarios_csv(&mut f, &scenarios)?;
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

        // ── New commands ─────────────────────────────────────────────────────
        Commands::Generate {
            skills_dir,
            output,
            tools,
        } => {
            cmd_generate(&skills_dir, &output, tools.as_deref())?;
        }

        Commands::Eval {
            config,
            data_dir,
            output,
            oxo_call,
            tools,
            repeats,
            mock,
        } => {
            cmd_eval(
                &config,
                &data_dir,
                &output,
                oxo_call,
                tools.as_deref(),
                repeats,
                mock,
            )?;
        }

        Commands::Summary { results_dir } => {
            cmd_summary(&results_dir)?;
        }

        Commands::InitConfig { output } => {
            BenchConfig::write_default(&output)?;
            println!(
                "{} Default config written to {}",
                "✓".green().bold(),
                output.display().to_string().cyan()
            );
        }
    }

    Ok(())
}

// ── Generate command ─────────────────────────────────────────────────────────

fn cmd_generate(
    skills_dir: &std::path::Path,
    output_dir: &std::path::Path,
    tools_filter: Option<&str>,
) -> anyhow::Result<()> {
    if !skills_dir.exists() {
        anyhow::bail!(
            "Skills directory not found: {}\n\
             Hint: run from the repository root or use --skills-dir to specify the path.",
            skills_dir.display()
        );
    }

    std::fs::create_dir_all(output_dir)?;

    println!(
        "{} Loading skills from {} ...",
        "→".cyan().bold(),
        skills_dir.display().to_string().cyan()
    );

    let mut skills = load_skills_from_dir(skills_dir)?;

    // Apply tool filter if specified.
    if let Some(filter) = tools_filter {
        let allowed: std::collections::HashSet<&str> = filter.split(',').collect();
        skills.retain(|s| allowed.contains(s.name.as_str()));
    }

    println!(
        "  {} skill files loaded ({} with examples)",
        skills.len(),
        skills.iter().filter(|s| !s.examples.is_empty()).count()
    );

    // ── Generate scenarios ────────────────────────────────────────────────
    let mut all_scenarios: Vec<Scenario> = Vec::new();
    for skill in &skills {
        let scenarios = generate_scenarios(skill);
        all_scenarios.extend(scenarios);
    }

    let ref_csv_path = output_dir.join("reference_commands.csv");
    {
        let mut f = std::fs::File::create(&ref_csv_path)?;
        write_scenarios_csv(&mut f, &all_scenarios)?;
    }
    println!(
        "{} {} ({} scenarios across {} tools)",
        "✓".green().bold(),
        ref_csv_path.display().to_string().cyan(),
        all_scenarios.len(),
        skills.len()
    );

    // ── Generate descriptions ─────────────────────────────────────────────
    let mut all_descriptions: Vec<UsageDescription> = Vec::new();
    for scenario in &all_scenarios {
        let descs = generate_descriptions(scenario);
        all_descriptions.extend(descs);
    }

    let desc_csv_path = output_dir.join("usage_descriptions.csv");
    {
        let mut f = std::fs::File::create(&desc_csv_path)?;
        write_descriptions_csv(&mut f, &all_descriptions)?;
    }
    println!(
        "{} {} ({} descriptions)",
        "✓".green().bold(),
        desc_csv_path.display().to_string().cyan(),
        all_descriptions.len()
    );

    println!(
        "\n{} Generated data for {} tools → {}/",
        "✓".green().bold(),
        skills.len(),
        output_dir.display().to_string().cyan()
    );

    Ok(())
}

// ── Eval command ─────────────────────────────────────────────────────────────

fn cmd_eval(
    config_path: &std::path::Path,
    data_dir: &std::path::Path,
    output_dir: &std::path::Path,
    oxo_call_path: Option<String>,
    tools_filter: Option<&str>,
    repeats_override: Option<usize>,
    mock: bool,
) -> anyhow::Result<()> {
    // Load or generate config.
    let config = if config_path.exists() {
        BenchConfig::load(config_path)?
    } else {
        println!(
            "{} Config not found, using defaults (3 models, 3 repeats).",
            "→".cyan().bold()
        );
        BenchConfig::default()
    };

    let repeats = repeats_override.unwrap_or(config.benchmark.repeats);

    // Load scenarios and descriptions.
    let ref_csv_path = data_dir.join("reference_commands.csv");
    let desc_csv_path = data_dir.join("usage_descriptions.csv");

    if !ref_csv_path.exists() || !desc_csv_path.exists() {
        anyhow::bail!(
            "Benchmark data not found in {}.\n\
             Run 'oxo-bench generate' first to create reference_commands.csv and usage_descriptions.csv.",
            data_dir.display()
        );
    }

    let scenarios = load_scenarios_csv(&ref_csv_path)?;
    let mut descriptions = load_descriptions_csv(&desc_csv_path)?;

    // Apply tool filter.
    if let Some(filter) = tools_filter {
        let allowed: std::collections::HashSet<&str> = filter.split(',').collect();
        descriptions.retain(|d| allowed.contains(d.tool.as_str()));
    }

    println!(
        "{} Loaded {} scenarios, {} descriptions",
        "→".cyan().bold(),
        scenarios.len(),
        descriptions.len()
    );

    std::fs::create_dir_all(output_dir)?;

    let mut all_trials: Vec<TrialResult> = Vec::new();

    if mock {
        // ── Mock evaluation mode ──────────────────────────────────────────
        println!(
            "{} Mock evaluation: {} models × {} repeats × {} descriptions",
            "→".cyan().bold(),
            config.models.len(),
            repeats,
            descriptions.len()
        );
        println!(
            "{}",
            "  Using deterministic perturbation (no API calls required).".dimmed()
        );

        for (i, model_entry) in config.models.iter().enumerate() {
            println!(
                "[{}/{}] {} Evaluating {} (mock)...",
                i + 1,
                config.models.len(),
                "→".cyan().bold(),
                model_entry.name.cyan(),
            );
            let trials = run_mock_benchmark(&model_entry.name, repeats, &descriptions, &scenarios);
            println!(
                "      {} {} trials recorded",
                "✓".green().bold(),
                trials.len()
            );
            all_trials.extend(trials);
        }
    } else {
        // ── Real evaluation mode ──────────────────────────────────────────
        let binary = oxo_call_path
            .or_else(which_oxo_call)
            .unwrap_or_else(|| "oxo-call".to_string());

        println!(
            "{} Using oxo-call binary: {}",
            "→".cyan().bold(),
            binary.cyan()
        );

        if config.benchmark.parallel && config.models.len() > 1 {
            println!(
                "{} Evaluating {} models in parallel ({} repeats each)...",
                "→".cyan().bold(),
                config.models.len(),
                repeats,
            );

            let results: Vec<Vec<TrialResult>> = std::thread::scope(|scope| {
                let handles: Vec<_> = config
                    .models
                    .iter()
                    .map(|model_entry| {
                        let binary = binary.clone();
                        let descriptions = &descriptions;
                        let scenarios = &scenarios;
                        scope.spawn(move || {
                            let gtor = OxoCallGenerator {
                                binary_path: binary,
                            };
                            run_benchmark(
                                &model_entry.name,
                                repeats,
                                descriptions,
                                scenarios,
                                &gtor,
                            )
                        })
                    })
                    .collect();

                handles.into_iter().map(|h| h.join().unwrap()).collect()
            });

            for model_trials in results {
                all_trials.extend(model_trials);
            }
        } else {
            for (i, model_entry) in config.models.iter().enumerate() {
                println!(
                    "[{}/{}] {} Evaluating {} ({} repeats × {} descriptions)...",
                    i + 1,
                    config.models.len(),
                    "→".cyan().bold(),
                    model_entry.name.cyan(),
                    repeats,
                    descriptions.len(),
                );

                let gtor = OxoCallGenerator {
                    binary_path: binary.clone(),
                };
                let trials =
                    run_benchmark(&model_entry.name, repeats, &descriptions, &scenarios, &gtor);
                all_trials.extend(trials);
            }
        }
    }

    // ── Write result CSVs ─────────────────────────────────────────────────
    let trials_csv_path = output_dir.join("benchmark_trials.csv");
    {
        let mut f = std::fs::File::create(&trials_csv_path)?;
        write_trials_csv(&mut f, &all_trials)?;
    }
    println!(
        "{} {} ({} trials)",
        "✓".green().bold(),
        trials_csv_path.display().to_string().cyan(),
        all_trials.len()
    );

    let agg = aggregate_results(&all_trials);
    let agg_csv_path = output_dir.join("model_summary.csv");
    {
        let mut f = std::fs::File::create(&agg_csv_path)?;
        write_model_agg_csv(&mut f, &agg)?;
    }
    println!(
        "{} {} ({} models)",
        "✓".green().bold(),
        agg_csv_path.display().to_string().cyan(),
        agg.len()
    );

    // Per-model detailed CSVs.
    for model_agg in &agg {
        let model_trials: Vec<TrialResult> = all_trials
            .iter()
            .filter(|t| t.model == model_agg.model)
            .cloned()
            .collect();
        let safe_name = model_agg.model.replace(['/', ':'], "_");
        let model_csv_path = output_dir.join(format!("trials_{safe_name}.csv"));
        {
            let mut f = std::fs::File::create(&model_csv_path)?;
            write_trials_csv(&mut f, &model_trials)?;
        }
        println!(
            "  {} {} ({} trials)",
            "✓".green().bold(),
            model_csv_path.display().to_string().cyan(),
            model_trials.len()
        );
    }

    // Per-(tool, model) summary CSV.
    let tool_model_csv_path = output_dir.join("model_summary_by_tool.csv");
    {
        let tool_summaries = summarise_by_tool(&all_trials);
        let mut f = std::fs::File::create(&tool_model_csv_path)?;
        write_tool_model_summary_csv(&mut f, &tool_summaries)?;
        println!(
            "{} {} ({} tool × model rows)",
            "✓".green().bold(),
            tool_model_csv_path.display().to_string().cyan(),
            tool_summaries.len()
        );
    }

    // Print summary table.
    println!();
    print_agg_summary(&agg);

    Ok(())
}

// ── Summary command ──────────────────────────────────────────────────────────

fn cmd_summary(results_dir: &std::path::Path) -> anyhow::Result<()> {
    let agg_csv_path = results_dir.join("model_summary.csv");
    if !agg_csv_path.exists() {
        anyhow::bail!(
            "Summary file not found: {}\n\
             Run 'oxo-bench eval' first to generate results.",
            agg_csv_path.display()
        );
    }

    let agg = load_model_agg_csv(&agg_csv_path)?;
    print_agg_summary(&agg);
    Ok(())
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Try to find oxo-call in PATH or target/release or target/debug.
fn which_oxo_call() -> Option<String> {
    // Check PATH using the platform-appropriate lookup command.
    let lookup_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    if let Ok(path) = std::process::Command::new(lookup_cmd)
        .arg("oxo-call")
        .output()
        && path.status.success()
    {
        let p = String::from_utf8_lossy(&path.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if !p.is_empty() {
            return Some(p);
        }
    }
    // Check local build directories.
    let ext = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    for prefix in &["target/release/oxo-call", "target/debug/oxo-call"] {
        let candidate = format!("{prefix}{ext}");
        if std::path::Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

/// Print a formatted summary table of aggregate results.
fn print_agg_summary(agg: &[ModelAggResult]) {
    println!("{}", "═══ Model Comparison Summary ═══".bold().cyan());
    println!();
    println!(
        "{:<35} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10}",
        "Model".bold(),
        "Acc%".bold(),
        "Exact%".bold(),
        "Recall%".bold(),
        "Consist%".bold(),
        "Format%".bold(),
        "Latency".bold(),
    );
    println!("{}", "─".repeat(90).dimmed());
    for a in agg {
        println!(
            "{:<35} {:>7.1}% {:>7.1}% {:>7.1}% {:>7.1}% {:>7.1}% {:>8.0}ms",
            a.model.cyan(),
            a.accuracy * 100.0,
            a.exact_match_rate * 100.0,
            a.avg_flag_recall * 100.0,
            a.consistency * 100.0,
            a.format_valid_rate * 100.0,
            a.avg_latency_ms,
        );
    }
    println!("{}", "─".repeat(90).dimmed());
    if let Some(best) = agg.first() {
        println!(
            "\n{} Best model: {} (accuracy: {:.1}%)",
            "★".yellow().bold(),
            best.model.green().bold(),
            best.accuracy * 100.0
        );
    }
}

/// Load scenarios from a CSV file.
fn load_scenarios_csv(path: &std::path::Path) -> anyhow::Result<Vec<Scenario>> {
    let content = std::fs::read_to_string(path)?;
    let mut scenarios = Vec::new();
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if fields.len() >= 5 {
            scenarios.push(Scenario {
                tool: fields[0].clone(),
                scenario_id: fields[1].clone(),
                reference_args: fields[2].clone(),
                task_description: fields[3].clone(),
                category: fields[4].clone(),
            });
        }
    }
    Ok(scenarios)
}

/// Load descriptions from a CSV file.
fn load_descriptions_csv(path: &std::path::Path) -> anyhow::Result<Vec<UsageDescription>> {
    let content = std::fs::read_to_string(path)?;
    let mut descriptions = Vec::new();
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if fields.len() >= 5 {
            descriptions.push(UsageDescription {
                tool: fields[0].clone(),
                scenario_id: fields[1].clone(),
                desc_id: fields[2].clone(),
                user_level: fields[3].clone(),
                description: fields[4].clone(),
            });
        }
    }
    Ok(descriptions)
}

/// Load model aggregate results from a CSV file.
fn load_model_agg_csv(path: &std::path::Path) -> anyhow::Result<Vec<ModelAggResult>> {
    let content = std::fs::read_to_string(path)?;
    let mut results = Vec::new();
    for line in content.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_csv_line(line);
        if fields.len() >= 12 {
            results.push(ModelAggResult {
                model: fields[0].clone(),
                n_trials: fields[1].parse().unwrap_or(0),
                accuracy: fields[2].parse().unwrap_or(0.0),
                exact_match_rate: fields[3].parse().unwrap_or(0.0),
                avg_flag_recall: fields[4].parse().unwrap_or(0.0),
                avg_flag_precision: fields[5].parse().unwrap_or(0.0),
                avg_token_jaccard: fields[6].parse().unwrap_or(0.0),
                subcommand_match_rate: fields[7].parse().unwrap_or(0.0),
                consistency: fields[8].parse().unwrap_or(0.0),
                avg_latency_ms: fields[9].parse().unwrap_or(0.0),
                avg_tokens: fields[10].parse().unwrap_or(0.0),
                format_valid_rate: fields[11].parse().unwrap_or(0.0),
            });
        }
    }
    Ok(results)
}

/// Simple RFC 4180-compatible CSV line parser (handles quoted fields).
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                current.push(c);
            }
        } else if c == '"' {
            in_quotes = true;
        } else if c == ',' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}
