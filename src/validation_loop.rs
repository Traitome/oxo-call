//! Validation-Reflection Loop Module
//!
//! Integrates command generation, validation, reflection, and auto-fixing
//! into an iterative improvement cycle.

#![allow(dead_code)]

use crate::auto_fixer::AutoFixer;
use crate::command_validator::{CommandValidation, CommandValidator, ValidationStats};
use crate::constraint_graph::ConstraintGraph;
use crate::reflection_engine::{ReflectionEngine, ReflectionHistory};
use crate::subcommand_detector::SubcommandDef;

/// Result of the validation loop
#[derive(Debug, Clone)]
pub struct ValidationLoopResult {
    /// Final command (may be fixed or best attempt)
    pub final_command: String,
    /// Whether the final command passed validation
    pub is_valid: bool,
    /// Number of iterations performed
    pub iterations: usize,
    /// History of all attempts
    pub history: ReflectionHistory,
    /// Whether auto-fix was applied
    pub auto_fixed: bool,
    /// Final validation result
    pub final_validation: CommandValidation,
}

/// Validation-Reflection Loop
#[derive(Debug, Clone)]
pub struct ValidationReflectionLoop {
    /// Command validator
    validator: CommandValidator,
    /// Reflection engine
    reflector: ReflectionEngine,
    /// Auto fixer
    auto_fixer: AutoFixer,
    /// Maximum iterations
    max_iterations: usize,
    /// Valid flags for context
    valid_flags: Vec<String>,
    /// Example commands
    examples: Vec<String>,
}

impl ValidationReflectionLoop {
    /// Create a new validation loop
    pub fn new(constraint_graph: ConstraintGraph, subcommands: Vec<SubcommandDef>) -> Self {
        Self {
            validator: CommandValidator::new(constraint_graph.clone(), subcommands),
            reflector: ReflectionEngine::new(),
            auto_fixer: AutoFixer::new(),
            max_iterations: 3,
            valid_flags: Vec::new(),
            examples: Vec::new(),
        }
    }

    /// Set valid flags for context
    pub fn with_valid_flags(mut self, flags: Vec<String>) -> Self {
        self.valid_flags = flags;
        self
    }

    /// Set example commands
    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Enable aggressive auto-fixing
    pub fn aggressive(mut self) -> Self {
        self.auto_fixer = AutoFixer::new().aggressive();
        self
    }

