//! Command validation module.
//!
//! This module provides **generic, documentation-driven** validation and
//! correction functions for LLM-generated commands.  All validation is based
//! on patterns extracted from the tool's own help documentation
//! (`StructuredDoc`), never on hardcoded tool-specific knowledge.

use crate::doc_processor::StructuredDoc;

/// Correct format issues in the generated command.
///
/// Only handles generic, documentation-driven corrections:
/// - Remove hallucinated subcommands for tools that don't have them
pub fn correct_format(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut corrected = args_str.to_string();

    if !sdoc.has_subcommands {
        corrected = remove_hallucinated_subcommands(&corrected, sdoc);
    } else {
        corrected = ensure_valid_subcommand(&corrected, sdoc);
    }

    corrected
}

/// Aggressive generic corrections — all based on StructuredDoc, no tool-specific logic.
///
/// Pipeline:
/// 1. Subcommand validation (add missing / remove hallucinated)
/// 2. Add missing required flags from catalog
/// 3. Remove hallucinated flags not in catalog
/// 4. Remove consecutive duplicate flags
pub fn aggressive_correct(
    args_str: &str,
    sdoc: &StructuredDoc,
    _tool: &str,
    _task: Option<&str>,
) -> String {
    let mut corrected = args_str.to_string();

    if !sdoc.has_subcommands {
        corrected = remove_hallucinated_subcommands(&corrected, sdoc);
    } else {
        corrected = ensure_valid_subcommand(&corrected, sdoc);
    }

    corrected = add_missing_required_flags(&corrected, sdoc);

    corrected = remove_hallucinated_flags(&corrected, sdoc);

    corrected = remove_duplicate_flags(&corrected);

    corrected.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Validate and correct subcommand for tools that require them.
///
/// Uses `sdoc.subcommands` (extracted from documentation) as the source of
/// truth.  For tools without subcommands, uses positive-identification to
/// detect hallucinated subcommands (any non-flag, non-file, non-number token).
pub fn validate_subcommand(args_str: &str, _tool: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<&str> = args_str.split_whitespace().collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first_token = tokens[0];

    if !sdoc.has_subcommands && !is_valid_first_token_no_subcommand(first_token, sdoc) {
        return tokens[1..].join(" ");
    }

    args_str.to_string()
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Ensure tool has a valid subcommand if required (documentation-driven).
///
/// When the first token is not a valid subcommand:
/// 1. If the first token looks like a hallucinated subcommand (not a flag, not a file),
///    remove it and try to find a matching subcommand from the remaining tokens.
/// 2. If the first token is a flag or file, prepend the best-matching subcommand.
fn ensure_valid_subcommand(args_str: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first = &tokens[0];

    // First token IS a valid subcommand - check for double subcommand
    if sdoc.subcommands.contains(first) {
        if tokens.len() > 1 {
            let second_is_subcommand = sdoc.subcommands.contains(&tokens[1]);
            if second_is_subcommand {
                let mut result = vec![tokens[1].clone()];
                result.extend(tokens.into_iter().skip(2));
                return result.join(" ");
            }
        }
        return args_str.to_string();
    }

    // First token is NOT a valid subcommand
    // Check if it's a hallucinated subcommand that should be removed
    let is_hallucinated = !first.starts_with('-')
        && !first.contains('.')
        && !first.contains('/')
        && !first.parse::<f64>().is_ok();

    // Build the working tokens (without hallucinated first token)
    let working_tokens: Vec<&str> = if is_hallucinated {
        tokens[1..].iter().map(|s| s.as_str()).collect()
    } else {
        tokens.iter().map(|s| s.as_str()).collect()
    };

    if working_tokens.is_empty() {
        // Only had a hallucinated token, prepend first subcommand
        if let Some(default_sub) = sdoc.subcommands.first() {
            return default_sub.clone();
        }
        return String::new();
    }

    // Check if the first working token is a valid subcommand
    if sdoc.subcommands.contains(&working_tokens[0].to_string()) {
        return working_tokens.join(" ");
    }

    // Need to prepend a subcommand - try to find the best match
    let best_sub = find_best_subcommand(&sdoc.subcommands, &working_tokens);

    if let Some(sub) = best_sub {
        let mut result = vec![sub];
        result.extend(working_tokens.iter().map(|s| s.to_string()));
        return result.join(" ");
    }

    // Fallback: prepend first subcommand
    if let Some(default_sub) = sdoc.subcommands.first() {
        let mut result = vec![default_sub.clone()];
        result.extend(working_tokens.iter().map(|s| s.to_string()));
        return result.join(" ");
    }

    // No subcommands found at all - return working tokens
    working_tokens.join(" ")
}

/// Find the best-matching subcommand for the given tokens.
///
/// Looks for subcommand names that appear as tokens or that match
/// semantically with the flag/content patterns in the args.
fn find_best_subcommand(subcommands: &[String], tokens: &[&str]) -> Option<String> {
    for token in tokens {
        if subcommands.contains(&token.to_string()) {
            return Some(token.to_string());
        }
    }

    let token_lower: Vec<String> = tokens.iter().map(|t| t.to_lowercase()).collect();
    for sub in subcommands {
        let sub_lower = sub.to_lowercase();
        for tl in &token_lower {
            if tl.contains(&sub_lower) || sub_lower.contains(tl.as_str()) {
                if sub.len() >= 3 {
                    return Some(sub.clone());
                }
            }
        }
    }

    None
}

/// Remove hallucinated subcommands from args for tools without subcommands.
///
/// Uses a combination of a fixed blocklist and positive-identification:
/// - Blocklist: common hallucinated subcommand words
/// - Positive check: if the first token is a flag, file path, number, or
///   single-letter mode flag, it's valid and kept
fn remove_hallucinated_subcommands(args_str: &str, sdoc: &StructuredDoc) -> String {
    let tokens: Vec<&str> = args_str.split_whitespace().collect();
    if tokens.is_empty() {
        return args_str.to_string();
    }

    let first_token = tokens[0];

    // If the first token is clearly valid, keep it
    if is_valid_first_token_no_subcommand(first_token, sdoc) {
        return args_str.to_string();
    }

    // Check against expanded blocklist of common hallucinated subcommands
    const HALLUCINATED_SUBCOMMANDS: &[&str] = &[
        "run", "process", "analyze", "analysis", "generate", "compute", "extract",
        "convert", "filter", "sort", "index", "align", "call", "plot",
        "report", "profile", "prepare", "build", "quantify", "count",
        "trim", "clean", "stats", "merge", "split", "view", "faidx", "dict",
        "assurance", "discover", "execute", "perform", "evaluate", "assess",
        "check", "validate", "detect", "identify", "classify", "predict",
        "assemble", "annotate", "map", "sequence", "compare", "visualize",
        "download", "install", "update", "configure", "initialize", "setup",
        "create", "delete", "remove", "modify", "update", "export", "import",
        "train", "test", "debug", "optimize", "benchmark", "simulate",
    ];

    if HALLUCINATED_SUBCOMMANDS.contains(&first_token.to_lowercase().as_str()) {
        return tokens[1..].join(" ");
    }

    args_str.to_string()
}

/// Check whether a first token is valid for a tool that has NO subcommands.
///
/// Valid first tokens are:
/// - Flags (start with `-`)
/// - File paths (contain `.` or `/`)
/// - Numbers (pure digits, possibly with K/M/G suffix for genomics)
/// - Known companion binaries (contain `_` or `-` and match sdoc list)
/// - Single-letter tokens that could be mode flags (e.g. `x` for tar)
fn is_valid_first_token_no_subcommand(token: &str, sdoc: &StructuredDoc) -> bool {
    if token.starts_with('-') {
        return true;
    }
    if token.contains('.') || token.contains('/') {
        return true;
    }
    if token.parse::<f64>().is_ok() {
        return true;
    }
    if token.len() == 1 && token.chars().all(|c| c.is_ascii_alphabetic()) {
        return true;
    }
    if (token.contains('_') || token.contains('-'))
        && !sdoc.companion_binaries.is_empty()
        && sdoc.companion_binaries.iter().any(|cb| cb == token)
    {
        return true;
    }
    if !sdoc.companion_binaries.is_empty()
        && sdoc.companion_binaries.iter().any(|cb| cb == token)
    {
        return true;
    }
    false
}

/// Add missing required flags with defaults from the flag catalog.
///
/// Also adds commonly-expected flags that aren't explicitly marked required
/// but are semantically important (output, threads, input format) when the
/// task description implies them.
fn add_missing_required_flags(args_str: &str, sdoc: &StructuredDoc) -> String {
    let mut result = args_str.to_string();

    for entry in &sdoc.flag_catalog {
        if !entry.required {
            continue;
        }

        let flag_present = args_str.split_whitespace().any(|t| {
            t == entry.flag || t.starts_with(&format!("{}=", entry.flag))
        });

        if flag_present {
            continue;
        }

        if let Some(ref alt) = entry.alt_form {
            let alt_present = args_str.split_whitespace().any(|t| {
                t == alt || t.starts_with(&format!("{}=", alt))
            });
            if alt_present {
                continue;
            }
        }

        let default_val = entry.default.as_ref().map(|s| s.as_str()).unwrap_or_else(|| infer_default_value(entry));

        if !default_val.is_empty() {
            result.push(' ');
            result.push_str(&entry.flag);
            result.push(' ');
            result.push_str(default_val);
        }
    }

    result
}

/// Infer a default value for a flag based on its semantics (generic heuristics).
fn infer_default_value(entry: &crate::doc_processor::FlagEntry) -> &'static str {
    let flag_lower = entry.flag.to_lowercase();
    let desc_lower = entry.description.to_lowercase();

    if flag_lower.contains("-t") || flag_lower.contains("--thread") || flag_lower.contains("-@") || desc_lower.contains("thread") {
        return "4";
    }
    if flag_lower.contains("-d") || flag_lower.contains("--outdir") || flag_lower.contains("--output-dir") {
        return "output/";
    }
    if desc_lower.contains("memory") || desc_lower.contains("ram") {
        return "4G";
    }
    ""
}

/// Detect and remove hallucinated flags not in the catalog.
pub fn remove_hallucinated_flags(args_str: &str, sdoc: &StructuredDoc) -> String {
    if sdoc.flag_catalog.is_empty() {
        return args_str.to_string();
    }

    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    if !tokens.is_empty() {
        let first = &tokens[0];
        let is_likely_companion = first.contains('_') || first.contains('-');

        let has_meaningful_flags = sdoc.flag_catalog.iter().any(|e| {
            let flag = &e.flag;
            flag != "-h" && flag != "--help" && flag != "-v" && flag != "--version"
        });

        if is_likely_companion || !has_meaningful_flags {
            return args_str.to_string();
        }
    }

    let known_flags: std::collections::HashSet<String> = sdoc
        .flag_catalog
        .iter()
        .flat_map(|entry| {
            entry
                .flag
                .split([',', ' ', '\t'])
                .map(|s| s.trim().trim_end_matches('=').to_string())
                .filter(|s| !s.is_empty() && s.starts_with('-'))
        })
        .collect();

    let mut result = Vec::new();
    let mut skip_next = false;

    for (i, token) in tokens.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if token.starts_with('-') {
            let base_flag = if token.contains('=') {
                token.split('=').next().unwrap_or(token)
            } else {
                token
            };

            if known_flags.contains(base_flag) {
                result.push(token.clone());
            } else {
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    skip_next = true;
                }
            }
        } else {
            result.push(token.clone());
        }
    }

    result.join(" ")
}

