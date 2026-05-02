//! Generic CLI Help Parser
//!
//! A fallback parser for CLI help output that doesn't match any specialized
//! parser format. Uses regex patterns to extract common CLI structures.
//!
//! ## Supported Patterns
//!
//! - GNU style: `--flag=value`, `-f value`, `-fvalue`
//! - Unix style: `-f`, `--flag`
//! - Short help: one-line per flag with tab-separated description
//! - Long help: multi-line with indented descriptions

use super::HelpParser;
use crate::schema::{
    CliSchema, CliStyle, FlagSchema, ParamType, PositionalSchema, SubcommandSchema,
};
use regex::Regex;

/// Generic help parser using regex patterns
pub struct GenericParser;

impl HelpParser for GenericParser {
    fn name(&self) -> &str {
        "generic"
    }

    fn can_parse(&self, _help: &str) -> bool {
        // Generic parser can always attempt to parse
        true
    }

    fn parse(&self, tool: &str, help: &str) -> CliSchema {
        let cli_style = CliStyle::detect_from_help(help);

        let mut schema = CliSchema::minimal(tool, cli_style);
        schema.schema_source = self.name().to_string();
        schema.usage_summary = extract_usage_generic(help);

        // Parse flags using multiple regex patterns
        schema.flags = parse_flags_generic(help);

        // Parse positional arguments
        schema.positionals = parse_positionals_generic(help);

        // Parse subcommands if present
        schema.subcommands = parse_subcommands_generic(help);

        // Calculate doc quality
        schema.doc_quality = calculate_doc_quality_generic(&schema);

        schema
    }
}

/// Extract usage line using generic patterns
fn extract_usage_generic(help: &str) -> String {
    // Match various usage patterns
    let patterns = [
        r"^usage:\s+(.+)$",
        r"^Usage:\s+(.+)$",
        r"^USAGE:\s+(.+)$",
        r"^SYNOPSIS\s+(.+)$",
        r"^\s*tool\s+\[.*\].*$", // Generic tool [options] pattern
    ];

    for line in help.lines() {
        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern)
                && let Some(caps) = re.captures(line)
            {
                if let Some(m) = caps.get(1) {
                    return m.as_str().trim().to_string();
                } else {
                    return line.trim().to_string();
                }
            }
        }
    }

    String::new()
}

/// Parse flags using generic regex patterns
fn parse_flags_generic(help: &str) -> Vec<FlagSchema> {
    let mut flags = Vec::new();
    let mut seen_flags: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Pattern 1: --flag=VALUE or --flag VALUE
    // Pattern 2: -f VALUE, -fVALUE
    // Pattern 3: -f, --flag (combined short and long)
    // Pattern 4: Tab-separated: -f\tDescription

    let patterns = [
        // Short and long combined: -f, --flag VALUE (must be BEFORE single --flag)
        // (?m) enables multiline mode so ^ matches start of each line
        // Short and long combined: supports both -f, --flag and -f/--flag formats
        r"(?m)^(-[a-zA-Z0-9@])[,/]\s*--([a-zA-Z0-9_-]+)\s+([^\s]+)(?:\s+(.+))?",
        // Short and long bool: supports both -f, --flag and -f/--flag formats
        r"(?m)^(-[a-zA-Z0-9@])[,/]\s*--([a-zA-Z0-9_-]+)(?:\s+(.+))?",
        // Long flag with value: --output=FILE, --output=none, --output FILE
        r"(?m)^(--[a-zA-Z0-9_-]+)[=\s]+([^\s]+)(?:\s+(.+))?",
        // Single long flag: --flag
        r"(?m)^(--[a-zA-Z0-9_-]+)(?:\s+(.+))?",
        // Multi-letter short flag with description: -GL description, -doMaf description
        // Must come before single-letter patterns to avoid -G matching before -GL
        r"(?m)^(-[a-zA-Z][a-zA-Z0-9_]+)\s+(.+)",
        // Single short flag with value: -f VALUE, -1 <mates1>
        r"(?m)^(-[a-zA-Z0-9@])\s+([^\s]+)(?:\s+(.+))?",
        // Short flag with attached value: -jX, -a=, -C=X, -B[X]
        r#"(?m)^(-[a-zA-Z])([=A-Za-z0-9_<>'"\[\]|]+)(?:\s+(.+))?"#,
        // Tab-separated format
        r"(?m)^(-[a-zA-Z0-9@])\t(.+)",
        r"(?m)^(--[a-zA-Z0-9_-]+)\t(.+)",
    ];

    for line in help.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Only process lines that look like flag definitions (start with - or --).
        // This prevents matching hyphens inside prose text like "file-for-file" or "paired-end".
        if !line.starts_with('-') {
            continue;
        }

        for (i, pattern) in patterns.iter().enumerate() {
            if let Ok(re) = Regex::new(pattern)
                && let Some(caps) = re.captures(line)
            {
                if let Some(flag) = extract_flag_from_caps(i, &caps) {
                    // Avoid duplicates - check both primary name and all aliases
                    let key = flag.name.clone();
                    if !seen_flags.contains(&key) {
                        seen_flags.insert(key);
                        for alias in &flag.aliases {
                            seen_flags.insert(alias.clone());
                        }
                        flags.push(flag);
                    }
                }
                break; // Only use first matching pattern
            }
        }
    }

    flags
}

