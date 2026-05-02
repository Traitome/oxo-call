//! Deterministic post-processing of LLM-generated command arguments.
//!
//! This module provides 100% deterministic operations to fix common errors
//! in LLM-generated CLI arguments:
//! - Subcommand injection (for subcommand-style tools)
//! - Flag whitelist enforcement (remove hallucinated flags)
//! - Required flag/positional validation

use super::types::{CliSchema, CliStyle, ParamType};

/// Post-process LLM-generated args using schema constraints.
pub fn schema_post_process(args: &[String], schema: &CliSchema, task: &str) -> Vec<String> {
    if args.is_empty() {
        return args.to_vec();
    }

    let mut tokens = args.to_vec();

    // Phase 1: Subcommand deterministic injection
    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        tokens = fix_subcommand(tokens, schema, task);
    }

    // Phase 2: Flag whitelist enforcement (remove hallucinated flags)
    let subcmd = detect_subcmd_from_tokens(&tokens, schema);
    tokens = remove_invalid_flags(tokens, schema, subcmd.as_deref());

    // Phase 3: Required flag injection (add missing required flags)
    tokens = inject_required_flags(tokens, schema, subcmd.as_deref());

    tokens
}

/// Phase 1: Fix missing or wrong subcommand using schema.
fn fix_subcommand(tokens: Vec<String>, schema: &CliSchema, task: &str) -> Vec<String> {
    if tokens.is_empty() {
        return tokens;
    }

    let first = &tokens[0];
    let is_known_subcmd = schema.subcommands.iter().any(|s| s.name == *first);

    if is_known_subcmd {
        return tokens;
    }

    let suggested = schema.select_subcommand(task);
    if let Some(subcmd) = suggested {
        let mut fixed = vec![subcmd.name.clone()];
        fixed.extend(tokens);
        return fixed;
    }

    tokens
}

/// Detect if a token looks like a positional argument (file path, number, etc.)
#[allow(dead_code)]
fn looks_like_positional(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    if token.starts_with('-') {
        return false;
    }
    if token.contains('.') {
        return true;
    }
    if token.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    false
}

/// Detect which subcommand is being used from the token list.
fn detect_subcmd_from_tokens(tokens: &[String], schema: &CliSchema) -> Option<String> {
    if schema.cli_style != CliStyle::Subcommand {
        return None;
    }
    tokens.first().and_then(|t| {
        schema
            .subcommands
            .iter()
            .find(|s| s.name == *t)
            .map(|s| s.name.clone())
    })
}

/// Phase 2: Remove flags that are not in the schema whitelist.
///
/// This eliminates hallucinated flags that small models frequently generate.
/// Keeps the flag's value token if it's not a flag itself.
pub fn remove_invalid_flags(
    tokens: Vec<String>,
    schema: &CliSchema,
    subcommand: Option<&str>,
) -> Vec<String> {
    let valid_flags = schema.all_flag_names(subcommand);
    if valid_flags.is_empty() {
        return tokens;
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let token = &tokens[i];

        if token.starts_with('-') && token.contains('=') {
            // --flag=value form: validate the flag part before '='
            let flag_name = token.split('=').next().unwrap_or(token);
            if valid_flags.contains(&flag_name) {
                result.push(token.clone());
            }
            i += 1;
        } else if token.starts_with('-') {
            // -flag or --flag form (without =)
            let flag_name = token.as_str();
            let is_valid = valid_flags.contains(&flag_name);

            if is_valid {
                result.push(token.clone());
                // If this flag takes a value, consume the next token if it's a value
                let takes_value = schema
                    .get_flag(flag_name, subcommand)
                    .is_some_and(|f| !matches!(f.param_type, ParamType::Bool));
                if takes_value && i + 1 < tokens.len() {
                    let next = &tokens[i + 1];
                    if !next.starts_with('-') {
                        result.push(next.clone());
                        i += 1;
                    }
                }
            } else {
                // Invalid flag: skip it AND skip the next token if it looks like a value
                if i + 1 < tokens.len() {
                    let next = &tokens[i + 1];
                    if !next.starts_with('-') {
                        i += 1;
                    }
                }
            }
            i += 1;
        } else {
            // Not a flag (positional, subcommand, or value that belongs to a preceding flag)
            result.push(token.clone());
            i += 1;
        }
    }

    result
}

