//! CLI Schema Intermediate Representation (IR)
//!
//! A unified formal model for representing command-line tool interfaces.
//! This schema captures the structure of CLI tools in a way that enables:
//! 1. Deterministic intent parsing
//! 2. Constrained command generation
//! 3. Schema validation (eliminating hallucination)
//!
//! ## Design Philosophy
//!
//! The Schema IR serves as the "boundary" in HDA (Hierarchical Deterministic Architecture):
//! - It defines what flags/subcommands/parameters are valid for a tool
//! - It constrains LLM generation to only produce schema-compliant outputs
//! - It enables validation of generated commands against the formal spec

use serde::{Deserialize, Serialize};

/// CLI argument style classification
///
/// Different tools follow different conventions for argument ordering.
/// This classification enables style-aware parsing and generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CliStyle {
    /// Git-style: `tool subcommand [options] [args]`
    /// Examples: samtools sort, git commit, docker run
    #[default]
    Subcommand,

    /// GNU-style: `tool [options] input output`
    /// Examples: fastp -i in.fq -o out.fq, minimap2 -a ref.fa reads.fq
    FlagsFirst,

    /// Unix-style: `tool input output [options]`
    /// Examples: admixture data.bed 5 --cv=10, prodigal -i genome.fna -a proteins.faa
    /// Note: Some tools like admixture use pure positional (no flags for main args)
    Positional,

    /// Mixed style: `tool input [options] output`
    /// Examples: some tools accept input first, then flags, then output
    Hybrid,
}

impl CliStyle {
    /// Detect CLI style from help output
    pub fn detect_from_help(help: &str) -> Self {
        let help_lower = help.to_lowercase();

        // Check for subcommand-style patterns
        if help_lower.contains("commands:")
            || help_lower.contains("subcommands:")
            || help_lower.contains("usage: tool <command>")
            || help_lower.contains("usage: tool command")
            || help_lower.contains("usage: prog <command>")
            || help_lower.contains("usage: prog command")
            || help_lower.contains("available commands")
            || help_lower.contains("available subcommands")
        {
            return CliStyle::Subcommand;
        }

        // Check for {cmd1,cmd2,...} pattern in usage (common in Python argparse)
        if help_lower.contains("usage:") && help_lower.contains("{") && help_lower.contains("}") {
            // Check if the {} contains command names
            if let Some(start) = help_lower.find('{')
                && let Some(end) = help_lower[start..].find('}')
            {
                let inner = &help_lower[start + 1..start + end];
                let candidates: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
                if candidates.len() >= 2
                    && candidates.iter().all(|c| {
                        !c.is_empty()
                            && c.chars()
                                .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '-')
                    })
                {
                    return CliStyle::Subcommand;
                }
            }
        }

        // Check for positional-style patterns (no flags for main args)
        // Tools like admixture show: "admixture input.bed K [options]"
        if help_lower.contains("input.bed")
            || help_lower.contains("usage:")
                && !help_lower.contains("-i")
                && !help_lower.contains("--input")
        {
            // Check if there are optional flags after positionals
            if help_lower.contains("-") || help_lower.contains("--") {
                return CliStyle::Hybrid;
            }
            return CliStyle::Positional;
        }

        // Check for flags-first patterns
        if help_lower.contains("-i")
            || help_lower.contains("--input")
            || help_lower.contains("usage: tool -")
        {
            return CliStyle::FlagsFirst;
        }

        // Default: assume flags-first for most modern tools
        CliStyle::FlagsFirst
    }

    /// Get the expected argument order for this style
    #[allow(dead_code)]
    pub fn argument_order(&self) -> ArgumentOrder {
        match self {
            CliStyle::Subcommand => ArgumentOrder::SubcommandFlagsPositionals,
            CliStyle::FlagsFirst => ArgumentOrder::FlagsPositionals,
            CliStyle::Positional => ArgumentOrder::PositionalsFlags,
            CliStyle::Hybrid => ArgumentOrder::PositionalsFlagsPositionals,
        }
    }
}

/// Expected argument ordering for command generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ArgumentOrder {
    /// `subcommand flags positionals` (e.g., samtools sort -@ 4 input.bam)
    SubcommandFlagsPositionals,
    /// `flags positionals` (e.g., fastp -i in.fq -o out.fq)
    FlagsPositionals,
    /// `positionals flags` (e.g., admixture data.bed 5 --cv=10)
    PositionalsFlags,
    /// `positionals flags positionals` (mixed)
    PositionalsFlagsPositionals,
}

