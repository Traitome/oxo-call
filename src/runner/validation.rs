//! Post-generation validation of LLM-generated commands.
//!
//! Validates that generated command arguments are consistent with the tool's
//! documentation:
//! - **Flag validation** (#135): Checks flags against the extracted flag catalog.
//! - **Subcommand validation** (#137): Checks that the first positional argument
//!   is a valid subcommand when the tool is known to use subcommands.
//!
//! Validation is best-effort: warnings are emitted but execution is not blocked,
//! since tools often have flags that aren't captured by `--help` extraction.

use crate::doc_processor::{FlagEntry, StructuredDoc};

/// Result of validating generated arguments against tool documentation.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Flags in the generated command that were NOT found in the doc catalog.
    pub unknown_flags: Vec<String>,
    /// The subcommand used, if any.
    pub subcommand: Option<String>,
    /// Whether the subcommand was found in the doc's command list.
    pub subcommand_valid: Option<bool>,
    /// Human-readable warnings for the user.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Returns true if no issues were detected.
    #[allow(dead_code)]
    pub fn is_clean(&self) -> bool {
        self.unknown_flags.is_empty() && self.subcommand_valid != Some(false)
    }
}

/// Validate generated arguments against the structured documentation.
///
/// # Arguments
/// * `args` - The generated command arguments (as individual tokens).
/// * `structured_doc` - The structured documentation from `DocProcessor`.
///
/// # Returns
/// A `ValidationResult` with any detected issues.
pub fn validate_args(args: &[String], structured_doc: &StructuredDoc) -> ValidationResult {
    let mut result = ValidationResult::default();

    // ── Flag validation ──────────────────────────────────────────────────
    if !structured_doc.flag_catalog.is_empty() {
        let known_flags = build_flag_set(&structured_doc.flag_catalog);
        for arg in args {
            if !arg.starts_with('-') {
                continue;
            }
            // Normalise: split combined short flags like -abc → -a, -b, -c
            // but keep long flags like --output intact.
            let flags_to_check = expand_flags(arg);
            for flag in flags_to_check {
                if !known_flags.contains(flag.as_str()) {
                    result.unknown_flags.push(flag.clone());
                }
            }
        }
        if !result.unknown_flags.is_empty() {
            result.warnings.push(format!(
                "Flag(s) not found in documentation: {}. Verify they are valid.",
                result.unknown_flags.join(", ")
            ));
        }
    }

    // ── Subcommand validation ────────────────────────────────────────────
    if !structured_doc.commands.is_empty() {
        let known_cmds = parse_subcommands(&structured_doc.commands);
        if !known_cmds.is_empty() {
            // The first non-flag argument is the likely subcommand.
            if let Some(first_positional) = args.iter().find(|a| !a.starts_with('-')) {
                result.subcommand = Some(first_positional.clone());
                let valid = known_cmds
                    .iter()
                    .any(|c| c.eq_ignore_ascii_case(first_positional));
                result.subcommand_valid = Some(valid);
                if !valid {
                    result.warnings.push(format!(
                        "Subcommand '{}' not found in documentation. Known subcommands: {}",
                        first_positional,
                        known_cmds.join(", ")
                    ));
                }
            }
        }
    }

    result
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Build a set of known flag names from the flag catalog.
fn build_flag_set(catalog: &[FlagEntry]) -> std::collections::HashSet<String> {
    let mut set = std::collections::HashSet::new();
    for entry in catalog {
        // Flags may be like "-o", "--output", "-@ INT", "--threads INT".
        // We extract the flag portion (before any type/value).
        for part in entry.flag.split([',', ' ', '\t']) {
            let part = part.trim();
            if part.starts_with('-') {
                // Strip trailing `=` if present (e.g., `--output=`)
                let flag = part.trim_end_matches('=');
                set.insert(flag.to_string());
            }
        }
    }
    set
}

/// Expand combined short flags (e.g., `-abc` → `["-a", "-b", "-c"]`).
/// Long flags (`--foo`) are returned as-is.
fn expand_flags(arg: &str) -> Vec<String> {
    if arg.starts_with("--") {
        // Long flag: split at '=' and return the flag part.
        let flag = arg.split('=').next().unwrap_or(arg);
        vec![flag.to_string()]
    } else if arg.starts_with('-') && arg.len() > 2 {
        // Could be combined short flags like -abc, or a short flag with value
        // like -o output.bam. We handle the common case: if all chars after `-`
        // are ASCII letters, treat as combined flags. Otherwise, return as-is.
        let rest = &arg[1..];
        if rest.chars().all(|c| c.is_ascii_alphabetic()) {
            rest.chars().map(|c| format!("-{c}")).collect()
        } else {
            vec![arg.to_string()]
        }
    } else {
        vec![arg.to_string()]
    }
}

/// Parse the subcommands section into individual command names.
fn parse_subcommands(commands_section: &str) -> Vec<String> {
    let mut cmds = Vec::new();
    for line in commands_section.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Each line typically starts with the command name, followed by
        // whitespace and a description.  Take the first token.
        if let Some(cmd) = trimmed.split_whitespace().next() {
            // Skip lines that are clearly headers or separators.
            if cmd.starts_with('-')
                || cmd.starts_with('=')
                || cmd.contains(':')
                || cmd.to_uppercase() == cmd && cmd.len() > 3
            {
                continue;
            }
            cmds.push(cmd.to_string());
        }
    }
    cmds
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_structured_doc(flags: Vec<FlagEntry>, commands: &str) -> StructuredDoc {
        StructuredDoc {
            flag_catalog: flags,
            commands: commands.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_valid_flags_pass() {
        let doc = make_structured_doc(
            vec![
                FlagEntry {
                    flag: "-o".to_string(),
                    description: "output file".to_string(),
                },
                FlagEntry {
                    flag: "-@ INT".to_string(),
                    description: "threads".to_string(),
                },
            ],
            "",
        );
        let args: Vec<String> = vec!["-o", "out.bam", "-@", "8"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = validate_args(&args, &doc);
        assert!(result.is_clean());
        assert!(result.unknown_flags.is_empty());
    }

    #[test]
    fn test_unknown_flag_detected() {
        let doc = make_structured_doc(
            vec![FlagEntry {
                flag: "-o".to_string(),
                description: "output".to_string(),
            }],
            "",
        );
        let args: Vec<String> = vec!["-o", "out.bam", "--nonexistent"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = validate_args(&args, &doc);
        assert!(!result.is_clean());
        assert!(result.unknown_flags.contains(&"--nonexistent".to_string()));
    }

    #[test]
    fn test_valid_subcommand() {
        let doc = make_structured_doc(
            vec![],
            "sort      Sort BAM file\nindex     Create index\nview      View BAM",
        );
        let args: Vec<String> = vec!["sort", "-o", "out.bam", "in.bam"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = validate_args(&args, &doc);
        assert_eq!(result.subcommand, Some("sort".to_string()));
        assert_eq!(result.subcommand_valid, Some(true));
    }

    #[test]
    fn test_invalid_subcommand() {
        let doc = make_structured_doc(vec![], "sort      Sort BAM file\nindex     Create index");
        let args: Vec<String> = vec!["frobnicate", "-o", "out.bam"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = validate_args(&args, &doc);
        assert_eq!(result.subcommand_valid, Some(false));
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_expand_combined_short_flags() {
        let expanded = expand_flags("-abc");
        assert_eq!(expanded, vec!["-a", "-b", "-c"]);
    }

    #[test]
    fn test_expand_long_flag() {
        let expanded = expand_flags("--output=file.bam");
        assert_eq!(expanded, vec!["--output"]);
    }

    #[test]
    fn test_no_validation_without_catalog() {
        let doc = make_structured_doc(vec![], "");
        let args: Vec<String> = vec!["--whatever", "-xyz"]
            .into_iter()
            .map(String::from)
            .collect();
        let result = validate_args(&args, &doc);
        assert!(result.is_clean());
    }

    #[test]
    fn test_parse_subcommands() {
        let cmds = parse_subcommands("sort    Sort BAM\nindex   Create index\n\n");
        assert_eq!(cmds, vec!["sort", "index"]);
    }

    #[test]
    fn test_parse_subcommands_skips_headers() {
        let cmds = parse_subcommands("COMMANDS:\nsort    Sort BAM\n--- separator ---\n");
        // "COMMANDS:" is all-caps header (len > 3), should be skipped
        // "---" starts with '-', should be skipped
        assert_eq!(cmds, vec!["sort"]);
    }
}
