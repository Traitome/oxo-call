//! Python argparse Help Parser
//!
//! Parses help output from Python tools using argparse (e.g., most bioPython tools).
//!
//! ## Argparse Format Recognition
//!
//! Typical Python argparse help output:
//! ```ignore
//! usage: tool [options]
//!
//! optional arguments:
//!   -h, --help            show this help message and exit
//!   -i INPUT, --input INPUT
//!                         Input file
//!   -o OUTPUT, --output OUTPUT
//!                         Output file
//!   -v, --verbose         Verbose output
//!
//! positional arguments:
//!   input_file            Input file path
//!   output_file           Output file path
//! ```

use super::HelpParser;
use crate::schema::{
    CliSchema, CliStyle, FlagSchema, ParamType, PositionalSchema, SubcommandSchema,
};

/// Python argparse help parser
pub struct PythonArgparseParser;

impl HelpParser for PythonArgparseParser {
    fn name(&self) -> &str {
        "python-argparse"
    }

    fn can_parse(&self, help: &str) -> bool {
        let help_lower = help.to_lowercase();
        help_lower.contains("optional arguments:")
            || help_lower.contains("positional arguments:")
            || help_lower.contains("show this help message and exit")
            || help_lower.contains("usage: python")
    }

    fn parse(&self, tool: &str, help: &str) -> CliSchema {
        let cli_style = CliStyle::detect_from_help(help);

        let mut schema = CliSchema::minimal(tool, cli_style);
        schema.schema_source = self.name().to_string();
        schema.usage_summary = extract_usage(help);

        // Parse optional arguments (flags)
        schema.flags = parse_optional_arguments(help);

        // Parse positional arguments
        schema.positionals = parse_positional_arguments(help);

        // Detect subcommands if present
        schema.subcommands = parse_subcommands(help);

        // Calculate doc quality based on extraction success
        schema.doc_quality = calculate_doc_quality(&schema);

        schema
    }
}

/// Extract usage line
fn extract_usage(help: &str) -> String {
    for line in help.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("usage:") {
            return line.trim().to_string();
        }
    }
    String::new()
}

/// Parse optional arguments section
fn parse_optional_arguments(help: &str) -> Vec<FlagSchema> {
    let mut flags = Vec::new();
    let mut in_optional_section = false;

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Detect section start
        if line_lower.contains("optional arguments:") || line_lower.contains("options:") {
            in_optional_section = true;
            continue;
        }

        // Detect section end
        if in_optional_section && line_lower.contains("positional arguments:")
            || line_lower.contains("arguments:")
            || line_lower.starts_with("usage:")
        {
            in_optional_section = false;
        }

        if !in_optional_section {
            continue;
        }

        // Parse flag line
        if let Some(flag) = parse_flag_line(line) {
            flags.push(flag);
        }
    }

    flags
}

/// Parse a single flag line
fn parse_flag_line(line: &str) -> Option<FlagSchema> {
    // Argparse format: "-h, --help            show this help message and exit"
    // or: "-i INPUT, --input INPUT    Input file"

    let trimmed = line.trim();
    if !trimmed.starts_with('-') {
        return None;
    }

    // Split on whitespace to get flags and description
    let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();
    if parts.is_empty() {
        return None;
    }

    let flag_part = parts[0].trim();
    let description = if parts.len() > 1 {
        parts[1].trim().to_string()
    } else {
        String::new()
    };

    // Parse flag names and parameter type
    let (names, param_type) = parse_flag_names_and_type(flag_part);

    if names.is_empty() {
        return None;
    }

    // Primary name is the first one
    let primary = names[0].to_string();
    let aliases = names[1..].iter().map(|s| s.to_string()).collect();

    Some(FlagSchema {
        name: primary,
        aliases,
        param_type,
        description,
        default: None,
        required: false,
        long_description: None,
    })
}

/// Parse flag names and determine parameter type
fn parse_flag_names_and_type(flag_part: &str) -> (Vec<&str>, ParamType) {
    // Examples:
    // "-h, --help" -> (["-h", "--help"], Bool)
    // "-i INPUT, --input INPUT" -> (["-i", "--input"], File)

    let mut names = Vec::new();
    let mut param_type = ParamType::Bool;
    let mut found_placeholder = false;

    // Split by comma
    for segment in flag_part.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        // Check if segment has a placeholder
        // "-i INPUT" means the flag takes a value
        let parts: Vec<&str> = segment.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let flag_name = parts[0];
        if flag_name.starts_with('-') {
            names.push(flag_name);
        }

        // If there's a placeholder after the flag
        if parts.len() > 1 {
            let placeholder = parts[1];
            found_placeholder = true;

            // Infer type from placeholder name
            param_type = infer_type_from_placeholder(placeholder);
        }
    }

    if !found_placeholder && !names.is_empty() {
        param_type = ParamType::Bool;
    }

    (names, param_type)
}