/// Parameter type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamType {
    /// Integer parameter (e.g., -@ 4)
    Int,
    /// Float parameter (e.g., --threshold 0.5)
    Float,
    /// String parameter (e.g., --format BAM)
    String,
    /// File path parameter (e.g., -i input.bam)
    File,
    /// Boolean flag (e.g., --verbose, present or absent)
    Bool,
    /// Enum parameter with allowed values (e.g., --format bam|sam|cram)
    Enum(Vec<String>),
}

impl ParamType {
    /// Check if a value matches this type
    #[allow(dead_code)]
    pub fn validate_value(&self, value: &str) -> bool {
        match self {
            ParamType::Int => value.parse::<i64>().is_ok(),
            ParamType::Float => value.parse::<f64>().is_ok(),
            ParamType::String => !value.is_empty(),
            ParamType::File => !value.is_empty(),
            ParamType::Bool => {
                // Bool flags don't have values - they're present/absent
                // If checking a value, it should be empty or a bool-like string
                value.is_empty()
                    || value == "true"
                    || value == "false"
                    || value == "yes"
                    || value == "no"
            }
            ParamType::Enum(allowed) => allowed.iter().any(|v| v == value),
        }
    }

    /// Get a hint for LLM about expected value format
    #[allow(dead_code)]
    pub fn llm_hint(&self) -> String {
        match self {
            ParamType::Int => "integer".to_string(),
            ParamType::Float => "number".to_string(),
            ParamType::String => "text".to_string(),
            ParamType::File => "file path".to_string(),
            ParamType::Bool => "boolean (no value needed)".to_string(),
            ParamType::Enum(values) => format!("one of: {}", values.join("|")),
        }
    }
}

/// Flag schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSchema {
    /// Primary flag name (e.g., "-o", "--output")
    pub name: String,

    /// Aliases (e.g., ["-o", "--output", "--out"])
    pub aliases: Vec<String>,

    /// Parameter type
    pub param_type: ParamType,

    /// Brief description
    pub description: String,

    /// Default value if flag is optional
    pub default: Option<String>,

    /// Whether this flag is required
    pub required: bool,

    /// Long-form description with examples
    pub long_description: Option<String>,
}

impl FlagSchema {
    /// Check if a flag name matches this schema (including aliases)
    pub fn matches_name(&self, name: &str) -> bool {
        self.name == name || self.aliases.iter().any(|a| a == name)
    }

    /// Get all valid names for this flag
    pub fn all_names(&self) -> Vec<&str> {
        let mut names = vec![self.name.as_str()];
        names.extend(self.aliases.iter().map(|s| s.as_str()));
        names
    }
}

/// Positional parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionalSchema {
    /// Position index (0-based)
    pub position: usize,

    /// Parameter name placeholder (e.g., "INPUT", "K", "OUTPUT")
    pub name: String,

    /// Parameter type
    pub param_type: ParamType,

    /// Brief description
    pub description: String,

    /// Whether this positional is required
    pub required: bool,

    /// Default value if optional
    pub default: Option<String>,
}

/// Constraint rule for flag combinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintRule {
    /// Flag A requires flag B (e.g., -o requires input format)
    Requires(String, String),

    /// Flag A and flag B are mutually exclusive
    MutuallyExclusive(String, String),

    /// Flag A implies flag B with specific value
    ImpliesValue(String, String, String),

    /// All flags in set must be present together
    AllRequired(Vec<String>),

    /// At least one flag from set must be present
    AtLeastOne(Vec<String>),
}

/// Subcommand schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubcommandSchema {
    /// Subcommand name (e.g., "sort", "index", "quant")
    pub name: String,

    /// Brief description
    pub description: String,

    /// Usage pattern (e.g., "sort [-@ INT] [-o FILE] INPUT")
    pub usage_pattern: String,

    /// Flags specific to this subcommand
    pub flags: Vec<FlagSchema>,

    /// Positional parameters
    pub positionals: Vec<PositionalSchema>,

    /// Constraint rules
    pub constraints: Vec<ConstraintRule>,

    /// Task keywords that indicate this subcommand
    /// (e.g., "quantify" → quant, "sort" → sort, "merge" → merge)
    pub task_keywords: Vec<String>,
}

impl SubcommandSchema {
    /// Get all valid flag names for this subcommand
    pub fn all_flag_names(&self) -> Vec<&str> {
        self.flags.iter().flat_map(|f| f.all_names()).collect()
    }