/// Phase 3: Inject missing required flags.
///
/// For flags marked as required in the schema but absent from the generated
/// command, inject them with a placeholder value based on the flag's type.
pub fn inject_required_flags(
    tokens: Vec<String>,
    schema: &CliSchema,
    subcommand: Option<&str>,
) -> Vec<String> {
    let valid_flags = schema.all_flag_names(subcommand);
    if valid_flags.is_empty() {
        return tokens;
    }

    let used_flag_names: Vec<String> = tokens
        .iter()
        .filter(|t| t.starts_with('-'))
        .map(|t| t.split('=').next().unwrap_or(t).to_string())
        .collect();

    let required_flags: Vec<_> = if let Some(subcmd) = subcommand {
        schema
            .get_subcommand(subcmd)
            .map(|s| s.flags.iter().filter(|f| f.required).collect())
            .unwrap_or_default()
    } else {
        schema.flags.iter().filter(|f| f.required).collect()
    };

    let mut additions = Vec::new();
    for flag in &required_flags {
        let is_used = flag
            .all_names()
            .iter()
            .any(|n| used_flag_names.iter().any(|u| u == n));

        if !is_used {
            match &flag.param_type {
                ParamType::Bool => {
                    additions.push(flag.name.clone());
                }
                ParamType::File => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    } else {
                        additions.push(format!("{}=OUTPUT", flag.name));
                    }
                }
                ParamType::Int => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    } else {
                        additions.push(flag.name.clone());
                        additions.push("1".to_string());
                    }
                }
                _ => {
                    if let Some(default) = &flag.default {
                        additions.push(format!("{}={}", flag.name, default));
                    }
                }
            }
        }
    }

    let mut result = tokens;
    result.extend(additions);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CliStyle, FlagSchema, ParamType, SubcommandSchema};

    fn test_subcommand_schema() -> CliSchema {
        CliSchema {
            tool: "samtools".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: "Tools for BAM".to_string(),
            subcommands: vec![
                SubcommandSchema {
                    name: "sort".to_string(),
                    description: "Sort BAM".to_string(),
                    usage_pattern: "samtools sort -o output.bam input.bam".to_string(),
                    flags: vec![
                        FlagSchema {
                            name: "-@".to_string(),
                            aliases: vec!["--threads".to_string()],
                            param_type: ParamType::Int,
                            description: "Threads".to_string(),
                            default: Some("1".to_string()),
                            required: false,
                            long_description: None,
                        },
                        FlagSchema {
                            name: "-o".to_string(),
                            aliases: vec!["--output".to_string()],
                            param_type: ParamType::File,
                            description: "Output file".to_string(),
                            default: None,
                            required: true,
                            long_description: None,
                        },
                    ],
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec!["sort".to_string(), "coordinate".to_string()],
                },
                SubcommandSchema {
                    name: "index".to_string(),
                    description: "Index BAM".to_string(),
                    usage_pattern: "samtools index input.bam".to_string(),
                    flags: Vec::new(),
                    positionals: Vec::new(),
                    constraints: Vec::new(),
                    task_keywords: vec!["index".to_string()],
                },
            ],
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        }
    }

    #[test]
    fn test_fix_subcommand_inject_missing() {
        let schema = test_subcommand_schema();
        let tokens = vec!["-@".to_string(), "4".to_string(), "input.bam".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam by coordinate");
        assert_eq!(fixed[0], "sort");
    }

    #[test]
    fn test_fix_subcommand_keep_existing() {
        let schema = test_subcommand_schema();
        let tokens = vec!["sort".to_string(), "-@".to_string(), "4".to_string()];
        let fixed = fix_subcommand(tokens, &schema, "sort bam by coordinate");
        assert_eq!(fixed[0], "sort");
        assert_eq!(fixed.len(), 3);
    }

    #[test]
    fn test_remove_invalid_flags() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "--invalid".to_string(),
            "value".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "input.bam".to_string(),
        ];
        let cleaned = remove_invalid_flags(tokens, &schema, Some("sort"));
        assert!(!cleaned.iter().any(|t| t == "--invalid"));
        assert!(cleaned.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_schema_post_process_full() {
        let schema = test_subcommand_schema();
        let args: Vec<String> = vec!["-@".to_string(), "4".to_string(), "input.bam".to_string()];
        let result = schema_post_process(&args, &schema, "sort bam by coordinate");
        assert!(result[0] == "sort");
        assert!(result.iter().any(|t| t == "-@"));
    }

    #[test]
    fn test_inject_required_flags_missing() {
        let schema = test_subcommand_schema();
        let tokens = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "input.bam".to_string(),
        ];
        let result = inject_required_flags(tokens, &schema, Some("sort"));
        assert!(result.iter().any(|t| t.contains("-o")));
    }
}
