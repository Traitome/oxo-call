//! Benchmark report generation.
//!
//! Formats results from workflow and LLM benchmarks into human-readable
//! tables and machine-readable JSON for integration with CI/reporting pipelines.

use crate::bench::llm::ModelBenchResult;
use crate::bench::workflow::BenchWorkflowResult;
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
    let mut map: std::collections::HashMap<String, ModelSummary> =
        std::collections::HashMap::new();

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::workflow::bench_workflow_expand;

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
}
