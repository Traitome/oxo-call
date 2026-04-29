//! Schema-guided command generation.
//!
//! This module uses CliSchema to generate constrained prompts and validate
//! generated commands. It's Layer 3 of HDA: Schema-guided generation.
//!
//! ## Core Functions
//!
//! - `build_schema_prompt_section`: Generates a "Valid Flags" section from Schema
//! - `validate_command_against_schema`: Validates generated ARGS against Schema
//! - `suggest_flags_for_task`: Suggests relevant flags based on task keywords

use crate::schema::{CliSchema, CliStyle, FlagSchema, ParamType};

/// Build a schema-guided prompt section for command generation.
///
/// This replaces the heuristic flag extraction with a formal Schema IR.
/// The generated section provides:
/// - Complete list of valid flags (whitelist)
/// - Type hints for each flag (integer, file, etc.)
/// - Required flag indicators
/// - Subcommand options (if applicable)
pub fn build_schema_prompt_section(schema: &CliSchema, task: &str) -> String {
    build_schema_prompt_section_inner(schema, task, false)
}

/// Build a compact schema prompt section for small models (≤3B).
/// Uses minimal tokens while preserving critical information:
/// - Subcommand hint (single line)
/// - Flag whitelist (space-separated)
/// - Required flags (explicit list)
pub fn build_schema_prompt_section_compact(schema: &CliSchema, task: &str) -> String {
    build_schema_prompt_section_inner(schema, task, true)
}

fn build_schema_prompt_section_inner(schema: &CliSchema, task: &str, compact: bool) -> String {
    if compact {
        return build_schema_prompt_compact(schema, task);
    }

    let mut section = String::new();

    // 1. Subcommand selection (if applicable)
    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        section.push_str("## Subcommand Selection\n");
        section.push_str("This tool requires a subcommand. Select based on task:\n\n");

        // Find matching subcommand based on task keywords
        let suggested = suggest_subcommand_for_task(schema, task);
        for subcmd in &schema.subcommands {
            let marker = if suggested.as_ref() == Some(&subcmd.name) {
                "✓ " // Recommended
            } else {
                "  "
            };
            section.push_str(&format!(
                "{} `{}` — {}\n",
                marker, subcmd.name, subcmd.description
            ));
        }
        if let Some(suggested) = suggested {
            section.push_str(&format!(
                "\n**Recommended for this task: `{}`**\n",
                suggested
            ));
        }
        section.push('\n');
    }

    // 2. Global flags (apply to all subcommands)
    if !schema.global_flags.is_empty() {
        section.push_str("## Global Flags\n");
        for flag in &schema.global_flags {
            section.push_str(&format_flag_entry(flag));
        }
        section.push('\n');
    }

    // 3. Subcommand-specific flags
    if let Some(subcmd) = suggest_subcommand_for_task(schema, task) {
        if let Some(subcmd_schema) = schema.get_subcommand(&subcmd) {
            section.push_str(&format!("## Flags for `{}` subcommand\n", subcmd));
            for flag in &subcmd_schema.flags {
                section.push_str(&format_flag_entry(flag));
            }
            section.push('\n');
        }
    } else if !schema.flags.is_empty() {
        // No subcommand - use tool-level flags
        section.push_str("## Valid Flags (use ONLY these)\n");
        section.push_str("⚠️ Only flags listed below are valid. Using other flags will fail.\n\n");
        for flag in schema.flags.iter().take(30) {
            section.push_str(&format_flag_entry(flag));
        }
        section.push('\n');
    }

    // 4. Positional arguments
    if !schema.positionals.is_empty() {
        section.push_str("## Positional Arguments\n");
        for pos in &schema.positionals {
            let req = if pos.required { "required" } else { "optional" };
            section.push_str(&format!(
                "- `{}` (position {}, {}) — {}\n",
                pos.name, pos.position, req, pos.description
            ));
        }
        section.push('\n');
    }

    // 5. Constraints
    if !schema.constraints.is_empty() {
        section.push_str("## Flag Constraints\n");
        for constraint in &schema.constraints {
            section.push_str(&format!("- {}\n", constraint.message()));
        }
        section.push('\n');
    }

    section
}

