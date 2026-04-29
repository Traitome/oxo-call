//! Reflection Engine Module
//!
//! Analyzes validation failures and generates insights for fixing them.
//! Provides LLM-based reflection and rule-based reflection.

#![allow(dead_code)]

use crate::command_validator::CommandValidation;
use crate::constraint_graph::Violation;

/// Reflection result containing analysis and guidance
#[derive(Debug, Clone)]
pub struct Reflection {
    /// Analysis of what went wrong
    pub analysis: String,
    /// Specific guidance on how to fix
    pub guidance: String,
    /// Confidence in the reflection (0.0 - 1.0)
    pub confidence: f32,
    /// Suggested approach for next iteration
    pub approach: Approach,
}

/// Approach for fixing the issue
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Approach {
    /// Use specific flags from whitelist
    UseWhitelistFlags,
    /// Add missing subcommand
    AddSubcommand,
    /// Remove hallucinated flags
    RemoveHallucinations,
    /// Fix flag format
    FixFormat,
    /// Reconsider parameter combination
    ReconsiderCombination,
    /// Use example as template
    UseExample,
}

/// Reflection Engine
#[derive(Debug, Clone)]
pub struct ReflectionEngine {
    /// Enable LLM-based reflection
    use_llm: bool,
    /// Maximum reflection depth
    max_depth: usize,
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionEngine {
    /// Create a new reflection engine
    pub fn new() -> Self {
        Self {
            use_llm: false, // Rule-based by default
            max_depth: 3,
        }
    }

    /// Enable LLM-based reflection
    pub fn with_llm(mut self) -> Self {
        self.use_llm = true;
        self
    }

    /// Reflect on a validation failure
    pub fn reflect(
        &self,
        generated_command: &str,
        validation: &CommandValidation,
        valid_flags: &[String],
        examples: &[String],
    ) -> Reflection {
        if self.use_llm {
            // LLM-based reflection would go here
            // For now, fall back to rule-based
            self.rule_based_reflect(generated_command, validation, valid_flags, examples)
        } else {
            self.rule_based_reflect(generated_command, validation, valid_flags, examples)
        }
    }

    /// Rule-based reflection
    fn rule_based_reflect(
        &self,
        generated_command: &str,
        validation: &CommandValidation,
        valid_flags: &[String],
        examples: &[String],
    ) -> Reflection {
        let mut issues = Vec::new();
        let mut guidance_parts = Vec::new();
        let mut approach = Approach::UseWhitelistFlags;

        // Analyze violations
        for violation in &validation.violations {
            match violation {
                Violation::HallucinatedFlag { flag, suggestion } => {
                    issues.push(format!("Flag '{}' is not in the valid flags list", flag));

                    if let Some(suggested) = suggestion {
                        guidance_parts.push(format!(
                            "Replace '{}' with '{}' (similar valid flag)",
                            flag, suggested
                        ));
                    } else {
                        guidance_parts.push(format!(
                            "Remove '{}' - it's not a valid flag for this tool/subcommand",
                            flag
                        ));
                    }

                    approach = Approach::RemoveHallucinations;
                }
                Violation::MissingRequired { flag, context } => {
                    let ctx_desc = context
                        .as_ref()
                        .map(|c| format!(" (required when {})", c))
                        .unwrap_or_default();

                    issues.push(format!("Missing required flag '{}' {}", flag, ctx_desc));
                    guidance_parts.push(format!("Add the required flag '{}'", flag));
                    approach = Approach::UseWhitelistFlags;
                }
                Violation::MutuallyExclusive { flag1, flag2 } => {
                    issues.push(format!(
                        "Cannot use both '{}' and '{}' together",
                        flag1, flag2
                    ));
                    guidance_parts.push(format!(
                        "Choose either '{}' or '{}', not both",
                        flag1, flag2
                    ));
                    approach = Approach::ReconsiderCombination;
                }
                Violation::MissingDependency { flag, requires } => {
                    issues.push(format!(
                        "Flag '{}' requires '{}' to be present",
                        flag, requires
                    ));
                    guidance_parts.push(format!("Add '{}' when using '{}'", requires, flag));
                    approach = Approach::UseWhitelistFlags;
                }
                Violation::InvalidFormat {
                    flag,
                    expected,
                    got,
                } => {
                    let got_desc = got
                        .as_ref()
                        .map(|g| format!(" (got '{}')", g))
                        .unwrap_or_default();

                    issues.push(format!("Flag '{}' has wrong format{}", flag, got_desc));
                    guidance_parts.push(format!("Fix '{}' to have format: {}", flag, expected));
                    approach = Approach::FixFormat;
                }
            }
        }

        // Check for missing subcommand
        if validation.detected_subcommand.is_none() && !valid_flags.is_empty() {
            // Check if first part looks like a flag
            let parts: Vec<&str> = generated_command.split_whitespace().collect();
            if !parts.is_empty() && parts[0].starts_with('-') {
                issues.push("Missing subcommand - command starts with a flag".to_string());
                guidance_parts.push(
                    "Start with the subcommand name (e.g., 'sort', 'view', 'intersect') before flags"
                        .to_string(),
                );
                approach = Approach::AddSubcommand;
            }
        }

        // Build analysis
        let analysis = if issues.is_empty() {
            "No specific issues identified".to_string()
        } else {
            format!("Found {} issue(s): {}", issues.len(), issues.join("; "))
        };

        // Build guidance
        let guidance = if guidance_parts.is_empty() {
            "Review the valid flags list and try again".to_string()
        } else {
            guidance_parts.join(". ")
        };

        // Calculate confidence based on issue clarity
        let confidence = if issues.is_empty() {
            0.5
        } else {
            (0.3 + 0.7 * (1.0 - (issues.len() as f32 / 5.0).min(1.0))).min(0.95)
        };

        // If we have examples, suggest using them
        if !examples.is_empty() && approach == Approach::UseWhitelistFlags {
            approach = Approach::UseExample;
        }

        Reflection {
            analysis,
            guidance,
            confidence,
            approach,
        }
    }

    /// Generate a reflection prompt for LLM
    pub fn build_reflection_prompt(
        &self,
        generated: &str,
        validation: &CommandValidation,
        valid_flags: &[String],
        examples: &[String],
    ) -> String {
        let mut prompt = format!(
            "You are a CLI command analysis expert. Review this generated command:\n\n\
             Generated: {}\n\n",
            generated
        );

        if !validation.violations.is_empty() {
            prompt.push_str("Validation Errors:\n");
            for (i, v) in validation.violations.iter().enumerate() {
                prompt.push_str(&format!("{}. {:?}\n", i + 1, v));
            }
            prompt.push('\n');
        }

        prompt.push_str("Valid flags for this tool:\n");
        for flag in valid_flags.iter().take(20) {
            prompt.push_str(&format!("- {}\n", flag));
        }

        if !examples.is_empty() {
            prompt.push_str("\nCorrect examples:\n");
            for (i, ex) in examples.iter().take(3).enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, ex));
            }
        }

        prompt.push_str(
            "\nAnalyze why errors occurred and provide specific fix guidance. \
             Be concise and actionable.",
        );

        prompt
    }

    /// Build an improved generation prompt based on reflection
    pub fn build_improved_prompt(
        &self,
        original_task: &str,
        reflection: &Reflection,
        previous_attempts: &[String],
        valid_flags: &[String],
        examples: &[String],
    ) -> String {
        let mut prompt = format!(
            "Generate a command for: {}\n\n\
             Previous attempts failed. Analysis: {}\n\
             Guidance: {}\n\n",
            original_task, reflection.analysis, reflection.guidance
        );

        if !previous_attempts.is_empty() {
            prompt.push_str("Previous attempts (do not repeat these):\n");
            for (i, attempt) in previous_attempts.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, attempt));
            }
            prompt.push('\n');
        }

        prompt.push_str("Valid flags to use:\n");
        for flag in valid_flags.iter().take(20) {
            prompt.push_str(&format!("- {}\n", flag));
        }

        if !examples.is_empty() {
            prompt.push_str("\nFollow these correct examples:\n");
            for ex in examples.iter().take(3) {
                prompt.push_str(&format!("- {}\n", ex));
            }
        }

        match reflection.approach {
            Approach::AddSubcommand => {
                prompt.push_str("\nCRITICAL: Start with the subcommand name!\n");
            }
            Approach::RemoveHallucinations => {
                prompt.push_str("\nCRITICAL: Use ONLY flags from the valid list above!\n");
            }
            Approach::UseExample => {
                prompt.push_str("\nCRITICAL: Follow the pattern in the examples closely!\n");
            }
            _ => {}
        }

        prompt.push_str("\nGenerate only the command arguments (no tool name):");

        prompt
    }

    /// Check if we should retry based on reflection
    pub fn should_retry(&self, reflection: &Reflection, iteration: usize) -> bool {
        if iteration >= self.max_depth {
            return false;
        }

        // Only retry if confidence is reasonable
        reflection.confidence > 0.4
    }
}

