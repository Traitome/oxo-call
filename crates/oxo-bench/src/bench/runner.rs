//! Benchmark execution engine.
//!
//! Runs each (description, scenario) pair through a *command generator*
//! (normally the `oxo-call dry-run --json` subprocess) and records latency,
//! token count, and comparison metrics against the reference command.
//!
//! The generator is injected as a trait so that unit tests can substitute a
//! mock without requiring a real API token or the `oxo-call` binary.

use crate::bench::compare::compare_commands;
use crate::bench::scenario::{Scenario, UsageDescription};
use std::io::Write;
use std::time::Instant;

// ── Data types ───────────────────────────────────────────────────────────────

/// Result of a single benchmark trial (one description × one model × one repeat).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrialResult {
    pub tool: String,
    pub category: String,
    pub scenario_id: String,
    pub desc_id: String,
    pub model: String,
    pub ablation: String,
    pub repeat_idx: usize,
    pub generated_args: String,
    pub reference_args: String,
    pub exact_match: bool,
    pub token_jaccard: f64,
    pub flag_recall: f64,
    pub flag_precision: f64,
    pub flag_group_recall: f64,
    pub flag_group_precision: f64,
    /// Jaccard similarity over flag–value groups (order-aware pairing).
    pub flag_group_jaccard: f64,
    /// Whether positional arguments appear in the correct relative order.
    pub positional_order_match: f64,
    pub subcommand_match: bool,
    pub accuracy_score: f64,
    pub latency_ms: f64,
    pub tokens: usize,
    pub format_valid: bool,
}

/// Aggregate metrics for a single model across all trials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelAggResult {
    pub model: String,
    pub ablation: String,
    pub n_trials: usize,
    pub accuracy: f64,
    pub exact_match_rate: f64,
    pub avg_flag_recall: f64,
    pub avg_flag_precision: f64,
    pub avg_token_jaccard: f64,
    pub subcommand_match_rate: f64,
    pub consistency: f64,
    pub avg_latency_ms: f64,
    pub avg_tokens: f64,
    pub format_valid_rate: f64,
}

// ── Command generator trait ──────────────────────────────────────────────────

/// Response returned by a command generator.
#[derive(Debug, Clone)]
pub struct GeneratedCommand {
    /// The ARGS string (flags + positional args, without the tool name prefix).
    pub args: String,
    /// Whether the response had valid `ARGS:` / `EXPLANATION:` format.
    pub format_valid: bool,
    /// Approximate number of tokens in the raw response.
    pub tokens: usize,
}

/// Trait for pluggable command generators (real oxo-call or mock).
pub trait CommandGenerator: Send + Sync {
    fn generate(&self, tool: &str, task: &str, model: &str) -> GeneratedCommand;
}

/// Mock generator that returns the reference args directly (for unit testing).
pub struct EchoGenerator;

impl CommandGenerator for EchoGenerator {
    fn generate(&self, _tool: &str, task: &str, _model: &str) -> GeneratedCommand {
        // Return the task as-is for testing purposes.
        GeneratedCommand {
            args: task.to_string(),
            format_valid: true,
            tokens: task.split_whitespace().count(),
        }
    }
}

/// Mock generator that always returns a fixed response.
pub struct FixedGenerator {
    pub args: String,
    pub format_valid: bool,
}

impl CommandGenerator for FixedGenerator {
    fn generate(&self, _tool: &str, _task: &str, _model: &str) -> GeneratedCommand {
        GeneratedCommand {
            args: self.args.clone(),
            format_valid: self.format_valid,
            tokens: self.args.split_whitespace().count(),
        }
    }
}

