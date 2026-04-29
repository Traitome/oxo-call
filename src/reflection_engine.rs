//! Reflection Engine Module (HDA)
//!
//! Analyzes validation failures and generates insights for fixing them.
//! Uses schema-based ValidationError instead of constraint_graph::Violation.

#![allow(dead_code)]

use crate::command_validator::CommandValidation;
use crate::schema::ValidationError;

#[derive(Debug, Clone)]
pub struct Reflection {
    pub analysis: String,
    pub guidance: String,
    pub confidence: f32,
    pub approach: Approach,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Approach {
    UseWhitelistFlags,
    AddSubcommand,
    RemoveHallucinations,
    FixFormat,
    ReconsiderCombination,
    UseExample,
}

#[derive(Debug, Clone)]
pub struct ReflectionEngine {
    use_llm: bool,
    max_depth: usize,
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionEngine {
    pub fn new() -> Self {
        Self {
            use_llm: false,
            max_depth: 3,
        }
    }

    pub fn with_llm(mut self) -> Self {
        self.use_llm = true;
        self
    }

    pub fn reflect(
        &self,
        generated_command: &str,
        validation: &CommandValidation,
        valid_flags: &[String],
        examples: &[String],
    ) -> Reflection {
        self.rule_based_reflect(generated_command, validation, valid_flags, examples)
    }

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

        for error in &validation.errors {
            match error {
                ValidationError::InvalidFlag {
                    flag,
                    valid_flags: schema_flags,
                } => {
                    issues.push(format!("Flag '{}' is not in the valid flags list", flag));
                    if let Some(closest) = schema_flags.first() {
                        guidance_parts.push(format!(
                            "Replace '{}' with '{}' (similar valid flag)",
                            flag, closest
                        ));
                    } else {
                        guidance_parts.push(format!(
                            "Remove '{}' - it's not a valid flag for this tool/subcommand",
                            flag
                        ));
                    }
                    approach = Approach::RemoveHallucinations;
                }
                ValidationError::MissingRequiredFlag { flag } => {
                    issues.push(format!("Missing required flag '{}'", flag));
                    guidance_parts.push(format!("Add the required flag '{}'", flag));
                    approach = Approach::UseWhitelistFlags;
                }
                ValidationError::MissingSubcommand { expected } => {
                    issues.push(format!("Missing subcommand '{}'", expected));
                    guidance_parts.push(format!("Add subcommand '{}' at the beginning", expected));
                    approach = Approach::AddSubcommand;
                }
                ValidationError::WrongSubcommand { expected, actual } => {
                    issues.push(format!(
                        "Wrong subcommand: expected '{}', got '{}'",
                        expected, actual
                    ));
                    guidance_parts.push(format!("Replace '{}' with '{}'", actual, expected));
                    approach = Approach::AddSubcommand;
                }
                ValidationError::ConstraintViolation { message } => {
                    issues.push(message.clone());
                    guidance_parts.push(message.clone());
                    approach = Approach::ReconsiderCombination;
                }
                ValidationError::WrongValueType {
                    flag,
                    expected_type,
                    actual_value,
                } => {
                    issues.push(format!(
                        "Flag '{}' has wrong format (expected {}, got '{}')",
                        flag, expected_type, actual_value
                    ));
                    guidance_parts
                        .push(format!("Fix '{}' to have format: {}", flag, expected_type));
                    approach = Approach::FixFormat;
                }
                ValidationError::MissingPositional { position, name } => {
                    issues.push(format!(
                        "Missing positional argument '{}' at position {}",
                        name, position
                    ));
                    guidance_parts.push(format!("Add the positional argument '{}'", name));
                    approach = Approach::UseWhitelistFlags;
                }
            }
        }

        if validation.detected_subcommand.is_none() && !valid_flags.is_empty() {
            let parts: Vec<&str> = generated_command.split_whitespace().collect();
            if !parts.is_empty() && parts[0].starts_with('-') {
                issues.push("Missing subcommand - command starts with a flag".to_string());
                guidance_parts.push(
                    "Start with the subcommand name (e.g., 'sort', 'view', 'intersect') before flags".to_string(),
                );
                approach = Approach::AddSubcommand;
            }
        }

        let analysis = if issues.is_empty() {
            "No specific issues identified".to_string()
        } else {
            format!("Found {} issue(s): {}", issues.len(), issues.join("; "))
        };

        let guidance = if guidance_parts.is_empty() {
            "Review the valid flags list and try again".to_string()
        } else {
            guidance_parts.join(". ")
        };

        let confidence = if issues.is_empty() {
            0.5
        } else {
            (0.3 + 0.7 * (1.0 - (issues.len() as f32 / 5.0).min(1.0))).min(0.95)
        };

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

    pub fn build_reflection_prompt(
        &self,
        generated: &str,
        validation: &CommandValidation,
        valid_flags: &[String],
        examples: &[String],
    ) -> String {
        let mut prompt = format!(
            "You are a CLI command analysis expert. Review this generated command:\n\nGenerated: {}\n\n",
            generated
        );

        if !validation.errors.is_empty() {
            prompt.push_str("Validation Errors:\n");
            for (i, e) in validation.errors.iter().enumerate() {
                prompt.push_str(&format!("{}. {:?}\n", i + 1, e));
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

        prompt.push_str("\nAnalyze why errors occurred and provide specific fix guidance. Be concise and actionable.");
        prompt
    }

    pub fn build_improved_prompt(
        &self,
        original_task: &str,
        reflection: &Reflection,
        previous_attempts: &[String],
        valid_flags: &[String],
        examples: &[String],
    ) -> String {
        let mut prompt = format!(
            "Generate a command for: {}\n\nPrevious attempts failed. Analysis: {}\nGuidance: {}\n\n",
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

    pub fn should_retry(&self, reflection: &Reflection, iteration: usize) -> bool {
        if iteration >= self.max_depth {
            return false;
        }
        reflection.confidence > 0.4
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReflectionHistory {
    pub attempts: Vec<AttemptRecord>,
    pub current_iteration: usize,
}

#[derive(Debug, Clone)]
pub struct AttemptRecord {
    pub command: String,
    pub validation: CommandValidation,
    pub reflection: Reflection,
}

impl ReflectionHistory {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn previous_commands(&self) -> Vec<String> {
        self.attempts.iter().map(|a| a.command.clone()).collect()
    }

    pub fn best_attempt(&self) -> Option<&AttemptRecord> {
        self.attempts
            .iter()
            .min_by_key(|a| a.validation.errors.len())
    }

    pub fn is_making_progress(&self) -> bool {
        if self.attempts.len() < 2 {
            return true;
        }
        let last = &self.attempts[self.attempts.len() - 1];
        let prev = &self.attempts[self.attempts.len() - 2];
        last.validation.errors.len() <= prev.validation.errors.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ValidationError;

    #[test]
    fn test_reflection_on_invalid_flag() {
        let engine = ReflectionEngine::new();
        let validation = CommandValidation {
            is_valid: false,
            detected_subcommand: Some("sort".to_string()),
            errors: vec![ValidationError::InvalidFlag {
                flag: "--fast".to_string(),
                valid_flags: vec!["-@".to_string()],
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
        assert_eq!(reflection.approach, Approach::RemoveHallucinations);
    }

    #[test]
    fn test_reflection_on_missing_subcommand() {
        let engine = ReflectionEngine::new();
        let validation = CommandValidation {
            is_valid: false,
            detected_subcommand: None,
            errors: vec![],
            suggestions: vec![],
        };

        let reflection = engine.reflect("-o out.bam in.bam", &validation, &["-o".to_string()], &[]);
        assert!(reflection.analysis.contains("Missing subcommand"));
        assert_eq!(reflection.approach, Approach::AddSubcommand);
    }
}
