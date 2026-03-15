//! Benchmark report generation.
//!
//! Formats results from workflow and LLM benchmarks into human-readable
//! tables, machine-readable JSON, and CSV files for integration with
//! CI/reporting pipelines and the docs/ landing page.

use crate::bench::llm::{EvalTask, ModelBenchResult};
use crate::bench::workflow::BenchWorkflowResult;
use crate::sim::omics::OmicsScenario;
use std::io::Write;

/// Print a Markdown table of workflow benchmark results.
pub fn print_workflow_report<W: Write>(
    writer: &mut W,
    results: &[BenchWorkflowResult],
) -> std::io::Result<()> {
    writeln!(writer, "## Workflow benchmark results\n")?;
    writeln!(
        writer,
        "| Workflow | Tasks | Parse (µs) | Expand (µs) | Cycle? |"
    )?;
    writeln!(
        writer,
        "|----------|-------|------------|-------------|--------|"
    )?;
    for r in results {
        writeln!(
            writer,
            "| {} | {} | {:.1} | {:.1} | {} |",
            r.workflow_name,
            r.expanded_tasks,
            r.parse_ns as f64 / 1_000.0,
            r.expand_ns as f64 / 1_000.0,
            if r.has_cycle { "⚠ YES" } else { "✓ no" },
        )?;
    }
    Ok(())
}

/// Print a Markdown table of LLM model benchmark results.
pub fn print_model_report<W: Write>(
    writer: &mut W,
    results: &[ModelBenchResult],
) -> std::io::Result<()> {
    writeln!(writer, "## LLM model evaluation results\n")?;
    writeln!(
        writer,
        "| Model | Tool | Category | Accuracy | Format% | Consistency |"
    )?;
    writeln!(
        writer,
        "|-------|------|----------|----------|---------|-------------|"
    )?;
    for r in results {
        writeln!(
            writer,
            "| {} | {} | {} | {:.0}% | {:.0}% | {:.0}% |",
            r.model,
            r.tool,
            r.category,
            r.accuracy() * 100.0,
            r.format_validity_rate * 100.0,
            r.self_consistency_rate * 100.0,
        )?;
    }
    Ok(())
}

/// Summarise LLM results by model (aggregate across tools).
pub fn summarise_by_model(results: &[ModelBenchResult]) -> Vec<ModelSummary> {
    let mut map: std::collections::HashMap<String, ModelSummary> = std::collections::HashMap::new();

    for r in results {
        let entry = map
            .entry(r.model.clone())
            .or_insert_with(|| ModelSummary::new(&r.model));
        entry.n_tasks += 1;
        entry.total_correct += r.correct_count;
        entry.total_attempts += r.total_count;
        entry.sum_format_rate += r.format_validity_rate;
        entry.sum_consistency += r.self_consistency_rate;
        if let Some(lat) = r.avg_latency_ms {
            entry.sum_latency_ms += lat;
            entry.n_latency_samples += 1;
        }
    }

    let mut summaries: Vec<ModelSummary> = map.into_values().collect();
    summaries.sort_by(|a, b| {
        b.overall_accuracy()
            .partial_cmp(&a.overall_accuracy())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    summaries
}

/// Aggregate statistics for a single model across all evaluated tasks.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelSummary {
    pub model: String,
    pub n_tasks: usize,
    pub total_correct: usize,
    pub total_attempts: usize,
    pub sum_format_rate: f64,
    pub sum_consistency: f64,
    pub sum_latency_ms: f64,
    pub n_latency_samples: usize,
}

impl ModelSummary {
    fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            n_tasks: 0,
            total_correct: 0,
            total_attempts: 0,
            sum_format_rate: 0.0,
            sum_consistency: 0.0,
            sum_latency_ms: 0.0,
            n_latency_samples: 0,
        }
    }

    pub fn overall_accuracy(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.total_correct as f64 / self.total_attempts as f64
        }
    }

    pub fn avg_format_rate(&self) -> f64 {
        if self.n_tasks == 0 {
            0.0
        } else {
            self.sum_format_rate / self.n_tasks as f64
        }
    }

    pub fn avg_consistency(&self) -> f64 {
        if self.n_tasks == 0 {
            0.0
        } else {
            self.sum_consistency / self.n_tasks as f64
        }
    }

    pub fn avg_latency_ms(&self) -> Option<f64> {
        if self.n_latency_samples == 0 {
            None
        } else {
            Some(self.sum_latency_ms / self.n_latency_samples as f64)
        }
    }
}