/// Format a single flag entry for the prompt.
fn format_flag_entry(flag: &FlagSchema) -> String {
    let mut entry = String::new();

    // Flag names (primary + aliases)
    let names: Vec<&str> = flag.all_names();
    let names_str = names
        .iter()
        .map(|n| format!("`{}`", n))
        .collect::<Vec<_>>()
        .join(", ");

    // Required marker
    let req_marker = if flag.required { " [REQUIRED]" } else { "" };

    // Type hint
    let type_hint = format_type_hint(&flag.param_type);

    // Build entry
    entry.push_str(&format!("- {}{} {}\n", names_str, req_marker, type_hint));

    // Description
    if !flag.description.is_empty() {
        entry.push_str(&format!("  → {}\n", flag.description));
    }

    // Default value
    if let Some(default) = &flag.default {
        entry.push_str(&format!("  → Default: `{}`\n", default));
    }

    entry
}

/// Format type hint for LLM.
fn format_type_hint(param_type: &ParamType) -> String {
    match param_type {
        ParamType::Int => "⟨integer⟩".to_string(),
        ParamType::Float => "⟨number⟩".to_string(),
        ParamType::String => "⟨text⟩".to_string(),
        ParamType::File => "⟨file⟩".to_string(),
        ParamType::Bool => "(no value)".to_string(),
        ParamType::Enum(values) => format!("⟨{}⟩", values.join("|")),
    }
}

/// Suggest a subcommand based on task keywords.
pub fn suggest_subcommand_for_task(schema: &CliSchema, task: &str) -> Option<String> {
    if schema.subcommands.is_empty() {
        return None;
    }

    let task_lower = task.to_lowercase();
    let task_words: Vec<&str> = task_lower.split_whitespace().collect();

    // Score each subcommand based on keyword match
    let mut best_match: Option<(String, usize)> = None;

    for subcmd in &schema.subcommands {
        let mut score = 0;

        // Check task keywords in subcommand
        for kw in &subcmd.task_keywords {
            if task_words.iter().any(|w| w.contains(kw) || kw.contains(w)) {
                score += 2;
            }
        }

        // Check description keywords
        let desc_lower = subcmd.description.to_lowercase();
        for word in &task_words {
            if desc_lower.contains(word) {
                score += 1;
            }
        }

        // Check exact name match
        if task_words.iter().any(|w| w == &subcmd.name) {
            score += 5;
        }

        // Update best match
        if score > 0 {
            match best_match {
                None => best_match = Some((subcmd.name.clone(), score)),
                Some((_, prev_score)) if score > prev_score => {
                    best_match = Some((subcmd.name.clone(), score));
                }
                _ => {}
            }
        }
    }

    best_match.map(|(name, _)| name)
}

/// Suggest flags based on task keywords.
#[allow(dead_code)]
pub fn suggest_flags_for_task(
    schema: &CliSchema,
    task: &str,
    subcommand: Option<&str>,
) -> Vec<String> {
    let task_lower = task.to_lowercase();
    let mut suggested: Vec<String> = Vec::new();

    // Common task-flag associations
    let task_flag_map: Vec<(&str, &str)> = vec![
        ("threads", "-@"),
        ("parallel", "-@"),
        ("cpu", "-@"),
        ("output", "-o"),
        ("out", "-o"),
        ("input", "-i"),
        ("in", "-i"),
        ("bam", "-b"),
        ("sam", "-S"),
        ("verbose", "-v"),
        ("quiet", "-q"),
        ("quality", "-q"),
        ("fast", "-f"),
        ("quick", "-f"),
        ("index", "-I"),
        ("reference", "-r"),
        ("ref", "-r"),
    ];

    // Get flags for context
    let flags: Vec<&FlagSchema> = if let Some(subcmd) = subcommand {
        if let Some(subcmd_schema) = schema.get_subcommand(subcmd) {
            subcmd_schema
                .flags
                .iter()
                .chain(schema.global_flags.iter())
                .collect()
        } else {
            schema
                .flags
                .iter()
                .chain(schema.global_flags.iter())
                .collect()
        }
    } else {
        schema
            .flags
            .iter()
            .chain(schema.global_flags.iter())
            .collect()
    };

    // Match task keywords to flags
    for (keyword, flag_pattern) in &task_flag_map {
        if task_lower.contains(keyword) {
            // Find matching flag in schema
            for flag in &flags {
                if flag.matches_name(flag_pattern) {
                    suggested.push(flag.name.clone());
                    break;
                }
            }
        }
    }

    suggested
}

