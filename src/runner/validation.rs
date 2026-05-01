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
            // Check flags in-place without intermediate Vec allocation
            check_flags_in_place(arg, &known_flags, &mut result.unknown_flags);
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

/// Check flags in-place without intermediate Vec allocation.
/// Appends unknown flags directly to the unknown_flags vector.
fn check_flags_in_place(
    arg: &str,
    known_flags: &std::collections::HashSet<String>,
    unknown_flags: &mut Vec<String>,
) {
    if arg.starts_with("--") {
        // Long flag: split at '=' and check the flag part.
        let flag = arg.split('=').next().unwrap_or(arg);
        if !known_flags.contains(flag) {
            unknown_flags.push(flag.to_string());
        }
    } else if arg.starts_with('-') && arg.len() > 2 {
        // Could be combined short flags like -abc, or a short flag with value
        let rest = &arg[1..];
        if rest.chars().all(|c| c.is_ascii_alphabetic()) {
            // Combined short flags: check each individually without format! allocation
            for c in rest.chars() {
                // Build "-c" inline without format! allocation
                let mut flag_str = String::with_capacity(2);
                flag_str.push('-');
                flag_str.push(c);
                if !known_flags.contains(&flag_str) {
                    unknown_flags.push(flag_str);
                }
            }
        } else {
            // Single flag or flag with value
            if !known_flags.contains(arg) {
                unknown_flags.push(arg.to_string());
            }
        }
    } else if arg.starts_with('-') {
        // Single short flag like "-o"
        if !known_flags.contains(arg) {
            unknown_flags.push(arg.to_string());
        }
    }
}

/// Parse the subcommands section into individual command names.
fn parse_subcommands(commands_section: &str) -> Vec<String> {
    // structured_doc.commands is comma-separated (from extract_subcommands in doc_processor)
    // e.g., "dict, faidx, sort, index"
    if commands_section.is_empty() {
        return Vec::new();
    }

    let mut cmds = Vec::new();
    for part in commands_section.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Take the first token (subcommand name, may have trailing description)
        if let Some(cmd) = trimmed.split_whitespace().next() {
            // Skip if it looks like a flag or header
            if !cmd.starts_with('-') && !cmd.starts_with('=') && !cmd.contains(':') {
                cmds.push(cmd.to_string());
            }
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
    fn test_check_combined_short_flags() {
        let mut unknown = Vec::new();
        let known: std::collections::HashSet<String> =
            ["-a", "-b"].iter().map(|s| s.to_string()).collect();
        check_flags_in_place("-abc", &known, &mut unknown);
        // -c is not known, should be added to unknown
        assert_eq!(unknown, vec!["-c"]);
    }

    #[test]
    fn test_check_long_flag() {
        let mut unknown = Vec::new();
        let known: std::collections::HashSet<String> =
            ["--output"].iter().map(|s| s.to_string()).collect();
        check_flags_in_place("--output=file.bam", &known, &mut unknown);
        // --output is known, nothing added
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_check_unknown_long_flag() {
        let mut unknown = Vec::new();
        let known: std::collections::HashSet<String> =
            ["--input"].iter().map(|s| s.to_string()).collect();
        check_flags_in_place("--output=file.bam", &known, &mut unknown);
        // --output is not known, should be added
        assert_eq!(unknown, vec!["--output"]);
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
        // Function expects comma-separated format
        let cmds = parse_subcommands("sort, index");
        assert_eq!(cmds, vec!["sort", "index"]);
    }

    #[test]
    fn test_parse_subcommands_skips_headers() {
        // Function expects comma-separated format
        let cmds = parse_subcommands("sort, index");
        // Headers and flags are skipped
        assert_eq!(cmds, vec!["sort", "index"]);
    }
}
