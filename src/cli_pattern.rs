//! CLI Pattern Classification Module
//!
//! This module provides intelligent classification of CLI tools into different
//! pattern categories, enabling tailored processing strategies for each type.

#![allow(dead_code)]

use regex::Regex;
use std::sync::LazyLock;

/// CLI tool pattern types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliPattern {
    /// Simple tool: awk, git, curl
    /// Usage: tool [options] [arguments]
    Simple,

    /// Subcommand-based tool: samtools, bcftools, bedtools, gatk
    /// Usage: tool <subcommand> [options] [arguments]
    Subcommand {
        /// Default subcommand if any
        default: Option<String>,
    },

    /// Meta-tool/wrapper: deeptools, picard
    /// Usage: tool <module> [options] or standalone modules
    MetaTool {
        /// Whether command requires tool name prefix
        requires_prefix: bool,
    },

    /// Multi-entry tool: agat (multiple binaries sharing docs)
    /// Usage: tool_subcommand [options]
    MultiEntry {
        /// Available entry points
        entries: Vec<String>,
    },
}

impl CliPattern {
    /// Get a human-readable description of the pattern
    pub fn description(&self) -> &'static str {
        match self {
            CliPattern::Simple => "Simple command with flags",
            CliPattern::Subcommand { .. } => "Subcommand-based tool",
            CliPattern::MetaTool { .. } => "Meta-tool with modules",
            CliPattern::MultiEntry { .. } => "Multi-entry tool suite",
        }
    }

    /// Check if this pattern requires subcommand detection
    pub fn requires_subcommand(&self) -> bool {
        matches!(
            self,
            CliPattern::Subcommand { .. } | CliPattern::MultiEntry { .. }
        )
    }

    /// Check if this pattern is a meta-tool
    pub fn is_meta_tool(&self) -> bool {
        matches!(self, CliPattern::MetaTool { .. })
    }
}

/// Pre-compiled regex patterns for classification
static SUBCOMMAND_USAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)usage:\s*\S+\s+<command|subcommand|cmd>").unwrap());

static COMMANDS_SECTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(commands|available\s+commands|subcommands):\s*$").unwrap()
});

static AGAT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)another\s+gff\s+analysis\s+toolkit|agat_\w+").unwrap());

static META_TOOL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(available\s+modules|computeMatrix|plotHeatmap|bamCoverage)").unwrap()
});

static PICARD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)picard\.jar|broad\s+institute").unwrap());

/// CLI Pattern Classifier
#[derive(Debug, Clone, Default)]
pub struct CliPatternClassifier;

impl CliPatternClassifier {
    /// Create a new classifier
    pub fn new() -> Self {
        Self
    }

    /// Classify a tool based on its documentation and name
    pub fn classify(&self, raw_doc: &str, tool_name: &str) -> CliPattern {
        let _doc_lower = raw_doc.to_lowercase();
        let name_lower = tool_name.to_lowercase();

        // Check 1: Multi-entry tools (agat)
        if self.is_multi_entry_tool(&name_lower, raw_doc) {
            return CliPattern::MultiEntry {
                entries: self.extract_multi_entry_points(raw_doc, tool_name),
            };
        }

        // Check 2: Meta-tools (deeptools, picard)
        if self.is_meta_tool(raw_doc, &name_lower) {
            return CliPattern::MetaTool {
                requires_prefix: !name_lower.contains("picard"),
            };
        }

        // Check 3: Subcommand-based tools
        if self.is_subcommand_tool(raw_doc, &name_lower) {
            // Check for default subcommand
            let default = self.detect_default_subcommand(raw_doc, tool_name);
            return CliPattern::Subcommand { default };
        }

        // Default: Simple tool
        CliPattern::Simple
    }

    /// Check if tool is a multi-entry suite like agat
    fn is_multi_entry_tool(&self, name_lower: &str, raw_doc: &str) -> bool {
        name_lower.starts_with("agat") || AGAT_RE.is_match(raw_doc)
    }

