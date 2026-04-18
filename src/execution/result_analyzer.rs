//! Result Analyzer — post-execution analysis and learning.
//!
//! Analyzes command execution results to extract actionable insights
//! that feed back into the knowledge layer for continuous improvement.

use crate::knowledge::error_db::ErrorCategory;
use serde::{Deserialize, Serialize};

/// Analysis of a completed command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAnalysis {
    /// Whether the execution was successful.
    pub success: bool,
    /// Error category (if failed).
    pub error_category: Option<ErrorCategory>,
    /// Key patterns detected in output.
    pub output_patterns: Vec<OutputPattern>,
    /// Suggested improvements for future runs.
    pub improvements: Vec<String>,
    /// Resource usage hints inferred from output.
    pub resource_hints: ResourceHints,
}

/// Detected pattern in command output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPattern {
    /// Pattern category.
    pub category: PatternCategory,
    /// Description.
    pub description: String,
}

/// Categories of output patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Performance-related (e.g. "processed 1M reads in 30s").
    Performance,
    /// Quality metrics (e.g. "mapping rate: 95%").
    QualityMetric,
    /// Warning that may indicate an issue.
    Warning,
    /// File creation/modification.
    FileOutput,
}

/// Resource usage hints inferred from output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceHints {
    /// Suggested thread count based on observed performance.
    pub suggested_threads: Option<usize>,
    /// Estimated memory usage.
    pub estimated_memory_gb: Option<f32>,
    /// Whether the task was I/O bound.
    pub io_bound: bool,
}

/// The Result Analyzer.
pub struct ResultAnalyzer;

