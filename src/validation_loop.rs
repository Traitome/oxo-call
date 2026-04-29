//! Validation-Reflection Loop Module (HDA)
//!
//! Integrates command generation, validation, and reflection
//! into an iterative improvement cycle using schema-based validation.

#![allow(dead_code)]

use crate::command_validator::{CommandValidation, CommandValidator, ValidationStats};
use crate::reflection_engine::{ReflectionEngine, ReflectionHistory};
use crate::schema::CliSchema;
use crate::subcommand_detector::SubcommandDef;

#[derive(Debug, Clone)]
pub struct ValidationLoopResult {
    pub final_command: String,
    pub is_valid: bool,
    pub iterations: usize,
    pub history: ReflectionHistory,
    pub auto_fixed: bool,
    pub final_validation: CommandValidation,
}

#[derive(Debug, Clone)]
pub struct ValidationReflectionLoop {
    validator: CommandValidator,
    reflector: ReflectionEngine,
    max_iterations: usize,
    valid_flags: Vec<String>,
    examples: Vec<String>,
}

impl ValidationReflectionLoop {
    pub fn new(schema: Option<CliSchema>, subcommands: Vec<SubcommandDef>) -> Self {
        Self {
            validator: CommandValidator::new(schema, subcommands),
            reflector: ReflectionEngine::new(),
            max_iterations: 3,
            valid_flags: Vec::new(),
            examples: Vec::new(),
        }
    }

    pub fn with_valid_flags(mut self, flags: Vec<String>) -> Self {
        self.valid_flags = flags;
        self
    }

    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn process(
        &self,
        generated: &str,
        expected_subcommand: Option<&str>,
    ) -> ValidationLoopResult {
        let mut history = ReflectionHistory::new();
        let mut current = generated.to_string();
        let mut auto_fixed = false;

        if let Some(validation) = self.try_auto_fix(&mut current, expected_subcommand) {
            if validation.is_valid {
                return ValidationLoopResult {
                    final_command: current,
                    is_valid: true,
                    iterations: 1,
                    history,
                    auto_fixed: true,
                    final_validation: validation,
                };
            }
            auto_fixed = true;
        }

        for iteration in 0..self.max_iterations {
            let validation = self.validator.validate(&current, expected_subcommand);

            if validation.is_valid {
                return ValidationLoopResult {
                    final_command: current,
                    is_valid: true,
                    iterations: iteration + 1,
                    history,
                    auto_fixed,
                    final_validation: validation,
                };
            }

            let reflection =
                self.reflector
                    .reflect(&current, &validation, &self.valid_flags, &self.examples);

            history.add_attempt(current.clone(), validation.clone(), reflection.clone());

            if !self.reflector.should_retry(&reflection, iteration) {
                break;
            }

            if let Some(fixed) = self.validator.auto_fix(&current, expected_subcommand) {
                current = fixed;
                auto_fixed = true;
            } else {
                break;
            }
        }

        let final_validation = self.validator.validate(&current, expected_subcommand);

        ValidationLoopResult {
            final_command: current,
            is_valid: final_validation.is_valid,
            iterations: history.attempts.len(),
            history,
            auto_fixed,
            final_validation,
        }
    }

    fn try_auto_fix(
        &self,
        command: &mut String,
        expected_subcommand: Option<&str>,
    ) -> Option<CommandValidation> {
        let validation = self.validator.validate(command, expected_subcommand);

        if validation.is_valid {
            return Some(validation);
        }

        if let Some(fixed) = self.validator.auto_fix(command, expected_subcommand) {
            *command = fixed;
            let validation = self.validator.validate(command, expected_subcommand);
            return Some(validation);
        }

        None
    }

    pub fn validate_batch(&self, commands: &[(String, Option<String>)]) -> BatchValidationResult {
        let mut results = Vec::new();
        let mut stats = ValidationStats::default();

        for (command, expected_subcmd) in commands {
            let result = self.process(command, expected_subcmd.as_deref());
            stats.total += 1;

            if result.is_valid {
                stats.valid += 1;
            } else {
                stats.invalid += 1;
                stats.errors.extend(result.final_validation.errors.clone());
            }

            results.push(result);
        }

        BatchValidationResult { results, stats }
    }