/// Run a mock benchmark with deterministic perturbation.
///
/// Simulates the full oxo-call pipeline (with docs/skills grounding).
///
/// Because benchmark scenarios are extracted from the very skill files that
/// oxo-call loads into the LLM prompt, the model sees the *exact* reference
/// example — meaning error rates should be extremely low.  Furthermore,
/// with `temperature = 0` all repeats of the same description receive the
/// same input and should produce identical output (deterministic).
///
/// Perturbation rates (per model):
///
/// - "gpt-4o-mini" → 0.5 %
/// - "gpt-4o" (not mini) → 0.3 %
/// - "claude" → 0.4 %
/// - Others → 0.1 %
pub fn run_mock_benchmark(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
) -> Vec<TrialResult> {
    let scenario_map: std::collections::HashMap<&str, &Scenario> = scenarios
        .iter()
        .map(|s| (s.scenario_id.as_str(), s))
        .collect();

    let mut results = Vec::new();

    // Docs-first grounding: skill files contain the exact reference examples,
    // so error rates are very low.
    let perturbation_rate = if model.contains("gpt-4o-mini") {
        0.005
    } else if model.contains("gpt-4o") {
        0.003
    } else if model.contains("claude") {
        0.004
    } else {
        0.001
    };

    for desc in descriptions {
        let scenario = match scenario_map.get(desc.scenario_id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        for repeat in 0..repeats {
            let start = Instant::now();

            // With temperature = 0 and identical prompt context, all repeats
            // of the same (model, desc_id) pair produce the same output.
            // Pass `0` as repeat index so every repeat shares one hash.
            let generated_args = perturb_args(
                &scenario.reference_args,
                model,
                &desc.desc_id,
                0, // deterministic across repeats (temp=0)
                perturbation_rate,
            );

            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let tokens = generated_args.split_whitespace().count();

            let cmp = compare_commands(&generated_args, &scenario.reference_args);

            results.push(TrialResult {
                tool: desc.tool.clone(),
                category: scenario.category.clone(),
                scenario_id: desc.scenario_id.clone(),
                desc_id: desc.desc_id.clone(),
                model: model.to_string(),
                ablation: "full".to_string(),
                repeat_idx: repeat,
                generated_args,
                reference_args: scenario.reference_args.clone(),
                exact_match: cmp.exact_match,
                token_jaccard: cmp.token_jaccard,
                flag_recall: cmp.flag_recall,
                flag_precision: cmp.flag_precision,
                flag_group_recall: cmp.flag_group_recall,
                flag_group_precision: cmp.flag_group_precision,
                flag_group_jaccard: cmp.flag_group_jaccard,
                positional_order_match: cmp.positional_order_match,
                subcommand_match: cmp.subcommand_match,
                accuracy_score: cmp.accuracy_score(),
                latency_ms,
                tokens,
                format_valid: true,
            });
        }
    }

    results
}

/// Run a mock **baseline** benchmark that simulates a bare LLM (no tool
/// docs / skills).
///
/// Without skill documentation the LLM must guess flag names, subcommands,
/// and file conventions from its parametric knowledge alone — resulting in
/// substantially higher error rates.  Each repeat is hashed independently
/// (simulating non-deterministic sampling at temperature > 0), so
/// consistency is also much lower than the enhanced mock.
///
/// Perturbation rates (per model):
///
/// - "gpt-4o-mini" → 55 %
/// - "gpt-4o" (not mini) → 30 %
/// - "claude" → 40 %
/// - Others → 25 %
///
/// Results carry the model name suffixed with `(baseline)` so they can
/// be clearly distinguished from the enhanced results.
pub fn run_mock_baseline(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
) -> Vec<TrialResult> {
    let scenario_map: std::collections::HashMap<&str, &Scenario> = scenarios
        .iter()
        .map(|s| (s.scenario_id.as_str(), s))
        .collect();

    let mut results = Vec::new();

    // Without docs the LLM has to guess — much higher error rates.
    let perturbation_rate = if model.contains("gpt-4o-mini") {
        0.55
    } else if model.contains("gpt-4o") {
        0.30
    } else if model.contains("claude") {
        0.40
    } else {
        0.25
    };

    let baseline_model = format!("{model} (baseline)");

    for desc in descriptions {
        let scenario = match scenario_map.get(desc.scenario_id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        for repeat in 0..repeats {
            let start = Instant::now();

            // Use a distinct hash namespace ("baseline:") so the perturbation
            // pattern differs from the enhanced mock.
            let generated_args = perturb_args(
                &scenario.reference_args,
                &format!("baseline:{model}"),
                &desc.desc_id,
                repeat,
                perturbation_rate,
            );

            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
            let tokens = generated_args.split_whitespace().count();

            let cmp = compare_commands(&generated_args, &scenario.reference_args);

            results.push(TrialResult {
                tool: desc.tool.clone(),
                category: scenario.category.clone(),
                scenario_id: desc.scenario_id.clone(),
                desc_id: desc.desc_id.clone(),
                model: baseline_model.clone(),
                ablation: "bare".to_string(),
                repeat_idx: repeat,
                generated_args,
                reference_args: scenario.reference_args.clone(),
                exact_match: cmp.exact_match,
                token_jaccard: cmp.token_jaccard,
                flag_recall: cmp.flag_recall,
                flag_precision: cmp.flag_precision,
                flag_group_recall: cmp.flag_group_recall,
                flag_group_precision: cmp.flag_group_precision,
                flag_group_jaccard: cmp.flag_group_jaccard,
                positional_order_match: cmp.positional_order_match,
                subcommand_match: cmp.subcommand_match,
                accuracy_score: cmp.accuracy_score(),
                latency_ms,
                tokens,
                format_valid: true,
            });
        }
    }

    results
}

// ── Baseline comparison ──────────────────────────────────────────────────────

/// Side-by-side comparison of a model's enhanced (with docs/skills) vs
/// baseline (bare LLM) aggregate metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaselineComparison {
    pub model: String,
    pub enhanced_accuracy: f64,
    pub baseline_accuracy: f64,
    pub accuracy_delta: f64,
    pub enhanced_exact_match: f64,
    pub baseline_exact_match: f64,
    pub exact_match_delta: f64,
    pub enhanced_flag_recall: f64,
    pub baseline_flag_recall: f64,
    pub flag_recall_delta: f64,
    pub enhanced_consistency: f64,
    pub baseline_consistency: f64,
    pub consistency_delta: f64,
}

/// Compute per-model baseline comparisons from enhanced and baseline trials.
///
/// For each model that appears in *both* sets (ignoring the ` (baseline)`
/// suffix on the baseline side), produce a [`BaselineComparison`].
pub fn compute_baseline_comparison(
    enhanced_trials: &[TrialResult],
    baseline_trials: &[TrialResult],
) -> Vec<BaselineComparison> {
    let enhanced_agg = aggregate_results(enhanced_trials);
    let baseline_agg = aggregate_results(baseline_trials);

    // Build a lookup from the baseline model name (strip " (baseline)" suffix).
    let baseline_map: std::collections::HashMap<String, &ModelAggResult> = baseline_agg
        .iter()
        .map(|a| {
            let base = a
                .model
                .strip_suffix(" (baseline)")
                .unwrap_or(&a.model)
                .to_string();
            (base, a)
        })
        .collect();

    let mut comparisons: Vec<BaselineComparison> = enhanced_agg
        .iter()
        .filter_map(|enh| {
            let base = baseline_map.get(&enh.model)?;
            Some(BaselineComparison {
                model: enh.model.clone(),
                enhanced_accuracy: enh.accuracy,
                baseline_accuracy: base.accuracy,
                accuracy_delta: enh.accuracy - base.accuracy,
                enhanced_exact_match: enh.exact_match_rate,
                baseline_exact_match: base.exact_match_rate,
                exact_match_delta: enh.exact_match_rate - base.exact_match_rate,
                enhanced_flag_recall: enh.avg_flag_recall,
                baseline_flag_recall: base.avg_flag_recall,
                flag_recall_delta: enh.avg_flag_recall - base.avg_flag_recall,
                enhanced_consistency: enh.consistency,
                baseline_consistency: base.consistency,
                consistency_delta: enh.consistency - base.consistency,
            })
        })
        .collect();

    comparisons.sort_by(|a, b| {
        b.accuracy_delta
            .partial_cmp(&a.accuracy_delta)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    comparisons
}

/// Write baseline comparison results to CSV.
///
/// Columns: `model,enhanced_accuracy,baseline_accuracy,accuracy_delta,
///           enhanced_exact_match,baseline_exact_match,exact_match_delta,
///           enhanced_flag_recall,baseline_flag_recall,flag_recall_delta,
///           enhanced_consistency,baseline_consistency,consistency_delta`
pub fn write_baseline_comparison_csv<W: Write>(
    writer: &mut W,
    comparisons: &[BaselineComparison],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "model,enhanced_accuracy,baseline_accuracy,accuracy_delta,\
         enhanced_exact_match,baseline_exact_match,exact_match_delta,\
         enhanced_flag_recall,baseline_flag_recall,flag_recall_delta,\
         enhanced_consistency,baseline_consistency,consistency_delta"
    )?;
    for c in comparisons {
        writeln!(
            writer,
            "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            c.model,
            c.enhanced_accuracy,
            c.baseline_accuracy,
            c.accuracy_delta,
            c.enhanced_exact_match,
            c.baseline_exact_match,
            c.exact_match_delta,
            c.enhanced_flag_recall,
            c.baseline_flag_recall,
            c.flag_recall_delta,
            c.enhanced_consistency,
            c.baseline_consistency,
            c.consistency_delta,
        )?;
    }
    Ok(())
}

/// Apply deterministic perturbation to reference args.
///
/// Uses a simple hash of the inputs to decide whether to drop, reorder, or
/// replace a flag — producing consistent results across runs.
fn perturb_args(
    reference_args: &str,
    model: &str,
    desc_id: &str,
    repeat: usize,
    perturbation_rate: f64,
) -> String {
    let tokens: Vec<&str> = reference_args.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    // Compute a deterministic hash to decide perturbation.
    let hash_input = format!("{model}:{desc_id}:{repeat}");
    let hash = simple_hash(&hash_input);

    // Decide whether to perturb this trial at all.
    let frac = (hash % 1000) as f64 / 1000.0;
    if frac >= perturbation_rate {
        // No perturbation — return exact reference.
        return reference_args.to_string();
    }

    // Apply one perturbation based on hash.
    let perturbation_type = hash % 4;
    let mut result_tokens: Vec<String> = tokens.iter().map(|&s| s.to_string()).collect();

    match perturbation_type {
        0 => {
            // Drop a non-first token (simulate missing flag).
            if result_tokens.len() > 2 {
                let idx = 1 + (hash / 7) as usize % (result_tokens.len() - 1);
                result_tokens.remove(idx);
            }
        }
        1 => {
            // Swap two adjacent tokens (simulate flag reordering).
            if result_tokens.len() > 2 {
                let idx = 1 + (hash / 11) as usize % (result_tokens.len() - 2);
                result_tokens.swap(idx, idx + 1);
            }
        }
        2 => {
            // Add an extra flag (simulate hallucination).
            result_tokens.push("--extra-flag".to_string());
        }
        _ => {
            // Replace a file path with a different name.
            if result_tokens.len() > 1 {
                let idx = 1 + (hash / 13) as usize % (result_tokens.len() - 1);
                if !result_tokens[idx].starts_with('-') {
                    result_tokens[idx] = "other_file.bam".to_string();
                }
            }
        }
    }

    result_tokens.join(" ")
}

/// Simple deterministic hash for perturbation decisions.
fn simple_hash(input: &str) -> u64 {
    let mut h: u64 = 5381;
    for byte in input.bytes() {
        h = h.wrapping_mul(33).wrapping_add(byte as u64);
    }
    h
}

/// Generator that calls `oxo-call dry-run --json` as a subprocess.
///
/// Supports ablation scenarios via `--no-skill`, `--no-doc`, and `--no-prompt`
/// flags, and provider/API base configuration via environment variables.
pub struct OxoCallGenerator {
    pub binary_path: String,
    /// Ablation scenario controlling which grounding sources are available.
    pub scenario: crate::config::AblationScenario,
    /// Optional provider hint (e.g. "ollama").
    pub provider: Option<String>,
    /// Optional API base URL (e.g. "http://localhost:11434").
    pub api_base: Option<String>,
    /// Optional API key.
    pub api_key: Option<String>,
}

impl CommandGenerator for OxoCallGenerator {
    fn generate(&self, tool: &str, task: &str, model: &str) -> GeneratedCommand {
        let mut cmd = std::process::Command::new(&self.binary_path);
        cmd.args(["dry-run", "--json", "-m", model]);

        // Ablation flags.
        if !self.scenario.use_skill() {
            cmd.arg("--no-skill");
        }
        if !self.scenario.use_doc() {
            cmd.arg("--no-doc");
        }
        if !self.scenario.use_prompt() {
            cmd.arg("--no-prompt");
        }

        cmd.args([tool, task]);

        // Pass provider configuration via environment.
        if let Some(ref base) = self.api_base {
            cmd.env("OXO_CALL_API_BASE", base);
        }
        if let Some(ref key) = self.api_key {
            cmd.env("OXO_CALL_API_KEY", key);
        }

        let output = cmd.output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                parse_dry_run_json(&stdout)
            }
            _ => GeneratedCommand {
                args: String::new(),
                format_valid: false,
                tokens: 0,
            },
        }
    }
}

/// Parse the JSON output of `oxo-call dry-run --json`.
fn parse_dry_run_json(json_str: &str) -> GeneratedCommand {
    // Expected format: { "command": "tool args ...", "args": "args ...", ... }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
        let args = value
            .get("args")
            .and_then(|v| v.as_str())
            .or_else(|| {
                // Fallback: extract args from "command" by removing the tool prefix.
                value.get("command").and_then(|v| {
                    v.as_str()
                        .map(|cmd| cmd.split_once(' ').map(|(_, rest)| rest).unwrap_or(cmd))
                })
            })
            .unwrap_or("")
            .to_string();

        // Estimate token count from the generated args (not the full JSON envelope).
        let tokens = args.split_whitespace().count();

        GeneratedCommand {
            args,
            format_valid: true,
            tokens,
        }
    } else {
        GeneratedCommand {
            args: String::new(),
            format_valid: false,
            tokens: 0,
        }
    }
}

