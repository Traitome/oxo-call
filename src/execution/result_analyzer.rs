//! Result Analyzer — post-execution analysis and learning.
//!
//! Analyzes command execution results to extract actionable insights
//! that feed back into the knowledge layer for continuous improvement.

use crate::knowledge::error_db::ErrorCategory;
use serde::{Deserialize, Serialize};

/// Analysis of a completed command execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct ResourceHints {
    /// Suggested thread count based on observed performance.
    pub suggested_threads: Option<usize>,
    /// Estimated memory usage.
    pub estimated_memory_gb: Option<f32>,
    /// Whether the task was I/O bound.
    pub io_bound: bool,
}

/// The Result Analyzer.
#[allow(dead_code)]
pub struct ResultAnalyzer;

impl Default for ResultAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
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
        if combined.contains("processed") || combined.contains("elapsed") {
            patterns.push(OutputPattern {
                category: PatternCategory::Performance,
                description: "Performance metrics detected in output".to_string(),
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

        improvements
    }

    /// Infer resource usage hints from output.
    fn infer_resource_hints(&self, _stdout: &str, stderr: &str) -> ResourceHints {
        let lower = stderr.to_lowercase();
        let io_bound = lower.contains("i/o") || lower.contains("disk");

        ResourceHints {
            suggested_threads: None,
            estimated_memory_gb: None,
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
}