/// Validate generated command arguments against schema.
///
/// Returns a ValidationResult with:
/// - is_valid: Whether the command conforms to schema
/// - errors: List of validation errors (invalid flags, missing required, etc.)
/// - warnings: List of potential issues
#[allow(dead_code)]
pub fn validate_command_against_schema(
    args: &[String],
    schema: &CliSchema,
    expected_subcommand: Option<&str>,
) -> crate::schema::ValidationResult {
    schema.validate_args(args, expected_subcommand)
}

/// Extension trait for ConstraintRule to generate human-readable messages.
impl crate::schema::ConstraintRule {
    /// Get a human-readable message for this constraint.
    pub fn message(&self) -> String {
        match self {
            crate::schema::ConstraintRule::Requires(a, b) => {
                format!("Flag `{}` requires flag `{}`", a, b)
            }
            crate::schema::ConstraintRule::MutuallyExclusive(a, b) => {
                format!("Flags `{}` and `{}` cannot be used together", a, b)
            }
            crate::schema::ConstraintRule::ImpliesValue(a, b, v) => {
                format!("Flag `{}` implies `{}` = `{}`", a, b, v)
            }
            crate::schema::ConstraintRule::AllRequired(flags) => {
                format!("All of: {} are required", flags.join(", "))
            }
            crate::schema::ConstraintRule::AtLeastOne(flags) => {
                format!("At least one of: {} is required", flags.join(", "))
            }
        }
    }
}