impl Default for ResultAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ResultAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze the results of a command execution.
    pub fn analyze(
        &self,
        _tool: &str,
        exit_code: i32,
        stdout: &str,
        stderr: &str,
    ) -> ExecutionAnalysis {
        let success = exit_code == 0;
        let error_category = if !success {
            Some(ErrorCategory::classify(stderr))
        } else {
            None
        };

        let output_patterns = self.detect_patterns(stdout, stderr);
        let improvements = self.suggest_improvements(&output_patterns, stderr);
        let resource_hints = self.infer_resource_hints(stdout, stderr);

        ExecutionAnalysis {
            success,
            error_category,
            output_patterns,
            improvements,
            resource_hints,
        }
    }

    /// Detect patterns in command output.
    fn detect_patterns(&self, stdout: &str, stderr: &str) -> Vec<OutputPattern> {
        let mut patterns = Vec::new();
        let combined = format!("{stdout}\n{stderr}").to_lowercase();

        // Performance patterns.
        if combined.contains("processed")
            || combined.contains("elapsed")
            || combined.contains("wall clock")
            || combined.contains("real\t")
        {
            patterns.push(OutputPattern {
                category: PatternCategory::Performance,
                description: "Performance metrics detected in output".to_string(),
            });
        }

        // Threading / parallelism hints.
        if combined.contains("thread") || combined.contains("worker") || combined.contains("core") {
            patterns.push(OutputPattern {
                category: PatternCategory::Performance,
                description: "Threading / parallelism information detected".to_string(),
            });
        }

        // Memory usage hints.
        if combined.contains("memory")
            || combined.contains("peak rss")
            || combined.contains("max resident")
            || combined.contains("heap")
        {
            patterns.push(OutputPattern {
                category: PatternCategory::Performance,
                description: "Memory usage information detected".to_string(),
            });
        }

        // Quality metrics.
        let quality_indicators = [
            "mapping rate",
            "mapped",
            "properly paired",
            "duplicate",
            "quality",
            "coverage",
            "gc content",
            "error rate",
            "phred",
            "base quality",
        ];
        for ind in quality_indicators {
            if combined.contains(ind) {
                patterns.push(OutputPattern {
                    category: PatternCategory::QualityMetric,
                    description: format!("Quality metric detected: {ind}"),
                });
                break;
            }
        }

        // Warnings.
        if combined.contains("warning") || combined.contains("[w::") {
            patterns.push(OutputPattern {
                category: PatternCategory::Warning,
                description: "Warnings detected in output".to_string(),
            });
        }

        // File creation.
        if combined.contains("wrote ") || combined.contains("output:") || combined.contains("saved")
        {
            patterns.push(OutputPattern {
                category: PatternCategory::FileOutput,
                description: "File creation/output detected".to_string(),
            });
        }

        patterns
    }

    /// Suggest improvements based on detected patterns and output.
    fn suggest_improvements(&self, patterns: &[OutputPattern], stderr: &str) -> Vec<String> {
        let mut improvements = Vec::new();

        if patterns
            .iter()
            .any(|p| p.category == PatternCategory::Warning)
        {
            improvements.push(
                "Review warnings in stderr — they may indicate data quality issues".to_string(),
            );
        }

        let lower = stderr.to_lowercase();
        if lower.contains("sort order") || lower.contains("not sorted") {
            improvements.push("Input file may need to be sorted first (samtools sort)".to_string());
        }

        if lower.contains("no index") || lower.contains("index file") {
            improvements.push(
                "Consider creating an index (samtools index) for faster random access".to_string(),
            );
        }

        if lower.contains("out of memory") || lower.contains("cannot allocate") {
            improvements.push(
                "Reduce thread count or use a smaller chunk size to lower memory usage".to_string(),
            );
        }

        if lower.contains("truncated") || lower.contains("unexpected eof") {
            improvements.push(
                "Input file may be corrupted or incomplete — verify integrity first".to_string(),
            );
        }

        improvements
    }

    /// Infer resource usage hints from output.
    fn infer_resource_hints(&self, stdout: &str, stderr: &str) -> ResourceHints {
        let combined = format!("{stdout}\n{stderr}").to_lowercase();
        let io_bound = combined.contains("i/o") || combined.contains("disk");

        // Try to detect thread count from output (e.g., "using 4 threads").
        let suggested_threads = regex::Regex::new(r"(?:using|threads?:?)\s*(\d+)")
            .ok()
            .and_then(|re| re.captures(&combined))
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<usize>().ok());

        // Try to detect memory from output (e.g., "peak RSS: 2.5 GB").
        let estimated_memory_gb = regex::Regex::new(r"(\d+\.?\d*)\s*(?:gb|gib)")
            .ok()
            .and_then(|re| re.captures(&combined))
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f32>().ok());

        ResourceHints {
            suggested_threads,
            estimated_memory_gb,
            io_bound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_success() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 0, "1000 reads processed", "");
        assert!(result.success);
        assert!(result.error_category.is_none());
    }

    #[test]
    fn test_analyze_failure() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 1, "", "samtools: No such file or directory");
        assert!(!result.success);
        assert_eq!(result.error_category, Some(ErrorCategory::MissingInput));
    }

    #[test]
    fn test_detect_quality_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze(
            "samtools",
            0,
            "1000 + 0 mapped (95.00%)\n50 + 0 properly paired",
            "",
        );
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.category == PatternCategory::QualityMetric)
        );
    }

    #[test]
    fn test_suggest_sort_improvement() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 1, "", "file is not sorted");
        assert!(result.improvements.iter().any(|s| s.contains("sorted")));
    }

    // ── Expanded result analyzer tests ───────────────────────────────────────

    #[test]
    fn test_detect_performance_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 0, "elapsed: 12.3s", "");
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.category == PatternCategory::Performance)
        );
    }

    #[test]
    fn test_detect_threading_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("bwa", 0, "using 8 threads", "");
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.description.contains("Threading"))
        );
    }

    #[test]
    fn test_detect_memory_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("star", 0, "", "peak RSS: 32 GB memory used");
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.description.contains("Memory"))
        );
    }

    #[test]
    fn test_detect_warning_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("bcftools", 0, "", "[W::vcf_parse] warning: contig");
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.category == PatternCategory::Warning)
        );
    }

    #[test]
    fn test_detect_file_output_patterns() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 0, "wrote 50000 alignments", "");
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.category == PatternCategory::FileOutput)
        );
    }

    #[test]
    fn test_suggest_oom_improvement() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("star", 1, "", "fatal: out of memory");
        assert!(result.improvements.iter().any(|s| s.contains("memory")));
    }

    #[test]
    fn test_suggest_truncated_improvement() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 1, "", "truncated file");
        assert!(result.improvements.iter().any(|s| s.contains("corrupted")));
    }

    #[test]
    fn test_infer_thread_hint() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("bwa", 0, "using 8 threads for alignment", "");
        assert_eq!(result.resource_hints.suggested_threads, Some(8));
    }

    #[test]
    fn test_infer_memory_hint() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("star", 0, "", "peak RSS: 2.5 GB");
        assert!((result.resource_hints.estimated_memory_gb.unwrap() - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_infer_io_bound() {
        let analyzer = ResultAnalyzer::new();
        let result = analyzer.analyze("samtools", 0, "", "bottleneck: disk I/O");
        assert!(result.resource_hints.io_bound);
    }

    #[test]
    fn test_no_false_positive_on_error_rate() {
        let analyzer = ResultAnalyzer::new();
        // "error rate" is a quality metric, not a real error.
        let result = analyzer.analyze("samtools", 0, "error rate: 0.01%", "");
        assert!(result.success);
        assert!(
            result
                .output_patterns
                .iter()
                .any(|p| p.category == PatternCategory::QualityMetric)
        );
    }
}