    /// Check if a flag is valid for this subcommand
    #[allow(dead_code)]
    pub fn is_valid_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f.matches_name(name))
    }

    /// Get flag schema by name
    pub fn get_flag(&self, name: &str) -> Option<&FlagSchema> {
        self.flags.iter().find(|f| f.matches_name(name))
    }
}

/// Global flags that apply to all subcommands (or to tool without subcommands)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GlobalFlags {
    /// Global flags (e.g., --version, --help, --verbose)
    pub flags: Vec<FlagSchema>,
}

/// Complete CLI Schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSchema {
    /// Tool name
    pub tool: String,

    /// Tool version (for compatibility checks)
    pub version: Option<String>,

    /// CLI argument style
    pub cli_style: CliStyle,

    /// Brief tool description
    pub description: String,

    /// Subcommands (for subcommand-style tools)
    pub subcommands: Vec<SubcommandSchema>,

    /// Global flags (apply to all subcommands)
    pub global_flags: Vec<FlagSchema>,

    /// Flags for non-subcommand tools
    pub flags: Vec<FlagSchema>,

    /// Positional parameters for non-subcommand tools
    pub positionals: Vec<PositionalSchema>,

    /// Usage pattern summary
    pub usage_summary: String,

    /// Constraint rules
    pub constraints: Vec<ConstraintRule>,

    /// Documentation quality score (0.0-1.0)
    pub doc_quality: f32,

    /// Source of schema (e.g., "python-argparse", "rust-clap", "generic-regex")
    pub schema_source: String,
}

impl CliSchema {
    /// Create a minimal schema for a tool
    pub fn minimal(tool: &str, cli_style: CliStyle) -> Self {
        Self {
            tool: tool.to_string(),
            version: None,
            cli_style,
            description: String::new(),
            subcommands: Vec::new(),
            global_flags: Vec::new(),
            flags: Vec::new(),
            positionals: Vec::new(),
            usage_summary: String::new(),
            constraints: Vec::new(),
            doc_quality: 0.0,
            schema_source: "minimal".to_string(),
        }
    }

    /// Get subcommand by name
    pub fn get_subcommand(&self, name: &str) -> Option<&SubcommandSchema> {
        self.subcommands.iter().find(|s| s.name == name)
    }

    /// Select subcommand based on task description
    pub fn select_subcommand(&self, task: &str) -> Option<&SubcommandSchema> {
        if self.subcommands.is_empty() {
            return None;
        }

        let task_lower = task.to_lowercase();
        let task_words: Vec<&str> = task_lower.split_whitespace().collect();

        let mut best: Option<(&SubcommandSchema, usize)> = None;

        for subcmd in &self.subcommands {
            let mut score = 0;

            if task_words.iter().any(|w| *w == subcmd.name) {
                score += 100;
            }

            for kw in &subcmd.task_keywords {
                let kw_lower = kw.to_lowercase();
                if task_words
                    .iter()
                    .any(|w| *w == kw_lower || w.contains(&kw_lower))
                {
                    score += 10;
                }
                if task_lower.contains(&kw_lower) {
                    score += 5;
                }
            }

            let desc_lower = subcmd.description.to_lowercase();
            for word in &task_words {
                if desc_lower.contains(word) {
                    score += 2;
                }
            }

            if score > 0 {
                match best {
                    None => best = Some((subcmd, score)),
                    Some((_, prev)) if score > prev => best = Some((subcmd, score)),
                    _ => {}
                }
            }
        }

        best.map(|(s, _)| s)
    }

    /// Get all valid flag names (global + subcommand-specific)
    pub fn all_flag_names(&self, subcommand: Option<&str>) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .global_flags
            .iter()
            .flat_map(|f| f.all_names())
            .collect();

        if let Some(subcmd) = subcommand {
            if let Some(schema) = self.get_subcommand(subcmd) {
                names.extend(schema.all_flag_names());
            }
        } else {
            names.extend(self.flags.iter().flat_map(|f| f.all_names()));
        }