// ── Benchmark runner ─────────────────────────────────────────────────────────

/// Run the benchmark for a single model over all descriptions.
///
/// For each description, calls the generator `repeats` times and records
/// comparison metrics against the corresponding scenario reference command.
///
/// `ablation_label` is stored in each trial result for downstream analysis
/// (e.g. "full", "bare", "skill", etc.).
pub fn run_benchmark(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
    generator: &(dyn CommandGenerator + 'static),
    ablation_label: &str,
) -> Vec<TrialResult> {
    run_benchmark_with_callback(
        model,
        repeats,
        descriptions,
        scenarios,
        generator,
        ablation_label,
        None,
    )
}

/// Run benchmark with optional callback for incremental result writing.
pub fn run_benchmark_with_callback(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
    generator: &(dyn CommandGenerator + 'static),
    ablation_label: &str,
    callback: Option<&dyn TrialCallback>,
) -> Vec<TrialResult> {
    // Build scenario lookup map.
    let scenario_map: std::collections::HashMap<&str, &Scenario> = scenarios
        .iter()
        .map(|s| (s.scenario_id.as_str(), s))
        .collect();

    let mut results = Vec::new();

    for desc in descriptions {
        let scenario = match scenario_map.get(desc.scenario_id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        for repeat in 0..repeats {
            let start = Instant::now();
            let resp = generator.generate(&desc.tool, &desc.description, model);
            let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

            let cmp = compare_commands(&resp.args, &scenario.reference_args);

            let trial = TrialResult {
                tool: desc.tool.clone(),
                category: scenario.category.clone(),
                scenario_id: desc.scenario_id.clone(),
                desc_id: desc.desc_id.clone(),
                model: model.to_string(),
                ablation: ablation_label.to_string(),
                repeat_idx: repeat,
                generated_args: resp.args,
                reference_args: scenario.reference_args.clone(),
                exact_match: cmp.exact_match,
                token_jaccard: cmp.token_jaccard,
                flag_recall: cmp.flag_recall,
                flag_precision: cmp.flag_precision,
                flag_group_recall: cmp.flag_group_recall,
                flag_group_precision: cmp.flag_group_precision,
                flag_group_jaccard: cmp.flag_group_jaccard,
                positional_order_match: cmp.positional_order_match,
                subcommand_match: cmp.subcommand_match,
                accuracy_score: cmp.accuracy_score(),
                latency_ms,
                tokens: resp.tokens,
                format_valid: resp.format_valid,
            };

            if let Some(cb) = callback {
                cb.on_trial(&trial);
            }

            results.push(trial);
        }
    }

    results
}

/// Aggregate trial results into per-model summary metrics.
pub fn aggregate_results(trials: &[TrialResult]) -> Vec<ModelAggResult> {
    let mut by_model_ablation: std::collections::HashMap<(&str, &str), Vec<&TrialResult>> =
        std::collections::HashMap::new();
    for t in trials {
        by_model_ablation
            .entry((t.model.as_str(), t.ablation.as_str()))
            .or_default()
            .push(t);
    }

    let mut agg: Vec<ModelAggResult> = by_model_ablation
        .into_iter()
        .map(|((model, ablation), trials)| {
            let n = trials.len() as f64;
            ModelAggResult {
                model: model.to_string(),
                ablation: ablation.to_string(),
                n_trials: trials.len(),
                accuracy: trials.iter().map(|t| t.accuracy_score).sum::<f64>() / n,
                exact_match_rate: trials.iter().filter(|t| t.exact_match).count() as f64 / n,
                avg_flag_recall: trials.iter().map(|t| t.flag_recall).sum::<f64>() / n,
                avg_flag_precision: trials.iter().map(|t| t.flag_precision).sum::<f64>() / n,
                avg_token_jaccard: trials.iter().map(|t| t.token_jaccard).sum::<f64>() / n,
                subcommand_match_rate: trials.iter().filter(|t| t.subcommand_match).count() as f64
                    / n,
                consistency: compute_trial_consistency(&trials),
                avg_latency_ms: trials.iter().map(|t| t.latency_ms).sum::<f64>() / n,
                avg_tokens: trials.iter().map(|t| t.tokens as f64).sum::<f64>() / n,
                format_valid_rate: trials.iter().filter(|t| t.format_valid).count() as f64 / n,
            }
        })
        .collect();

    agg.sort_by(|a, b| {
        b.accuracy
            .partial_cmp(&a.accuracy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    agg
}

/// Run benchmark with parallel processing of descriptions.
///
/// Uses rayon to process descriptions in parallel while maintaining
/// thread-safe incremental result writing through the callback.
pub fn run_benchmark_parallel(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
    generator: &(dyn CommandGenerator + 'static),
    ablation_label: &str,
    callback: Option<&dyn TrialCallback>,
    num_threads: usize,
) -> Vec<TrialResult> {
    use rayon::prelude::*;

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let scenario_map: std::collections::HashMap<&str, &Scenario> = scenarios
        .iter()
        .map(|s| (s.scenario_id.as_str(), s))
        .collect();

    let model = model.to_string();
    let ablation_label = ablation_label.to_string();

    pool.install(|| {
        descriptions
            .par_iter()
            .filter_map(|desc| {
                let scenario = scenario_map.get(desc.scenario_id.as_str())?;

                let mut trials = Vec::new();
                for repeat in 0..repeats {
                    let start = Instant::now();
                    let resp = generator.generate(&desc.tool, &desc.description, &model);
                    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

                    let cmp = compare_commands(&resp.args, &scenario.reference_args);

                    let trial = TrialResult {
                        tool: desc.tool.clone(),
                        category: scenario.category.clone(),
                        scenario_id: desc.scenario_id.clone(),
                        desc_id: desc.desc_id.clone(),
                        model: model.clone(),
                        ablation: ablation_label.clone(),
                        repeat_idx: repeat,
                        generated_args: resp.args,
                        reference_args: scenario.reference_args.clone(),
                        exact_match: cmp.exact_match,
                        token_jaccard: cmp.token_jaccard,
                        flag_recall: cmp.flag_recall,
                        flag_precision: cmp.flag_precision,
                        flag_group_recall: cmp.flag_group_recall,
                        flag_group_precision: cmp.flag_group_precision,
                        flag_group_jaccard: cmp.flag_group_jaccard,
                        positional_order_match: cmp.positional_order_match,
                        subcommand_match: cmp.subcommand_match,
                        accuracy_score: cmp.accuracy_score(),
                        latency_ms,
                        tokens: resp.tokens,
                        format_valid: resp.format_valid,
                    };

                    if let Some(cb) = callback {
                        cb.on_trial(&trial);
                    }

                    trials.push(trial);
                }
                Some(trials)
            })
            .flatten()
            .collect()
    })
}

/// Compute consistency: for each (scenario_id, desc_id) group, check if all
/// repeat runs produced the same generated_args.
fn compute_trial_consistency(trials: &[&TrialResult]) -> f64 {
    let mut groups: std::collections::HashMap<(&str, &str), Vec<&str>> =
        std::collections::HashMap::new();
    for t in trials {
        groups
            .entry((t.scenario_id.as_str(), t.desc_id.as_str()))
            .or_default()
            .push(t.generated_args.as_str());
    }

    if groups.is_empty() {
        return 1.0;
    }

    let consistent = groups
        .values()
        .filter(|responses| responses.len() <= 1 || responses.iter().all(|r| *r == responses[0]))
        .count();

    consistent as f64 / groups.len() as f64
}

// ── CSV export ───────────────────────────────────────────────────────────────

/// Write detailed trial results to CSV.
pub fn write_trials_csv<W: Write>(writer: &mut W, trials: &[TrialResult]) -> std::io::Result<()> {
    writeln!(
        writer,
        "tool,category,scenario_id,desc_id,model,ablation,repeat,generated_args,reference_args,\
         exact_match,token_jaccard,flag_recall,flag_precision,\
         flag_group_recall,flag_group_precision,flag_group_jaccard,positional_order_match,\
         subcommand_match,accuracy_score,latency_ms,tokens,format_valid"
    )?;
    for t in trials {
        let gen_esc = csv_escape(&t.generated_args);
        let ref_esc = csv_escape(&t.reference_args);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.1},{},{}",
            t.tool,
            t.category,
            t.scenario_id,
            t.desc_id,
            t.model,
            t.ablation,
            t.repeat_idx,
            gen_esc,
            ref_esc,
            t.exact_match,
            t.token_jaccard,
            t.flag_recall,
            t.flag_precision,
            t.flag_group_recall,
            t.flag_group_precision,
            t.flag_group_jaccard,
            t.positional_order_match,
            t.subcommand_match,
            t.accuracy_score,
            t.latency_ms,
            t.tokens,
            t.format_valid,
        )?;
    }
    Ok(())
}

/// Write aggregate model results to CSV.
pub fn write_model_agg_csv<W: Write>(
    writer: &mut W,
    agg: &[ModelAggResult],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "model,ablation,n_trials,accuracy,exact_match_rate,avg_flag_recall,avg_flag_precision,\
         avg_token_jaccard,subcommand_match_rate,consistency,avg_latency_ms,avg_tokens,\
         format_valid_rate"
    )?;
    for a in agg {
        writeln!(
            writer,
            "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.1},{:.1},{:.4}",
            a.model,
            a.ablation,
            a.n_trials,
            a.accuracy,
            a.exact_match_rate,
            a.avg_flag_recall,
            a.avg_flag_precision,
            a.avg_token_jaccard,
            a.subcommand_match_rate,
            a.consistency,
            a.avg_latency_ms,
            a.avg_tokens,
            a.format_valid_rate,
        )?;
    }
    Ok(())
}

fn csv_escape(field: &str) -> String {
    // Normalize newlines and carriage returns to spaces to prevent CSV corruption
    let normalized = field.replace('\r', " ").replace('\n', " ");
    if normalized.contains(',') || normalized.contains('"') {
        format!("\"{}\"", normalized.replace('"', "\"\""))
    } else {
        normalized
    }
}

/// Aggregate metrics for a single (tool, model) pair.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolModelSummary {
    pub tool: String,
    pub category: String,
    pub model: String,
    pub ablation: String,
    pub n_trials: usize,
    pub accuracy: f64,
    pub exact_match_rate: f64,
    pub avg_flag_recall: f64,
    pub consistency: f64,
}

/// Summarise trial results per (tool, model, ablation) tuple, sorted by tool then model.
pub fn summarise_by_tool(trials: &[TrialResult]) -> Vec<ToolModelSummary> {
    let mut by_tool_model_ablation: std::collections::HashMap<
        (String, String, String),
        Vec<&TrialResult>,
    > = std::collections::HashMap::new();
    for t in trials {
        by_tool_model_ablation
            .entry((t.tool.clone(), t.model.clone(), t.ablation.clone()))
            .or_default()
            .push(t);
    }

    let mut summaries: Vec<ToolModelSummary> = by_tool_model_ablation
        .into_iter()
        .map(|((tool, model, ablation), trials)| {
            let n = trials.len() as f64;
            // Gather scenario-level consistency
            let mut groups: std::collections::HashMap<(&str, &str), Vec<&str>> =
                std::collections::HashMap::new();
            for t in &trials {
                groups
                    .entry((t.scenario_id.as_str(), t.desc_id.as_str()))
                    .or_default()
                    .push(t.generated_args.as_str());
            }
            let consistency = if groups.is_empty() {
                1.0
            } else {
                let consistent = groups
                    .values()
                    .filter(|responses| {
                        responses.len() <= 1 || responses.iter().all(|r| *r == responses[0])
                    })
                    .count();
                consistent as f64 / groups.len() as f64
            };

            // Use category from first trial (all same tool → same category)
            let category = trials
                .first()
                .map(|t| t.category.clone())
                .unwrap_or_default();

            ToolModelSummary {
                tool,
                category,
                model,
                ablation,
                n_trials: trials.len(),
                accuracy: trials.iter().map(|t| t.accuracy_score).sum::<f64>() / n,
                exact_match_rate: trials.iter().filter(|t| t.exact_match).count() as f64 / n,
                avg_flag_recall: trials.iter().map(|t| t.flag_recall).sum::<f64>() / n,
                consistency,
            }
        })
        .collect();

    summaries.sort_by(|a, b| {
        a.tool
            .cmp(&b.tool)
            .then(a.model.cmp(&b.model))
            .then(a.ablation.cmp(&b.ablation))
    });
    summaries
}

/// Aggregate metrics for a single (category, model) pair.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryModelSummary {
    pub category: String,
    pub model: String,
    pub ablation: String,
    pub n_tools: usize,
    pub n_trials: usize,
    pub accuracy: f64,
    pub exact_match_rate: f64,
    pub avg_flag_recall: f64,
    pub avg_flag_precision: f64,
    pub consistency: f64,
    /// 95% confidence interval half-width for accuracy.
    pub accuracy_ci95: f64,
    /// 95% confidence interval half-width for exact match rate.
    pub exact_match_ci95: f64,
}

/// Summarise trial results per (category, model, ablation) tuple, sorted by category then model.
pub fn summarise_by_category(trials: &[TrialResult]) -> Vec<CategoryModelSummary> {
    let mut by_cat_model_ablation: std::collections::HashMap<
        (String, String, String),
        Vec<&TrialResult>,
    > = std::collections::HashMap::new();
    for t in trials {
        by_cat_model_ablation
            .entry((t.category.clone(), t.model.clone(), t.ablation.clone()))
            .or_default()
            .push(t);
    }

    let mut summaries: Vec<CategoryModelSummary> = by_cat_model_ablation
        .into_iter()
        .map(|((category, model, ablation), trials)| {
            let n = trials.len() as f64;

            let accuracy = trials.iter().map(|t| t.accuracy_score).sum::<f64>() / n;
            let exact_match_rate = trials.iter().filter(|t| t.exact_match).count() as f64 / n;
            let avg_flag_recall = trials.iter().map(|t| t.flag_recall).sum::<f64>() / n;
            let avg_flag_precision = trials.iter().map(|t| t.flag_precision).sum::<f64>() / n;

            // Count distinct tools in this category.
            let tools: std::collections::HashSet<&str> =
                trials.iter().map(|t| t.tool.as_str()).collect();

            // Consistency within this category.
            let mut groups: std::collections::HashMap<(&str, &str), Vec<&str>> =
                std::collections::HashMap::new();
            for t in &trials {
                groups
                    .entry((t.scenario_id.as_str(), t.desc_id.as_str()))
                    .or_default()
                    .push(t.generated_args.as_str());
            }
            let consistency = if groups.is_empty() {
                1.0
            } else {
                let consistent = groups
                    .values()
                    .filter(|responses| {
                        responses.len() <= 1 || responses.iter().all(|r| *r == responses[0])
                    })
                    .count();
                consistent as f64 / groups.len() as f64
            };

            // 95% CI using normal approximation for proportion (Wald interval).
            let z = 1.96_f64;
            let accuracy_ci95 = z * (accuracy * (1.0 - accuracy) / n).sqrt();
            let exact_match_ci95 = z * (exact_match_rate * (1.0 - exact_match_rate) / n).sqrt();

            CategoryModelSummary {
                category,
                model,
                ablation,
                n_tools: tools.len(),
                n_trials: trials.len(),
                accuracy,
                exact_match_rate,
                avg_flag_recall,
                avg_flag_precision,
                consistency,
                accuracy_ci95,
                exact_match_ci95,
            }
        })
        .collect();

    summaries.sort_by(|a, b| a.category.cmp(&b.category).then(a.model.cmp(&b.model)));
    summaries
}

/// Write per-(category, model, ablation) summary to CSV.
pub fn write_category_summary_csv<W: Write>(
    writer: &mut W,
    summaries: &[CategoryModelSummary],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "category,model,ablation,n_tools,n_trials,accuracy,exact_match_rate,\
         avg_flag_recall,avg_flag_precision,consistency,accuracy_ci95,exact_match_ci95"
    )?;
    for s in summaries {
        writeln!(
            writer,
            "{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.6},{:.6}",
            s.category,
            s.model,
            s.ablation,
            s.n_tools,
            s.n_trials,
            s.accuracy,
            s.exact_match_rate,
            s.avg_flag_recall,
            s.avg_flag_precision,
            s.consistency,
            s.accuracy_ci95,
            s.exact_match_ci95,
        )?;
    }
    Ok(())
}

/// Compute a 95% confidence interval half-width for a proportion using
/// the Wald normal approximation: z * sqrt(p*(1-p)/n).
pub fn proportion_ci95(p: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let z = 1.96_f64;
    z * (p * (1.0 - p) / n as f64).sqrt()
}

/// Error category for a single trial failure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorCategory {
    /// A flag was dropped (missing from generated output).
    MissingFlag,
    /// An extra flag was added (hallucinated).
    ExtraFlag,
    /// A value was replaced with a different one.
    WrongValue,
    /// Flags were reordered (token order differs but set matches).
    FlagReorder,
    /// Subcommand mismatch (first token differs).
    WrongSubcommand,
    /// Format invalid (no ARGS:/EXPLANATION: lines).
    FormatError,
    /// Complete miss (empty output).
    EmptyOutput,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::MissingFlag => write!(f, "missing_flag"),
            ErrorCategory::ExtraFlag => write!(f, "extra_flag"),
            ErrorCategory::WrongValue => write!(f, "wrong_value"),
            ErrorCategory::FlagReorder => write!(f, "flag_reorder"),
            ErrorCategory::WrongSubcommand => write!(f, "wrong_subcommand"),
            ErrorCategory::FormatError => write!(f, "format_error"),
            ErrorCategory::EmptyOutput => write!(f, "empty_output"),
        }
    }
}

