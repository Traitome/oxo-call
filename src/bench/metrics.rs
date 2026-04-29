//! Metrics for RAG Benchmark
//!
//! Calculates accuracy metrics and generates comparison reports.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accuracy metrics for a single test run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyMetrics {
    /// Total number of test cases
    pub total: usize,
    /// Number of correct predictions
    pub correct: usize,
    /// Number of incorrect predictions
    pub incorrect: usize,
    /// Number of skipped/failed tests
    pub skipped: usize,
    /// Detailed results by pattern type
    pub by_pattern: HashMap<String, PatternMetrics>,
    /// Detailed results by tool
    pub by_tool: HashMap<String, ToolMetrics>,
    /// Detailed results by difficulty
    pub by_difficulty: HashMap<String, DifficultyMetrics>,
}

impl AccuracyMetrics {
    pub fn new() -> Self {
        Self {
            total: 0,
            correct: 0,
            incorrect: 0,
            skipped: 0,
            by_pattern: HashMap::new(),
            by_tool: HashMap::new(),
            by_difficulty: HashMap::new(),
        }
    }

    /// Calculate overall accuracy (0.0 - 1.0)
    pub fn accuracy(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.correct as f32 / self.total as f32
        }
    }

    /// Calculate accuracy excluding skipped
    pub fn accuracy_excluding_skipped(&self) -> f32 {
        let valid = self.total - self.skipped;
        if valid == 0 {
            0.0
        } else {
            self.correct as f32 / valid as f32
        }
    }

    /// Add a test result
    pub fn add_result(
        &mut self,
        tool: &str,
        pattern: &str,
        difficulty: &str,
        is_correct: bool,
        skipped: bool,
    ) {
        self.total += 1;

        if skipped {
            self.skipped += 1;
        } else if is_correct {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }

        // Update pattern metrics
        let pattern_metrics = self.by_pattern.entry(pattern.to_string()).or_default();
        pattern_metrics.add_result(is_correct, skipped);

        // Update tool metrics
        let tool_metrics = self.by_tool.entry(tool.to_string()).or_default();
        tool_metrics.add_result(is_correct, skipped);

        // Update difficulty metrics
        let diff_metrics = self
            .by_difficulty
            .entry(difficulty.to_string())
            .or_default();
        diff_metrics.add_result(is_correct, skipped);
    }

    /// Print summary report
    pub fn print_summary(&self) {
        println!("\n=== Accuracy Metrics ===");
        println!("Total Cases: {}", self.total);
        println!(
            "Correct:     {} ({:.1}%)",
            self.correct,
            self.accuracy() * 100.0
        );
        println!("Incorrect:   {}", self.incorrect);
        println!("Skipped:     {}", self.skipped);
        println!(
            "Accuracy (excl. skipped): {:.1}%",
            self.accuracy_excluding_skipped() * 100.0
        );

        println!("\n--- By Pattern Type ---");
        for (pattern, metrics) in &self.by_pattern {
            println!(
                "  {}: {:.1}% ({} correct / {} total)",
                pattern,
                metrics.accuracy() * 100.0,
                metrics.correct,
                metrics.total
            );
        }

        println!("\n--- By Tool ---");
        for (tool, metrics) in &self.by_tool {
            println!(
                "  {}: {:.1}% ({} correct / {} total)",
                tool,
                metrics.accuracy() * 100.0,
                metrics.correct,
                metrics.total
            );
        }

        println!("\n--- By Difficulty ---");
        for (difficulty, metrics) in &self.by_difficulty {
            println!(
                "  {}: {:.1}% ({} correct / {} total)",
                difficulty,
                metrics.accuracy() * 100.0,
                metrics.correct,
                metrics.total
            );
        }
    }
}

impl Default for AccuracyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics for a specific pattern type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetrics {
    pub total: usize,
    pub correct: usize,
    pub incorrect: usize,
    pub skipped: usize,
}

impl PatternMetrics {
    pub fn new() -> Self {
        Self {
            total: 0,
            correct: 0,
            incorrect: 0,
            skipped: 0,
        }
    }