    pub fn diagnose(&self, command: &str) -> CommandDiagnosis {
        let (subcommand, args) = self.validator.parse_command(command);
        let validation = self.validator.validate(command, None);

        CommandDiagnosis {
            parsed_subcommand: subcommand,
            parsed_args: args,
            validation,
            valid_flags_sample: self.valid_flags.iter().take(10).cloned().collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchValidationResult {
    pub results: Vec<ValidationLoopResult>,
    pub stats: ValidationStats,
}

impl BatchValidationResult {
    pub fn accuracy(&self) -> f32 {
        self.stats.accuracy()
    }

    pub fn fixed_results(&self) -> Vec<&ValidationLoopResult> {
        self.results.iter().filter(|r| r.auto_fixed).collect()
    }

    pub fn invalid_results(&self) -> Vec<&ValidationLoopResult> {
        self.results.iter().filter(|r| !r.is_valid).collect()
    }

    pub fn print_report(&self) {
        println!("\n=== Validation Report ===");
        println!("Total commands: {}", self.stats.total);
        println!(
            "Valid: {} ({:.1}%)",
            self.stats.valid,
            self.accuracy() * 100.0
        );
        println!("Invalid: {}", self.stats.invalid);
        println!("Auto-fixed: {}", self.fixed_results().len());

        if !self.stats.errors.is_empty() {
            println!("\nError breakdown:");
            let counts = self.stats.error_counts();
            for (error_type, count) in counts {
                println!("  - {}: {}", error_type, count);
            }
        }

        let invalid = self.invalid_results();
        if !invalid.is_empty() {
            println!("\nStill invalid after fixes ({}):", invalid.len());
            for (i, result) in invalid.iter().take(5).enumerate() {
                println!(
                    "  {}. {} -> {} (after {} iterations)",
                    i + 1,
                    result
                        .history
                        .attempts
                        .first()
                        .map(|a| a.command.clone())
                        .unwrap_or_default(),
                    result.final_command,
                    result.iterations
                );
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandDiagnosis {
    pub parsed_subcommand: Option<String>,
    pub parsed_args: Vec<crate::command_validator::ParsedArg>,
    pub validation: CommandValidation,
    pub valid_flags_sample: Vec<String>,
}

impl CommandDiagnosis {
    pub fn print(&self) {
        println!("\n=== Command Diagnosis ===");
        println!(
            "Parsed subcommand: {:?}",
            self.parsed_subcommand.as_deref().unwrap_or("(none)")
        );
        println!("Parsed args count: {}", self.parsed_args.len());
        println!(
            "Validation: {}",
            if self.validation.is_valid {
                "✓ VALID"
            } else {
                "✗ INVALID"
            }
        );

        if !self.validation.errors.is_empty() {
            println!("\nErrors:");
            for (i, e) in self.validation.errors.iter().enumerate() {
                println!("  {}. {:?}", i + 1, e);
            }
        }

        println!(
            "\nValid flags (sample): {}",
            self.valid_flags_sample.join(", ")
        );
    }
}

pub struct ValidationLoopBuilder {
    schema: Option<CliSchema>,
    subcommands: Vec<SubcommandDef>,
    valid_flags: Vec<String>,
    examples: Vec<String>,
    max_iterations: usize,
}

impl Default for ValidationLoopBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationLoopBuilder {
    pub fn new() -> Self {
        Self {
            schema: None,
            subcommands: Vec::new(),
            valid_flags: Vec::new(),
            examples: Vec::new(),
            max_iterations: 3,
        }
    }

    pub fn schema(mut self, schema: CliSchema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn subcommands(mut self, sc: Vec<SubcommandDef>) -> Self {
        self.subcommands = sc;
        self
    }

    pub fn valid_flags(mut self, flags: Vec<String>) -> Self {
        self.valid_flags = flags;
        self
    }

    pub fn examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    pub fn build(self) -> ValidationReflectionLoop {
        ValidationReflectionLoop::new(self.schema, self.subcommands)
            .with_valid_flags(self.valid_flags)
            .with_examples(self.examples)
            .with_max_iterations(self.max_iterations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_loop_with_valid_command() {
        let loop_ = ValidationReflectionLoop::new(None, vec![]).with_max_iterations(2);
        let result = loop_.process("sort in.bam", None);
        assert!(result.is_valid);
        assert_eq!(result.iterations, 1);
    }

    #[test]
    fn test_batch_validation() {
        let loop_ = ValidationReflectionLoop::new(None, vec![]);
        let commands = vec![
            ("sort in.bam".to_string(), None),
            ("view in.bam".to_string(), None),
        ];
        let batch = loop_.validate_batch(&commands);
        assert_eq!(batch.stats.total, 2);
        assert!(batch.accuracy() >= 0.0);
    }
}