/// Classify the error type for a non-exact-match trial.
pub fn classify_error(trial: &TrialResult) -> ErrorCategory {
    if !trial.format_valid {
        return ErrorCategory::FormatError;
    }
    if trial.generated_args.trim().is_empty() {
        return ErrorCategory::EmptyOutput;
    }
    if !trial.subcommand_match {
        return ErrorCategory::WrongSubcommand;
    }

    let gen_tokens: std::collections::HashSet<&str> =
        trial.generated_args.split_whitespace().collect();
    let ref_tokens: std::collections::HashSet<&str> =
        trial.reference_args.split_whitespace().collect();

    let missing = ref_tokens.difference(&gen_tokens).count();
    let extra = gen_tokens.difference(&ref_tokens).count();

    // If sets are identical but order differs, it's a reorder.
    if missing == 0 && extra == 0 && !trial.exact_match {
        return ErrorCategory::FlagReorder;
    }

    // If there are extra tokens but no missing ones, it's hallucination.
    if extra > 0 && missing == 0 {
        return ErrorCategory::ExtraFlag;
    }

    // If there are missing tokens but no extra ones, something was dropped.
    if missing > 0 && extra == 0 {
        return ErrorCategory::MissingFlag;
    }

    // Both missing and extra — likely a value was replaced.
    ErrorCategory::WrongValue
}

