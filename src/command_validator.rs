//! Command Validator Module (HDA)
//!
//! Validates generated commands against CLI schema constraints.
//! Replaces the old constraint_graph-based validation with schema-driven validation.

#![allow(dead_code)]

use crate::schema::{CliSchema, ValidationError, ValidationResult};
use crate::subcommand_detector::SubcommandDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArg {
    pub name: String,
    pub value: Option<String>,
    pub is_positional: bool,
}

impl ParsedArg {
    pub fn flag(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            is_positional: false,
        }
    }

    pub fn flag_with_value(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
            is_positional: false,
        }
    }

    pub fn positional(value: impl Into<String>) -> Self {
        Self {
            name: value.into(),
            value: None,
            is_positional: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandValidation {
    pub is_valid: bool,
    pub detected_subcommand: Option<String>,
    pub errors: Vec<ValidationError>,
    pub suggestions: Vec<FixSuggestion>,
}

#[derive(Debug, Clone)]
pub struct FixSuggestion {
    pub fix_type: FixType,
    pub description: String,
    pub original: String,
    pub replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixType {
    ReplaceFlag,
    AddSubcommand,
    RemoveFlag,
    FixFormat,
}

#[derive(Debug, Clone)]
pub struct CommandValidator {
    schema: Option<CliSchema>,
    subcommands: Vec<SubcommandDef>,
}

impl CommandValidator {
    pub fn new(schema: Option<CliSchema>, subcommands: Vec<SubcommandDef>) -> Self {
        Self {
            schema,
            subcommands,
        }
    }

    pub fn with_schema(mut self, schema: CliSchema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn parse_command(&self, command: &str) -> (Option<String>, Vec<ParsedArg>) {
        let trimmed = command.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        if parts.is_empty() {
            return (None, Vec::new());
        }

        let (subcommand, args_start) = self.detect_subcommand(&parts);

        let mut args = Vec::new();
        let mut i = args_start;

        while i < parts.len() {
            let part = parts[i];

            if part.starts_with('-') {
                let flag_name = part.to_string();

                if i + 1 < parts.len() && !parts[i + 1].starts_with('-') {
                    let value = parts[i + 1].to_string();
                    args.push(ParsedArg::flag_with_value(flag_name, value));
                    i += 2;
                } else if let Some(pos) = part.find('=') {
                    let name = part[..pos].to_string();
                    let value = part[pos + 1..].to_string();
                    args.push(ParsedArg::flag_with_value(name, value));
                    i += 1;
                } else {
                    args.push(ParsedArg::flag(flag_name));
                    i += 1;
                }
            } else {
                args.push(ParsedArg::positional(part));
                i += 1;
            }
        }

        (subcommand, args)
    }

    fn detect_subcommand(&self, parts: &[&str]) -> (Option<String>, usize) {
        if parts.is_empty() {
            return (None, 0);
        }

        let first = parts[0];
        for subcmd in &self.subcommands {
            if subcmd.name == first {
                return (Some(subcmd.name.clone()), 1);
            }
        }

        if let Some(ref schema) = self.schema
            && let Some(matched) = schema.select_subcommand(first)
        {
            return (Some(matched.name.clone()), 1);
        }

        (None, 0)
    }

    pub fn validate(&self, command: &str, expected_subcommand: Option<&str>) -> CommandValidation {
        let (detected_subcmd, args) = self.parse_command(command);

        let flag_args: Vec<(String, Option<String>)> = args
            .iter()
            .filter(|a| !a.is_positional)
            .map(|a| (a.name.clone(), a.value.clone()))
            .collect();

        let schema_result: ValidationResult = if let Some(ref schema) = self.schema {
            schema.validate_command(&flag_args, detected_subcmd.as_deref())
        } else {
            ValidationResult::valid()
        };

        let errors = if schema_result.is_valid {
            Vec::new()
        } else {
            schema_result.errors.clone()
        };

        let suggestions =
            self.generate_suggestions(&errors, &detected_subcmd, expected_subcommand, command);

        CommandValidation {
            is_valid: schema_result.is_valid
                && self.check_subcommand(&detected_subcmd, expected_subcommand),
            detected_subcommand: detected_subcmd,
            errors,
            suggestions,
        }
    }

    fn check_subcommand(&self, detected: &Option<String>, expected: Option<&str>) -> bool {
        match (detected, expected) {
            (_, None) => true,
            (Some(d), Some(e)) => d == e,
            (None, Some(_)) => false,
        }
    }

    fn generate_suggestions(
        &self,
        errors: &[ValidationError],
        detected_subcmd: &Option<String>,
        expected_subcommand: Option<&str>,
        original_command: &str,
    ) -> Vec<FixSuggestion> {
        let mut suggestions = Vec::new();

        for error in errors {
            match error {
                ValidationError::InvalidFlag { flag, valid_flags } => {
                    let best_match = self.find_closest_flag(flag, valid_flags);
                    if let Some(replacement) = best_match {
                        suggestions.push(FixSuggestion {
                            fix_type: FixType::ReplaceFlag,
                            description: format!(
                                "Replace invalid flag '{}' with '{}'",
                                flag, replacement
                            ),
                            original: flag.clone(),
                            replacement,
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
                ValidationError::MissingRequiredFlag { flag } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::AddSubcommand,
                        description: format!("Add required flag '{}'", flag),
                        original: String::new(),
                        replacement: flag.clone(),
                    });
                }
                ValidationError::MissingSubcommand { expected } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::AddSubcommand,
                        description: format!(
                            "Add missing subcommand '{}' at the beginning",
                            expected
                        ),
                        original: original_command.to_string(),
                        replacement: format!("{} {}", expected, original_command),
                    });
                }
                ValidationError::WrongSubcommand { expected, actual } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::ReplaceFlag,
                        description: format!("Replace subcommand '{}' with '{}'", actual, expected),
                        original: actual.clone(),
                        replacement: expected.clone(),
                    });
                }
                ValidationError::ConstraintViolation { message } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::FixFormat,
                        description: message.clone(),
                        original: String::new(),
                        replacement: String::new(),
                    });
                }
                ValidationError::WrongValueType {
                    flag,
                    expected_type,
                    actual_value,
                } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::FixFormat,
                        description: format!(
                            "Flag {}: expected {}, got '{}'",
                            flag, expected_type, actual_value
                        ),
                        original: actual_value.clone(),
                        replacement: format!("<{}>", expected_type),
                    });
                }
                ValidationError::MissingPositional { position, name } => {
                    suggestions.push(FixSuggestion {
                        fix_type: FixType::AddSubcommand,
                        description: format!(
                            "Missing positional argument '{}' at position {}",
                            name, position
                        ),
                        original: String::new(),
                        replacement: name.clone(),
                    });
                }
            }
        }

        if let Some(expected) = expected_subcommand
            && detected_subcmd.as_deref() != Some(expected)
            && !suggestions
                .iter()
                .any(|s| matches!(s.fix_type, FixType::AddSubcommand))
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

    fn find_closest_flag(&self, flag: &str, valid_flags: &[String]) -> Option<String> {
        let flag_lower = flag.to_lowercase().trim_start_matches('-').to_string();
        let mut best: Option<(String, usize)> = None;

        for valid in valid_flags {
            let valid_lower = valid.to_lowercase().trim_start_matches('-').to_string();
            if valid_lower == flag_lower {
                return Some(valid.clone());
            }
            let dist = levenshtein_distance(&flag_lower, &valid_lower);
            if dist <= 2 && best.as_ref().is_none_or(|(_, d)| dist < *d) {
                best = Some((valid.clone(), dist));
            }
        }

        best.map(|(s, _)| s)
    }

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
                    let pattern = format!("{}\\s+\\S+", regex::escape(&suggestion.original));
                    fixed = regex::Regex::new(&pattern)
                        .ok()?
                        .replace(&fixed, "")
                        .to_string();
                    fixed = fixed.replace(&suggestion.original, "");
                }
                FixType::AddSubcommand if suggestion.original.is_empty() => {
                    fixed = format!("{} {}", suggestion.replacement, fixed);
                }
                _ => {}
            }
        }

        fixed = fixed.split_whitespace().collect::<Vec<_>>().join(" ");

        let revalidation = self.validate(&fixed, expected_subcmd);
        if revalidation.is_valid {
            Some(fixed)
        } else {
            None
        }
    }

    pub fn get_stats(&self, commands: &[&str]) -> ValidationStats {
        let mut stats = ValidationStats::default();

        for cmd in commands {
            let validation = self.validate(cmd, None);
            stats.total += 1;

            if validation.is_valid {
                stats.valid += 1;
            } else {
                stats.invalid += 1;
                stats.errors.extend(validation.errors);
            }
        }

        stats
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub total: usize,
    pub valid: usize,
    pub invalid: usize,
    pub errors: Vec<ValidationError>,
}