/// Print a Markdown summary table grouped by model.
pub fn print_model_summary<W: Write>(
    writer: &mut W,
    summaries: &[ModelSummary],
) -> std::io::Result<()> {
    writeln!(writer, "## Model summary\n")?;
    writeln!(
        writer,
        "| Model | Tasks | Accuracy | Format% | Consistency | Avg Latency |"
    )?;
    writeln!(
        writer,
        "|-------|-------|----------|---------|-------------|-------------|"
    )?;
    for s in summaries {
        let lat = s
            .avg_latency_ms()
            .map(|ms| format!("{ms:.0} ms"))
            .unwrap_or_else(|| "N/A".to_string());
        writeln!(
            writer,
            "| {} | {} | {:.0}% | {:.0}% | {:.0}% | {} |",
            s.model,
            s.n_tasks,
            s.overall_accuracy() * 100.0,
            s.avg_format_rate() * 100.0,
            s.avg_consistency() * 100.0,
            lat,
        )?;
    }
    Ok(())
}

// ── CSV export ────────────────────────────────────────────────────────────────

/// Write workflow benchmark results as a CSV file.
///
/// Columns: `workflow,expanded_tasks,parse_us,expand_us,cycle_free`
pub fn write_workflow_csv<W: Write>(
    writer: &mut W,
    results: &[BenchWorkflowResult],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "workflow,expanded_tasks,parse_us,expand_us,cycle_free"
    )?;
    for r in results {
        writeln!(
            writer,
            "{},{},{:.1},{:.1},{}",
            r.workflow_name,
            r.expanded_tasks,
            r.parse_ns as f64 / 1_000.0,
            r.expand_ns as f64 / 1_000.0,
            !r.has_cycle,
        )?;
    }
    Ok(())
}

/// Write omics simulation scenario statistics as a CSV file.
///
/// Columns: `scenario_id,assay,n_samples,read_len_bp,reads_per_sample,error_rate,total_reads`
pub fn write_scenarios_csv<W: Write>(
    writer: &mut W,
    scenarios: &[OmicsScenario],
) -> std::io::Result<()> {
    writeln!(
        writer,
        "scenario_id,assay,n_samples,read_len_bp,reads_per_sample,error_rate,total_reads"
    )?;
    for s in scenarios {
        let total = s.samples.len() * s.reads_per_sample;
        writeln!(
            writer,
            "{},{},{},{},{},{:.3},{}",
            s.id,
            s.assay,
            s.samples.len(),
            s.read_len,
            s.reads_per_sample,
            s.error_rate,
            total,
        )?;
    }
    Ok(())
}

