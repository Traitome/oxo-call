//! CLI Help Output Parsers
//!
//! Different CLI tools use different help output formats based on their
//! underlying framework (Python argparse, Rust clap, Go flag, etc.).
//! This module provides specialized parsers for each style.
//!
//! ## Design Philosophy
//!
//! Parsing help output should be **deterministic** - no LLM involved.
//! This is Layer 2 of HDA: Help → Structured Schema IR.
//!
//! ## Parser Selection
//!
//! The `detect_parser_type()` function analyzes help output structure
//! and selects the appropriate specialized parser. If no specialized
//! parser matches, it falls back to the generic regex parser.

pub mod generic;
pub mod python;

use crate::schema::CliSchema;

/// Parser trait for help output parsing
pub trait HelpParser {
    /// Parser name for identification
    fn name(&self) -> &str;

    /// Check if this parser can handle the given help output
    fn can_parse(&self, help: &str) -> bool;

    /// Parse help output into CliSchema
    fn parse(&self, tool: &str, help: &str) -> CliSchema;
}

/// Detect which parser type to use based on help output structure
pub fn detect_parser_type(help: &str) -> ParserType {
    let help_lower = help.to_lowercase();

    // Python argparse patterns
    if help_lower.contains("usage: python")
        || help_lower.contains("optional arguments:")
        || help_lower.contains("positional arguments:")
        || help_lower.contains("show this help message and exit")
    {
        return ParserType::PythonArgparse;
    }

    // Rust clap patterns (often colored, table format)
    if help_lower.contains("options:")
        && help_lower
            .lines()
            .any(|l| l.contains("-") && l.contains("--"))
        && !help_lower.contains("optional arguments:")
    {
        return ParserType::RustClap;
    }

    // Go flag patterns (simple flag list)
    if help_lower
        .lines()
        .any(|l| l.trim().starts_with("-") && l.contains("\t") && l.len() < 80)
    {
        return ParserType::GoFlag;
    }

    // Default: generic regex parser
    ParserType::Generic
}

/// Parser type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserType {
    PythonArgparse,
    RustClap,
    GoFlag,
    Generic,
}

impl ParserType {
    /// Get the parser implementation for this type
    pub fn parser(&self) -> Box<dyn HelpParser> {
        match self {
            ParserType::PythonArgparse => Box::new(python::PythonArgparseParser),
            ParserType::RustClap => Box::new(generic::GenericParser), // TODO: specialized clap parser
            ParserType::GoFlag => Box::new(generic::GenericParser),   // TODO: specialized go parser
            ParserType::Generic => Box::new(generic::GenericParser),
        }
    }
}

/// Parse help output using the appropriate parser
pub fn parse_help(tool: &str, help: &str) -> CliSchema {
    let parser_type = detect_parser_type(help);
    let parser = parser_type.parser();

    if parser.can_parse(help) {
        parser.parse(tool, help)
    } else {
        // Fallback to generic parser
        generic::GenericParser.parse(tool, help)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_python_argparse() {
        let help = "usage: tool [options]\noptional arguments:\n  -h, --help  show this help message and exit";
        assert_eq!(detect_parser_type(help), ParserType::PythonArgparse);
    }

    #[test]
    fn test_detect_generic() {
        let help = "Tool usage: tool input output\nFlags: -v verbose";
        assert_eq!(detect_parser_type(help), ParserType::Generic);
    }
}