impl ValidationStats {
    pub fn accuracy(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        self.valid as f32 / self.total as f32
    }

    pub fn error_counts(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for error in &self.errors {
            let key = match error {
                ValidationError::InvalidFlag { .. } => "invalid_flag",
                ValidationError::MissingRequiredFlag { .. } => "missing_required",
                ValidationError::MissingSubcommand { .. } => "missing_subcommand",
                ValidationError::WrongSubcommand { .. } => "wrong_subcommand",
                ValidationError::ConstraintViolation { .. } => "constraint_violation",
                ValidationError::WrongValueType { .. } => "wrong_value_type",
                ValidationError::MissingPositional { .. } => "missing_positional",
            };
            *counts.entry(key.to_string()).or_insert(0) += 1;
        }
        counts
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *cell = j;
    }

    for (i, ac) in a.chars().enumerate() {
        for (j, bc) in b.chars().enumerate() {
            let cost = if ac == bc { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_with_subcommand() {
        let validator = CommandValidator::new(
            None,
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
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_parse_flag_with_value() {
        let validator = CommandValidator::new(None, vec![]);
        let (_, args) = validator.parse_command("--output file.txt -t 8");
        assert_eq!(args[0].name, "--output");
        assert_eq!(args[0].value, Some("file.txt".to_string()));
        assert_eq!(args[1].name, "-t");
        assert_eq!(args[1].value, Some("8".to_string()));
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("output", "output"), 0);
        assert_eq!(levenshtein_distance("output", "ouput"), 1);
        assert_eq!(levenshtein_distance("abc", "xyz"), 3);
    }
}
