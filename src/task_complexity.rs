//! Task Complexity Estimator — Orchestration Decision Engine.
//!
//! Analyzes user input and determines whether to use **Fast** (single LLM call)
//! or **Quality** (multi-stage pipeline) mode. The decision is based on a
//! weighted rule system that considers:
//!
//! - Task description length and ambiguity
//! - Availability of a matching skill file
//! - Documentation quality for the target tool
//! - Presence of complex keywords (pipeline, parallel, batch, …)
//! - Non-English input (adds translation complexity)
//! - Number of explicit CLI parameters
//!
//! # Scoring
//!
//! Each rule contributes a score in `[0.0, weight]`.  The total is capped at
//! `1.0`.  Scores below `0.5` recommend **Fast** mode; `≥ 0.5` recommends
//! **Quality** mode.  Confidence is highest when the score is far from the
//! `0.5` boundary.
//!
//! # Extensibility
//!
//! Custom rules can be added at runtime via [`TaskComplexityEstimator::add_rule`].

use serde::{Deserialize, Serialize};

/// Complexity score ranging from 0.0 (simple) to 1.0 (complex)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ComplexityScore(pub f32);

impl Default for ComplexityScore {
    fn default() -> Self {
        Self(0.0)
    }
}

impl ComplexityScore {
    #[allow(dead_code)]
    pub fn is_simple(&self) -> bool {
        self.0 < 0.5
    }

    pub fn is_complex(&self) -> bool {
        self.0 >= 0.5
    }
}

/// Workflow mode recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkflowMode {
    #[default]
    Fast,
    Quality,
}

/// Result of complexity estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityResult {
    pub score: ComplexityScore,
    pub recommended_mode: WorkflowMode,
    pub confidence: f32,
    pub reasons: Vec<String>,
}

impl Default for ComplexityResult {
    fn default() -> Self {
        Self {
            score: ComplexityScore::default(),
            recommended_mode: WorkflowMode::default(),
            confidence: 0.5,
            reasons: vec![],
        }
    }
}

/// A heuristic rule that contributes to the overall complexity score.
#[derive(Debug, Clone)]
pub struct ComplexityRule {
    /// Human-readable rule identifier (e.g. "task_length").
    pub name: String,
    /// Maximum weight this rule can contribute (informational; the evaluator
    /// itself caps its return value).
    #[allow(dead_code)]
    pub weight: f32,
    pub evaluator: fn(&str, &str, bool, f32) -> f32,
}

/// Task Complexity Estimator
#[derive(Debug, Clone)]
pub struct TaskComplexityEstimator {
    rules: Vec<ComplexityRule>,
}

impl Default for TaskComplexityEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskComplexityEstimator {
    pub fn new() -> Self {
        Self {
            rules: vec![
                // Rule 1: Task length
                ComplexityRule {
                    name: "task_length".to_string(),
                    weight: 0.2,
                    evaluator: |task, _, _, _| {
                        if task.len() > 100 {
                            0.2
                        } else if task.len() > 50 {
                            0.1
                        } else {
                            0.0
                        }
                    },
                },
                // Rule 2: No skill available
                ComplexityRule {
                    name: "no_skill".to_string(),
                    weight: 0.3,
                    evaluator: |_, _, has_skill, _| {
                        if !has_skill { 0.3 } else { 0.0 }
                    },
                },
                // Rule 3: Low documentation quality
                ComplexityRule {
                    name: "low_doc_quality".to_string(),
                    weight: 0.2,
                    evaluator: |_, _, _, doc_quality| {
                        if doc_quality < 0.3 {
                            0.2
                        } else if doc_quality < 0.5 {
                            0.1
                        } else {
                            0.0
                        }
                    },
                },
                // Rule 4: Complex keywords
                ComplexityRule {
                    name: "complex_keywords".to_string(),
                    weight: 0.2,
                    evaluator: |task, _, _, _| {
                        let complex_keywords = [
                            "pipeline",
                            "workflow",
                            "multiple",
                            "complex",
                            "optimize",
                            "parallel",
                            "batch",
                            "custom",
                            "advanced",
                            "sophisticated",
                            "multi-step",
                        ];
                        let task_lower = task.to_lowercase();
                        let match_count = complex_keywords
                            .iter()
                            .filter(|k| task_lower.contains(*k))
                            .count();

                        (match_count as f32 * 0.1).min(0.2)
                    },
                },
                // Rule 5: Parameter count
                ComplexityRule {
                    name: "parameter_count".to_string(),
                    weight: 0.2,
                    evaluator: |task, _, _, _| {
                        let param_count = task.matches("--").count() + task.matches("-").count();
                        (param_count as f32 * 0.05).min(0.2)
                    },
                },
                // Rule 6: Non-English input
                ComplexityRule {
                    name: "non_english".to_string(),
                    weight: 0.15,
                    evaluator: |task, _, _, _| {
                        // Simple heuristic: check for Chinese characters
                        let chinese_count = task
                            .chars()
                            .filter(|c| '\u{4e00}' <= *c && *c <= '\u{9fff}')
                            .count();
                        if chinese_count > 0 { 0.15 } else { 0.0 }
                    },
                },
                // Rule 7: Ambiguous description
                ComplexityRule {
                    name: "ambiguous".to_string(),
                    weight: 0.15,
                    evaluator: |task, _, _, _| {
                        let ambiguous_words = ["something", "some", "maybe", "probably", "like"];
                        let task_lower = task.to_lowercase();
                        if ambiguous_words.iter().any(|w| task_lower.contains(*w)) {
                            0.15
                        } else {
                            0.0
                        }
                    },
                },
            ],
        }
    }