        names
    }

    /// Check if a flag is valid for given context
    #[allow(dead_code)]
    pub fn is_valid_flag(&self, name: &str, subcommand: Option<&str>) -> bool {
        // Check global flags
        if self.global_flags.iter().any(|f| f.matches_name(name)) {
            return true;
        }

        // Check subcommand-specific flags
        if let Some(subcmd) = subcommand
            && let Some(schema) = self.get_subcommand(subcmd)
        {
            return schema.is_valid_flag(name);
        }

        // Check tool-level flags (for non-subcommand tools)
        self.flags.iter().any(|f| f.matches_name(name))
    }

    /// Get flag schema by name
    pub fn get_flag(&self, name: &str, subcommand: Option<&str>) -> Option<&FlagSchema> {
        // Check global flags first
        if let Some(f) = self.global_flags.iter().find(|f| f.matches_name(name)) {
            return Some(f);
        }

        // Check subcommand-specific flags
        if let Some(subcmd) = subcommand
            && let Some(schema) = self.get_subcommand(subcmd)
        {
            return schema.get_flag(name);
        }

        // Check tool-level flags
        self.flags.iter().find(|f| f.matches_name(name))
    }

    /// Validate generated command (flag, value) pairs against schema
    pub fn validate_command(
        &self,
        flag_args: &[(String, Option<String>)],
        subcommand: Option<&str>,
    ) -> ValidationResult {
        let args: Vec<String> = flag_args
            .iter()
            .flat_map(|(name, value)| {
                let mut v = vec![name.clone()];
                if let Some(val) = value {
                    v.push(val.clone());
                }
                v
            })
            .collect();
        self.validate_args(&args, subcommand)
    }

    /// Validate generated arguments against schema
    pub fn validate_args(&self, args: &[String], subcommand: Option<&str>) -> ValidationResult {
        let mut errors = Vec::new();

        // 1. Check subcommand if expected
        if self.cli_style == CliStyle::Subcommand {
            if let Some(first) = args.first() {
                if !self.subcommands.iter().any(|s| &s.name == first)
                    && let Some(expected) = subcommand
                {
                    // Subcommand was detected but not used
                    errors.push(ValidationError::WrongSubcommand {
                        expected: expected.to_string(),
                        actual: first.clone(),
                    });
                }
            } else if let Some(expected) = subcommand {
                errors.push(ValidationError::MissingSubcommand {
                    expected: expected.to_string(),
                });
            }
        }

        // 2. Check all flags against whitelist
        let valid_flags = self.all_flag_names(subcommand);
        let mut used_flags = Vec::new();

        for arg in args.iter() {
            if arg.starts_with('-') {
                // Check if flag is valid
                if !valid_flags.iter().any(|v| *v == arg) {
                    errors.push(ValidationError::InvalidFlag {
                        flag: arg.clone(),
                        valid_flags: valid_flags.iter().map(|s| s.to_string()).collect(),
                    });
                }
                used_flags.push(arg.clone());
            }
        }

        // 3. Check required flags
        let required_flags = self.get_required_flags(subcommand);
        for req in &required_flags {
            if !used_flags.iter().any(|u| {
                self.get_flag(req, subcommand)
                    .map(|f| f.matches_name(u))
                    .unwrap_or(false)
            }) {
                errors.push(ValidationError::MissingRequiredFlag { flag: req.clone() });
            }
        }

        // 4. Check constraints
        for constraint in &self.constraints {
            if let Some(error) = self.validate_constraint(constraint, &used_flags) {
                errors.push(error);
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings: Vec::new(), // Reserved for future warning messages
            used_flags,
            valid_flags: valid_flags.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Get required flags for a context
    fn get_required_flags(&self, subcommand: Option<&str>) -> Vec<String> {
        let mut required = Vec::new();

        // Global required flags
        for flag in &self.global_flags {
            if flag.required {
                required.push(flag.name.clone());
            }
        }

        // Subcommand/tool required flags
        if let Some(subcmd) = subcommand {
            if let Some(schema) = self.get_subcommand(subcmd) {
                for flag in &schema.flags {
                    if flag.required {
                        required.push(flag.name.clone());
                    }
                }
            }
        } else {
            for flag in &self.flags {
                if flag.required {
                    required.push(flag.name.clone());
                }
            }
        }

        required
    }

    /// Validate a single constraint
    fn validate_constraint(
        &self,
        constraint: &ConstraintRule,
        used_flags: &[String],
    ) -> Option<ValidationError> {
        match constraint {
            ConstraintRule::Requires(a, b) => {
                if used_flags.iter().any(|f| f == a) && !used_flags.iter().any(|f| f == b) {
                    return Some(ValidationError::ConstraintViolation {
                        message: format!("Flag {} requires flag {}", a, b),
                    });
                }
            }
            ConstraintRule::MutuallyExclusive(a, b) => {
                if used_flags.iter().any(|f| f == a) && used_flags.iter().any(|f| f == b) {
                    return Some(ValidationError::ConstraintViolation {
                        message: format!("Flags {} and {} are mutually exclusive", a, b),
                    });
                }
            }
            ConstraintRule::AllRequired(flags) => {
                let all_present = flags.iter().all(|f| used_flags.iter().any(|u| u == f));
                if !all_present {
                    return Some(ValidationError::ConstraintViolation {
                        message: format!("All flags in {} are required together", flags.join(", ")),
                    });
                }
            }
            ConstraintRule::AtLeastOne(flags) => {
                let any_present = flags.iter().any(|f| used_flags.iter().any(|u| u == f));
                if !any_present {
                    return Some(ValidationError::ConstraintViolation {
                        message: format!("At least one flag from {} is required", flags.join(", ")),
                    });
                }
            }
            _ => {}
        }
        None
    }
}

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    /// Used a flag that doesn't exist in schema
    InvalidFlag {
        flag: String,
        valid_flags: Vec<String>,
    },

    /// Missing a required flag
    MissingRequiredFlag { flag: String },

    /// Missing expected subcommand
    MissingSubcommand { expected: String },

    /// Wrong subcommand used
    WrongSubcommand { expected: String, actual: String },

    /// Constraint violation
    ConstraintViolation { message: String },

    /// Wrong value type
    WrongValueType {
        flag: String,
        expected_type: String,
        actual_value: String,
    },

    /// Missing positional argument
    MissingPositional { position: usize, name: String },
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether args are valid
    pub is_valid: bool,

    /// Validation errors
    pub errors: Vec<ValidationError>,

    /// Validation warnings
    pub warnings: Vec<String>,

    /// Flags used in args
    pub used_flags: Vec<String>,

    /// All valid flags for context
    pub valid_flags: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            used_flags: Vec::new(),
            valid_flags: Vec::new(),
        }
    }

    /// Get error summary for display
    #[allow(dead_code)]
    pub fn error_summary(&self) -> String {
        self.errors
            .iter()
            .map(|e| match e {
                ValidationError::InvalidFlag { flag, .. } => {
                    format!("Invalid flag: {} (not in schema)", flag)
                }
                ValidationError::MissingRequiredFlag { flag } => {
                    format!("Missing required flag: {}", flag)
                }
                ValidationError::MissingSubcommand { expected } => {
                    format!("Missing subcommand: {}", expected)
                }
                ValidationError::WrongSubcommand { expected, actual } => {
                    format!("Wrong subcommand: expected {}, got {}", expected, actual)
                }
                ValidationError::ConstraintViolation { message } => message.clone(),
                ValidationError::WrongValueType {
                    flag,
                    expected_type,
                    actual_value,
                } => format!(
                    "Flag {}: expected {}, got {}",
                    flag, expected_type, actual_value
                ),
                ValidationError::MissingPositional { position, name } => format!(
                    "Missing positional argument {} (position {})",
                    name, position
                ),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_style_detection() {
        // Subcommand-style help
        let help = "Usage: samtools <command> [options]\nCommands: sort, view, index";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::Subcommand);

        // Flags-first help
        let help = "Usage: fastp -i INPUT -o OUTPUT [options]";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::FlagsFirst);
    }

    #[test]
    fn test_param_type_validation() {
        assert!(ParamType::Int.validate_value("4"));
        assert!(!ParamType::Int.validate_value("abc"));

        assert!(ParamType::Enum(vec!["bam".to_string(), "sam".to_string()]).validate_value("bam"));
        assert!(!ParamType::Enum(vec!["bam".to_string()]).validate_value("cram"));
    }

    #[test]
    fn test_flag_schema_matching() {
        let flag = FlagSchema {
            name: "-o".to_string(),
            aliases: vec!["--output".to_string()],
            param_type: ParamType::File,
            description: "Output file".to_string(),
            default: None,
            required: false,
            long_description: None,
        };

        assert!(flag.matches_name("-o"));
        assert!(flag.matches_name("--output"));
        assert!(!flag.matches_name("-i"));
    }

    #[test]
    fn test_subcommand_select() {
        let schema = CliSchema {
            tool: "salmon".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: "Salmon".to_string(),
            subcommands: vec![
                SubcommandSchema {
                    name: "quant".to_string(),
                    description: "Quantify expression".to_string(),
                    usage_pattern: String::new(),
                    flags: vec![],
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["quantify".to_string(), "expression".to_string()],
                },
                SubcommandSchema {
                    name: "index".to_string(),
                    description: "Build index".to_string(),
                    usage_pattern: String::new(),
                    flags: vec![],
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["index".to_string(), "build".to_string()],
                },
            ],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.9,
            schema_source: "test".to_string(),
        };

        let selected = schema.select_subcommand("quantify reads from fastq");
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "quant");

        let selected = schema.select_subcommand("build index from reference");
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "index");
    }

    #[test]
    fn test_schema_validation() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: "Test tool".to_string(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![FlagSchema {
                name: "-i".to_string(),
                aliases: vec!["--input".to_string()],
                param_type: ParamType::File,
                description: "Input".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            positionals: vec![],
            usage_summary: "test -i INPUT [options]".to_string(),
            constraints: vec![],
            doc_quality: 1.0,
            schema_source: "test".to_string(),
        };

        // Missing required flag
        let result = schema.validate_args(&["-o".to_string(), "output.bam".to_string()], None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::MissingRequiredFlag { .. }))
        );

        // Invalid flag
        let result = schema.validate_args(&["-invalid".to_string()], None);
        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, ValidationError::InvalidFlag { .. }))
        );

        // Valid args
        let result = schema.validate_args(&["-i".to_string(), "input.bam".to_string()], None);
        assert!(result.is_valid);
    }

    #[test]
    fn test_cli_style_detect_subcommand_braces() {
        let help = "usage: tool {init,run,status} [options]";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::Subcommand);
    }

    #[test]
    fn test_cli_style_detect_positional() {
        let help = "usage: admixture input.bed K [options]\nOptions:\n  --cv  cross-validation";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::Hybrid);
    }

    #[test]
    fn test_cli_style_detect_flags_first_with_input() {
        let help = "usage: fastp -i INPUT -o OUTPUT\nOptions:\n  -i  input\n  -o  output";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::FlagsFirst);
    }

    #[test]
    fn test_cli_style_argument_order() {
        assert_eq!(CliStyle::Subcommand.argument_order(), ArgumentOrder::SubcommandFlagsPositionals);
        assert_eq!(CliStyle::FlagsFirst.argument_order(), ArgumentOrder::FlagsPositionals);
        assert_eq!(CliStyle::Positional.argument_order(), ArgumentOrder::PositionalsFlags);
        assert_eq!(CliStyle::Hybrid.argument_order(), ArgumentOrder::PositionalsFlagsPositionals);
    }

    #[test]
    fn test_param_type_validate() {
        assert!(ParamType::Float.validate_value("3.14"));
        assert!(!ParamType::Float.validate_value("abc"));
        assert!(ParamType::String.validate_value("hello"));
        assert!(!ParamType::String.validate_value(""));
        assert!(ParamType::File.validate_value("/path/to/file"));
        assert!(ParamType::Bool.validate_value(""));
        assert!(ParamType::Bool.validate_value("true"));
        assert!(ParamType::Bool.validate_value("false"));
        assert!(ParamType::Bool.validate_value("yes"));
        assert!(ParamType::Bool.validate_value("no"));
        assert!(!ParamType::Bool.validate_value("maybe"));
    }

    #[test]
    fn test_param_type_llm_hint() {
        assert_eq!(ParamType::Int.llm_hint(), "integer");
        assert_eq!(ParamType::Float.llm_hint(), "number");
        assert_eq!(ParamType::String.llm_hint(), "text");
        assert_eq!(ParamType::File.llm_hint(), "file path");
        assert_eq!(ParamType::Bool.llm_hint(), "boolean (no value needed)");
        assert_eq!(ParamType::Enum(vec!["a".to_string(), "b".to_string()]).llm_hint(), "one of: a|b");
    }

    #[test]
    fn test_flag_schema_all_names() {
        let flag = FlagSchema {
            name: "-o".to_string(),
            aliases: vec!["--output".to_string()],
            param_type: ParamType::File,
            description: "Output".to_string(),
            default: None,
            required: false,
            long_description: None,
        };
        let names = flag.all_names();
        assert_eq!(names, vec!["-o", "--output"]);
    }

    #[test]
    fn test_cli_schema_is_valid_flag() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "run".to_string(),
                description: String::new(),
                usage_pattern: String::new(),
                flags: vec![FlagSchema {
                    name: "-t".to_string(),
                    aliases: vec![],
                    param_type: ParamType::Int,
                    description: "Threads".to_string(),
                    default: None,
                    required: false,
                    long_description: None,
                }],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![FlagSchema {
                name: "-v".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            flags: vec![FlagSchema {
                name: "-o".to_string(),
                aliases: vec![],
                param_type: ParamType::File,
                description: "Output".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        assert!(schema.is_valid_flag("-v", None));
        assert!(schema.is_valid_flag("-o", None));
        assert!(!schema.is_valid_flag("-x", None));
        assert!(schema.is_valid_flag("-v", Some("run")));
        assert!(schema.is_valid_flag("-t", Some("run")));
        assert!(!schema.is_valid_flag("-o", Some("run")));
    }

    #[test]
    fn test_cli_schema_get_flag() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![FlagSchema {
                name: "-v".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            flags: vec![FlagSchema {
                name: "-o".to_string(),
                aliases: vec![],
                param_type: ParamType::File,
                description: "Output".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        assert!(schema.get_flag("-v", None).is_some());
        assert!(schema.get_flag("-o", None).is_some());
        assert!(schema.get_flag("-x", None).is_none());
    }

    #[test]
    fn test_cli_schema_all_flag_names() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "sort".to_string(),
                description: String::new(),
                usage_pattern: String::new(),
                flags: vec![FlagSchema {
                    name: "-@".to_string(),
                    aliases: vec![],
                    param_type: ParamType::Int,
                    description: "Threads".to_string(),
                    default: None,
                    required: false,
                    long_description: None,
                }],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![FlagSchema {
                name: "-v".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let names = schema.all_flag_names(Some("sort"));
        assert!(names.contains(&"-v"));
        assert!(names.contains(&"-@"));
    }

    #[test]
    fn test_subcommand_schema_methods() {
        let subcmd = SubcommandSchema {
            name: "sort".to_string(),
            description: "Sort data".to_string(),
            usage_pattern: "sort [options] INPUT".to_string(),
            flags: vec![FlagSchema {
                name: "-@".to_string(),
                aliases: vec!["--threads".to_string()],
                param_type: ParamType::Int,
                description: "Threads".to_string(),
                default: None,
                required: false,
                long_description: None,
            }],
            positionals: vec![],
            constraints: vec![],
            task_keywords: vec![],
        };
        assert!(subcmd.is_valid_flag("-@"));
        assert!(subcmd.is_valid_flag("--threads"));
        assert!(!subcmd.is_valid_flag("-x"));
        assert!(subcmd.get_flag("-@").is_some());
        assert!(subcmd.get_flag("-x").is_none());
        let names = subcmd.all_flag_names();
        assert!(names.contains(&"-@"));
        assert!(names.contains(&"--threads"));
    }

    #[test]
    fn test_cli_style_detect_hybrid_style() {
        let help = "usage: admixture input.bed K [options] -t 4";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::Hybrid);
    }

    #[test]
    fn test_cli_style_detect_flags_first_via_input() {
        let help = "tool -i input.txt -o output.txt";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::FlagsFirst);
    }

    #[test]
    fn test_cli_style_detect_default_flags_first() {
        let help = "some random text without patterns";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::FlagsFirst);
    }

    #[test]
    fn test_cli_style_detect_positional_style() {
        let help = "usage: mytool input.bed K";
        assert_eq!(CliStyle::detect_from_help(help), CliStyle::Positional);
    }

    #[test]
    fn test_validate_args_subcommand_wrong() {
        let schema = CliSchema {
            tool: "bwa".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "mem".to_string(),
                description: "BWA-MEM".to_string(),
                usage_pattern: String::new(),
                flags: vec![],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&["index".to_string()], Some("mem"));
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::WrongSubcommand { .. })));
    }

    #[test]
    fn test_validate_args_missing_subcommand() {
        let schema = CliSchema {
            tool: "bwa".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "mem".to_string(),
                description: "BWA-MEM".to_string(),
                usage_pattern: String::new(),
                flags: vec![],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&[], Some("mem"));
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingSubcommand { .. })));
    }

    #[test]
    fn test_validate_args_constraints() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![
                FlagSchema {
                    name: "-a".to_string(),
                    aliases: vec![],
                    param_type: ParamType::Bool,
                    description: "Flag A".to_string(),
                    default: None,
                    required: false,
                    long_description: None,
                },
                FlagSchema {
                    name: "-b".to_string(),
                    aliases: vec![],
                    param_type: ParamType::Bool,
                    description: "Flag B".to_string(),
                    default: None,
                    required: false,
                    long_description: None,
                },
                FlagSchema {
                    name: "-c".to_string(),
                    aliases: vec![],
                    param_type: ParamType::Bool,
                    description: "Flag C".to_string(),
                    default: None,
                    required: false,
                    long_description: None,
                },
            ],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![
                ConstraintRule::Requires("-a".to_string(), "-b".to_string()),
                ConstraintRule::MutuallyExclusive("-a".to_string(), "-c".to_string()),
                ConstraintRule::AllRequired(vec!["-b".to_string(), "-c".to_string()]),
                ConstraintRule::AtLeastOne(vec!["-a".to_string(), "-b".to_string()]),
            ],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&["-a".to_string()], None);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::ConstraintViolation { .. })));
    }

    #[test]
    fn test_validate_args_at_least_one_missing() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![ConstraintRule::AtLeastOne(vec!["-a".to_string(), "-b".to_string()])],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&[], None);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_args_all_required_missing() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![ConstraintRule::AllRequired(vec!["-a".to_string(), "-b".to_string()])],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&[], None);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_validate_args_mutually_exclusive() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![ConstraintRule::MutuallyExclusive("-a".to_string(), "-b".to_string())],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&["-a".to_string(), "-b".to_string()], None);
        assert!(!result.is_valid);
    }

    #[test]
    fn test_get_required_flags_subcommand() {
        let schema = CliSchema {
            tool: "test".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "run".to_string(),
                description: String::new(),
                usage_pattern: String::new(),
                flags: vec![FlagSchema {
                    name: "-i".to_string(),
                    aliases: vec![],
                    param_type: ParamType::File,
                    description: "Input".to_string(),
                    default: None,
                    required: true,
                    long_description: None,
                }],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![FlagSchema {
                name: "-v".to_string(),
                aliases: vec![],
                param_type: ParamType::Bool,
                description: "Verbose".to_string(),
                default: None,
                required: true,
                long_description: None,
            }],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.validate_args(&["run".to_string()], Some("run"));
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::MissingRequiredFlag { .. })));
    }

    #[test]
    fn test_validation_result_error_summary() {
        let result = ValidationResult {
            is_valid: false,
            errors: vec![
                ValidationError::InvalidFlag { flag: "-x".to_string(), valid_flags: vec![] },
                ValidationError::MissingRequiredFlag { flag: "-i".to_string() },
                ValidationError::MissingSubcommand { expected: "mem".to_string() },
                ValidationError::WrongSubcommand { expected: "mem".to_string(), actual: "index".to_string() },
                ValidationError::ConstraintViolation { message: "test constraint".to_string() },
                ValidationError::WrongValueType { flag: "-t".to_string(), expected_type: "Int".to_string(), actual_value: "abc".to_string() },
                ValidationError::MissingPositional { position: 0, name: "INPUT".to_string() },
            ],
            warnings: Vec::new(),
            used_flags: Vec::new(),
            valid_flags: Vec::new(),
        };
        let summary = result.error_summary();
        assert!(summary.contains("Invalid flag"));
        assert!(summary.contains("Missing required flag"));
        assert!(summary.contains("Missing subcommand"));
        assert!(summary.contains("Wrong subcommand"));
        assert!(summary.contains("test constraint"));
        assert!(summary.contains("expected Int"));
        assert!(summary.contains("Missing positional"));
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::valid();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_select_subcommand_keyword_match() {
        let schema = CliSchema {
            tool: "bwa".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![
                SubcommandSchema {
                    name: "mem".to_string(),
                    description: "BWA-MEM algorithm for alignment".to_string(),
                    usage_pattern: String::new(),
                    flags: vec![],
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["align".to_string(), "mapping".to_string()],
                },
                SubcommandSchema {
                    name: "index".to_string(),
                    description: "Index reference genome".to_string(),
                    usage_pattern: String::new(),
                    flags: vec![],
                    positionals: vec![],
                    constraints: vec![],
                    task_keywords: vec!["index".to_string(), "build".to_string()],
                },
            ],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.select_subcommand("align reads to reference");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "mem");
    }

    #[test]
    fn test_select_subcommand_description_match() {
        let schema = CliSchema {
            tool: "tool".to_string(),
            version: None,
            cli_style: CliStyle::Subcommand,
            description: String::new(),
            subcommands: vec![SubcommandSchema {
                name: "sort".to_string(),
                description: "Sort alignment by coordinate".to_string(),
                usage_pattern: String::new(),
                flags: vec![],
                positionals: vec![],
                constraints: vec![],
                task_keywords: vec![],
            }],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        let result = schema.select_subcommand("coordinate sort my BAM file");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "sort");
    }

    #[test]
    fn test_select_subcommand_empty() {
        let schema = CliSchema {
            tool: "tool".to_string(),
            version: None,
            cli_style: CliStyle::FlagsFirst,
            description: String::new(),
            subcommands: vec![],
            global_flags: vec![],
            flags: vec![],
            positionals: vec![],
            usage_summary: String::new(),
            constraints: vec![],
            doc_quality: 0.5,
            schema_source: "test".to_string(),
        };
        assert!(schema.select_subcommand("do something").is_none());
    }
}