    pub fn accuracy(&self) -> f32 {
        if self.total == self.skipped {
            0.0
        } else {
            self.correct as f32 / (self.total - self.skipped) as f32
        }
    }

    pub fn add_result(&mut self, is_correct: bool, skipped: bool) {
        self.total += 1;
        if skipped {
            self.skipped += 1;
        } else if is_correct {
            self.correct += 1;
        } else {
            self.incorrect += 1;
        }
    }
}

impl Default for PatternMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics for a specific tool
pub type ToolMetrics = PatternMetrics;

/// Metrics for a specific difficulty level
pub type DifficultyMetrics = PatternMetrics;

/// Comparison between baseline and RAG-enhanced results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub baseline: AccuracyMetrics,
    pub rag_enhanced: AccuracyMetrics,
    pub improvement: ImprovementMetrics,
}

impl ComparisonReport {
    pub fn new(baseline: AccuracyMetrics, rag_enhanced: AccuracyMetrics) -> Self {
        let improvement = ImprovementMetrics::calculate(&baseline, &rag_enhanced);
        Self {
            baseline,
            rag_enhanced,
            improvement,
        }
    }

    /// Print detailed comparison report
    pub fn print_report(&self) {
        println!("\n========================================");
        println!("   RAG Benchmark Comparison Report");
        println!("========================================\n");

        println!("Overall Accuracy:");
        println!("  Baseline:     {:.1}%", self.baseline.accuracy() * 100.0);
        println!(
            "  RAG Enhanced: {:.1}%",
            self.rag_enhanced.accuracy() * 100.0
        );
        println!(
            "  Improvement:  {:+.1}% ({}x better)",
            self.improvement.accuracy_delta * 100.0,
            self.improvement.relative_improvement
        );

        println!("\n--- Pattern Type Breakdown ---");
        for pattern in self.baseline.by_pattern.keys() {
            let base = self.baseline.by_pattern.get(pattern).unwrap();
            let rag = self.rag_enhanced.by_pattern.get(pattern).unwrap();

            let base_acc = base.accuracy() * 100.0;
            let rag_acc = rag.accuracy() * 100.0;
            let delta = rag_acc - base_acc;

            println!("  {}:", pattern);
            println!(
                "    Baseline: {:.1}% | RAG: {:.1}% | Δ: {:+.1}%",
                base_acc, rag_acc, delta
            );
        }

        println!("\n--- Difficulty Breakdown ---");
        for difficulty in ["easy", "medium", "hard"] {
            if let Some(base) = self.baseline.by_difficulty.get(difficulty) {
                let rag = self.rag_enhanced.by_difficulty.get(difficulty).unwrap();

                let base_acc = base.accuracy() * 100.0;
                let rag_acc = rag.accuracy() * 100.0;
                let delta = rag_acc - base_acc;

                println!("  {}:", difficulty);
                println!(
                    "    Baseline: {:.1}% | RAG: {:.1}% | Δ: {:+.1}%",
                    base_acc, rag_acc, delta
                );
            }
        }

        println!("\n--- Per-Tool Comparison ---");
        let mut tools: Vec<_> = self.baseline.by_tool.keys().collect();
        tools.sort();

        for tool in tools {
            let base = self.baseline.by_tool.get(tool).unwrap();
            let rag = self.rag_enhanced.by_tool.get(tool).unwrap();

            let base_acc = base.accuracy() * 100.0;
            let rag_acc = rag.accuracy() * 100.0;

            println!(
                "  {:12} {:5.1}% → {:5.1}% ({:+.1}%)",
                tool,
                base_acc,
                rag_acc,
                rag_acc - base_acc
            );
        }

        println!("\n========================================");
    }
}

/// Improvement metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementMetrics {
    /// Absolute accuracy improvement (0.0 - 1.0)
    pub accuracy_delta: f32,
    /// Relative improvement (1.0 = no change, 2.0 = 2x better)
    pub relative_improvement: f32,
    /// Number of additional correct predictions
    pub additional_correct: i32,
}

