//! Command Validator Module
//!
//! Validates generated commands against documentation constraints
//! to detect hallucinations and enforce parameter rules.

#![allow(dead_code)]

use crate::constraint_graph::{ConstraintGraph, Violation};
use crate::subcommand_detector::SubcommandDef;

/// Parsed command argument
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArg {
    /// Flag name (e.g., "-o", "--output")
    pub name: String,
    /// Optional value
    pub value: Option<String>,
    /// Whether this is a positional argument (not a flag)
    pub is_positional: bool,
}

impl ParsedArg {
    /// Create a new flag argument
    pub fn flag(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            is_positional: false,
        }
    }

    /// Create a new flag with value
    pub fn flag_with_value(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
            is_positional: false,
        }
    }

    /// Create a positional argument
    pub fn positional(value: impl Into<String>) -> Self {
        Self {
            name: value.into(),
            value: None,
            is_positional: true,
        }
    }
}

/// Command validation result
#[derive(Debug, Clone)]
pub struct CommandValidation {
    /// Whether the command passed validation
    pub is_valid: bool,
    /// Detected subcommand (if any)
    pub detected_subcommand: Option<String>,
    /// List of validation violations
    pub violations: Vec<Violation>,
    /// Suggestions for fixes
    pub suggestions: Vec<FixSuggestion>,
}

/// Fix suggestion for a violation
#[derive(Debug, Clone)]
pub struct FixSuggestion {
    /// Type of fix
    pub fix_type: FixType,
    /// Description of the fix
    pub description: String,
    /// Original text
    pub original: String,
    /// Replacement text
    pub replacement: String,
}

/// Type of fix
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixType {
    /// Replace hallucinated flag with correct one
    ReplaceFlag,
    /// Add missing subcommand
    AddSubcommand,
    /// Remove invalid flag
    RemoveFlag,
    /// Fix flag format
    FixFormat,
}

/// Command Validator
#[derive(Debug, Clone)]
pub struct CommandValidator {
    /// Constraint graph for validation
    constraint_graph: ConstraintGraph,
    /// Known subcommands
    subcommands: Vec<SubcommandDef>,
}

impl CommandValidator {
    /// Create a new command validator
    pub fn new(constraint_graph: ConstraintGraph, subcommands: Vec<SubcommandDef>) -> Self {
        Self {
            constraint_graph,
            subcommands,
        }
    }

    /// Parse command string into arguments
    pub fn parse_command(&self, command: &str) -> (Option<String>, Vec<ParsedArg>) {
        let trimmed = command.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            return (None, Vec::new());
        }

        // First, try to detect subcommand
        let (subcommand, args_start) = self.detect_subcommand(&parts);

        // Parse remaining parts as arguments
        let mut args = Vec::new();
        let mut i = args_start;

        while i < parts.len() {
            let part = parts[i];

            if part.starts_with('-') {
                // This is a flag
                let flag_name = part.to_string();

                // Check if next part is a value (not a flag)
                if i + 1 < parts.len() && !parts[i + 1].starts_with('-') {
                    let value = parts[i + 1].to_string();
                    args.push(ParsedArg::flag_with_value(flag_name, value));
                    i += 2;
                } else if let Some(pos) = part.find('=') {
                    // Flag with = syntax: --flag=value
                    let name = part[..pos].to_string();
                    let value = part[pos + 1..].to_string();
                    args.push(ParsedArg::flag_with_value(name, value));
                    i += 1;
                } else {
                    // Flag without value
                    args.push(ParsedArg::flag(flag_name));
                    i += 1;
                }
            } else {
                // Positional argument
                args.push(ParsedArg::positional(part));
                i += 1;
            }
        }