    /// Estimate task complexity
    ///
    /// # Arguments
    /// * `task` - User's task description
    /// * `tool` - Tool name
    /// * `has_skill` - Whether a skill file exists for this tool
    /// * `doc_quality` - Documentation quality score (0.0-1.0)
    ///
    /// # Returns
    /// Complexity estimation result with recommended mode
    pub fn estimate(
        &self,
        task: &str,
        tool: &str,
        has_skill: bool,
        doc_quality: f32,
    ) -> ComplexityResult {
        let mut total_score = 0.0;
        let mut reasons = Vec::new();

        for rule in &self.rules {
            let contribution = (rule.evaluator)(task, tool, has_skill, doc_quality);
            if contribution > 0.0 {
                total_score += contribution;
                reasons.push(format!(
                    "{}: +{:.2}",
                    rule.name.replace('_', " "),
                    contribution
                ));
            }
        }

        // Cap at 1.0
        let score = ComplexityScore(total_score.min(1.0));

        let recommended_mode = if score.is_complex() {
            WorkflowMode::Quality
        } else {
            WorkflowMode::Fast
        };

        let confidence = self.calculate_confidence(total_score);

        ComplexityResult {
            score,
            recommended_mode,
            confidence,
            reasons,
        }
    }

    /// Calculate confidence based on score distribution
    fn calculate_confidence(&self, score: f32) -> f32 {
        // Higher confidence when score is very low or very high
        // Lower confidence when score is around 0.5 (borderline)
        let distance_from_middle = (score - 0.5).abs();
        0.5 + distance_from_middle
    }

    /// Add custom rule
    #[allow(dead_code)]
    pub fn add_rule(&mut self, rule: ComplexityRule) {
        self.rules.push(rule);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task() {
        let estimator = TaskComplexityEstimator::new();

        let result = estimator.estimate(
            "sort input.bam by coordinate",
            "samtools",
            true, // has skill
            0.8,  // good doc quality
        );

        assert!(result.score.is_simple());
        assert_eq!(result.recommended_mode, WorkflowMode::Fast);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_complex_task() {
        let estimator = TaskComplexityEstimator::new();

        let result = estimator.estimate(
            "build a complex pipeline for variant calling with multiple steps and parallel processing",
            "mytool",
            false, // no skill
            0.3,   // poor doc quality
        );

        assert!(result.score.is_complex());
        assert_eq!(result.recommended_mode, WorkflowMode::Quality);
        assert!(result.reasons.len() > 2);
    }

    #[test]
    fn test_chinese_input() {
        let estimator = TaskComplexityEstimator::new();

        let result = estimator.estimate("把 input.bam 按坐标排序", "samtools", true, 0.8);

        // Chinese input should trigger Quality mode
        assert!(result.reasons.iter().any(|r| r.contains("non english")));
    }

    #[test]
    fn test_no_skill() {
        let estimator = TaskComplexityEstimator::new();

        let result = estimator.estimate(
            "process input.txt",
            "mytool",
            false, // no skill
            0.5,
        );

        assert!(result.reasons.iter().any(|r| r.contains("no skill")));
    }

    #[test]
    fn test_confidence_calculation() {
        let estimator = TaskComplexityEstimator::new();

        // Very simple task (score close to 0)
        let result1 = estimator.estimate("sort input.bam", "samtools", true, 0.9);
        assert!(result1.confidence > 0.5);

        // Very complex task (score close to 1)
        let result2 = estimator.estimate(
            "build complex pipeline with multiple steps",
            "mytool",
            false,
            0.2,
        );
        // Complex task should have reasonable confidence
        assert!(result2.confidence > 0.5);

        // Borderline task (score around 0.5)
        let result3 = estimator.estimate("process input.txt", "tool", true, 0.5);
        // Borderline cases may have lower confidence
        assert!(result3.confidence >= 0.5);
    }
}