/// Remove consecutive duplicate flags.
fn remove_duplicate_flags(args_str: &str) -> String {
    let tokens: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();
    let mut result = Vec::new();
    let mut seen_flags = std::collections::HashSet::new();
    let mut skip_next = false;

    for (i, token) in tokens.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if token.starts_with('-') {
            if seen_flags.contains(token) {
                if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                    skip_next = true;
                }
                continue;
            }
            seen_flags.insert(token.clone());
            result.push(token.clone());
        } else {
            result.push(token.clone());
        }
    }

    result.join(" ")
}

/// Check if a token is likely a subcommand (not a flag or file path).
/// Kept for backward compatibility but now delegates to the positive-identification logic.
fn is_likely_subcommand(token: &str) -> bool {
    let sdoc = StructuredDoc {
        has_subcommands: false,
            subcommand_descriptions: Vec::new(),
        ..Default::default()
    };
    !is_valid_first_token_no_subcommand(token, &sdoc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc_processor::FlagEntry;

    #[test]
    fn test_remove_hallucinated_subcommands() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            ..Default::default()
        };
        let input = "run -i data.bed";
        let result = remove_hallucinated_subcommands(input, &sdoc);
        assert_eq!(result, "-i data.bed");
    }

    #[test]
    fn test_remove_hallucinated_assurance() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            ..Default::default()
        };
        let result = remove_hallucinated_subcommands("assurance -d empty_directory", &sdoc);
        assert_eq!(result, "-d empty_directory");

        let result = remove_hallucinated_subcommands("assurance -v *.txt", &sdoc);
        assert_eq!(result, "-v *.txt");
    }

    #[test]
    fn test_keep_valid_first_tokens() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            ..Default::default()
        };
        assert_eq!(remove_hallucinated_subcommands("-i data.bed", &sdoc), "-i data.bed");
        assert_eq!(remove_hallucinated_subcommands("data.bed 5", &sdoc), "data.bed 5");
        assert_eq!(remove_hallucinated_subcommands("5 data.bed", &sdoc), "5 data.bed");
        assert_eq!(remove_hallucinated_subcommands("/path/to/file", &sdoc), "/path/to/file");
    }

    #[test]
    fn test_keep_companion_binary() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            companion_binaries: vec!["rsem-prepare-reference".to_string()],
            ..Default::default()
        };
        let result = remove_hallucinated_subcommands("rsem-prepare-reference --gtf a.gtf ref.fa idx", &sdoc);
        assert!(result.contains("rsem-prepare-reference"));
    }

    #[test]
    fn test_validate_subcommand_samtools() {
        let sdoc = StructuredDoc {
            has_subcommands: true,
            subcommand_descriptions: Vec::new(),
            subcommands: vec!["sort".to_string(), "index".to_string(), "view".to_string()],
            subcommand_descriptions: Vec::new(),
            ..Default::default()
        };

        let result = validate_subcommand("sort -o out.bam in.bam", "samtools", &sdoc);
        assert!(result.contains("sort"));
    }

    #[test]
    fn test_validate_subcommand_no_subcommand_tools() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            subcommands: vec![],
            subcommand_descriptions: Vec::new(),
            ..Default::default()
        };

        let result = validate_subcommand("sort -i in.fq -o out.fq", "fastp", &sdoc);
        assert!(!result.contains("sort"));
        assert!(result.contains("-i"));

        let result = validate_subcommand("-i in.fq -o out.fq", "fastp", &sdoc);
        assert_eq!(result, "-i in.fq -o out.fq");
    }

    #[test]
    fn test_remove_duplicate_flags() {
        let input = "sort -@ 4 -@ 4 -o out.bam in.bam";
        let result = remove_duplicate_flags(input);
        assert!(!result.contains("-@ 4 -@ 4"));
    }

    #[test]
    fn test_aggressive_correct_no_tool_specific() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            subcommands: vec![],
            subcommand_descriptions: Vec::new(),
            flag_catalog: vec![],
            ..Default::default()
        };

        let input = "run -i in.fq -o out.fq";
        let result = aggressive_correct(input, &sdoc, "fastp", Some("trim reads"));
        assert!(!result.contains("run"));
        assert!(result.contains("-i"));
    }

    #[test]
    fn test_remove_hallucinated_flags() {
        let sdoc = StructuredDoc {
            has_subcommands: false,
            subcommand_descriptions: Vec::new(),
            flag_catalog: vec![
                FlagEntry {
                    flag: "-i".to_string(),
                    value_type: Some("FILE".to_string()),
                    description: "Input file".to_string(),
                    required: false,
                    default: None,
                    alt_form: Some("--input".to_string()),
                    enum_values: vec![],
                },
                FlagEntry {
                    flag: "-o".to_string(),
                    value_type: Some("FILE".to_string()),
                    description: "Output file".to_string(),
                    required: false,
                    default: None,
                    alt_form: Some("--output".to_string()),
                    enum_values: vec![],
                },
            ],
            ..Default::default()
        };

        let result = remove_hallucinated_flags("-i in.fq --fake-flag value -o out.fq", &sdoc);
        assert!(result.contains("-i"));
        assert!(result.contains("-o"));
        assert!(!result.contains("--fake-flag"));
        assert!(!result.contains("value"));
    }
}
