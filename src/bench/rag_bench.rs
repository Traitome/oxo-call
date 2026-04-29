//! RAG Benchmark Runner (HDA)
//!
//! Executes benchmark tests using schema-based document processing.

#![allow(dead_code)]

use crate::bench::metrics::{AccuracyMetrics, ComparisonReport, TestResult};
use crate::bench::test_cases::{TestCase, TestSuite};
use crate::doc_processor::DocProcessor;
use crate::schema::parse_help;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub model: String,
    pub enable_rag: bool,
    pub max_retries: usize,
    pub timeout_secs: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            model: "github-copilot/claude-3.5-sonnet".to_string(),
            enable_rag: true,
            max_retries: 1,
            timeout_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub config: BenchmarkConfig,
    pub metrics: AccuracyMetrics,
    pub test_results: Vec<TestResult>,
    pub total_duration_ms: u64,
    pub avg_latency_ms: f64,
}

impl BenchmarkResult {
    pub fn print_summary(&self) {
        println!("\n=== Benchmark Result ===");
        println!("Model: {}", self.config.model);
        println!("RAG Enabled: {}", self.config.enable_rag);
        println!(
            "Total Duration: {:.1}s",
            self.total_duration_ms as f64 / 1000.0
        );
        println!("Average Latency: {:.1}ms", self.avg_latency_ms);
        self.metrics.print_summary();
    }
}

pub struct RagBenchmark {
    config: BenchmarkConfig,
    doc_processor: DocProcessor,
}