/// Write the canonical LLM evaluation task catalog as a CSV file.
///
/// Columns: `category,tool,task_description,required_patterns`
pub fn write_eval_tasks_csv<W: Write>(writer: &mut W, tasks: &[EvalTask]) -> std::io::Result<()> {
    writeln!(writer, "category,tool,task_description,required_patterns")?;
    for t in tasks {
        // Escape double-quotes inside fields by doubling them (RFC 4180).
        let task_esc = t.task.replace('"', "\"\"");
        let patterns_esc = t.required_patterns.join(";").replace('"', "\"\"");
        writeln!(
            writer,
            "{},{},\"{}\",\"{}\"",
            t.category, t.tool, task_esc, patterns_esc,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::llm::canonical_eval_tasks;
    use crate::bench::workflow::bench_workflow_expand;
    use crate::sim::omics::canonical_scenarios;

    #[test]
    fn test_print_workflow_report() {
        let results = bench_workflow_expand(1);
        let mut buf = Vec::new();
        print_workflow_report(&mut buf, &results).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Workflow benchmark results"));
        assert!(text.contains("rnaseq"));
    }

    #[test]
    fn test_write_workflow_csv_header_and_rows() {
        let results = bench_workflow_expand(1);
        let mut buf = Vec::new();
        write_workflow_csv(&mut buf, &results).unwrap();
        let text = String::from_utf8(buf).unwrap();
        // Header
        assert!(text.starts_with("workflow,expanded_tasks,parse_us,expand_us,cycle_free"));
        // One row per workflow
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), results.len());
        // Each row has 5 comma-separated fields
        for line in &data_lines {
            assert_eq!(line.split(',').count(), 5, "bad CSV row: {line}");
        }
        // rnaseq row should contain "true" for cycle_free
        assert!(text.contains("rnaseq,"), "rnaseq missing from workflow CSV");
        assert!(text.contains(",true"), "all workflows should be cycle-free");
    }

    #[test]
    fn test_write_scenarios_csv_header_and_rows() {
        let scenarios = canonical_scenarios();
        let mut buf = Vec::new();
        write_scenarios_csv(&mut buf, &scenarios).unwrap();
        let text = String::from_utf8(buf).unwrap();
        // Header
        assert!(text.starts_with(
            "scenario_id,assay,n_samples,read_len_bp,reads_per_sample,error_rate,total_reads"
        ));
        // One data row per scenario
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), scenarios.len());
        // Each row has 7 fields
        for line in &data_lines {
            assert_eq!(line.split(',').count(), 7, "bad CSV row: {line}");
        }
    }

    #[test]
    fn test_write_eval_tasks_csv_header_and_rows() {
        let tasks = canonical_eval_tasks();
        let mut buf = Vec::new();
        write_eval_tasks_csv(&mut buf, &tasks).unwrap();
        let text = String::from_utf8(buf).unwrap();
        // Header
        assert!(text.starts_with("category,tool,task_description,required_patterns"));
        // Correct number of data lines
        let data_lines: Vec<&str> = text.lines().skip(1).filter(|l| !l.is_empty()).collect();
        assert_eq!(data_lines.len(), tasks.len());
        // Quoted task descriptions are present
        assert!(
            text.contains("alignment"),
            "category 'alignment' missing from eval CSV"
        );
        assert!(
            text.contains("samtools"),
            "tool 'samtools' missing from eval CSV"
        );
    }

    #[test]
    fn test_summarise_by_model() {
        let results = vec![
            ModelBenchResult {
                model: "gpt-4o-mini".to_string(),
                tool: "samtools".to_string(),
                task_summary: "sort bam".to_string(),
                category: "qc".to_string(),
                correct_count: 3,
                total_count: 3,
                format_validity_rate: 1.0,
                self_consistency_rate: 1.0,
                avg_latency_ms: Some(250.0),
            },
            ModelBenchResult {
                model: "gpt-4o-mini".to_string(),
                tool: "bwa".to_string(),
                task_summary: "align reads".to_string(),
                category: "alignment".to_string(),
                correct_count: 2,
                total_count: 3,
                format_validity_rate: 1.0,
                self_consistency_rate: 0.67,
                avg_latency_ms: Some(300.0),
            },
        ];
        let summaries = summarise_by_model(&results);
        assert_eq!(summaries.len(), 1);
        let s = &summaries[0];
        assert_eq!(s.model, "gpt-4o-mini");
        assert_eq!(s.total_correct, 5);
        assert_eq!(s.total_attempts, 6);
    }

    // ─── print_model_report ───────────────────────────────────────────────────

    #[test]
    fn test_print_model_report_header_and_rows() {
        let results = vec![
            ModelBenchResult {
                model: "gpt-4o".to_string(),
                tool: "samtools".to_string(),
                task_summary: "sort bam".to_string(),
                category: "alignment".to_string(),
                correct_count: 3,
                total_count: 3,
                format_validity_rate: 1.0,
                self_consistency_rate: 1.0,
                avg_latency_ms: None,
            },
            ModelBenchResult {
                model: "gpt-4o-mini".to_string(),
                tool: "bwa".to_string(),
                task_summary: "align reads".to_string(),
                category: "alignment".to_string(),
                correct_count: 2,
                total_count: 3,
                format_validity_rate: 0.9,
                self_consistency_rate: 0.8,
                avg_latency_ms: Some(100.0),
            },
        ];
        let mut buf = Vec::new();
        print_model_report(&mut buf, &results).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("LLM model evaluation results"));
        assert!(text.contains("gpt-4o"));
        assert!(text.contains("gpt-4o-mini"));
        assert!(text.contains("samtools"));
        assert!(text.contains("bwa"));
    }

    #[test]
    fn test_print_model_report_empty() {
        let mut buf = Vec::new();
        print_model_report(&mut buf, &[]).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("LLM model evaluation results"));
    }

    // ─── print_model_summary ──────────────────────────────────────────────────

    #[test]
    fn test_print_model_summary_header_and_rows() {
        let results = vec![ModelBenchResult {
            model: "gpt-4o".to_string(),
            tool: "samtools".to_string(),
            task_summary: "sort bam".to_string(),
            category: "alignment".to_string(),
            correct_count: 5,
            total_count: 5,
            format_validity_rate: 1.0,
            self_consistency_rate: 0.9,
            avg_latency_ms: Some(200.0),
        }];
        let summaries = summarise_by_model(&results);
        let mut buf = Vec::new();
        print_model_summary(&mut buf, &summaries).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("Model summary"));
        assert!(text.contains("gpt-4o"));
        assert!(text.contains("200 ms"));
    }

    #[test]
    fn test_print_model_summary_no_latency() {
        let results = vec![ModelBenchResult {
            model: "test-model".to_string(),
            tool: "samtools".to_string(),
            task_summary: "sort bam".to_string(),
            category: "alignment".to_string(),
            correct_count: 3,
            total_count: 5,
            format_validity_rate: 0.8,
            self_consistency_rate: 0.7,
            avg_latency_ms: None, // No latency samples
        }];
        let summaries = summarise_by_model(&results);
        let mut buf = Vec::new();
        print_model_summary(&mut buf, &summaries).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains("N/A"), "missing latency should show N/A");
    }

    // ─── ModelSummary methods ─────────────────────────────────────────────────

    #[test]
    fn test_model_summary_no_attempts_zero_accuracy() {
        let s = ModelSummary {
            model: "test".to_string(),
            n_tasks: 0,
            total_correct: 0,
            total_attempts: 0,
            sum_format_rate: 0.0,
            sum_consistency: 0.0,
            sum_latency_ms: 0.0,
            n_latency_samples: 0,
        };
        assert_eq!(s.overall_accuracy(), 0.0);
        assert_eq!(s.avg_format_rate(), 0.0);
        assert_eq!(s.avg_consistency(), 0.0);
        assert!(s.avg_latency_ms().is_none());
    }

    #[test]
    fn test_model_summary_with_data() {
        let s = ModelSummary {
            model: "gpt-4o".to_string(),
            n_tasks: 2,
            total_correct: 4,
            total_attempts: 5,
            sum_format_rate: 1.8,
            sum_consistency: 1.6,
            sum_latency_ms: 500.0,
            n_latency_samples: 2,
        };
        assert!((s.overall_accuracy() - 0.8).abs() < 1e-9);
        assert!((s.avg_format_rate() - 0.9).abs() < 1e-9);
        assert!((s.avg_consistency() - 0.8).abs() < 1e-9);
        assert_eq!(s.avg_latency_ms(), Some(250.0));
    }
}