/// Infer parameter type from placeholder name
fn infer_type_from_placeholder(placeholder: &str) -> ParamType {
    let placeholder_lower = placeholder.to_lowercase();

    if placeholder_lower.contains("file")
        || placeholder_lower.ends_with(".bam")
        || placeholder_lower.ends_with(".fa")
        || placeholder_lower.ends_with(".fq")
        || placeholder_lower.ends_with(".fasta")
        || placeholder_lower.ends_with(".fastq")
        || placeholder_lower.contains("path")
        || placeholder_lower.contains("input")
        || placeholder_lower.contains("output")
    {
        ParamType::File
    } else if placeholder_lower.contains("int")
        || placeholder_lower.contains("num")
        || placeholder_lower.contains("count")
        || placeholder_lower == "k"
    {
        ParamType::Int
    } else if placeholder_lower.contains("float")
        || placeholder_lower.contains("threshold")
        || placeholder_lower.contains("prob")
    {
        ParamType::Float
    } else {
        ParamType::String
    }
}

/// Parse positional arguments section
fn parse_positional_arguments(help: &str) -> Vec<PositionalSchema> {
    let mut positionals = Vec::new();
    let mut in_positional_section = false;
    let mut position = 0;

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Detect section start
        if line_lower.contains("positional arguments:") {
            in_positional_section = true;
            continue;
        }

        // Detect section end
        if in_positional_section && line_lower.contains("optional arguments:")
            || line_lower.contains("options:")
            || line_lower.starts_with("usage:")
        {
            in_positional_section = false;
        }

        if !in_positional_section {
            continue;
        }

        // Parse positional argument line
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('-') {
            continue;
        }

        // Split on whitespace to get name and description
        let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].trim();
        if name.is_empty() {
            continue;
        }

        // Skip if name looks like a description continuation
        if !name
            .chars()
            .next()
            .map(|c| c.is_alphanumeric())
            .unwrap_or(false)
        {
            continue;
        }

        let description = if parts.len() > 1 {
            parts[1].trim().to_string()
        } else {
            String::new()
        };

        // Infer type from name
        let param_type = infer_type_from_placeholder(name);

        positionals.push(PositionalSchema {
            position,
            name: name.to_string(),
            param_type,
            description,
            required: true,
            default: None,
        });

        position += 1;
    }

    positionals
}

/// Parse subcommands section
fn parse_subcommands(help: &str) -> Vec<SubcommandSchema> {
    let mut subcommands = Vec::new();
    let mut in_commands_section = false;

    for line in help.lines() {
        let line_lower = line.to_lowercase().trim().to_string();

        // Detect section start
        if line_lower.contains("commands:") || line_lower.contains("subcommands:") {
            in_commands_section = true;
            continue;
        }

        // Detect section end
        if in_commands_section && line_lower.starts_with("usage:") {
            in_commands_section = false;
        }

        if !in_commands_section {
            continue;
        }

        // Parse command line
        // Format: "  sort        sort BAM file"
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Split on whitespace
        let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();
        if parts.is_empty() {
            continue;
        }

        let name = parts[0].trim();
        if name.is_empty() || name.starts_with('-') {
            continue;
        }

        let description = if parts.len() > 1 {
            parts[1].trim().to_string()
        } else {
            String::new()
        };

        // Extract task keywords from description
        let task_keywords = extract_task_keywords(&description);

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

    subcommands
}

/// Extract task keywords from description
fn extract_task_keywords(description: &str) -> Vec<String> {
    // Simple keyword extraction: first few words
    let words: Vec<String> = description
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .take(5)
        .map(|w| w.to_lowercase())
        .collect();
    words
}

/// Calculate documentation quality score
fn calculate_doc_quality(schema: &CliSchema) -> f32 {
    let mut score = 0.0;

    // Has usage summary
    if !schema.usage_summary.is_empty() {
        score += 0.2;
    }

    // Has flags
    score += (schema.flags.len().min(10) as f32) * 0.05;

    // Has positionals
    score += (schema.positionals.len().min(5) as f32) * 0.05;

    // Has subcommands (for subcommand-style tools)
    if schema.cli_style == CliStyle::Subcommand && !schema.subcommands.is_empty() {
        score += 0.2;
    }

    // All flags have descriptions
    let flags_with_desc = schema
        .flags
        .iter()
        .filter(|f| !f.description.is_empty())
        .count();
    if flags_with_desc == schema.flags.len() && !schema.flags.is_empty() {
        score += 0.2;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_argparse_parser() {
        let help = "usage: tool [options]\n\noptional arguments:\n  -h, --help  show this help message and exit\n  -i INPUT    input file\n";
        let parser = PythonArgparseParser;

        assert!(parser.can_parse(help));

        let schema = parser.parse("tool", help);
        assert_eq!(schema.flags.len(), 2);
    }

    #[test]
    fn test_parse_flag_line() {
        let line = "-i INPUT, --input INPUT    Input file path";
        let flag = parse_flag_line(line).unwrap();

        assert_eq!(flag.name, "-i");
        assert!(flag.aliases.contains(&"--input".to_string()));
        assert_eq!(flag.param_type, ParamType::File);
    }

    #[test]
    fn test_infer_type_from_placeholder() {
        assert_eq!(infer_type_from_placeholder("INPUT.bam"), ParamType::File);
        assert_eq!(infer_type_from_placeholder("INT"), ParamType::Int);
        assert_eq!(infer_type_from_placeholder("THRESHOLD"), ParamType::Float);
    }
}