/// Reflection history for iterative improvement
#[derive(Debug, Clone, Default)]
pub struct ReflectionHistory {
    /// All attempts made
    pub attempts: Vec<AttemptRecord>,
    /// Current iteration
    pub current_iteration: usize,
}

/// Record of one attempt
#[derive(Debug, Clone)]
pub struct AttemptRecord {
    /// Generated command
    pub command: String,
    /// Validation result
    pub validation: CommandValidation,
    /// Reflection on the attempt
    pub reflection: Reflection,
}

impl ReflectionHistory {
    /// Create new history
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an attempt
    pub fn add_attempt(
        &mut self,
        command: String,
        validation: CommandValidation,
        reflection: Reflection,
    ) {
        self.attempts.push(AttemptRecord {
            command,
            validation,
            reflection,
        });
        self.current_iteration += 1;
    }

    /// Get all previous command attempts
    pub fn previous_commands(&self) -> Vec<String> {
        self.attempts.iter().map(|a| a.command.clone()).collect()
    }

    /// Get the best attempt so far (least violations)
    pub fn best_attempt(&self) -> Option<&AttemptRecord> {
        self.attempts
            .iter()
            .min_by_key(|a| a.validation.violations.len())
    }

    /// Check if we're making progress
    pub fn is_making_progress(&self) -> bool {
        if self.attempts.len() < 2 {
            return true;
        }

        let last = &self.attempts[self.attempts.len() - 1];
        let prev = &self.attempts[self.attempts.len() - 2];

        last.validation.violations.len() <= prev.validation.violations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_on_hallucinated_flag() {
        let engine = ReflectionEngine::new();

        let validation = CommandValidation {
            is_valid: false,
            detected_subcommand: Some("sort".to_string()),
            violations: vec![Violation::HallucinatedFlag {
                flag: "--fast".to_string(),
                suggestion: Some("-@".to_string()),
            }],
            suggestions: vec![],
        };

        let reflection = engine.reflect(
            "sort --fast in.bam",
            &validation,
            &["-@".to_string(), "-o".to_string()],
            &[],
        );

        assert!(reflection.analysis.contains("--fast"));
        assert!(reflection.guidance.contains("-@"));
        assert_eq!(reflection.approach, Approach::RemoveHallucinations);
    }

    #[test]
    fn test_reflection_on_missing_subcommand() {
        let engine = ReflectionEngine::new();

        let validation = CommandValidation {
            is_valid: false,
            detected_subcommand: None,
            violations: vec![],
            suggestions: vec![],
        };

        let reflection = engine.reflect("-o out.bam in.bam", &validation, &["-o".to_string()], &[]);

        assert!(reflection.analysis.contains("Missing subcommand"));
        assert_eq!(reflection.approach, Approach::AddSubcommand);
    }
}