impl RagBenchmark {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config: config.clone(),
            doc_processor: DocProcessor::new(),
        }
    }

    pub fn run_comparison(&self, suite: &TestSuite) -> ComparisonReport {
        println!("\n========================================");
        println!("Running RAG Benchmark Comparison");
        println!("Test Suite: {}", suite.name);
        println!("Total Cases: {}", suite.cases.len());
        println!("========================================\n");

        println!("[1/2] Running BASELINE (doc mode without schema)...");
        let baseline_config = BenchmarkConfig {
            enable_rag: false,
            ..self.config.clone()
        };
        let baseline = self.run_with_config(&suite.cases, &baseline_config);
        baseline.print_summary();

        println!("\n[2/2] Running SCHEMA-ENHANCED mode...");
        let rag_config = BenchmarkConfig {
            enable_rag: true,
            ..self.config.clone()
        };
        let rag_enhanced = self.run_with_config(&suite.cases, &rag_config);
        rag_enhanced.print_summary();

        ComparisonReport::new(baseline.metrics, rag_enhanced.metrics)
    }

    fn run_with_config(&self, cases: &[TestCase], config: &BenchmarkConfig) -> BenchmarkResult {
        let mut metrics = AccuracyMetrics::new();
        let mut test_results = Vec::new();
        let start_time = Instant::now();
        let mut total_latency_ms = 0u64;

        for (i, case) in cases.iter().enumerate() {
            println!(
                "  [{}/{}] Testing: {} - {}",
                i + 1,
                cases.len(),
                case.tool,
                case.task
            );
            let result = self.run_single_test(case, config);
            metrics.add_result(
                &case.tool,
                case.pattern_type.as_str(),
                &case.difficulty,
                result.is_correct,
                result.error_message.is_some(),
            );
            total_latency_ms += result.latency_ms;
            test_results.push(result);
        }

        let total_duration_ms = start_time.elapsed().as_millis() as u64;
        let avg_latency_ms = if !test_results.is_empty() {
            total_latency_ms as f64 / test_results.len() as f64
        } else {
            0.0
        };

        BenchmarkResult {
            config: config.clone(),
            metrics,
            test_results,
            total_duration_ms,
            avg_latency_ms,
        }
    }

    fn run_single_test(&self, case: &TestCase, config: &BenchmarkConfig) -> TestResult {
        let start_time = Instant::now();
        let documentation = self.fetch_tool_documentation(&case.tool);
        let _structured_doc = self.doc_processor.process(&documentation);

        if config.enable_rag {
            let _schema = parse_help(&case.tool, &documentation);
        }

        let predicted = match self.generate_command(&case.tool, &case.task, config) {
            Ok(cmd) => cmd,
            Err(e) => {
                let latency_ms = start_time.elapsed().as_millis() as u64;
                let mut result = TestResult::new(&case.tool, &case.task, &case.expected_args, "");
                result.latency_ms = latency_ms;
                result.error_message = Some(e);
                return result;
            }
        };

        let latency_ms = start_time.elapsed().as_millis() as u64;
        let mut result = TestResult::new(&case.tool, &case.task, &case.expected_args, &predicted);
        result.latency_ms = latency_ms;
        result
    }

    fn fetch_tool_documentation(&self, tool: &str) -> String {
        match Command::new(tool).arg("--help").output() {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    self.get_mock_documentation(tool)
                }
            }
            Err(_) => self.get_mock_documentation(tool),
        }
    }

    fn get_mock_documentation(&self, tool: &str) -> String {
        match tool {
            "samtools" => r#"Usage: samtools <command> [options]

Commands:
  sort      Sort alignment file
  index     Index alignment file
  view      View alignment file
  flagstat  Generate flag statistics

Options:
  -o FILE   Output file
  -@ INT    Number of threads
  -b        Output BAM format
  -h        Include header
"#
            .to_string(),
            "bcftools" => r#"Usage: bcftools <command> [options]

Commands:
  view      View VCF/BCF file
  filter    Filter variants
  query     Query VCF/BCF

Options:
  -h        Include header
  -i EXPR   Filter expression
  -o FILE   Output file
"#
            .to_string(),
            "bedtools" => r#"Usage: bedtools <command> [options]

Commands:
  intersect Find overlapping intervals
  merge     Merge overlapping regions

Options:
  -a FILE   Input file A
  -b FILE   Input file B
"#
            .to_string(),
            _ => format!(
                "Usage: {} [options]\n\nOptions:\n  -o FILE  Output file\n  -v       Verbose",
                tool
            ),
        }
    }

    fn generate_command(
        &self,
        tool: &str,
        task: &str,
        config: &BenchmarkConfig,
    ) -> Result<String, String> {
        let mut cmd = Command::new("./target/release/oxo-call");
        cmd.arg("run")
            .arg("--tool")
            .arg(tool)
            .arg("--task")
            .arg(task)
            .arg("--dry-run");
        if !config.model.is_empty() {
            cmd.arg("--model").arg(&config.model);
        }
        cmd.arg("--timeout").arg(config.timeout_secs.to_string());

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute oxo-call: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "oxo-call failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_args_from_output(&stdout, tool)
    }

    fn parse_args_from_output(&self, output: &str, tool: &str) -> Result<String, String> {
        for line in output.lines() {
            if line.trim().starts_with("ARGS:") {
                return Ok(line.trim_start_matches("ARGS:").trim().to_string());
            }
            if line.contains(tool)
                && !line.contains("dry-run")
                && let Some(pos) = line.find(' ')
            {
                return Ok(line[pos..].trim().to_string());
            }
        }
        Ok(output
            .lines()
            .find(|l| l.contains(tool))
            .map(|l| l.to_string())
            .unwrap_or_default())
    }
}

pub fn run_quick_benchmark() -> ComparisonReport {
    let config = BenchmarkConfig::default();
    let benchmark = RagBenchmark::new(config);
    let suite = crate::bench::test_cases::create_quick_suite();
    benchmark.run_comparison(&suite)
}

pub fn run_full_benchmark() -> ComparisonReport {
    let config = BenchmarkConfig::default();
    let benchmark = RagBenchmark::new(config);
    let suite = crate::bench::test_cases::create_bioinformatics_suite();
    benchmark.run_comparison(&suite)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_documentation() {
        let benchmark = RagBenchmark::new(BenchmarkConfig::default());
        let doc = benchmark.get_mock_documentation("samtools");
        assert!(doc.contains("sort"));
    }

    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert!(config.enable_rag);
    }
}
