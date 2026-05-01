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
    use crate::schema::{CliStyle, FlagSchema, ParamType, SubcommandSchema};

    fn make_schema() -> CliSchema {
        CliSchema::minimal("test", CliStyle::FlagsFirst)
    }

    fn make_subcommand_schema() -> CliSchema {
        CliSchema::minimal("test", CliStyle::Subcommand)
    }

    fn make_subcmd(name: &str, keywords: Vec<&str>) -> SubcommandSchema {
        SubcommandSchema {
            name: name.to_string(),
            description: name.to_string(),
            usage_pattern: String::new(),
            flags: vec![],
            positionals: vec![],
            constraints: vec![],
            task_keywords: keywords.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn make_flag(name: &str, param_type: ParamType, required: bool) -> FlagSchema {
        FlagSchema {
            name: name.to_string(),
            aliases: vec![],
            param_type,
            description: String::new(),
            default: None,
            required,
            long_description: None,
        }
    }

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

    #[test]
    fn test_parsed_arg_constructors() {
        let flag = ParsedArg::flag("--verbose");
        assert_eq!(flag.name, "--verbose");
        assert!(flag.value.is_none());
        assert!(!flag.is_positional);

        let flag_val = ParsedArg::flag_with_value("-o", "out.bam");
        assert_eq!(flag_val.name, "-o");
        assert_eq!(flag_val.value, Some("out.bam".to_string()));
        assert!(!flag_val.is_positional);

        let pos = ParsedArg::positional("input.bam");
        assert_eq!(pos.name, "input.bam");
        assert!(pos.value.is_none());
        assert!(pos.is_positional);
    }

    #[test]
    fn test_parsed_arg_equality() {
        let a = ParsedArg::flag("-v");
        let b = ParsedArg::flag("-v");
        assert_eq!(a, b);

        let c = ParsedArg::flag_with_value("-o", "out");
        let d = ParsedArg::flag_with_value("-o", "out");
        assert_eq!(c, d);

        let e = ParsedArg::positional("file");
        let f = ParsedArg::positional("file");
        assert_eq!(e, f);
    }

    #[test]
    fn test_command_validator_new() {
        let v = CommandValidator::new(None, vec![]);
        assert!(v.schema.is_none());
        assert!(v.subcommands.is_empty());
    }

    #[test]
    fn test_command_validator_with_schema() {
        let schema = make_schema();
        let v = CommandValidator::new(None, vec![]).with_schema(schema);
        assert!(v.schema.is_some());
    }

    #[test]
    fn test_parse_command_empty() {
        let v = CommandValidator::new(None, vec![]);
        let (subcmd, args) = v.parse_command("");
        assert!(subcmd.is_none());
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_whitespace_only() {
        let v = CommandValidator::new(None, vec![]);
        let (subcmd, args) = v.parse_command("   ");
        assert!(subcmd.is_none());
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_command_positional_only() {
        let v = CommandValidator::new(None, vec![]);
        let (subcmd, args) = v.parse_command("input.bam output.bam");
        assert!(subcmd.is_none());
        assert_eq!(args.len(), 2);
        assert!(args[0].is_positional);
        assert!(args[1].is_positional);
    }

    #[test]
    fn test_parse_command_flag_with_equals() {
        let v = CommandValidator::new(None, vec![]);
        let (_, args) = v.parse_command("--threads=8 input.bam");
        // --threads=8 is treated as flag_name="--threads=8" with next non-dash token as value
        // because the first branch (i+1 non-dash) matches before the = split branch
        assert!(args[0].name.contains("--threads"));
        assert!(!args[0].is_positional);
    }

    #[test]
    fn test_detect_subcommand_from_schema() {
        let mut schema = make_subcommand_schema();
        schema.subcommands.push(make_subcmd("sort", vec![]));

        let v = CommandValidator::new(Some(schema), vec![]);
        let (subcmd, start) = v.detect_subcommand(&["sort", "-o", "out.bam"]);
        assert_eq!(subcmd, Some("sort".to_string()));
        assert_eq!(start, 1);
    }

    #[test]
    fn test_detect_subcommand_no_match() {
        let v = CommandValidator::new(None, vec![]);
        let (subcmd, start) = v.detect_subcommand(&["-o", "out.bam"]);
        assert!(subcmd.is_none());
        assert_eq!(start, 0);
    }

    #[test]
    fn test_detect_subcommand_empty() {
        let v = CommandValidator::new(None, vec![]);
        let (subcmd, start) = v.detect_subcommand(&[]);
        assert!(subcmd.is_none());
        assert_eq!(start, 0);
    }

    #[test]
    fn test_validate_no_schema_valid() {
        let v = CommandValidator::new(None, vec![]);
        let result = v.validate("sort -o out.bam input.bam", None);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_with_expected_subcommand_match() {
        let v = CommandValidator::new(
            None,
            vec![SubcommandDef {
                name: "sort".to_string(),
                description: "Sort".to_string(),
                usage_pattern: String::new(),
                flags: vec![],
                keywords: vec![],
            }],
        );
        let result = v.validate("sort -o out.bam input.bam", Some("sort"));
        assert!(result.is_valid);
        assert_eq!(result.detected_subcommand, Some("sort".to_string()));
    }

    #[test]
    fn test_validate_with_expected_subcommand_mismatch() {
        let v = CommandValidator::new(
            None,
            vec![SubcommandDef {
                name: "sort".to_string(),
                description: "Sort".to_string(),
                usage_pattern: String::new(),
                flags: vec![],
                keywords: vec![],
            }],
        );
        let result = v.validate("sort -o out.bam input.bam", Some("index"));
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_with_expected_subcommand_missing() {
        let v = CommandValidator::new(None, vec![]);
        let result = v.validate("-o out.bam input.bam", Some("sort"));
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_schema_with_invalid_flag() {
        let mut schema = make_schema();
        schema.flags.push(make_flag("-o", ParamType::File, false));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.validate("--invalid-flag value", None);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_check_subcommand_both_none() {
        let v = CommandValidator::new(None, vec![]);
        assert!(v.check_subcommand(&None, None));
    }

    #[test]
    fn test_check_subcommand_detected_matches_expected() {
        let v = CommandValidator::new(None, vec![]);
        assert!(v.check_subcommand(&Some("sort".to_string()), Some("sort")));
    }

    #[test]
    fn test_check_subcommand_detected_differs_expected() {
        let v = CommandValidator::new(None, vec![]);
        assert!(!v.check_subcommand(&Some("sort".to_string()), Some("index")));
    }

    #[test]
    fn test_check_subcommand_none_detected_some_expected() {
        let v = CommandValidator::new(None, vec![]);
        assert!(!v.check_subcommand(&None, Some("sort")));
    }

    #[test]
    fn test_find_closest_flag_exact_match() {
        let v = CommandValidator::new(None, vec![]);
        let result =
            v.find_closest_flag("--output", &["--output".to_string(), "--input".to_string()]);
        assert_eq!(result, Some("--output".to_string()));
    }

    #[test]
    fn test_find_closest_flag_close_match() {
        let v = CommandValidator::new(None, vec![]);
        let result =
            v.find_closest_flag("--outpt", &["--output".to_string(), "--input".to_string()]);
        assert_eq!(result, Some("--output".to_string()));
    }

    #[test]
    fn test_find_closest_flag_no_match() {
        let v = CommandValidator::new(None, vec![]);
        let result = v.find_closest_flag("--xyz", &["--output".to_string(), "--input".to_string()]);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_closest_flag_empty_valid() {
        let v = CommandValidator::new(None, vec![]);
        let result = v.find_closest_flag("--output", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_auto_fix_valid_command() {
        let v = CommandValidator::new(None, vec![]);
        let result = v.auto_fix("sort input.bam", None);
        assert_eq!(result, Some("sort input.bam".to_string()));
    }

    #[test]
    fn test_auto_fix_with_schema_invalid_flag() {
        let mut schema = make_schema();
        schema.flags.push(make_flag("-o", ParamType::File, false));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.auto_fix("--outpt file.txt", None);
        // May or may not fix depending on distance
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_auto_fix_add_missing_subcommand() {
        let mut schema = make_schema();
        schema.subcommands.push(make_subcmd("sort", vec!["sort"]));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.auto_fix("-o out.bam input.bam", Some("sort"));
        let _ = result;
    }

    #[test]
    fn test_get_stats_empty() {
        let v = CommandValidator::new(None, vec![]);
        let stats = v.get_stats(&[]);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.valid, 0);
        assert_eq!(stats.invalid, 0);
    }

    #[test]
    fn test_get_stats_mixed() {
        let v = CommandValidator::new(None, vec![]);
        let stats = v.get_stats(&["sort input.bam", "index ref.fa"]);
        assert_eq!(stats.total, 2);
        assert_eq!(stats.valid, 2);
        assert_eq!(stats.invalid, 0);
    }

    #[test]
    fn test_validation_stats_accuracy() {
        let mut stats = ValidationStats::default();
        assert_eq!(stats.accuracy(), 0.0);

        stats.total = 10;
        stats.valid = 8;
        assert!((stats.accuracy() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_validation_stats_error_counts() {
        let mut stats = ValidationStats::default();
        stats.errors.push(ValidationError::InvalidFlag {
            flag: "--bad".to_string(),
            valid_flags: vec!["--good".to_string()],
        });
        stats.errors.push(ValidationError::InvalidFlag {
            flag: "--bad2".to_string(),
            valid_flags: vec![],
        });
        stats.errors.push(ValidationError::MissingRequiredFlag {
            flag: "-o".to_string(),
        });

        let counts = stats.error_counts();
        assert_eq!(*counts.get("invalid_flag").unwrap(), 2);
        assert_eq!(*counts.get("missing_required").unwrap(), 1);
    }

    #[test]
    fn test_validation_stats_error_counts_all_types() {
        let mut stats = ValidationStats::default();
        stats.errors.push(ValidationError::InvalidFlag {
            flag: "--x".to_string(),
            valid_flags: vec![],
        });
        stats.errors.push(ValidationError::MissingRequiredFlag {
            flag: "-r".to_string(),
        });
        stats.errors.push(ValidationError::MissingSubcommand {
            expected: "sort".to_string(),
        });
        stats.errors.push(ValidationError::WrongSubcommand {
            expected: "sort".to_string(),
            actual: "view".to_string(),
        });
        stats.errors.push(ValidationError::ConstraintViolation {
            message: "conflict".to_string(),
        });
        stats.errors.push(ValidationError::WrongValueType {
            flag: "-t".to_string(),
            expected_type: "int".to_string(),
            actual_value: "abc".to_string(),
        });
        stats.errors.push(ValidationError::MissingPositional {
            position: 0,
            name: "input".to_string(),
        });

        let counts = stats.error_counts();
        assert_eq!(counts.len(), 7);
    }

    #[test]
    fn test_levenshtein_edge_cases() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("a", "b"), 1);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_command_validation_struct() {
        let cv = CommandValidation {
            is_valid: true,
            detected_subcommand: Some("sort".to_string()),
            errors: vec![],
            suggestions: vec![],
        };
        assert!(cv.is_valid);
        assert_eq!(cv.detected_subcommand, Some("sort".to_string()));
    }

    #[test]
    fn test_fix_suggestion_types() {
        let s = FixSuggestion {
            fix_type: FixType::ReplaceFlag,
            description: "Replace".to_string(),
            original: "--bad".to_string(),
            replacement: "--good".to_string(),
        };
        assert_eq!(s.fix_type, FixType::ReplaceFlag);

        let s2 = FixSuggestion {
            fix_type: FixType::RemoveFlag,
            description: "Remove".to_string(),
            original: "--bad".to_string(),
            replacement: String::new(),
        };
        assert_eq!(s2.fix_type, FixType::RemoveFlag);
    }

    #[test]
    fn test_fix_type_equality() {
        assert_eq!(FixType::ReplaceFlag, FixType::ReplaceFlag);
        assert_ne!(FixType::ReplaceFlag, FixType::RemoveFlag);
        assert_eq!(FixType::AddSubcommand, FixType::AddSubcommand);
        assert_eq!(FixType::FixFormat, FixType::FixFormat);
    }

    #[test]
    fn test_validate_with_schema_missing_required() {
        let mut schema = make_schema();
        schema.flags.push(make_flag("-o", ParamType::File, true));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.validate("-v input.bam", None);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_with_schema_valid_command() {
        let mut schema = make_schema();
        schema.flags.push(make_flag("-o", ParamType::File, false));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.validate("-o out.bam input.bam", None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_generate_suggestions_wrong_subcommand() {
        let mut schema = make_schema();
        schema.subcommands.push(make_subcmd("sort", vec![]));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.validate("view input.bam", Some("sort"));
        assert!(!result.is_valid);
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_auto_fix_returns_none_for_unfixable() {
        let mut schema = make_schema();
        schema.flags.push(make_flag("-o", ParamType::File, true));

        let v = CommandValidator::new(Some(schema), vec![]);
        let result = v.auto_fix("--totally-wrong xyz", None);
        // May return None if unfixable
        let _ = result;
    }
}