/// Summary of error categories for a set of trials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorAnalysis {
    pub model: String,
    pub ablation: String,
    pub total_errors: usize,
    pub missing_flag: usize,
    pub extra_flag: usize,
    pub wrong_value: usize,
    pub flag_reorder: usize,
    pub wrong_subcommand: usize,
    pub format_error: usize,
    pub empty_output: usize,
}

/// Analyse error categories for all non-exact-match trials, grouped by model and ablation.
pub fn analyse_errors(trials: &[TrialResult]) -> Vec<ErrorAnalysis> {
    let mut by_model_ablation: std::collections::HashMap<(&str, &str), Vec<ErrorCategory>> =
        std::collections::HashMap::new();

    for t in trials {
        if !t.exact_match {
            let cat = classify_error(t);
            by_model_ablation
                .entry((t.model.as_str(), t.ablation.as_str()))
                .or_default()
                .push(cat);
        }
    }

    let mut analyses: Vec<ErrorAnalysis> = by_model_ablation
        .into_iter()
        .map(|((model, ablation), errors)| {
            let mut analysis = ErrorAnalysis {
                model: model.to_string(),
                ablation: ablation.to_string(),
                total_errors: errors.len(),
                missing_flag: 0,
                extra_flag: 0,
                wrong_value: 0,
                flag_reorder: 0,
                wrong_subcommand: 0,
                format_error: 0,
                empty_output: 0,
            };
            for e in &errors {
                match e {
                    ErrorCategory::MissingFlag => analysis.missing_flag += 1,
                    ErrorCategory::ExtraFlag => analysis.extra_flag += 1,
                    ErrorCategory::WrongValue => analysis.wrong_value += 1,
                    ErrorCategory::FlagReorder => analysis.flag_reorder += 1,
                    ErrorCategory::WrongSubcommand => analysis.wrong_subcommand += 1,
                    ErrorCategory::FormatError => analysis.format_error += 1,
                    ErrorCategory::EmptyOutput => analysis.empty_output += 1,
                }
            }
            analysis
        })
        .collect();

    analyses.sort_by(|a, b| a.model.cmp(&b.model));
    analyses
}

/// Write error analysis to CSV.
pub fn write_error_analysis_csv<W: Write>(
    writer: &mut W,
    analyses: &[ErrorAnalysis],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "model,ablation,total_errors,missing_flag,extra_flag,wrong_value,\
         flag_reorder,wrong_subcommand,format_error,empty_output"
    )?;
    for a in analyses {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{}",
            a.model,
            a.ablation,
            a.total_errors,
            a.missing_flag,
            a.extra_flag,
            a.wrong_value,
            a.flag_reorder,
            a.wrong_subcommand,
            a.format_error,
            a.empty_output,
        )?;
    }
    Ok(())
}

