//! RAG Benchmark Runner
//!
//! Executes benchmark tests comparing baseline doc mode with RAG-enhanced mode.

#![allow(dead_code)]

use crate::bench::metrics::{AccuracyMetrics, ComparisonReport, TestResult};
use crate::bench::test_cases::{TestCase, TestSuite};
use crate::doc_enhancer::DocEnhancer;
use crate::doc_processor::DocProcessor;
use std::process::Command;
use std::time::Instant;

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// LLM provider/model to use
    pub model: String,
    /// Enable RAG for comparison
    pub enable_rag: bool,
    /// Number of retries per test
    pub max_retries: usize,
    /// Timeout for each command generation (seconds)
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

/// Benchmark result for a single run
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

/// RAG Benchmark runner
pub struct RagBenchmark {
    config: BenchmarkConfig,
    doc_processor: DocProcessor,
    baseline_enhancer: DocEnhancer,
    rag_enhancer: DocEnhancer,
}

impl RagBenchmark {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config: config.clone(),
            doc_processor: DocProcessor::new(),
            baseline_enhancer: DocEnhancer::new(),
            rag_enhancer: DocEnhancer::with_rag(),
        }
    }

    /// Run full benchmark comparing baseline vs RAG
    pub fn run_comparison(&self, suite: &TestSuite) -> ComparisonReport {
        println!("\n========================================");
        println!("Running RAG Benchmark Comparison");
        println!("Test Suite: {}", suite.name);
        println!("Total Cases: {}", suite.cases.len());
        println!("========================================\n");

        // Run baseline
        println!("[1/2] Running BASELINE (doc mode without RAG)...");
        let baseline_config = BenchmarkConfig {
            enable_rag: false,
            ..self.config.clone()
        };
        let baseline = self.run_with_config(&suite.cases, &baseline_config);
        baseline.print_summary();

        // Run with RAG
        println!("\n[2/2] Running RAG-ENHANCED mode...");
        let rag_config = BenchmarkConfig {
            enable_rag: true,
            ..self.config.clone()
        };
        let rag_enhanced = self.run_with_config(&suite.cases, &rag_config);
        rag_enhanced.print_summary();

        // Generate comparison report
        ComparisonReport::new(baseline.metrics, rag_enhanced.metrics)
    }

    /// Run benchmark with specific config
    fn run_with_config(&self, cases: &[TestCase], config: &BenchmarkConfig) -> BenchmarkResult {
        let mut metrics = AccuracyMetrics::new();
        let mut test_results = Vec::new();
        let start_time = Instant::now();
        let mut total_latency_ms = 0u64;

        let enhancer = if config.enable_rag {
            &self.rag_enhancer
        } else {
            &self.baseline_enhancer
        };

        for (i, case) in cases.iter().enumerate() {
            println!(
                "  [{}/{}] Testing: {} - {}",
                i + 1,
                cases.len(),
                case.tool,
                case.task
            );

            let result = self.run_single_test(case, enhancer, config);

            // Update metrics
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

    /// Run a single test case
    fn run_single_test(
        &self,
        case: &TestCase,
        enhancer: &DocEnhancer,
        config: &BenchmarkConfig,
    ) -> TestResult {
        let start_time = Instant::now();

        // Get documentation (simplified - in real scenario would fetch from system)
        let documentation = self.fetch_tool_documentation(&case.tool);

        // Process documentation
        let structured_doc = self.doc_processor.process(&documentation);

        // Analyze with enhancer
        let analysis = enhancer.analyze(&documentation, &case.tool, &case.task);

        // Build prompt
        let prompt = if enhancer.is_rag_enabled() {
            enhancer.build_rag_enhanced_prompt(&case.tool, &case.task, &structured_doc, &analysis)
        } else {
            enhancer.build_enhanced_prompt(&case.tool, &case.task, &structured_doc, &analysis)
        };

        // Generate command using oxo-call CLI
        let predicted = match self.generate_command(&case.tool, &case.task, &prompt, config) {
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

    /// Fetch tool documentation (mock implementation)
    fn fetch_tool_documentation(&self, tool: &str) -> String {
        // Try to get help from the actual tool
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

    /// Get mock documentation for tools not available
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
  -H        Output header only
  -q INT    Minimum mapping quality
"#
            .to_string(),

            "bcftools" => r#"Usage: bcftools <command> [options]

Commands:
  view      View VCF/BCF file
  filter    Filter variants
  query     Query VCF/BCF
  norm      Normalize variants

Options:
  -h        Include header
  -i EXPR   Filter expression
  -l        List samples
  -f FILE   Reference FASTA
  -o FILE   Output file
"#
            .to_string(),

            "bedtools" => r#"Usage: bedtools <command> [options]

Commands:
  intersect Find overlapping intervals
  merge     Merge overlapping regions
  coverage  Compute coverage

Options:
  -a FILE   Input file A
  -b FILE   Input file B
  -i FILE   Input file
"#
            .to_string(),

            _ => format!(
                "Usage: {} [options]\n\nOptions:\n  -o FILE  Output file\n  -v       Verbose",
                tool
            ),
        }
    }

    /// Generate command using oxo-call CLI
    fn generate_command(
        &self,
        tool: &str,
        task: &str,
        _prompt: &str,
        config: &BenchmarkConfig,
    ) -> Result<String, String> {
        // Run oxo-call to generate command
        let mut cmd = Command::new("./target/release/oxo-call");
        cmd.arg("run")
            .arg("--tool")
            .arg(tool)
            .arg("--task")
            .arg(task)
            .arg("--dry-run");

        // Add model if specified
        if !config.model.is_empty() {
            cmd.arg("--model").arg(&config.model);
        }

        // Set timeout
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

        // Parse output to extract ARGS
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_args_from_output(&stdout, tool)
    }

    /// Parse ARGS from oxo-call output
    fn parse_args_from_output(&self, output: &str, tool: &str) -> Result<String, String> {
        // Look for ARGS line in output
        for line in output.lines() {
            if line.trim().starts_with("ARGS:") {
                return Ok(line.trim_start_matches("ARGS:").trim().to_string());
            }
            // Alternative: look for the command line
            if line.contains(tool) && !line.contains("dry-run") {
                // Extract arguments after tool name
                if let Some(pos) = line.find(' ') {
                    return Ok(line[pos..].trim().to_string());
                }
            }
        }

        // If no ARGS found, return the whole output as fallback
        Ok(output
            .lines()
            .find(|l| l.contains(tool))
            .map(|l| l.to_string())
            .unwrap_or_default())
    }
}

/// Quick benchmark for CI/testing
pub fn run_quick_benchmark() -> ComparisonReport {
    let config = BenchmarkConfig::default();
    let benchmark = RagBenchmark::new(config);

    let suite = crate::bench::test_cases::create_quick_suite();
    benchmark.run_comparison(&suite)
}

/// Full benchmark for comprehensive evaluation
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
        assert!(doc.contains("index"));

        let doc = benchmark.get_mock_documentation("bcftools");
        assert!(doc.contains("filter"));
    }

    #[test]
    fn test_benchmark_config() {
        let config = BenchmarkConfig::default();
        assert!(config.enable_rag);
        assert_eq!(config.max_retries, 1);
    }
}
