//! Confidence-driven workflow decision engine.
//!
//! This module replaces the heuristic Fast/Quality mode with a scientific
//! confidence estimation system based on:
//! - Schema completeness (how much of the tool interface is captured)
//! - Intent clarity (how clearly the task specifies the desired operation)
//! - Model capability (the model's instruction-following ability)
//!
//! ## Design Philosophy
//!
//! The confidence score determines the workflow strategy:
//! - **High Confidence (≥0.7)**: Single LLM call with schema constraints
//! - **Medium Confidence (0.4-0.7)**: Single call + validation + retry
//! - **Low Confidence (<0.4)**: Thinking mode + multi-stage reasoning
//!
//! This approach is deterministic and measurable, unlike the heuristic
//! Fast/Quality mode that had no scientific validation.

use serde::{Deserialize, Serialize};

/// Confidence level for workflow decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConfidenceLevel {
    /// High confidence: schema complete, intent clear, model capable
    /// → Single LLM call, no retry needed
    #[default]
    High,

    /// Medium confidence: partial schema, some ambiguity
    /// → Single call + validation + retry (max 2)
    Medium,

    /// Low confidence: schema incomplete, intent ambiguous, weak model
    /// → Thinking mode + multi-stage reasoning
    Low,
}

/// Result of confidence estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceResult {
    /// Overall confidence level
    pub level: ConfidenceLevel,

    /// Raw confidence score (0.0 to 1.0)
    pub score: f32,

    /// Component scores
    pub components: ConfidenceComponents,

    /// Recommended workflow strategy
    pub strategy: WorkflowStrategy,

    /// Explanation of the estimation
    pub explanation: String,
}

/// Individual components of confidence estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceComponents {
    /// Schema completeness: how much of tool interface is captured (0.0-1.0)
    /// Factors: number of flags extracted, flag description coverage, positional coverage
    pub schema_coverage: f32,

    /// Intent clarity: how clearly task specifies the operation (0.0-1.0)
    /// Factors: specific keywords, file mentions, subcommand match
    pub intent_clarity: f32,

    /// Model capability: model's instruction-following ability (0.0-1.0)
    /// Factors: parameter count, known model family benchmarks
    pub model_capability: f32,

    /// Skill availability: whether a skill file is available (0.0 or 1.0)
    pub skill_availability: f32,
}

impl Default for ConfidenceComponents {
    fn default() -> Self {
        Self {
            schema_coverage: 0.5,
            intent_clarity: 0.5,
            model_capability: 0.5,
            skill_availability: 0.0,
        }
    }
}

/// Workflow strategy recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStrategy {
    /// Single LLM call with schema constraints, no retry
    SingleCall,

    /// Single call + validation, retry up to 2 times if validation fails
    ValidationRetry { max_retries: u32 },

    /// Thinking mode enabled + multi-stage reasoning
    ThinkingMode { stages: Vec<ReasoningStage> },
}

/// Reasoning stage for multi-stage workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReasoningStage {
    /// Parse intent deterministically (keyword matching)
    IntentParsing,
    /// Generate schema-guided prompt
    SchemaPromptGeneration,
    /// Fill parameter values with constrained generation
    ValueFilling,
    /// Validate against schema
    Validation,
}

impl ConfidenceResult {
    /// Compute confidence from components using weighted average
    pub fn compute(components: ConfidenceComponents) -> Self {
        // Weights: schema_coverage is most important for 3B models
        let weights = [0.4, 0.3, 0.2, 0.1]; // schema, intent, model, skill

        let score = components.schema_coverage * weights[0]
            + components.intent_clarity * weights[1]
            + components.model_capability * weights[2]
            + components.skill_availability * weights[3];

        let level = if score >= 0.7 {
            ConfidenceLevel::High
        } else if score >= 0.4 {
            ConfidenceLevel::Medium
        } else {
            ConfidenceLevel::Low
        };

        let strategy = Self::determine_strategy(level, &components);
        let explanation = Self::explain(level, score, &components);

        Self {
            level,
            score,
            components,
            strategy,
            explanation,
        }
    }

    fn determine_strategy(
        level: ConfidenceLevel,
        components: &ConfidenceComponents,
    ) -> WorkflowStrategy {
        match level {
            ConfidenceLevel::High => WorkflowStrategy::SingleCall,
            ConfidenceLevel::Medium => WorkflowStrategy::ValidationRetry { max_retries: 2 },
            ConfidenceLevel::Low => {
                // For low confidence, use thinking mode if model supports it
                let stages = if components.model_capability < 0.6 {
                    vec![
                        ReasoningStage::IntentParsing,
                        ReasoningStage::SchemaPromptGeneration,
                        ReasoningStage::ValueFilling,
                        ReasoningStage::Validation,
                    ]
                } else {
                    vec![
                        ReasoningStage::SchemaPromptGeneration,
                        ReasoningStage::Validation,
                    ]
                };
                WorkflowStrategy::ThinkingMode { stages }
            }
        }
    }