    /// Extract entry points for multi-entry tools
    fn extract_multi_entry_points(&self, raw_doc: &str, tool_name: &str) -> Vec<String> {
        let mut entries = vec![];

        // For agat, look for agat_* commands in documentation
        let agat_cmd_re = Regex::new(r"agat_\w+").unwrap();
        for cap in agat_cmd_re.captures_iter(raw_doc) {
            if let Some(m) = cap.get(0) {
                let cmd = m.as_str().to_string();
                if !entries.contains(&cmd) {
                    entries.push(cmd);
                }
            }
        }

        // Also check the tool name itself if it follows the pattern
        if tool_name.starts_with("agat_") && !entries.contains(&tool_name.to_string()) {
            entries.push(tool_name.to_string());
        }

        // If no entries found, assume it's the tool name
        if entries.is_empty() {
            entries.push(tool_name.to_string());
        }

        entries.sort();
        entries
    }

    /// Check if tool is a meta-tool
    fn is_meta_tool(&self, raw_doc: &str, name_lower: &str) -> bool {
        META_TOOL_RE.is_match(raw_doc)
            || PICARD_RE.is_match(raw_doc)
            || name_lower.contains("deeptools")
    }

    /// Check if tool uses subcommands
    fn is_subcommand_tool(&self, raw_doc: &str, name_lower: &str) -> bool {
        // Check usage pattern
        if SUBCOMMAND_USAGE_RE.is_match(raw_doc) {
            return true;
        }

        // Check for commands section
        if COMMANDS_SECTION_RE.is_match(raw_doc) {
            return true;
        }

        // Known subcommand tools by name
        let known_subcommand_tools = [
            "samtools",
            "bcftools",
            "bedtools",
            "gatk",
            "tabix",
            "htsfile",
            "bamtools",
            "jvarkit",
            "bioalcidaejdk",
        ];
        if known_subcommand_tools.contains(&name_lower) {
            return true;
        }

        // Look for common patterns in documentation
        let doc_lower = raw_doc.to_lowercase();
        if doc_lower.contains("<command> [options]")
            || doc_lower.contains("<command> [flags]")
            || doc_lower.contains("available commands:")
        {
            return true;
        }

        false
    }

    /// Try to detect a default subcommand
    fn detect_default_subcommand(&self, raw_doc: &str, tool_name: &str) -> Option<String> {
        let doc_lower = raw_doc.to_lowercase();

        // Some tools have a default subcommand
        if tool_name == "gatk" && doc_lower.contains("markduplicates") {
            // GATK doesn't really have a default, but we can suggest common ones
            return None;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple_tools() {
        let classifier = CliPatternClassifier::new();

        // awk - simple tool
        let awk_doc = "Usage: awk [options] 'program' [file]\n\nOptions:\n  -F separator";
        assert_eq!(classifier.classify(awk_doc, "awk"), CliPattern::Simple);

        // git - actually subcommand-based
        let git_doc = "Usage: git <command> [<args>]\n\nCommands:\n  commit\n  push";
        assert!(matches!(
            classifier.classify(git_doc, "git"),
            CliPattern::Subcommand { .. }
        ));
    }

    #[test]
    fn test_classify_subcommand_tools() {
        let classifier = CliPatternClassifier::new();

        // samtools
        let samtools_doc = "Usage: samtools <command> [options]\n\nCommands:\n  sort\n  index";
        let pattern = classifier.classify(samtools_doc, "samtools");
        assert!(pattern.requires_subcommand());

        // bcftools
        let bcftools_doc = "Usage: bcftools <command> [options]\n\nCommands:\n  view\n  filter";
        let pattern = classifier.classify(bcftools_doc, "bcftools");
        assert!(pattern.requires_subcommand());
    }

    #[test]
    fn test_classify_agat() {
        let classifier = CliPatternClassifier::new();

        let agat_doc =
            "Another GFF Analysis Toolkit\n\nUsage: agat_convert_sp_gff2gtf.pl [options]";
        let pattern = classifier.classify(agat_doc, "agat");
        assert!(matches!(pattern, CliPattern::MultiEntry { .. }));

        if let CliPattern::MultiEntry { entries } = pattern {
            assert!(!entries.is_empty());
        }
    }
}