impl ImprovementMetrics {
    pub fn calculate(baseline: &AccuracyMetrics, rag: &AccuracyMetrics) -> Self {
        let accuracy_delta = rag.accuracy() - baseline.accuracy();

        let relative_improvement = if baseline.accuracy() == 0.0 {
            1.0
        } else {
            rag.accuracy() / baseline.accuracy()
        };

        let additional_correct = rag.correct as i32 - baseline.correct as i32;

        Self {
            accuracy_delta,
            relative_improvement,
            additional_correct,
        }
    }
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub tool: String,
    pub task: String,
    pub expected: String,
    pub predicted: String,
    pub is_correct: bool,
    pub similarity_score: f32,
    pub latency_ms: u64,
    pub error_message: Option<String>,
}

impl TestResult {
    pub fn new(
        tool: impl Into<String>,
        task: impl Into<String>,
        expected: impl Into<String>,
        predicted: impl Into<String>,
    ) -> Self {
        let expected = expected.into();
        let predicted = predicted.into();
        let is_correct = Self::is_exact_match(&expected, &predicted);
        let similarity_score = Self::calculate_similarity(&expected, &predicted);

        Self {
            tool: tool.into(),
            task: task.into(),
            expected,
            predicted,
            is_correct,
            similarity_score,
            latency_ms: 0,
            error_message: None,
        }
    }

    /// Check if expected and predicted are exact matches
    fn is_exact_match(expected: &str, predicted: &str) -> bool {
        let normalize = |s: &str| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        };
        normalize(expected) == normalize(predicted)
    }

    /// Calculate similarity score (0.0 - 1.0)
    fn calculate_similarity(expected: &str, predicted: &str) -> f32 {
        // Simple token-based Jaccard similarity
        let expected_tokens: std::collections::HashSet<_> = expected
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        let predicted_tokens: std::collections::HashSet<_> = predicted
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        if expected_tokens.is_empty() && predicted_tokens.is_empty() {
            return 1.0;
        }

        let intersection: std::collections::HashSet<_> = expected_tokens
            .intersection(&predicted_tokens)
            .cloned()
            .collect();
        let union: std::collections::HashSet<_> =
            expected_tokens.union(&predicted_tokens).cloned().collect();

        intersection.len() as f32 / union.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy_metrics() {
        let mut metrics = AccuracyMetrics::new();

        metrics.add_result("samtools", "subcommand", "easy", true, false);
        metrics.add_result("samtools", "subcommand", "easy", true, false);
        metrics.add_result("bedtools", "subcommand", "medium", false, false);

        assert_eq!(metrics.total, 3);
        assert_eq!(metrics.correct, 2);
        assert!((metrics.accuracy() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_comparison_report() {
        let mut baseline = AccuracyMetrics::new();
        baseline.add_result("samtools", "subcommand", "easy", true, false);
        baseline.add_result("samtools", "subcommand", "easy", false, false);

        let mut rag = AccuracyMetrics::new();
        rag.add_result("samtools", "subcommand", "easy", true, false);
        rag.add_result("samtools", "subcommand", "easy", true, false);

        let report = ComparisonReport::new(baseline, rag);

        assert!((report.improvement.accuracy_delta - 0.5).abs() < 0.01);
        assert_eq!(report.improvement.additional_correct, 1);
    }

    #[test]
    fn test_test_result_similarity() {
        let result = TestResult::new(
            "samtools",
            "sort BAM",
            "sort -o out.bam in.bam",
            "sort -o out.bam in.bam",
        );

        assert!(result.is_correct);
        assert_eq!(result.similarity_score, 1.0);

        let result2 = TestResult::new(
            "samtools",
            "sort BAM",
            "sort -o out.bam in.bam",
            "sort in.bam -o out.bam",
        );

        assert!(!result2.is_correct); // Order matters for exact match
        assert!(result2.similarity_score > 0.5);
    }
}