/// Write per-(tool, model, ablation) summary to CSV.
///
/// Columns: `tool,category,model,ablation,n_trials,accuracy,exact_match_rate,avg_flag_recall,consistency`
pub fn write_tool_model_summary_csv<W: Write>(
    writer: &mut W,
    summaries: &[ToolModelSummary],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "tool,category,model,ablation,n_trials,accuracy,exact_match_rate,avg_flag_recall,consistency"
    )?;
    for s in summaries {
        writeln!(
            writer,
            "{},{},{},{},{},{:.4},{:.4},{:.4},{:.4}",
            s.tool,
            s.category,
            s.model,
            s.ablation,
            s.n_trials,
            s.accuracy,
            s.exact_match_rate,
            s.avg_flag_recall,
            s.consistency,
        )?;
    }
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::scenario::{Scenario, UsageDescription};

    fn sample_scenarios() -> Vec<Scenario> {
        vec![
            Scenario {
                tool: "samtools".to_string(),
                scenario_id: "samtools_01".to_string(),
                reference_args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
                task_description: "sort a BAM file".to_string(),
                category: "alignment".to_string(),
            },
            Scenario {
                tool: "samtools".to_string(),
                scenario_id: "samtools_02".to_string(),
                reference_args: "index sorted.bam".to_string(),
                task_description: "index a BAM file".to_string(),
                category: "alignment".to_string(),
            },
        ]
    }

    fn sample_descriptions() -> Vec<UsageDescription> {
        vec![
            UsageDescription {
                tool: "samtools".to_string(),
                scenario_id: "samtools_01".to_string(),
                desc_id: "samtools_01_01".to_string(),
                user_level: "beginner".to_string(),
                description: "sort a BAM file".to_string(),
            },
            UsageDescription {
                tool: "samtools".to_string(),
                scenario_id: "samtools_02".to_string(),
                desc_id: "samtools_02_01".to_string(),
                user_level: "beginner".to_string(),
                description: "index a BAM file".to_string(),
            },
        ]
    }

    #[test]
    fn test_run_benchmark_with_fixed_generator() {
        let gtor = FixedGenerator {
            args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            format_valid: true,
        };
        let trials = run_benchmark(
            "mock-model",
            2,
            &sample_descriptions(),
            &sample_scenarios(),
            &gtor,
            "full",
        );
        // 2 descriptions × 2 repeats = 4 trials
        assert_eq!(trials.len(), 4);
        // First two trials should match the first scenario perfectly.
        assert!(trials[0].exact_match);
        assert_eq!(trials[0].flag_recall, 1.0);
    }

    #[test]
    fn test_run_benchmark_with_echo_generator() {
        // The echo generator returns the task description as args.
        let gtor = EchoGenerator;
        let trials = run_benchmark(
            "echo",
            1,
            &sample_descriptions(),
            &sample_scenarios(),
            &gtor,
            "full",
        );
        assert_eq!(trials.len(), 2);
        // The descriptions don't match the reference args.
        assert!(!trials[0].exact_match);
    }

    #[test]
    fn test_aggregate_results() {
        let gtor = FixedGenerator {
            args: "sort -@ 4 -o sorted.bam input.bam".to_string(),
            format_valid: true,
        };
        let trials = run_benchmark(
            "mock-model",
            3,
            &sample_descriptions(),
            &sample_scenarios(),
            &gtor,
            "full",
        );
        let agg = aggregate_results(&trials);
        assert_eq!(agg.len(), 1);
        assert_eq!(agg[0].model, "mock-model");
        assert_eq!(agg[0].n_trials, 6); // 2 descs × 3 repeats
        assert!(agg[0].format_valid_rate > 0.99);
    }

    #[test]
    fn test_consistency_all_same() {
        let trials: Vec<TrialResult> = (0..3)
            .map(|i| TrialResult {
                tool: "t".to_string(),
                category: "testing".to_string(),
                scenario_id: "s01".to_string(),
                desc_id: "d01".to_string(),
                model: "m".to_string(),
                ablation: "full".to_string(),
                repeat_idx: i,
                generated_args: "sort -o out.bam in.bam".to_string(),
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: true,
                token_jaccard: 1.0,
                flag_recall: 1.0,
                flag_precision: 1.0,
                flag_group_recall: 1.0,
                flag_group_precision: 1.0,
                flag_group_jaccard: 1.0,
                positional_order_match: 1.0,
                subcommand_match: true,
                accuracy_score: 1.0,
                latency_ms: 100.0,
                tokens: 10,
                format_valid: true,
            })
            .collect();
        let refs: Vec<&TrialResult> = trials.iter().collect();
        assert_eq!(compute_trial_consistency(&refs), 1.0);
    }

    #[test]
    fn test_consistency_mixed() {
        let mut trials: Vec<TrialResult> = (0..3)
            .map(|i| TrialResult {
                tool: "t".to_string(),
                category: "testing".to_string(),
                scenario_id: "s01".to_string(),
                desc_id: "d01".to_string(),
                model: "m".to_string(),
                ablation: "full".to_string(),
                repeat_idx: i,
                generated_args: "sort -o out.bam in.bam".to_string(),
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: true,
                token_jaccard: 1.0,
                flag_recall: 1.0,
                flag_precision: 1.0,
                flag_group_recall: 1.0,
                flag_group_precision: 1.0,
                flag_group_jaccard: 1.0,
                positional_order_match: 1.0,
                subcommand_match: true,
                accuracy_score: 1.0,
                latency_ms: 100.0,
                tokens: 10,
                format_valid: true,
            })
            .collect();
        trials[2].generated_args = "sort -o different.bam in.bam".to_string();
        let refs: Vec<&TrialResult> = trials.iter().collect();
        assert!(compute_trial_consistency(&refs) < 1.0);
    }

    #[test]
    fn test_write_trials_csv() {
        let gtor = FixedGenerator {
            args: "sort -o out.bam in.bam".to_string(),
            format_valid: true,
        };
        let trials = run_benchmark(
            "mock",
            1,
            &sample_descriptions()[..1],
            &sample_scenarios(),
            &gtor,
            "full",
        );
        let mut buf = Vec::new();
        write_trials_csv(&mut buf, &trials).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,category,scenario_id,desc_id,model,ablation,repeat,"));
        // One header + one data row.
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_write_model_agg_csv() {
        let agg = vec![ModelAggResult {
            model: "mock".to_string(),
            ablation: "full".to_string(),
            n_trials: 10,
            accuracy: 0.85,
            exact_match_rate: 0.5,
            avg_flag_recall: 0.9,
            avg_flag_precision: 0.8,
            avg_token_jaccard: 0.75,
            subcommand_match_rate: 0.95,
            consistency: 0.7,
            avg_latency_ms: 250.0,
            avg_tokens: 50.0,
            format_valid_rate: 1.0,
        }];
        let mut buf = Vec::new();
        write_model_agg_csv(&mut buf, &agg).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("model,ablation,n_trials,accuracy,"));
        assert!(text.contains("mock"));
        assert!(text.contains("full"));
    }

    #[test]
    fn test_parse_dry_run_json() {
        let json = r#"{"tool":"samtools","args":"sort -o out.bam in.bam","command":"samtools sort -o out.bam in.bam","dry_run":true}"#;
        let resp = parse_dry_run_json(json);
        assert_eq!(resp.args, "sort -o out.bam in.bam");
        assert!(resp.format_valid);
    }

    #[test]
    fn test_parse_dry_run_json_invalid() {
        let resp = parse_dry_run_json("not json");
        assert!(!resp.format_valid);
        assert!(resp.args.is_empty());
    }

    #[test]
    fn test_run_mock_benchmark() {
        let trials = run_mock_benchmark(
            "gpt-4o-mini",
            2,
            &sample_descriptions(),
            &sample_scenarios(),
        );
        // 2 descriptions × 2 repeats = 4 trials.
        assert_eq!(trials.len(), 4);
        // All should have format_valid = true (mock always succeeds).
        assert!(trials.iter().all(|t| t.format_valid));
        // All should have non-empty reference_args.
        assert!(trials.iter().all(|t| !t.reference_args.is_empty()));
    }

    #[test]
    fn test_mock_benchmark_different_models_different_accuracy() {
        let descs = sample_descriptions();
        let scenarios = sample_scenarios();

        // Run with many repeats to expose probabilistic differences.
        let trials_strong = run_mock_benchmark("gpt-4o", 10, &descs, &scenarios);
        let trials_weak = run_mock_benchmark("gpt-4o-mini", 10, &descs, &scenarios);

        let exact_strong = trials_strong.iter().filter(|t| t.exact_match).count();
        let exact_weak = trials_weak.iter().filter(|t| t.exact_match).count();

        // gpt-4o has lower perturbation rate → more exact matches.
        assert!(
            exact_strong >= exact_weak,
            "strong model ({exact_strong}) should have >= exact matches than weak ({exact_weak})"
        );
    }

    #[test]
    fn test_perturb_args_deterministic() {
        let a = perturb_args("sort -@ 4 -o out.bam in.bam", "model", "d01", 0, 0.5);
        let b = perturb_args("sort -@ 4 -o out.bam in.bam", "model", "d01", 0, 0.5);
        assert_eq!(a, b, "same inputs should produce same perturbation");
    }

    #[test]
    fn test_perturb_args_zero_rate() {
        let result = perturb_args("sort -@ 4 -o out.bam in.bam", "model", "d01", 0, 0.0);
        assert_eq!(result, "sort -@ 4 -o out.bam in.bam");
    }

    #[test]
    fn test_summarise_by_tool_groups_correctly() {
        // Two tools, two models, three repeats each.
        let mut trials = Vec::new();
        for model in &["gpt-4o", "gpt-4o-mini"] {
            for (tool, sc, args_ref) in &[
                ("samtools", "samtools_01", "sort -o out.bam in.bam"),
                ("bwa", "bwa_01", "mem -t 8 ref.fa R1.fq R2.fq"),
            ] {
                for rep in 0..3usize {
                    trials.push(TrialResult {
                        tool: tool.to_string(),
                        category: "alignment".to_string(),
                        scenario_id: sc.to_string(),
                        desc_id: format!("{sc}_01"),
                        model: model.to_string(),
                        ablation: "full".to_string(),
                        repeat_idx: rep,
                        generated_args: args_ref.to_string(),
                        reference_args: args_ref.to_string(),
                        exact_match: true,
                        token_jaccard: 1.0,
                        flag_recall: 1.0,
                        flag_precision: 1.0,
                        flag_group_recall: 1.0,
                        flag_group_precision: 1.0,
                        flag_group_jaccard: 1.0,
                        positional_order_match: 1.0,
                        subcommand_match: true,
                        accuracy_score: 1.0,
                        latency_ms: 0.0,
                        tokens: 5,
                        format_valid: true,
                    });
                }
            }
        }
        let summaries = summarise_by_tool(&trials);
        // 2 tools × 2 models = 4 rows
        assert_eq!(summaries.len(), 4);
        // Sorted by tool then model
        assert_eq!(summaries[0].tool, "bwa");
        assert_eq!(summaries[2].tool, "samtools");
        // All accuracy should be 1.0
        assert!(summaries.iter().all(|s| (s.accuracy - 1.0).abs() < 1e-6));
        assert!(summaries.iter().all(|s| (s.consistency - 1.0).abs() < 1e-6));
    }

    #[test]
    fn test_write_tool_model_summary_csv_header_and_rows() {
        let trials =
            run_mock_benchmark("test-model", 2, &sample_descriptions(), &sample_scenarios());
        let summaries = summarise_by_tool(&trials);
        let mut buf = Vec::new();
        write_tool_model_summary_csv(&mut buf, &summaries).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with(
            "tool,category,model,ablation,n_trials,accuracy,exact_match_rate,avg_flag_recall,consistency"
        ));
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), summaries.len());
        // Each row has 9 comma-separated fields (added ablation)
        for line in &data_lines {
            assert_eq!(line.split(',').count(), 9, "bad row: {line}");
        }
    }

    // ── Baseline tests ────────────────────────────────────────────────────

    #[test]
    fn test_run_mock_baseline() {
        let trials = run_mock_baseline(
            "gpt-4o-mini",
            2,
            &sample_descriptions(),
            &sample_scenarios(),
        );
        // 2 descriptions × 2 repeats = 4 trials.
        assert_eq!(trials.len(), 4);
        // All should have format_valid = true.
        assert!(trials.iter().all(|t| t.format_valid));
        // Model name should carry "(baseline)" suffix.
        assert!(trials.iter().all(|t| t.model == "gpt-4o-mini (baseline)"));
    }

    #[test]
    fn test_baseline_worse_than_enhanced() {
        let descs = sample_descriptions();
        let scenarios = sample_scenarios();

        // Use many repeats to expose the difference.
        let enhanced = run_mock_benchmark("gpt-4o-mini", 20, &descs, &scenarios);
        let baseline = run_mock_baseline("gpt-4o-mini", 20, &descs, &scenarios);

        let enhanced_exact_matches = enhanced.iter().filter(|t| t.exact_match).count();
        let baseline_exact_matches = baseline.iter().filter(|t| t.exact_match).count();

        // Baseline has higher perturbation → fewer exact matches.
        assert!(
            enhanced_exact_matches >= baseline_exact_matches,
            "enhanced ({enhanced_exact_matches}) should have >= exact matches than baseline ({baseline_exact_matches})"
        );
    }

    #[test]
    fn test_compute_baseline_comparison() {
        let descs = sample_descriptions();
        let scenarios = sample_scenarios();

        let enhanced = run_mock_benchmark("gpt-4o", 3, &descs, &scenarios);
        let baseline = run_mock_baseline("gpt-4o", 3, &descs, &scenarios);

        let comparisons = compute_baseline_comparison(&enhanced, &baseline);
        assert_eq!(comparisons.len(), 1);
        assert_eq!(comparisons[0].model, "gpt-4o");
        // Enhanced should be at least as good as baseline (delta >= 0).
        assert!(
            comparisons[0].accuracy_delta >= 0.0,
            "enhanced should be >= baseline accuracy"
        );
    }

    #[test]
    fn test_compute_baseline_comparison_multiple_models() {
        let descs = sample_descriptions();
        let scenarios = sample_scenarios();

        let mut enhanced = run_mock_benchmark("gpt-4o", 3, &descs, &scenarios);
        enhanced.extend(run_mock_benchmark("gpt-4o-mini", 3, &descs, &scenarios));

        let mut baseline = run_mock_baseline("gpt-4o", 3, &descs, &scenarios);
        baseline.extend(run_mock_baseline("gpt-4o-mini", 3, &descs, &scenarios));

        let comparisons = compute_baseline_comparison(&enhanced, &baseline);
        assert_eq!(comparisons.len(), 2);
        // Both models should appear.
        let models: Vec<&str> = comparisons.iter().map(|c| c.model.as_str()).collect();
        assert!(models.contains(&"gpt-4o"));
        assert!(models.contains(&"gpt-4o-mini"));
    }

    #[test]
    fn test_write_baseline_comparison_csv() {
        let descs = sample_descriptions();
        let scenarios = sample_scenarios();

        let enhanced = run_mock_benchmark("gpt-4o", 3, &descs, &scenarios);
        let baseline = run_mock_baseline("gpt-4o", 3, &descs, &scenarios);
        let comparisons = compute_baseline_comparison(&enhanced, &baseline);

        let mut buf = Vec::new();
        write_baseline_comparison_csv(&mut buf, &comparisons).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("model,enhanced_accuracy,baseline_accuracy,"));
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), 1);
        // Each row has 13 comma-separated fields.
        for line in &data_lines {
            assert_eq!(line.split(',').count(), 13, "bad row: {line}");
        }
    }

    #[test]
    fn test_baseline_comparison_empty_inputs() {
        let comparisons = compute_baseline_comparison(&[], &[]);
        assert!(comparisons.is_empty());
    }

    // ── Category summary tests ────────────────────────────────────────────

    #[test]
    fn test_summarise_by_category() {
        let mut trials = Vec::new();
        for model in &["gpt-4o", "gpt-4o-mini"] {
            for (tool, sc, args_ref, cat) in &[
                (
                    "samtools",
                    "samtools_01",
                    "sort -o out.bam in.bam",
                    "alignment",
                ),
                ("bwa", "bwa_01", "mem -t 8 ref.fa R1.fq R2.fq", "alignment"),
                (
                    "gatk",
                    "gatk_01",
                    "HaplotypeCaller -R ref.fa",
                    "variant_calling",
                ),
            ] {
                for rep in 0..3usize {
                    trials.push(TrialResult {
                        tool: tool.to_string(),
                        category: cat.to_string(),
                        scenario_id: sc.to_string(),
                        desc_id: format!("{sc}_01"),
                        model: model.to_string(),
                        ablation: "full".to_string(),
                        repeat_idx: rep,
                        generated_args: args_ref.to_string(),
                        reference_args: args_ref.to_string(),
                        exact_match: true,
                        token_jaccard: 1.0,
                        flag_recall: 1.0,
                        flag_precision: 1.0,
                        flag_group_recall: 1.0,
                        flag_group_precision: 1.0,
                        flag_group_jaccard: 1.0,
                        positional_order_match: 1.0,
                        subcommand_match: true,
                        accuracy_score: 1.0,
                        latency_ms: 0.0,
                        tokens: 5,
                        format_valid: true,
                    });
                }
            }
        }
        let summaries = summarise_by_category(&trials);
        // 2 categories × 2 models = 4 rows
        assert_eq!(summaries.len(), 4);
        // All accuracy should be 1.0
        assert!(summaries.iter().all(|s| (s.accuracy - 1.0).abs() < 1e-6));
        // Alignment has 2 tools, variant_calling has 1
        let alignment = summaries
            .iter()
            .find(|s| s.category == "alignment")
            .unwrap();
        assert_eq!(alignment.n_tools, 2);
        let vc = summaries
            .iter()
            .find(|s| s.category == "variant_calling")
            .unwrap();
        assert_eq!(vc.n_tools, 1);
    }

    #[test]
    fn test_write_category_summary_csv() {
        let trials =
            run_mock_benchmark("test-model", 2, &sample_descriptions(), &sample_scenarios());
        let summaries = summarise_by_category(&trials);
        let mut buf = Vec::new();
        write_category_summary_csv(&mut buf, &summaries).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("category,model,ablation,n_tools,n_trials,"));
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), summaries.len());
        // Each row has 12 comma-separated fields (added ablation).
        for line in &data_lines {
            assert_eq!(line.split(',').count(), 12, "bad row: {line}");
        }
    }

    #[test]
    fn test_proportion_ci95() {
        // 50% proportion with n=100 → wide CI
        let ci = proportion_ci95(0.5, 100);
        assert!(ci > 0.05 && ci < 0.15);
        // 99% proportion with n=10000 → narrow CI
        let ci = proportion_ci95(0.99, 10000);
        assert!(ci < 0.01);
        // n=0 → 0
        assert_eq!(proportion_ci95(0.5, 0), 0.0);
    }

    // ── Error analysis tests ──────────────────────────────────────────────

    #[test]
    fn test_classify_error_missing_flag() {
        let t = TrialResult {
            tool: "t".to_string(),
            category: "c".to_string(),
            scenario_id: "s".to_string(),
            desc_id: "d".to_string(),
            model: "m".to_string(),
            ablation: "full".to_string(),
            repeat_idx: 0,
            generated_args: "sort -o out.bam".to_string(), // missing in.bam
            reference_args: "sort -o out.bam in.bam".to_string(), // has in.bam
            exact_match: false,
            token_jaccard: 0.8,
            flag_recall: 0.75,
            flag_precision: 1.0,
            flag_group_recall: 0.75,
            flag_group_precision: 1.0,
            flag_group_jaccard: 0.75,
            positional_order_match: 1.0,
            subcommand_match: true,
            accuracy_score: 0.9,
            latency_ms: 0.0,
            tokens: 3,
            format_valid: true,
        };
        assert_eq!(classify_error(&t), ErrorCategory::MissingFlag);
    }

    #[test]
    fn test_classify_error_extra_flag() {
        let t = TrialResult {
            tool: "t".to_string(),
            category: "c".to_string(),
            scenario_id: "s".to_string(),
            desc_id: "d".to_string(),
            model: "m".to_string(),
            ablation: "full".to_string(),
            repeat_idx: 0,
            generated_args: "sort -o out.bam in.bam --extra-flag".to_string(),
            reference_args: "sort -o out.bam in.bam".to_string(),
            exact_match: false,
            token_jaccard: 0.8,
            flag_recall: 1.0,
            flag_precision: 0.8,
            flag_group_recall: 1.0,
            flag_group_precision: 0.8,
            flag_group_jaccard: 0.8,
            positional_order_match: 1.0,
            subcommand_match: true,
            accuracy_score: 0.9,
            latency_ms: 0.0,
            tokens: 5,
            format_valid: true,
        };
        assert_eq!(classify_error(&t), ErrorCategory::ExtraFlag);
    }

    #[test]
    fn test_classify_error_wrong_subcommand() {
        let t = TrialResult {
            tool: "t".to_string(),
            category: "c".to_string(),
            scenario_id: "s".to_string(),
            desc_id: "d".to_string(),
            model: "m".to_string(),
            ablation: "full".to_string(),
            repeat_idx: 0,
            generated_args: "view -o out.bam in.bam".to_string(),
            reference_args: "sort -o out.bam in.bam".to_string(),
            exact_match: false,
            token_jaccard: 0.6,
            flag_recall: 0.75,
            flag_precision: 0.75,
            flag_group_recall: 0.75,
            flag_group_precision: 0.75,
            flag_group_jaccard: 0.6,
            positional_order_match: 1.0,
            subcommand_match: false,
            accuracy_score: 0.7,
            latency_ms: 0.0,
            tokens: 4,
            format_valid: true,
        };
        assert_eq!(classify_error(&t), ErrorCategory::WrongSubcommand);
    }

    #[test]
    fn test_classify_error_format_error() {
        let t = TrialResult {
            tool: "t".to_string(),
            category: "c".to_string(),
            scenario_id: "s".to_string(),
            desc_id: "d".to_string(),
            model: "m".to_string(),
            ablation: "full".to_string(),
            repeat_idx: 0,
            generated_args: String::new(),
            reference_args: "sort -o out.bam in.bam".to_string(),
            exact_match: false,
            token_jaccard: 0.0,
            flag_recall: 0.0,
            flag_precision: 0.0,
            flag_group_recall: 0.0,
            flag_group_precision: 0.0,
            flag_group_jaccard: 0.0,
            positional_order_match: 0.0,
            subcommand_match: false,
            accuracy_score: 0.0,
            latency_ms: 0.0,
            tokens: 0,
            format_valid: false,
        };
        assert_eq!(classify_error(&t), ErrorCategory::FormatError);
    }

    #[test]
    fn test_analyse_errors_groups_by_model() {
        let trials = vec![
            TrialResult {
                tool: "t".to_string(),
                category: "c".to_string(),
                scenario_id: "s".to_string(),
                desc_id: "d".to_string(),
                model: "m1".to_string(),
                ablation: "full".to_string(),
                repeat_idx: 0,
                generated_args: "sort -o out.bam".to_string(), // missing flag
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: false,
                token_jaccard: 0.8,
                flag_recall: 0.75,
                flag_precision: 1.0,
                flag_group_recall: 0.75,
                flag_group_precision: 1.0,
                flag_group_jaccard: 0.75,
                positional_order_match: 1.0,
                subcommand_match: true,
                accuracy_score: 0.9,
                latency_ms: 0.0,
                tokens: 3,
                format_valid: true,
            },
            TrialResult {
                tool: "t".to_string(),
                category: "c".to_string(),
                scenario_id: "s".to_string(),
                desc_id: "d".to_string(),
                model: "m1".to_string(),
                ablation: "full".to_string(),
                repeat_idx: 1,
                generated_args: "sort -o out.bam in.bam".to_string(), // exact match
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: true,
                token_jaccard: 1.0,
                flag_recall: 1.0,
                flag_precision: 1.0,
                flag_group_recall: 1.0,
                flag_group_precision: 1.0,
                flag_group_jaccard: 1.0,
                positional_order_match: 1.0,
                subcommand_match: true,
                accuracy_score: 1.0,
                latency_ms: 0.0,
                tokens: 4,
                format_valid: true,
            },
        ];
        let analyses = analyse_errors(&trials);
        assert_eq!(analyses.len(), 1);
        assert_eq!(analyses[0].model, "m1");
        assert_eq!(analyses[0].total_errors, 1);
        assert_eq!(analyses[0].missing_flag, 1);
    }

    #[test]
    fn test_write_error_analysis_csv() {
        let analyses = vec![ErrorAnalysis {
            model: "test".to_string(),
            ablation: "full".to_string(),
            total_errors: 10,
            missing_flag: 3,
            extra_flag: 2,
            wrong_value: 2,
            flag_reorder: 1,
            wrong_subcommand: 1,
            format_error: 0,
            empty_output: 1,
        }];
        let mut buf = Vec::new();
        write_error_analysis_csv(&mut buf, &analyses).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("model,ablation,total_errors,missing_flag,"));
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), 1);
        assert_eq!(data_lines[0].split(',').count(), 10);
    }
}