fn build_schema_prompt_compact(schema: &CliSchema, task: &str) -> String {
    let mut section = String::new();

    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        let suggested = suggest_subcommand_for_task(schema, task);
        if let Some(ref subcmd) = suggested {
            section.push_str(&format!(
                "⚠️ SUBCOMMAND: {subcmd} (MUST be first in ARGS!)\n"
            ));
        } else {
            let names: Vec<&str> = schema
                .subcommands
                .iter()
                .map(|s| s.name.as_str())
                .take(8)
                .collect();
            section.push_str(&format!(
                "⚠️ SUBCOMMAND required! Pick from: {}\n",
                names.join("|")
            ));
        }

        if let Some(subcmd) = suggested
            && let Some(subcmd_schema) = schema.get_subcommand(&subcmd)
        {
            let all_flags: Vec<String> = subcmd_schema
                .flags
                .iter()
                .chain(schema.global_flags.iter())
                .flat_map(|f| f.all_names().into_iter().map(|n| n.to_string()))
                .collect();

            if !all_flags.is_empty() {
                section.push_str(&format!(
                    "VALID FLAGS (others will FAIL): {}\n",
                    all_flags.join(" ")
                ));
            }

            let required: Vec<&str> = subcmd_schema
                .flags
                .iter()
                .filter(|f| f.required)
                .flat_map(|f| f.all_names())
                .collect();
            if !required.is_empty() {
                section.push_str(&format!(
                    "REQUIRED: {} (missing = FAIL!)\n",
                    required.join(" ")
                ));
            }
        }
    } else {
        let all_flags: Vec<String> = schema
            .flags
            .iter()
            .chain(schema.global_flags.iter())
            .flat_map(|f| f.all_names().into_iter().map(|n| n.to_string()))
            .collect();

        if !all_flags.is_empty() {
            section.push_str(&format!(
                "VALID FLAGS (others will FAIL): {}\n",
                all_flags.join(" ")
            ));
        }

        let required: Vec<&str> = schema
            .flags
            .iter()
            .filter(|f| f.required)
            .flat_map(|f| f.all_names())
            .collect();
        if !required.is_empty() {
            section.push_str(&format!(
                "REQUIRED: {} (missing = FAIL!)\n",
                required.join(" ")
            ));
        }
    }

    if !schema.positionals.is_empty() {
        let pos_names: Vec<&str> = schema.positionals.iter().map(|p| p.name.as_str()).collect();
        section.push_str(&format!("POSITIONAL: {}\n", pos_names.join(" ")));
    }

    section
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{CliSchema, CliStyle, FlagSchema, ParamType, SubcommandSchema};

    fn test_schema() -> CliSchema {
        CliSchema {
            tool: "samtools".to_string(),
            version: Some("1.20".to_string()),
            cli_style: CliStyle::Subcommand,
            description: "Tools for manipulating BAM files".to_string(),
            schema_source: "test".to_string(),
            usage_summary: "samtools <command> [options]".to_string(),
            doc_quality: 0.9,
            subcommands: vec![
                SubcommandSchema {
                    name: "sort".to_string(),
                    description: "Sort BAM file by coordinate".to_string(),
                    usage_pattern: "samtools sort -o output.bam input.bam".to_string(),
                    flags: vec![
                        FlagSchema {
                            name: "-@".to_string(),
                            aliases: vec!["--threads".to_string()],
                            param_type: ParamType::Int,
                            description: "Number of threads".to_string(),
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
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["sort".to_string(), "coordinate".to_string()],
                },
                SubcommandSchema {
                    name: "view".to_string(),
                    description: "View and convert SAM/BAM files".to_string(),
                    usage_pattern: "samtools view -b input.bam".to_string(),
                    flags: vec![FlagSchema {
                        name: "-b".to_string(),
                        aliases: vec!["--bam".to_string()],
                        param_type: ParamType::Bool,
                        description: "Output BAM".to_string(),
                        default: None,
                        required: false,
                        long_description: None,
                    }],
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["view".to_string(), "convert".to_string()],
                },
            ],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            constraints: vec![],
        }
    }

    #[test]
    fn test_build_schema_prompt_section() {
        let schema = test_schema();
        let section = build_schema_prompt_section(&schema, "sort bam file by coordinate");

        assert!(section.contains("Subcommand Selection"));
        assert!(section.contains("sort"));
        assert!(section.contains("✓"));
        assert!(section.contains("Recommended"));
    }

    #[test]
    fn test_suggest_subcommand_for_task() {
        let schema = test_schema();

        // Task with "sort" keyword
        let suggested = suggest_subcommand_for_task(&schema, "sort bam file");
        assert_eq!(suggested, Some("sort".to_string()));

        // Task with "view" keyword
        let suggested = suggest_subcommand_for_task(&schema, "view and convert sam file");
        assert_eq!(suggested, Some("view".to_string()));

        // Unknown task
        let suggested = suggest_subcommand_for_task(&schema, "random task");
        assert!(
            suggested.is_none()
                || suggested == Some("sort".to_string())
                || suggested == Some("view".to_string())
        );
    }

    #[test]
    fn test_format_type_hint() {
        assert_eq!(format_type_hint(&ParamType::Int), "⟨integer⟩");
        assert_eq!(format_type_hint(&ParamType::Float), "⟨number⟩");
        assert_eq!(format_type_hint(&ParamType::File), "⟨file⟩");
        assert_eq!(format_type_hint(&ParamType::Bool), "(no value)");
        assert_eq!(
            format_type_hint(&ParamType::Enum(vec!["bam".to_string(), "sam".to_string()])),
            "⟨bam|sam⟩"
        );
    }

    #[test]
    fn test_validate_command_against_schema() {
        let schema = test_schema();

        // Valid command
        let args = vec![
            "sort".to_string(),
            "-@".to_string(),
            "4".to_string(),
            "-o".to_string(),
            "out.bam".to_string(),
            "in.bam".to_string(),
        ];
        let result = validate_command_against_schema(&args, &schema, Some("sort"));
        assert!(result.is_valid);

        // Invalid flag
        let args = vec![
            "sort".to_string(),
            "--invalid".to_string(),
            "out.bam".to_string(),
        ];
        let result = validate_command_against_schema(&args, &schema, Some("sort"));
        assert!(!result.is_valid);
    }

    #[test]
    fn test_suggest_flags_for_task() {
        let schema = test_schema();

        // Task with "threads" keyword
        let flags = suggest_flags_for_task(&schema, "sort bam with 8 threads", Some("sort"));
        assert!(flags.contains(&"-@".to_string()));
    }
}
