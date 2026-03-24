//! Benchmark execution engine.
//!
//! Runs each (description, scenario) pair through a *command generator*
//! (normally the `oxo-call dry-run --json` subprocess) and records latency,
//! token count, and comparison metrics against the reference command.
//!
//! The generator is injected as a trait so that unit tests can substitute a
//! mock without requiring a real API token or the `oxo-call` binary.

use crate::bench::compare::{compare_commands, compare_flag_groups};
use crate::bench::scenario::{Scenario, UsageDescription};
use std::io::Write;
use std::time::Instant;

// ── Data types ───────────────────────────────────────────────────────────────

/// Result of a single benchmark trial (one description × one model × one repeat).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrialResult {
    pub tool: String,
    pub scenario_id: String,
    pub desc_id: String,
    pub model: String,
    pub repeat_idx: usize,
    pub generated_args: String,
    pub reference_args: String,
    pub exact_match: bool,
    pub token_jaccard: f64,
    pub flag_recall: f64,
    pub flag_precision: f64,
    pub flag_group_recall: f64,
    pub flag_group_precision: f64,
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
pub trait CommandGenerator {
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

/// Generator that calls `oxo-call dry-run --json` as a subprocess.
pub struct OxoCallGenerator {
    pub binary_path: String,
}

impl CommandGenerator for OxoCallGenerator {
    fn generate(&self, tool: &str, task: &str, model: &str) -> GeneratedCommand {
        let output = std::process::Command::new(&self.binary_path)
            .args(["dry-run", "--json", "-m", model, tool, task])
            .output();

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

        let tokens = json_str.split_whitespace().count();

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
pub fn run_benchmark(
    model: &str,
    repeats: usize,
    descriptions: &[UsageDescription],
    scenarios: &[Scenario],
    generator: &dyn CommandGenerator,
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
            let (fg_recall, fg_precision) =
                compare_flag_groups(&resp.args, &scenario.reference_args);

            results.push(TrialResult {
                tool: desc.tool.clone(),
                scenario_id: desc.scenario_id.clone(),
                desc_id: desc.desc_id.clone(),
                model: model.to_string(),
                repeat_idx: repeat,
                generated_args: resp.args,
                reference_args: scenario.reference_args.clone(),
                exact_match: cmp.exact_match,
                token_jaccard: cmp.token_jaccard,
                flag_recall: cmp.flag_recall,
                flag_precision: cmp.flag_precision,
                flag_group_recall: fg_recall,
                flag_group_precision: fg_precision,
                subcommand_match: cmp.subcommand_match,
                accuracy_score: cmp.accuracy_score(),
                latency_ms,
                tokens: resp.tokens,
                format_valid: resp.format_valid,
            });
        }
    }

    results
}

/// Aggregate trial results into per-model summary metrics.
pub fn aggregate_results(trials: &[TrialResult]) -> Vec<ModelAggResult> {
    let mut by_model: std::collections::HashMap<&str, Vec<&TrialResult>> =
        std::collections::HashMap::new();
    for t in trials {
        by_model.entry(t.model.as_str()).or_default().push(t);
    }

    let mut agg: Vec<ModelAggResult> = by_model
        .into_iter()
        .map(|(model, trials)| {
            let n = trials.len() as f64;
            ModelAggResult {
                model: model.to_string(),
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
        "tool,scenario_id,desc_id,model,repeat,generated_args,reference_args,\
         exact_match,token_jaccard,flag_recall,flag_precision,\
         flag_group_recall,flag_group_precision,subcommand_match,\
         accuracy_score,latency_ms,tokens,format_valid"
    )?;
    for t in trials {
        let gen_esc = csv_escape(&t.generated_args);
        let ref_esc = csv_escape(&t.reference_args);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{},{:.4},{:.1},{},{}",
            t.tool,
            t.scenario_id,
            t.desc_id,
            t.model,
            t.repeat_idx,
            gen_esc,
            ref_esc,
            t.exact_match,
            t.token_jaccard,
            t.flag_recall,
            t.flag_precision,
            t.flag_group_recall,
            t.flag_group_precision,
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
        "model,n_trials,accuracy,exact_match_rate,avg_flag_recall,avg_flag_precision,\
         avg_token_jaccard,subcommand_match_rate,consistency,avg_latency_ms,avg_tokens,\
         format_valid_rate"
    )?;
    for a in agg {
        writeln!(
            writer,
            "{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.1},{:.1},{:.4}",
            a.model,
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
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
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
                scenario_id: "s01".to_string(),
                desc_id: "d01".to_string(),
                model: "m".to_string(),
                repeat_idx: i,
                generated_args: "sort -o out.bam in.bam".to_string(),
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: true,
                token_jaccard: 1.0,
                flag_recall: 1.0,
                flag_precision: 1.0,
                flag_group_recall: 1.0,
                flag_group_precision: 1.0,
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
                scenario_id: "s01".to_string(),
                desc_id: "d01".to_string(),
                model: "m".to_string(),
                repeat_idx: i,
                generated_args: "sort -o out.bam in.bam".to_string(),
                reference_args: "sort -o out.bam in.bam".to_string(),
                exact_match: true,
                token_jaccard: 1.0,
                flag_recall: 1.0,
                flag_precision: 1.0,
                flag_group_recall: 1.0,
                flag_group_precision: 1.0,
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
        );
        let mut buf = Vec::new();
        write_trials_csv(&mut buf, &trials).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.starts_with("tool,scenario_id,desc_id,model,repeat,"));
        // One header + one data row.
        let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_write_model_agg_csv() {
        let agg = vec![ModelAggResult {
            model: "mock".to_string(),
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
        assert!(text.starts_with("model,n_trials,accuracy,"));
        assert!(text.contains("mock"));
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
}