        (subcommand, args)
    }

    /// Detect subcommand from command parts
    fn detect_subcommand(&self, parts: &[&str]) -> (Option<String>, usize) {
        if parts.is_empty() {
            return (None, 0);
        }

        // Check if first part matches any known subcommand
        let first = parts[0];
        for subcmd in &self.subcommands {
            if subcmd.name == first {
                return (Some(subcmd.name.clone()), 1);
            }
        }

        // Also check aliases
        for subcmd in &self.subcommands {
            // Simple alias matching (could be enhanced)
            if (first.starts_with(&subcmd.name) || subcmd.name.starts_with(first))
                && self.similarity(first, &subcmd.name) > 0.7
            {
                return (Some(subcmd.name.clone()), 1);
            }
        }

        (None, 0)
    }

    /// Calculate string similarity
    fn similarity(&self, a: &str, b: &str) -> f32 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        if a_lower == b_lower {
            return 1.0;
        }

        // Simple prefix matching
        if a_lower.starts_with(&b_lower) || b_lower.starts_with(&a_lower) {
            let min_len = a_lower.len().min(b_lower.len());
            let max_len = a_lower.len().max(b_lower.len());
            return min_len as f32 / max_len as f32;
        }

        0.0
    }

    /// Validate a command
    pub fn validate(&self, command: &str, expected_subcommand: Option<&str>) -> CommandValidation {
        let (detected_subcmd, args) = self.parse_command(command);

        // Convert ParsedArg to format expected by constraint graph
        let flag_args: Vec<(String, Option<String>)> = args
            .iter()
            .filter(|a| !a.is_positional)
            .map(|a| (a.name.clone(), a.value.clone()))
            .collect();

        // Validate against constraint graph
        let validation_report = self
            .constraint_graph
            .validate(&flag_args, detected_subcmd.as_deref());

        // Generate fix suggestions
        let suggestions = self.generate_suggestions(
            &validation_report.violations,
            &detected_subcmd,
            expected_subcommand,
            command,
        );

        CommandValidation {
            is_valid: validation_report.is_valid,
            detected_subcommand: detected_subcmd,
            violations: validation_report.violations,
            suggestions,
        }
    }

    /// Generate fix suggestions for violations
    fn generate_suggestions(
        &self,
        violations: &[Violation],
        detected_subcmd: &Option<String>,
        expected_subcmd: Option<&str>,
        original_command: &str,
    ) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();

        for violation in violations {
            match violation {
                Violation::HallucinatedFlag { flag, suggestion } => {
                    if let Some(replacement) = suggestion {
                        suggestions.push(FixSuggestion {
                            fix_type: FixType::ReplaceFlag,
                            description: format!(
                                "Replace hallucinated flag '{}' with '{}'",
                                flag, replacement
                            ),
                            original: flag.clone(),
                            replacement: replacement.clone(),
                        });
                    } else {
                        suggestions.push(FixSuggestion {
                            fix_type: FixType::RemoveFlag,
                            description: format!("Remove invalid flag '{}'", flag),
                            original: flag.clone(),
                            replacement: String::new(),
                        });
                    }
                }
                Violation::MissingRequired { flag, context: _ } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::AddSubcommand,
                        description: format!("Add required flag '{}'", flag),
                        original: String::new(),
                        replacement: flag.clone(),
                    });
                }
                _ => {}
            }
        }

        // Check for missing subcommand
        if let Some(expected) = expected_subcmd
            && detected_subcmd.as_deref() != Some(expected)
        {
            suggestions.push(FixSuggestion {
                fix_type: FixType::AddSubcommand,
                description: format!("Add missing subcommand '{}' at the beginning", expected),
                original: original_command.to_string(),
                replacement: format!("{} {}", expected, original_command),
            });
        }

        suggestions
    }

    /// Auto-fix a command based on validation
    pub fn auto_fix(&self, command: &str, expected_subcmd: Option<&str>) -> Option<String> {
        let validation = self.validate(command, expected_subcmd);

        if validation.is_valid {
            return Some(command.to_string());
        }

        let mut fixed = command.to_string();

        for suggestion in &validation.suggestions {
            match suggestion.fix_type {
                FixType::ReplaceFlag => {
                    fixed = fixed.replace(&suggestion.original, &suggestion.replacement);
                }
                FixType::RemoveFlag => {
                    // Remove the flag and its value if present
                    let pattern = format!("{}\\s+\\S+", regex::escape(&suggestion.original));
                    fixed = regex::Regex::new(&pattern)
                        .ok()?
                        .replace(&fixed, "")
                        .to_string();
                    // Also try without value
                    fixed = fixed.replace(&suggestion.original, "");
                }
                FixType::AddSubcommand if suggestion.original.is_empty() => {
                    // Add at beginning
                    fixed = format!("{} {}", suggestion.replacement, fixed);
                }
                _ => {}
            }
        }

        // Clean up extra whitespace
        fixed = fixed.split_whitespace().collect::<Vec<_>>().join(" ");

        // Validate the fixed command
        let revalidation = self.validate(&fixed, expected_subcmd);
        if revalidation.is_valid {
            Some(fixed)
        } else {
            // If still invalid, return None
            None
        }
    }

    /// Get validation statistics
    pub fn get_stats(&self, commands: &[&str]) -> ValidationStats {
        let mut stats = ValidationStats::default();

        for cmd in commands {
            let validation = self.validate(cmd, None);
            stats.total += 1;

            if validation.is_valid {
                stats.valid += 1;
            } else {
                stats.invalid += 1;
                stats.violations.extend(validation.violations);
            }
        }

        stats
    }
}

/// Validation statistics
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// Total commands validated
    pub total: usize,
    /// Valid commands
    pub valid: usize,
    /// Invalid commands
    pub invalid: usize,
    /// All violations found
    pub violations: Vec<Violation>,
}

impl ValidationStats {
    /// Get accuracy rate
    pub fn accuracy(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.valid as f32 / self.total as f32
    }

    /// Get violation counts by type
    pub fn violation_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();

        for violation in &self.violations {
            let key = match violation {
                Violation::HallucinatedFlag { .. } => "hallucinated_flag",
                Violation::MissingRequired { .. } => "missing_required",
                Violation::MutuallyExclusive { .. } => "mutually_exclusive",
                Violation::MissingDependency { .. } => "missing_dependency",
                Violation::InvalidFormat { .. } => "invalid_format",
            };

            *counts.entry(key.to_string()).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_with_subcommand() {
        let validator = CommandValidator::new(
            ConstraintGraph::default(),
            vec![SubcommandDef {
                name: "sort".to_string(),
                description: "Sort BAM file".to_string(),
                usage_pattern: "[options] <in.bam>".to_string(),
                flags: vec!["-o".to_string(), "-@".to_string()],
                keywords: vec!["sort".to_string()],
            }],
        );

        let (subcmd, args) = validator.parse_command("sort -o out.bam in.bam");
        assert_eq!(subcmd, Some("sort".to_string()));
        assert_eq!(args.len(), 2); // -o with value, and in.bam positional
    }

    #[test]
    fn test_parse_flag_with_value() {
        let validator = CommandValidator::new(ConstraintGraph::default(), vec![]);

        let (_, args) = validator.parse_command("--output file.txt -t 8");

        assert_eq!(args[0].name, "--output");
        assert_eq!(args[0].value, Some("file.txt".to_string()));

        assert_eq!(args[1].name, "-t");
        assert_eq!(args[1].value, Some("8".to_string()));
    }
}