    fn explain(level: ConfidenceLevel, score: f32, components: &ConfidenceComponents) -> String {
        let level_str = match level {
            ConfidenceLevel::High => "High",
            ConfidenceLevel::Medium => "Medium",
            ConfidenceLevel::Low => "Low",
        };

        let reasons: Vec<String> = vec![
            format!(
                "Schema coverage: {:.0}%",
                components.schema_coverage * 100.0
            ),
            format!("Intent clarity: {:.0}%", components.intent_clarity * 100.0),
            format!(
                "Model capability: {:.0}%",
                components.model_capability * 100.0
            ),
        ];

        format!(
            "{} confidence (score {:.2}) based on: {}",
            level_str,
            score,
            reasons.join(", ")
        )
    }
}

/// Estimate confidence from available information
pub fn estimate_confidence(
    schema_flags_count: usize,
    schema_description_coverage: f32,
    task_keyword_match: bool,
    task_file_mentions: usize,
    model_param_count: Option<f32>,
    has_skill: bool,
) -> ConfidenceResult {
    // 1. Schema coverage
    // Score based on number of flags and description coverage
    let flag_score = if schema_flags_count >= 10 {
        1.0
    } else if schema_flags_count >= 5 {
        0.7
    } else if schema_flags_count >= 2 {
        0.5
    } else if schema_flags_count >= 1 {
        0.3
    } else {
        0.1 // No flags extracted = very low schema coverage
    };
    let schema_coverage = (flag_score * 0.6 + schema_description_coverage * 0.4).min(1.0_f32);

    // 2. Intent clarity
    // Score based on task keyword match and file mentions
    let keyword_score: f32 = if task_keyword_match { 0.7 } else { 0.3 };
    let file_score: f32 = if task_file_mentions >= 2 {
        0.8
    } else if task_file_mentions >= 1 {
        0.6
    } else {
        0.4
    };
    let intent_clarity = (keyword_score * 0.5 + file_score * 0.5).min(1.0_f32);

    // 3. Model capability
    // Based on parameter count and known benchmarks
    let model_capability = match model_param_count {
        Some(p) if p >= 7.0 => 0.85, // 7B+ models: high capability
        Some(p) if p >= 4.0 => 0.70, // 4-7B models: good capability
        Some(p) if p >= 3.0 => 0.60, // 3B models: moderate capability
        Some(p) if p >= 1.5 => 0.45, // 1.5B models: limited capability
        Some(_) => 0.30,             // <1.5B: weak capability
        None => 0.65,                // Unknown model: assume moderate
    };

    // 4. Skill availability
    let skill_availability = if has_skill { 1.0 } else { 0.0 };

    let components = ConfidenceComponents {
        schema_coverage,
        intent_clarity,
        model_capability,
        skill_availability,
    };

    ConfidenceResult::compute(components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_confidence() {
        let result = estimate_confidence(10, 0.8, true, 2, Some(7.0), true);
        assert_eq!(result.level, ConfidenceLevel::High);
        assert!(result.score >= 0.7);
    }

    #[test]
    fn test_medium_confidence() {
        let result = estimate_confidence(5, 0.5, true, 1, Some(3.0), false);
        assert_eq!(result.level, ConfidenceLevel::Medium);
        assert!(result.score >= 0.4 && result.score < 0.7);
    }

    #[test]
    fn test_low_confidence_no_schema() {
        let result = estimate_confidence(0, 0.0, false, 0, Some(1.5), false);
        assert_eq!(result.level, ConfidenceLevel::Low);
        assert!(result.score < 0.4);
    }

    #[test]
    fn test_strategy_determination() {
        let high = ConfidenceResult::compute(ConfidenceComponents {
            schema_coverage: 1.0,
            intent_clarity: 1.0,
            model_capability: 1.0,
            skill_availability: 1.0,
        });
        assert!(matches!(high.strategy, WorkflowStrategy::SingleCall));

        let low = ConfidenceResult::compute(ConfidenceComponents {
            schema_coverage: 0.1,
            intent_clarity: 0.1,
            model_capability: 0.3,
            skill_availability: 0.0,
        });
        assert!(matches!(
            low.strategy,
            WorkflowStrategy::ThinkingMode { .. }
        ));
    }

    #[test]
    fn test_model_capability_scoring() {
        // 7B model
        let result = estimate_confidence(5, 0.5, true, 1, Some(7.0), false);
        assert!(result.components.model_capability >= 0.85);

        // 3B model
        let result = estimate_confidence(5, 0.5, true, 1, Some(3.0), false);
        assert!(
            result.components.model_capability >= 0.55
                && result.components.model_capability <= 0.65
        );

        // 1B model
        let result = estimate_confidence(5, 0.5, true, 1, Some(1.0), false);
        assert!(result.components.model_capability < 0.35);
    }

    #[test]
    fn test_schema_coverage_impact() {
        // High schema coverage compensates for weak model
        let result = estimate_confidence(10, 0.8, true, 2, Some(1.5), true);
        // Even with weak model, high schema + skill should boost confidence
        assert!(result.score > 0.5);
    }
}