    /// Process a generated command through validation and fixing
    pub fn process(
        &self,
        generated: &str,
        expected_subcommand: Option<&str>,
    ) -> ValidationLoopResult {
        let mut history = ReflectionHistory::new();
        let mut current = generated.to_string();
        let mut auto_fixed = false;

        // Try auto-fix first (fast path)
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

        // Iterative validation-reflection loop
        for iteration in 0..self.max_iterations {
            // Validate current command
            let validation = self.validator.validate(&current, expected_subcommand);

            if validation.is_valid {
                // Success!
                return ValidationLoopResult {
                    final_command: current,
                    is_valid: true,
                    iterations: iteration + 1,
                    history,
                    auto_fixed,
                    final_validation: validation,
                };
            }

            // Reflect on the failure
            let reflection =
                self.reflector
                    .reflect(&current, &validation, &self.valid_flags, &self.examples);

            // Record attempt
            history.add_attempt(current.clone(), validation.clone(), reflection.clone());

            // Check if we should continue
            if !self.reflector.should_retry(&reflection, iteration) {
                break;
            }

            // For now, we rely on auto-fix in the next iteration
            // In a full implementation, this would regenerate with improved prompt
            if let Some(fixed) = self.auto_fixer.fix(&current, &validation) {
                current = fixed.fixed;
            } else {
                // No fix possible
                break;
            }
        }

        // Return best attempt
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

    /// Try to auto-fix the command
    fn try_auto_fix(
        &self,
        command: &mut String,
        expected_subcommand: Option<&str>,
    ) -> Option<CommandValidation> {
        let validation = self.validator.validate(command, expected_subcommand);

        if validation.is_valid {
            return Some(validation);
        }

        // Try quick fix first
        if let Some(quick_fixed) = self.auto_fixer.quick_fix(command, &self.valid_flags) {
            *command = quick_fixed;
            let validation = self.validator.validate(command, expected_subcommand);
            if validation.is_valid {
                return Some(validation);
            }
        }

        // Try full auto-fix
        if let Some(fix_result) = self.auto_fixer.fix(command, &validation) {
            *command = fix_result.fixed;
            let validation = self.validator.validate(command, expected_subcommand);
            return Some(validation);
        }

        None
    }

    /// Validate multiple commands and get statistics
    pub fn validate_batch(
        &self,
        commands: &[(String, Option<String>)], // (command, expected_subcommand)
    ) -> BatchValidationResult {
        let mut results = Vec::new();
        let mut stats = ValidationStats::default();

        for (command, expected_subcmd) in commands {
            let result = self.process(command, expected_subcmd.as_deref());
            stats.total += 1;

            if result.is_valid {
                stats.valid += 1;
            } else {
                stats.invalid += 1;
                stats
                    .violations
                    .extend(result.final_validation.violations.clone());
            }

            results.push(result);
        }

        BatchValidationResult { results, stats }
    }

    /// Get diagnostic information
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

/// Batch validation result
#[derive(Debug, Clone)]
pub struct BatchValidationResult {
    /// Individual results
    pub results: Vec<ValidationLoopResult>,
    /// Aggregate statistics
    pub stats: ValidationStats,
}

impl BatchValidationResult {
    /// Get accuracy rate
    pub fn accuracy(&self) -> f32 {
        self.stats.accuracy()
    }

    /// Get results that needed fixing
    pub fn fixed_results(&self) -> Vec<&ValidationLoopResult> {
        self.results.iter().filter(|r| r.auto_fixed).collect()
    }

    /// Get results that remained invalid
    pub fn invalid_results(&self) -> Vec<&ValidationLoopResult> {
        self.results.iter().filter(|r| !r.is_valid).collect()
    }

    /// Print summary report
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

        if !self.stats.violations.is_empty() {
            println!("\nViolation breakdown:");
            let counts = self.stats.violation_counts();
            for (violation_type, count) in counts {
                println!("  - {}: {}", violation_type, count);
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

/// Command diagnosis information
#[derive(Debug, Clone)]
pub struct CommandDiagnosis {
    /// Detected subcommand
    pub parsed_subcommand: Option<String>,
    /// Parsed arguments
    pub parsed_args: Vec<crate::command_validator::ParsedArg>,
    /// Validation result
    pub validation: CommandValidation,
    /// Sample of valid flags
    pub valid_flags_sample: Vec<String>,
}

impl CommandDiagnosis {
    /// Print diagnosis
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

        if !self.validation.violations.is_empty() {
            println!("\nViolations:");
            for (i, v) in self.validation.violations.iter().enumerate() {
                println!("  {}. {:?}", i + 1, v);
            }
        }

        println!(
            "\nValid flags (sample): {}",
            self.valid_flags_sample.join(", ")
        );
    }
}

/// Builder for ValidationReflectionLoop
pub struct ValidationLoopBuilder {
    constraint_graph: Option<ConstraintGraph>,
    subcommands: Vec<SubcommandDef>,
    valid_flags: Vec<String>,
    examples: Vec<String>,
    max_iterations: usize,
    aggressive: bool,
}

impl Default for ValidationLoopBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationLoopBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            constraint_graph: None,
            subcommands: Vec::new(),
            valid_flags: Vec::new(),
            examples: Vec::new(),
            max_iterations: 3,
            aggressive: false,
        }
    }

    /// Set constraint graph
    pub fn constraint_graph(mut self, cg: ConstraintGraph) -> Self {
        self.constraint_graph = Some(cg);
        self
    }

    /// Set subcommands
    pub fn subcommands(mut self, sc: Vec<SubcommandDef>) -> Self {
        self.subcommands = sc;
        self
    }

    /// Set valid flags
    pub fn valid_flags(mut self, flags: Vec<String>) -> Self {
        self.valid_flags = flags;
        self
    }

    /// Set examples
    pub fn examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    /// Set max iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Enable aggressive mode
    pub fn aggressive(mut self) -> Self {
        self.aggressive = true;
        self
    }

    /// Build the validation loop
    pub fn build(self) -> Option<ValidationReflectionLoop> {
        let cg = self.constraint_graph?;
        let mut loop_ = ValidationReflectionLoop::new(cg, self.subcommands)
            .with_valid_flags(self.valid_flags)
            .with_examples(self.examples)
            .with_max_iterations(self.max_iterations);

        if self.aggressive {
            loop_ = loop_.aggressive();
        }

        Some(loop_)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint_graph::ConstraintGraph;

    #[test]
    fn test_validation_loop_with_valid_command() {
        let cg = ConstraintGraph::default();
        let loop_ = ValidationReflectionLoop::new(cg, vec![]).with_max_iterations(2);

        // A simple valid command (no flags to validate)
        let result = loop_.process("sort in.bam", Some("sort"));

        assert!(result.is_valid);
        assert_eq!(result.iterations, 1);
    }

    #[test]
    fn test_batch_validation() {
        let cg = ConstraintGraph::default();
        let loop_ = ValidationReflectionLoop::new(cg, vec![]);

        let commands = vec![
            ("sort in.bam".to_string(), Some("sort".to_string())),
            ("view in.bam".to_string(), Some("view".to_string())),
        ];

        let batch = loop_.validate_batch(&commands);

        assert_eq!(batch.stats.total, 2);
        assert!(batch.accuracy() >= 0.0);
    }
}