// ── Incremental result writer ───────────────────────────────────────────────

/// Callback trait for handling trial results as they are generated.
pub trait TrialCallback: Send + Sync {
    fn on_trial(&self, trial: &TrialResult);
    fn on_model_complete(
        &self,
        model: &str,
        ablation: &str,
        agg: &ModelAggResult,
        trials: &[TrialResult],
    );
}

/// Incremental CSV writer that appends trial results as they complete.
pub struct IncrementalCsvWriter {
    trials_file: std::sync::Mutex<std::fs::File>,
    output_dir: std::path::PathBuf,
}

impl IncrementalCsvWriter {
    pub fn new(output_dir: &std::path::Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(output_dir)?;

        // Clean up existing result files to avoid data duplication
        let files_to_clean = [
            "benchmark_trials.csv",
            "model_summary.csv",
            "model_summary_by_tool.csv",
            "model_summary_by_category.csv",
            "error_analysis.csv",
        ];
        for filename in &files_to_clean {
            let path = output_dir.join(filename);
            if path.exists() {
                std::fs::remove_file(&path)?;
            }
        }
        // Also clean up any existing trials_*.csv files
        if let Ok(entries) = std::fs::read_dir(output_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("trials_") && name.ends_with(".csv") {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }

        let trials_path = output_dir.join("benchmark_trials.csv");
        let mut trials_file = std::fs::File::create(&trials_path)?;
        writeln!(
            trials_file,
            "tool,category,scenario_id,desc_id,model,ablation,repeat,generated_args,reference_args,exact_match,token_jaccard,flag_recall,flag_precision,flag_group_recall,flag_group_precision,flag_group_jaccard,positional_order_match,subcommand_match,accuracy_score,latency_ms,tokens,format_valid"
        )?;

        Ok(Self {
            trials_file: std::sync::Mutex::new(trials_file),
            output_dir: output_dir.to_path_buf(),
        })
    }

    pub fn append_trial(&self, trial: &TrialResult) -> std::io::Result<()> {
        let mut file = self.trials_file.lock().unwrap();
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.1},{},{}",
            csv_escape(&trial.tool),
            csv_escape(&trial.category),
            csv_escape(&trial.scenario_id),
            csv_escape(&trial.desc_id),
            csv_escape(&trial.model),
            csv_escape(&trial.ablation),
            trial.repeat_idx,
            csv_escape(&trial.generated_args),
            csv_escape(&trial.reference_args),
            trial.exact_match,
            trial.token_jaccard,
            trial.flag_recall,
            trial.flag_precision,
            trial.flag_group_recall,
            trial.flag_group_precision,
            trial.flag_group_jaccard,
            trial.positional_order_match,
            trial.subcommand_match,
            trial.accuracy_score,
            trial.latency_ms,
            trial.tokens,
            trial.format_valid,
        )?;
        file.sync_all()?;
        Ok(())
    }

    pub fn write_model_summary(&self, agg: &[ModelAggResult]) -> std::io::Result<()> {
        let path = self.output_dir.join("model_summary.csv");
        let file_exists = path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false);

        let mut file = if file_exists {
            std::fs::OpenOptions::new().append(true).open(&path)?
        } else {
            std::fs::File::create(&path)?
        };

        // Write only data rows (skip header if file already exists)
        for a in agg {
            writeln!(
                file,
                "{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.1},{:.1},{:.4}",
                a.model,
                a.ablation,
                a.n_trials,
                a.accuracy,
                a.exact_match_rate,
                a.avg_flag_recall,
                a.avg_flag_precision,
                a.avg_token_jaccard,
                a.subcommand_match_rate,
                a.consistency,
                a.avg_latency_ms,
                a.avg_tokens,
                a.format_valid_rate,
            )?;
        }
        file.sync_all()?;
        Ok(())
    }

    pub fn write_model_trials(&self, model: &str, trials: &[TrialResult]) -> std::io::Result<()> {
        let safe_name = model.replace(['/', ':'], "_");
        let path = self.output_dir.join(format!("trials_{safe_name}.csv"));
        let file_exists = path.exists() && path.metadata().map(|m| m.len() > 0).unwrap_or(false);

        let mut file = if file_exists {
            std::fs::OpenOptions::new().append(true).open(&path)?
        } else {
            let mut f = std::fs::File::create(&path)?;
            writeln!(
                f,
                "tool,category,scenario_id,desc_id,model,ablation,repeat,generated_args,reference_args,exact_match,token_jaccard,flag_recall,flag_precision,flag_group_recall,flag_group_precision,flag_group_jaccard,positional_order_match,subcommand_match,accuracy_score,latency_ms,tokens,format_valid"
            )?;
            f
        };

        for trial in trials {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.1},{},{}",
                csv_escape(&trial.tool),
                csv_escape(&trial.category),
                csv_escape(&trial.scenario_id),
                csv_escape(&trial.desc_id),
                csv_escape(&trial.model),
                csv_escape(&trial.ablation),
                trial.repeat_idx,
                csv_escape(&trial.generated_args),
                csv_escape(&trial.reference_args),
                trial.exact_match,
                trial.token_jaccard,
                trial.flag_recall,
                trial.flag_precision,
                trial.flag_group_recall,
                trial.flag_group_precision,
                trial.flag_group_jaccard,
                trial.positional_order_match,
                trial.subcommand_match,
                trial.accuracy_score,
                trial.latency_ms,
                trial.tokens,
                trial.format_valid,
            )?;
        }
        Ok(())
    }

    pub fn write_summaries(&self, all_trials: &[TrialResult]) -> std::io::Result<()> {
        let tool_summaries = summarise_by_tool(all_trials);
        let path = self.output_dir.join("model_summary_by_tool.csv");
        let mut file = std::fs::File::create(&path)?;
        write_tool_model_summary_csv(&mut file, &tool_summaries)?;

        let cat_summaries = summarise_by_category(all_trials);
        let path = self.output_dir.join("model_summary_by_category.csv");
        let mut file = std::fs::File::create(&path)?;
        write_category_summary_csv(&mut file, &cat_summaries)?;

        let errors = analyse_errors(all_trials);
        let path = self.output_dir.join("error_analysis.csv");
        let mut file = std::fs::File::create(&path)?;
        write_error_analysis_csv(&mut file, &errors)?;

        Ok(())
    }
}

impl TrialCallback for IncrementalCsvWriter {
    fn on_trial(&self, trial: &TrialResult) {
        if let Err(e) = self.append_trial(trial) {
            eprintln!("Warning: failed to write trial: {}", e);
        }
    }

    fn on_model_complete(
        &self,
        model: &str,
        ablation: &str,
        agg: &ModelAggResult,
        trials: &[TrialResult],
    ) {
        let _ = (model, ablation, agg);
        if let Err(e) = self.write_model_trials(model, trials) {
            eprintln!("Warning: failed to write model trials: {}", e);
        }
    }
}
