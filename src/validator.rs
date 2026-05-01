use crate::intent_mapper::LlmCommandFill;
use crate::tool_doc::ToolDoc;
use crate::tool_resolver::ToolRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    #[allow(dead_code)]
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }
}

pub struct Validator;

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(
        &self,
        _record: &ToolRecord,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        self.validate_flags(fill, doc, &mut errors, &mut warnings);
        self.validate_required_flags(fill, doc, &mut errors);
        self.validate_positionals(fill, doc, &mut errors, &mut warnings);
        self.validate_constraints(fill, doc, &mut errors);
        self.validate_skill_pitfalls(fill, doc, &mut warnings);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    fn validate_flags(
        &self,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
        errors: &mut Vec<String>,
        _warnings: &mut Vec<String>,
    ) {
        let valid_flags: Vec<&str> = doc.all_flag_names();

        let subcmd_flags: Vec<&str> = if let Some(ref subcmd_name) = fill.subcommand {
            doc.get_subcommand(subcmd_name)
                .map(|s| s.flags.iter().flat_map(|f| f.all_names()).collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let all_valid: Vec<&str> = valid_flags
            .iter()
            .chain(subcmd_flags.iter())
            .copied()
            .collect();

        for flag in fill.flags.keys() {
            if !all_valid.contains(&flag.as_str()) {
                errors.push(format!("Unknown flag: {}", flag));
            }
        }
    }

    fn validate_required_flags(
        &self,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
        errors: &mut Vec<String>,
    ) {
        let required_flag_groups: Vec<(&str, Vec<&str>)> =
            if let Some(ref subcmd_name) = fill.subcommand {
                doc.get_subcommand(subcmd_name)
                    .map(|s| {
                        s.flags
                            .iter()
                            .filter(|f| f.required)
                            .map(|f| (f.name.as_str(), f.all_names()))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                doc.flags
                    .iter()
                    .filter(|f| f.required)
                    .map(|f| (f.name.as_str(), f.all_names()))
                    .collect()
            };

        for (primary_name, all_names) in required_flag_groups {
            let provided = fill.flags.keys().any(|k| all_names.contains(&k.as_str()));
            if !provided {
                errors.push(format!("Missing required flag: {}", primary_name));
            }
        }
    }

    fn validate_positionals(
        &self,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) {
        let expected_positionals = if let Some(ref subcmd_name) = fill.subcommand {
            doc.get_subcommand(subcmd_name)
                .map(|s| s.positionals.len())
                .unwrap_or(0)
        } else {
            doc.positionals.len()
        };

        let required_positionals = if let Some(ref subcmd_name) = fill.subcommand {
            doc.get_subcommand(subcmd_name)
                .map(|s| s.positionals.iter().filter(|p| p.required).count())
                .unwrap_or(0)
        } else {
            doc.positionals.iter().filter(|p| p.required).count()
        };

        if fill.positionals.len() < required_positionals {
            errors.push(format!(
                "Missing required positional arguments: expected at least {}, got {}",
                required_positionals,
                fill.positionals.len()
            ));
        }

        if expected_positionals > 0 && fill.positionals.len() > expected_positionals {
            warnings.push(format!(
                "More positional arguments than expected: expected {}, got {}",
                expected_positionals,
                fill.positionals.len()
            ));
        }
    }

    fn validate_constraints(&self, fill: &LlmCommandFill, doc: &ToolDoc, errors: &mut Vec<String>) {
        use crate::schema::types::ConstraintRule;

        let constraints = &doc.constraints;

        for constraint in constraints {
            match constraint {
                ConstraintRule::MutuallyExclusive(a, b) => {
                    let has_a = fill.flags.keys().any(|k| k == a);
                    let has_b = fill.flags.keys().any(|k| k == b);
                    if has_a && has_b {
                        errors.push(format!(
                            "Mutually exclusive flags used together: {} and {}",
                            a, b
                        ));
                    }
                }
                ConstraintRule::Requires(a, b) => {
                    let has_a = fill.flags.keys().any(|k| k == a);
                    let has_b = fill.flags.keys().any(|k| k == b);
                    if has_a && !has_b {
                        errors.push(format!("Flag {} requires flag {}", a, b));
                    }
                }
                ConstraintRule::AtLeastOne(flags) => {
                    let any_present = flags.iter().any(|f| fill.flags.keys().any(|k| k == f));
                    if !any_present {
                        errors.push(format!(
                            "At least one of these flags must be present: {}",
                            flags.join(", ")
                        ));
                    }
                }
                ConstraintRule::AllRequired(flags) => {
                    let any_present = flags.iter().any(|f| fill.flags.keys().any(|k| k == f));
                    if any_present {
                        for f in flags {
                            if !fill.flags.keys().any(|k| k == f) {
                                errors.push(format!(
                                    "Flag {} is required when using this flag group",
                                    f
                                ));
                            }
                        }
                    }
                }
                ConstraintRule::ImpliesValue(a, b, val) => {
                    let has_a = fill.flags.keys().any(|k| k == a);
                    if has_a {
                        let b_val = fill.flags.get(b);
                        if b_val.is_none_or(|v| v != val) {
                            errors.push(format!("Flag {} requires {}={}", a, b, val));
                        }
                    }
                }
            }
        }
    }

    fn validate_skill_pitfalls(
        &self,
        fill: &LlmCommandFill,
        doc: &ToolDoc,
        warnings: &mut Vec<String>,
    ) {
        for pitfall in &doc.pitfalls {
            let pitfall_lower = pitfall.to_lowercase();
            let all_values: String = fill
                .flags
                .values()
                .chain(fill.positionals.iter())
                .cloned()
                .collect::<Vec<String>>()
                .join(" ")
                .to_lowercase();

            if pitfall_lower.contains("input")
                && pitfall_lower.contains("output")
                && all_values.contains(
                    &pitfall_lower
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
            {
                warnings.push(format!("Skill pitfall check: {}", pitfall));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::{CliStyle, ConstraintRule, ParamType};
    use crate::tool_doc::FlagDoc;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn make_record() -> ToolRecord {
        ToolRecord {
            name: "test".to_string(),
            resolved_path: PathBuf::from("/usr/bin/test"),
            interpreter: None,
            is_path_dependent: false,
            global_path: Some(PathBuf::from("/usr/bin/test")),
            version: None,
            companion_tools: Vec::new(),
        }
    }

    #[test]
    fn test_validate_unknown_flag() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "input".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Input,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--nonexistent".to_string(), "value".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Unknown flag")));
    }

    #[test]
    fn test_validate_valid_command() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "input".to_string(),
                default: None,
                required: false,
                category: crate::tool_doc::FlagCategory::Input,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-i".to_string(), "input.bam".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_missing_required_flag() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "input".to_string(),
                default: None,
                required: true,
                category: crate::tool_doc::FlagCategory::Input,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Missing required flag"))
        );
    }

    #[test]
    fn test_validate_required_flag_provided_via_alias() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![FlagDoc {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "input".to_string(),
                default: None,
                required: true,
                category: crate::tool_doc::FlagCategory::Input,
            }],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--input".to_string(), "in.bam".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_missing_positionals() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Positional,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: vec![crate::tool_doc::PositionalDoc {
                position: 0,
                name: "INPUT".to_string(),
                param_type: ParamType::File,
                description: "input file".to_string(),
                required: true,
                default: None,
            }],
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Missing required positional"))
        );
    }

    #[test]
    fn test_validate_extra_positionals_warning() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Positional,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: vec![crate::tool_doc::PositionalDoc {
                position: 0,
                name: "INPUT".to_string(),
                param_type: ParamType::File,
                description: "input file".to_string(),
                required: true,
                default: None,
            }],
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: vec!["in.bam".to_string(), "extra.bam".to_string()],
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(result.is_valid);
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("More positional"))
        );
    }

    #[test]
    fn test_validate_mutually_exclusive() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "mode a".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "mode b".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: vec![ConstraintRule::MutuallyExclusive(
                "-a".to_string(),
                "-b".to_string(),
            )],
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-a".to_string(), String::new());
                m.insert("-b".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Mutually exclusive"))
        );
    }

    #[test]
    fn test_validate_requires_constraint() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "enable a".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::String,
                    description: "b value".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: vec![ConstraintRule::Requires("-a".to_string(), "-b".to_string())],
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-a".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("requires")));
    }

    #[test]
    fn test_validate_at_least_one_constraint() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "a".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "b".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: vec![ConstraintRule::AtLeastOne(vec![
                "-a".to_string(),
                "-b".to_string(),
            ])],
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("At least one")));
    }

    #[test]
    fn test_validate_subcommand_flags() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: vec![crate::tool_doc::SubcommandDoc {
                name: "sort".to_string(),
                description: "sort".to_string(),
                usage_pattern: String::new(),
                flags: vec![FlagDoc {
                    name: "-@".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Int,
                    description: "threads".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::Performance,
                }],
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: Vec::new(),
            }],
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-@".to_string(), "4".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_subcommand_unknown_flag() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: vec![crate::tool_doc::SubcommandDoc {
                name: "sort".to_string(),
                description: "sort".to_string(),
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords: Vec::new(),
            }],
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: Some("sort".to_string()),
            flags: {
                let mut m = BTreeMap::new();
                m.insert("--bogus".to_string(), "value".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("Unknown flag")));
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult::invalid(vec!["error1".to_string(), "error2".to_string()]);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let result = ValidationResult::valid().with_warnings(vec!["warn1".to_string()]);
        assert!(result.is_valid);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_validate_all_required_constraint() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "a".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "b".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: vec![ConstraintRule::AllRequired(vec![
                "-a".to_string(),
                "-b".to_string(),
            ])],
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-a".to_string(), String::new());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("required when using this flag group"))
        );
    }

    #[test]
    fn test_validate_implies_value_constraint() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: vec![
                FlagDoc {
                    name: "-a".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::Bool,
                    description: "a".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
                FlagDoc {
                    name: "-b".to_string(),
                    aliases: Vec::new(),
                    param_type: ParamType::String,
                    description: "b".to_string(),
                    default: None,
                    required: false,
                    category: crate::tool_doc::FlagCategory::General,
                },
            ],
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: vec![ConstraintRule::ImpliesValue(
                "-a".to_string(),
                "-b".to_string(),
                "special".to_string(),
            )],
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };

        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-a".to_string(), String::new());
                m.insert("-b".to_string(), "wrong".to_string());
                m
            },
            positionals: Vec::new(),
        };

        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("requires") && e.contains("special"))
        );
    }

    #[test]
    fn test_check_pitfalls_input_output_conflict() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: vec!["input and output must be different files".to_string()],
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };
        let fill = LlmCommandFill {
            subcommand: None,
            flags: {
                let mut m = BTreeMap::new();
                m.insert("-i".to_string(), "input.bam".to_string());
                m.insert("-o".to_string(), "output.bam".to_string());
                m
            },
            positionals: Vec::new(),
        };
        let result = validator.validate(&record, &fill, &doc);
        assert!(result.warnings.iter().any(|w| w.contains("pitfall")));
    }

    #[test]
    fn test_check_pitfalls_no_pitfalls() {
        let validator = Validator::new();
        let record = make_record();
        let doc = ToolDoc {
            record: record.clone(),
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            schema_source: "generic".to_string(),
            doc_quality: 0.8,
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_patterns: Vec::new(),
            constraints: Vec::new(),
            examples: Vec::new(),
            concepts: Vec::new(),
            pitfalls: Vec::new(),
            raw_help: None,
            subcommand_helps: std::collections::HashMap::new(),
        };
        let fill = LlmCommandFill {
            subcommand: None,
            flags: BTreeMap::new(),
            positionals: Vec::new(),
        };
        let result = validator.validate(&record, &fill, &doc);
        assert!(!result.warnings.iter().any(|w| w.contains("pitfall")));
    }
}