/// Ensure a short flag has the `-` prefix
fn ensure_short_prefix(s: &str) -> String {
    if s.starts_with('-') {
        s.to_string()
    } else {
        format!("-{}", s)
    }
}

/// Ensure a long flag has the `--` prefix
fn ensure_long_prefix(s: &str) -> String {
    if s.starts_with("--") {
        s.to_string()
    } else {
        format!("--{}", s)
    }
}

/// Extract FlagSchema from regex captures based on pattern index
fn extract_flag_from_caps(pattern_idx: usize, caps: &regex::Captures) -> Option<FlagSchema> {
    match pattern_idx {
        // Pattern 0: -f, --flag VALUE
        0 => {
            let short = ensure_short_prefix(caps.get(1)?.as_str());
            let long = ensure_long_prefix(caps.get(2)?.as_str());
            let placeholder = caps.get(3)?.as_str();
            let description = caps
                .get(4)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let param_type = infer_type_generic(placeholder);

            Some(FlagSchema {
                name: long,
                aliases: vec![short],
                param_type,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 1: -f, --flag (bool)
        1 => {
            let short = ensure_short_prefix(caps.get(1)?.as_str());
            let long = ensure_long_prefix(caps.get(2)?.as_str());
            let description = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Some(FlagSchema {
                name: long,
                aliases: vec![short],
                param_type: ParamType::Bool,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 2: --flag=VALUE
        2 => {
            let name = ensure_long_prefix(caps.get(1)?.as_str());
            let placeholder = caps.get(2)?.as_str();
            let description = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let param_type = infer_type_generic(placeholder);

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 3: --flag
        3 => {
            let name = ensure_long_prefix(caps.get(1)?.as_str());
            let description = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type: ParamType::Bool,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 4: -GL description (multi-letter short flag like angsd)
        4 => {
            let name = ensure_short_prefix(caps.get(1)?.as_str());
            let description = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type: ParamType::Bool,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 5: -f VALUE
        5 => {
            let name = ensure_short_prefix(caps.get(1)?.as_str());
            let placeholder = caps.get(2)?.as_str();
            let description = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            let param_type = infer_type_generic(placeholder);

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 6: -fVALUE (attached value, no space)
        6 => {
            let name = ensure_short_prefix(caps.get(1)?.as_str());
            let _attached = caps.get(2)?.as_str();
            let description = caps
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();
            // The attached value indicates this flag takes a parameter
            let param_type = infer_type_generic(_attached);

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        // Pattern 7, 8: Tab-separated
        7 | 8 => {
            let name = ensure_short_prefix(caps.get(1)?.as_str());
            let description = caps
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_default();

            Some(FlagSchema {
                name,
                aliases: Vec::new(),
                param_type: ParamType::Bool,
                description,
                default: None,
                required: false,
                long_description: None,
            })
        }
        _ => None,
    }
}

/// Infer parameter type from placeholder name (generic version)
fn infer_type_generic(placeholder: &str) -> ParamType {
    let p = placeholder.to_lowercase();

    // File indicators
    if p.contains("file")
        || p.ends_with(".bam")
        || p.ends_with(".fa")
        || p.ends_with(".fq")
        || p.ends_with(".fasta")
        || p.ends_with(".fastq")
        || p.ends_with(".vcf")
        || p.ends_with(".gz")
        || p.contains("path")
        || p.contains("dir")
        || p.contains("output")
        || p.contains("input")
    {
        return ParamType::File;
    }

    // Integer indicators
    if p.contains("int")
        || p.contains("num")
        || p.contains("count")
        || p.contains("size")
        || p.contains("length")
        || p.contains("threads")
        || p.contains("cpu")
        || p == "n"
        || p == "k"
        || p == "m"
    {
        return ParamType::Int;
    }

    // Float indicators
    if p.contains("float")
        || p.contains("double")
        || p.contains("threshold")
        || p.contains("prob")
        || p.contains("rate")
        || p.contains("ratio")
        || p.contains("score")
        || p.contains("pvalue")
        || p.contains("evalue")
    {
        return ParamType::Float;
    }

    // String (default for non-bool placeholders)
    ParamType::String
}

/// Parse positional arguments using generic patterns
fn parse_positionals_generic(help: &str) -> Vec<PositionalSchema> {
    let mut positionals = Vec::new();
    let mut position: usize = 0;

    // Look for lines that look like positional args:
    // - No leading dash
    // - Typically uppercase or descriptive
    // - Often near "Arguments:" or "Parameters:" section

    let _in_args_section = false;

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Detect argument section start
        if line_lower.contains("arguments:")
            || line_lower.contains("parameters:")
            || line_lower.contains("inputs:")
        {
            // Mark section start - would need state tracking
            continue;
        }

        // Skip if line starts with flag indicator
        let trimmed = line.trim();
        if trimmed.starts_with('-')
            || trimmed.starts_with("usage")
            || trimmed.starts_with("options")
            || trimmed.is_empty()
        {
            continue;
        }

        // Try to parse as positional argument
        // Format: NAME description or NAME
        if let Some(pos) = parse_positional_line(trimmed, position) {
            positionals.push(pos);
            position += 1;
        }
    }

    positionals
}

/// Parse a single positional argument line
fn parse_positional_line(line: &str, position: usize) -> Option<PositionalSchema> {
    // Skip if doesn't look like a positional
    if line.is_empty() || line.starts_with('-') {
        return None;
    }

    // Split on double space or tab
    let parts: Vec<&str> = if line.contains("  ") {
        line.splitn(2, "  ").collect()
    } else if line.contains('\t') {
        line.splitn(2, '\t').collect()
    } else {
        vec![line]
    };

    let name = parts[0].trim();
    if name.is_empty() {
        return None;
    }

    // Name should be alphanumeric with possible underscores
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return None;
    }

    let description = if parts.len() > 1 {
        parts[1].trim().to_string()
    } else {
        String::new()
    };

    let param_type = infer_type_generic(name);

    Some(PositionalSchema {
        position,
        name: name.to_string(),
        param_type,
        description,
        required: true,
        default: None,
    })
}

/// Parse subcommands using generic patterns
fn parse_subcommands_generic(help: &str) -> Vec<SubcommandSchema> {
    let mut subcommands = Vec::new();
    let mut in_commands_section = false;
    let mut section_depth = 0;

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Detect commands section
        if line_lower.contains("commands:")
            || line_lower.contains("subcommands:")
            || line_lower.contains("available commands:")
            || line_lower == "commands"
            || line_lower == "subcommands"
        {
            in_commands_section = true;
            section_depth = 0;
            continue;
        }

        // End section
        if in_commands_section {
            // Empty line might end the section
            let trimmed = line.trim();
            if trimmed.is_empty() {
                section_depth += 1;
                if section_depth >= 2 {
                    in_commands_section = false;
                }
                continue;
            }
            section_depth = 0;

            // New section header ends commands
            if trimmed.starts_with("usage:")
                || line_lower.contains("options:")
                || line_lower.contains("arguments:")
                || line_lower.contains("description:")
                || line_lower.contains("examples:")
            {
                in_commands_section = false;
                continue;
            }

            // Skip flags
            if trimmed.starts_with('-') {
                continue;
            }

            // Parse command line - multiple formats:
            // 1. "  sort    Sort alignments by coordinate"
            // 2. "sort - Sort BAM file"
            // 3. "    index          Index a BAM file"
            let parts: Vec<&str> = if trimmed.contains("  ") {
                trimmed.splitn(2, "  ").collect()
            } else if trimmed.contains('\t') {
                trimmed.splitn(2, '\t').collect()
            } else if trimmed.contains(" - ") {
                trimmed.splitn(2, " - ").collect()
            } else {
                vec![trimmed]
            };

            let name = parts[0].trim().trim_end_matches(':');
            if name.is_empty() {
                continue;
            }

            // Name should be a valid command name
            if !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                continue;
            }

            // Skip very short names or common non-command words
            if name.len() < 2 || ["the", "and", "for", "use", "see", "all"].contains(&name) {
                continue;
            }

            let description = if parts.len() > 1 {
                parts[1].trim().trim_start_matches('-').trim().to_string()
            } else {
                String::new()
            };

            let task_keywords = extract_task_keywords_generic(&description);

            subcommands.push(SubcommandSchema {
                name: name.to_string(),
                description,
                usage_pattern: String::new(),
                flags: Vec::new(),
                positionals: Vec::new(),
                constraints: Vec::new(),
                task_keywords,
            });
        }
    }

    // If no subcommands found from "Commands:" section, try to extract from usage patterns
    if subcommands.is_empty() {
        subcommands = extract_subcommands_from_usage(help);
    }

    subcommands
}

/// Extract subcommands from usage patterns like "tool <command>" or "tool COMMAND"
fn extract_subcommands_from_usage(help: &str) -> Vec<SubcommandSchema> {
    let mut subcommands = Vec::new();

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Look for usage lines that suggest subcommand pattern
        if line_lower.starts_with("usage:")
            && (line_lower.contains("<command>")
                || line_lower.contains("<subcommand>")
                || line_lower.contains("<cmd>")
                || line_lower.contains("{")
                || line_lower.contains("command"))
        {
            // Extract subcommands from {cmd1,cmd2,...} pattern
            if let Some(start) = line.find('{')
                && let Some(end) = line[start..].find('}')
            {
                let inner = &line[start + 1..start + end];
                for name in inner.split(',') {
                    let name = name.trim();
                    if !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    {
                        let task_keywords = vec![name.to_lowercase()];
                        subcommands.push(SubcommandSchema {
                            name: name.to_string(),
                            description: String::new(),
                            usage_pattern: String::new(),
                            flags: Vec::new(),
                            positionals: Vec::new(),
                            constraints: Vec::new(),
                            task_keywords,
                        });
                    }
                }
            }
        }
    }

    subcommands
}

/// Extract task keywords from description
fn extract_task_keywords_generic(description: &str) -> Vec<String> {
    let mut keywords: Vec<String> = description
        .split_whitespace()
        .filter(|w| w.len() >= 3 && !w.starts_with('-') && !w.chars().all(|c| c.is_numeric()))
        .take(5)
        .map(|w| w.to_lowercase())
        .collect();

    let bio_verbs = [
        "sort",
        "index",
        "view",
        "merge",
        "convert",
        "extract",
        "filter",
        "count",
        "quantify",
        "align",
        "map",
        "assemble",
        "call",
        "annotate",
        "compare",
        "split",
        "join",
        "stats",
        "flagstat",
        "depth",
        "coverage",
        "mpileup",
        "cat",
        "collate",
        "reheader",
        "calmd",
        "fixmate",
        "markdup",
        "quickcheck",
        "reference",
        "faidx",
        "dict",
    ];
    for verb in bio_verbs {
        if description.to_lowercase().contains(verb) && !keywords.contains(&verb.to_string()) {
            keywords.push(verb.to_string());
        }
    }

    keywords
}

/// Calculate documentation quality score
fn calculate_doc_quality_generic(schema: &CliSchema) -> f32 {
    let mut score = 0.0;

    // Usage summary presence
    if !schema.usage_summary.is_empty() {
        score += 0.15;
    }

    // Flags coverage
    score += (schema.flags.len().min(15) as f32) * 0.03;

    // Positionals coverage
    score += (schema.positionals.len().min(5) as f32) * 0.04;

    // Subcommands for subcommand-style tools
    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        score += 0.15;
    }

    // Flags with descriptions ratio
    if !schema.flags.is_empty() {
        let desc_ratio = schema
            .flags
            .iter()
            .filter(|f| !f.description.is_empty())
            .count() as f32
            / schema.flags.len() as f32;
        score += desc_ratio * 0.2;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_parser_basic() {
        let help = "usage: tool [options]\n\nOptions:\n  -h, --help    Show help\n  -o FILE       Output file\n";
        let parser = GenericParser;

        assert!(parser.can_parse(help));

        let schema = parser.parse("tool", help);
        assert!(!schema.usage_summary.is_empty());
        assert!(schema.flags.len() >= 1);
    }

    #[test]
    fn test_extract_flag_long_with_value() {
        let line = "--output=FILE    Output file path";
        let patterns = [r"--([a-zA-Z0-9_-]+)[=\s]+([^\s]+)(?:\s+(.+))?"];

        if let Ok(re) = Regex::new(&patterns[0]) {
            if let Some(caps) = re.captures(line) {
                let flag = extract_flag_from_caps(2, &caps); // pattern 2 is now --flag=VALUE
                assert!(flag.is_some());
                let f = flag.unwrap();
                assert_eq!(f.name, "--output");
                assert_eq!(f.param_type, ParamType::File);
            }
        }
    }

    #[test]
    fn test_extract_flag_short_long_combined() {
        let line = "-v, --verbose    Enable verbose output";
        let pattern = r"-([a-zA-Z]),\s+--([a-zA-Z0-9_-]+)(?:\s+(.+))?";

        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(line) {
                let flag = extract_flag_from_caps(1, &caps); // pattern 1 is now -f, --flag (bool)
                assert!(flag.is_some());
                let f = flag.unwrap();
                assert_eq!(f.name, "--verbose");
                assert!(f.aliases.contains(&"-v".to_string()));
                assert_eq!(f.param_type, ParamType::Bool);
            }
        }
    }

    #[test]
    fn test_infer_type_generic() {
        assert_eq!(infer_type_generic("FILE"), ParamType::File);
        assert_eq!(infer_type_generic("OUTPUT.bam"), ParamType::File);
        assert_eq!(infer_type_generic("N"), ParamType::Int);
        assert_eq!(infer_type_generic("THREADS"), ParamType::Int);
        assert_eq!(infer_type_generic("THRESHOLD"), ParamType::Float);
        assert_eq!(infer_type_generic("NAME"), ParamType::String);
    }

    #[test]
    fn test_parse_positionals_generic() {
        let help = "Arguments:\n  input_file    Input file\n  output_file   Output file\n";
        let positionals = parse_positionals_generic(help);
        assert!(positionals.len() >= 2);
    }

    #[test]
    fn test_parse_subcommands_generic() {
        let help = "Commands:\n  sort    Sort the file\n  merge   Merge files\n";
        let subcommands = parse_subcommands_generic(help);
        assert!(subcommands.len() >= 2);
    }

    #[test]
    fn test_generic_parser_name() {
        let parser = GenericParser;
        assert_eq!(parser.name(), "generic");
    }

    #[test]
    fn test_generic_parser_can_always_parse() {
        let parser = GenericParser;
        assert!(parser.can_parse(""));
        assert!(parser.can_parse("anything"));
    }

    #[test]
    fn test_extract_usage_various_formats() {
        let help1 = "Usage: mytool [options] INPUT\nOptions:\n  -h  Help\n";
        assert!(!extract_usage_generic(help1).is_empty());

        let help2 = "usage: tool [options]\n";
        assert!(!extract_usage_generic(help2).is_empty());

        let help3 = "USAGE: tool INPUT OUTPUT\n";
        assert!(!extract_usage_generic(help3).is_empty());
    }

    #[test]
    fn test_extract_usage_no_match() {
        let help = "Some random text\nNo usage here\n";
        assert!(extract_usage_generic(help).is_empty());
    }

    #[test]
    fn test_parse_flags_long_flag_only() {
        let help = "Options:\n  --verbose    Enable verbose mode\n  --quiet      Suppress output\n";
        let flags = parse_flags_generic(help);
        assert!(flags.len() >= 2);
        assert!(flags.iter().any(|f| f.name == "--verbose"));
        assert!(flags.iter().any(|f| f.name == "--quiet"));
    }

    #[test]
    fn test_parse_flags_short_with_value() {
        let help = "Options:\n  -t N      Number of threads\n  -o FILE   Output file\n";
        let flags = parse_flags_generic(help);
        assert!(flags.iter().any(|f| f.name == "-t"));
        assert!(flags.iter().any(|f| f.name == "-o"));
    }

    #[test]
    fn test_parse_flags_tab_separated() {
        let help = "Options:\n-v\tVerbose mode\n--output\tOutput file\n";
        let flags = parse_flags_generic(help);
        assert!(flags.len() >= 1);
    }

    #[test]
    fn test_parse_flags_empty_help() {
        let flags = parse_flags_generic("");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_infer_type_generic_file_variants() {
        assert_eq!(infer_type_generic("INPUT.bam"), ParamType::File);
        assert_eq!(infer_type_generic("REF.fa"), ParamType::File);
        assert_eq!(infer_type_generic("READS.fq"), ParamType::File);
        assert_eq!(infer_type_generic("VARIANTS.vcf"), ParamType::File);
        assert_eq!(infer_type_generic("DATA.gz"), ParamType::File);
        assert_eq!(infer_type_generic("PATH"), ParamType::File);
        assert_eq!(infer_type_generic("DIR"), ParamType::File);
        assert_eq!(infer_type_generic("OUTPUT_FILE"), ParamType::File);
        assert_eq!(infer_type_generic("INPUT_PATH"), ParamType::File);
    }

    #[test]
    fn test_infer_type_generic_int_variants() {
        assert_eq!(infer_type_generic("INT"), ParamType::Int);
        assert_eq!(infer_type_generic("NUM"), ParamType::Int);
        assert_eq!(infer_type_generic("COUNT"), ParamType::Int);
        assert_eq!(infer_type_generic("SIZE"), ParamType::Int);
        assert_eq!(infer_type_generic("LENGTH"), ParamType::Int);
        assert_eq!(infer_type_generic("THREADS"), ParamType::Int);
        assert_eq!(infer_type_generic("CPU"), ParamType::Int);
        assert_eq!(infer_type_generic("K"), ParamType::Int);
        assert_eq!(infer_type_generic("M"), ParamType::Int);
    }

    #[test]
    fn test_infer_type_generic_float_variants() {
        assert_eq!(infer_type_generic("FLOAT"), ParamType::Float);
        assert_eq!(infer_type_generic("THRESHOLD"), ParamType::Float);
        assert_eq!(infer_type_generic("PROB"), ParamType::Float);
        assert_eq!(infer_type_generic("RATE"), ParamType::Float);
        assert_eq!(infer_type_generic("RATIO"), ParamType::Float);
        assert_eq!(infer_type_generic("SCORE"), ParamType::Float);
        assert_eq!(infer_type_generic("PVALUE"), ParamType::Float);
        assert_eq!(infer_type_generic("EVALUE"), ParamType::Float);
    }

    #[test]
    fn test_infer_type_generic_string() {
        assert_eq!(infer_type_generic("NAME"), ParamType::String);
        assert_eq!(infer_type_generic("PREFIX"), ParamType::String);
        assert_eq!(infer_type_generic("FORMAT"), ParamType::String);
    }

    #[test]
    fn test_parse_positional_line_valid() {
        let pos = parse_positional_line("INPUT    Input BAM file", 0);
        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert_eq!(pos.name, "INPUT");
        assert_eq!(pos.position, 0);
    }

    #[test]
    fn test_parse_positional_line_empty() {
        assert!(parse_positional_line("", 0).is_none());
    }

    #[test]
    fn test_parse_positional_line_starts_with_dash() {
        assert!(parse_positional_line("-flag", 0).is_none());
    }

    #[test]
    fn test_parse_positional_line_special_chars() {
        assert!(parse_positional_line("input@file", 0).is_none());
    }

    #[test]
    fn test_parse_positional_line_tab_separated() {
        let pos = parse_positional_line("OUTPUT\tOutput file", 1);
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().name, "OUTPUT");
    }

    #[test]
    fn test_parse_subcommands_from_usage_pattern() {
        let help = "Usage: tool {sort,merge,index} [options]\n";
        let subcmds = parse_subcommands_generic(help);
        assert!(
            subcmds.len() >= 3,
            "expected 3+ subcommands, got {}",
            subcmds.len()
        );
    }

    #[test]
    fn test_parse_subcommands_ends_at_options() {
        let help = "Commands:\n  sort    Sort data\n  merge   Merge data\n\nOptions:\n  -h  Help\n";
        let subcmds = parse_subcommands_generic(help);
        assert!(subcmds.len() >= 2);
    }

    #[test]
    fn test_parse_subcommands_dash_separator() {
        let help = "Commands:\n  sort - Sort alignments\n  index - Build index\n";
        let subcmds = parse_subcommands_generic(help);
        assert!(subcmds.len() >= 2);
    }

    #[test]
    fn test_extract_task_keywords_generic() {
        let keywords = extract_task_keywords_generic("Sort alignments by coordinate");
        assert!(keywords.contains(&"sort".to_string()));
    }

    #[test]
    fn test_calculate_doc_quality_empty() {
        let schema = CliSchema::minimal("tool", CliStyle::FlagsFirst);
        let score = calculate_doc_quality_generic(&schema);
        assert!(score < 0.2);
    }

    #[test]
    fn test_calculate_doc_quality_with_flags() {
        let mut schema = CliSchema::minimal("tool", CliStyle::FlagsFirst);
        schema.usage_summary = "tool [options]".to_string();
        schema.flags = vec![FlagSchema {
            name: "--output".to_string(),
            aliases: vec!["-o".to_string()],
            param_type: ParamType::File,
            description: "Output file".to_string(),
            default: None,
            required: false,
            long_description: None,
        }];
        let score = calculate_doc_quality_generic(&schema);
        assert!(score > 0.3);
    }

    #[test]
    fn test_calculate_doc_quality_subcommand_style() {
        let mut schema = CliSchema::minimal("tool", CliStyle::Subcommand);
        schema.subcommands = vec![SubcommandSchema {
            name: "sort".to_string(),
            description: "Sort data".to_string(),
            usage_pattern: String::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            constraints: Vec::new(),
            task_keywords: vec!["sort".to_string()],
        }];
        let score = calculate_doc_quality_generic(&schema);
        assert!(score >= 0.15);
    }

    #[test]
    fn test_generic_parser_full_parse() {
        let help = "Usage: mytool [options] INPUT OUTPUT\n\nOptions:\n  -h, --help       Show help\n  -o, --output=FILE  Output file\n  -t INT           Threads\n  --verbose        Verbose mode\n\nArguments:\n  INPUT   Input file\n  OUTPUT  Output file\n\nCommands:\n  sort    Sort data\n  merge   Merge data\n";
        let parser = GenericParser;
        let schema = parser.parse("mytool", help);
        assert!(!schema.usage_summary.is_empty());
        assert!(!schema.flags.is_empty());
        assert!(!schema.positionals.is_empty());
        assert!(!schema.subcommands.is_empty());
        assert!(schema.doc_quality > 0.0);
    }

    #[test]
    fn test_extract_subcommands_from_usage_no_braces() {
        let help = "Usage: tool <command> [options]\n";
        let subcmds = extract_subcommands_from_usage(help);
        assert!(subcmds.is_empty());
    }

    #[test]
    fn test_extract_subcommands_from_usage_with_braces() {
        let help = "Usage: tool {sort,merge,index} [options]\n";
        let subcmds = extract_subcommands_from_usage(help);
        assert_eq!(subcmds.len(), 3);
    }

    #[test]
    fn test_parse_subcommands_skips_short_names() {
        let help = "Commands:\n  a    Too short\n  sort Sort data\n";
        let subcmds = parse_subcommands_generic(help);
        assert!(subcmds.iter().all(|s| s.name.len() >= 2));
    }

    #[test]
    fn test_parse_subcommands_skips_common_words() {
        let help = "Commands:\n  the    The command\n  sort   Sort data\n";
        let subcmds = parse_subcommands_generic(help);
        assert!(subcmds.iter().all(|s| s.name != "the"));
    }
}

#[cfg(test)]
mod angsd_tests {
    use super::*;

    #[test]
    fn test_angsd_full_help() {
        let Ok(help) = std::process::Command::new("angsd").arg("--help").output() else {
            eprintln!("skipping angsd parser test because 'angsd' is not installed");
            return;
        };
        let help_str = String::from_utf8_lossy(&help.stderr);
        let flags = parse_flags_generic(&help_str);
        let names: Vec<_> = flags.iter().map(|f| f.name.as_str()).collect();
        println!("angsd flags: {:?}", names);
        assert!(names.contains(&"-GL"), "missing -GL, got: {:?}", names);
        assert!(names.contains(&"-bam"), "missing -bam, got: {:?}", names);
    }
}
