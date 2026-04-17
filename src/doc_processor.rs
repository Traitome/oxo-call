//! Intelligent lossless documentation processing for LLM consumption.
//!
//! This module provides smart documentation cleaning and structuring without
//! destructive compression. It preserves complete USAGE, EXAMPLES, and key
//! sections while removing only noise and redundancy.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Structured documentation with separated sections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StructuredDoc {
    /// Complete USAGE section (command structure)
    pub usage: String,
    /// Complete EXAMPLES section (concrete examples)
    pub examples: String,
    /// Compressed OPTIONS section (flags with brief descriptions)
    pub options: String,
    /// Subcommands list
    pub commands: String,
    /// Other useful information (description, parameters, etc.)
    pub other: String,
    /// Quick reference flags extracted from all sections
    pub quick_flags: Vec<String>,
}

impl fmt::Display for StructuredDoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();

        // USAGE - most important
        if !self.usage.is_empty() {
            output.push_str("=== USAGE ===\n");
            output.push_str(self.usage.trim());
            output.push_str("\n\n");
        }

        // EXAMPLES - concrete usage patterns
        if !self.examples.is_empty() {
            output.push_str("=== EXAMPLES ===\n");
            output.push_str(self.examples.trim());
            output.push_str("\n\n");
        }

        // COMMANDS - available subcommands
        if !self.commands.is_empty() {
            output.push_str("=== SUBCOMMANDS ===\n");
            output.push_str(&self.commands);
            output.push_str("\n\n");
        }

        // OPTIONS - compressed flags
        if !self.options.is_empty() {
            output.push_str("=== OPTIONS/FLAGS ===\n");
            output.push_str(self.options.trim());
            output.push_str("\n\n");
        }

        // Other useful info
        if !self.other.is_empty() {
            output.push_str(self.other.trim());
            output.push_str("\n\n");
        }

        // Quick reference flags
        if !self.quick_flags.is_empty() {
            output.push_str("=== QUICK REFERENCE FLAGS ===\n");
            output.push_str(
                &self
                    .quick_flags
                    .iter()
                    .take(30)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        write!(f, "{}", output.trim())
    }
}

/// Document processor with noise patterns and key section identifiers
#[allow(dead_code)]
pub struct DocProcessor {
    /// Noise patterns to remove
    noise_patterns: Vec<Regex>,
    /// Key section identifiers (complete preservation)
    #[allow(dead_code)]
    key_sections: Vec<String>,
}

impl Default for DocProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocProcessor {
    /// Create a new document processor with default patterns
    pub fn new() -> Self {
        let noise_patterns = vec![
            Regex::new(r"For more information.*").unwrap(),
            Regex::new(r"Report bugs to.*").unwrap(),
            Regex::new(r"See the full documentation.*").unwrap(),
            Regex::new(r"Homepage:.*").unwrap(),
            Regex::new(r"^\s*Version:.*$").unwrap(), // Only standalone version lines
            Regex::new(r"^\s*$").unwrap(),           // Empty lines (will be collapsed later)
        ];

        let key_sections = vec![
            "usage".to_string(),
            "examples".to_string(),
            "options".to_string(),
            "arguments".to_string(),
            "commands".to_string(),
            "parameters".to_string(),
            "flags".to_string(),
        ];

        DocProcessor {
            noise_patterns,
            key_sections,
        }
    }

    /// Process documentation (alias for clean_and_structure)
    pub fn process(&self, docs: &str) -> StructuredDoc {
        self.clean_and_structure(docs)
    }

    /// Clean and structure documentation for LLM consumption
    ///
    /// This is the main entry point for lossless documentation processing:
    /// 1. Remove noise (bug reports, links, version info)
    /// 2. Extract and preserve complete USAGE and EXAMPLES
    /// 3. Compress OPTIONS to essential flags
    /// 4. Extract subcommands
    /// 5. Build quick reference flags
    pub fn clean_and_structure(&self, docs: &str) -> StructuredDoc {
        // Step 1: Remove noise
        let cleaned = self.remove_noise(docs);

        // Step 2: Extract sections
        let sections = self.extract_sections(&cleaned);

        // Step 3: Build structured output
        let mut structured = StructuredDoc {
            usage: String::new(),
            examples: String::new(),
            options: String::new(),
            commands: String::new(),
            other: String::new(),
            quick_flags: Vec::new(),
        };

        for (section_name, content) in sections {
            let name_lower = section_name.to_lowercase();

            if name_lower.contains("usage") {
                structured.usage = content.clone();
            } else if name_lower.contains("example") {
                structured.examples.push_str(&content);
                structured.examples.push('\n');
            } else if name_lower.contains("option") || name_lower.contains("flag") {
                structured
                    .options
                    .push_str(&self.compress_options(&content));
                structured.options.push('\n');
            } else if name_lower.contains("command") {
                structured.commands = self.extract_subcommands(&content);
            } else if name_lower.contains("argument") || name_lower.contains("parameter") {
                structured
                    .other
                    .push_str(&format!("=== {} ===\n", section_name));
                structured.other.push_str(&content);
                structured.other.push_str("\n\n");
            }
        }

        // Step 4: Extract quick reference flags
        structured.quick_flags = self.extract_all_flags(&cleaned);

        structured
    }

    /// Process documentation for LLM with intelligent formatting
    ///
    /// This produces a LLM-ready string with clear section markers,
    /// preserving complete USAGE and EXAMPLES while compressing OPTIONS.
    #[allow(dead_code)]
    pub fn process_for_llm(&self, docs: &str) -> String {
        let structured = self.clean_and_structure(docs);
        self.format_structured_doc(&structured)
    }

    /// Format structured documentation for LLM consumption
    #[allow(dead_code)]
    fn format_structured_doc(&self, doc: &StructuredDoc) -> String {
        let mut output = String::new();

        // USAGE - most critical, show first
        if !doc.usage.is_empty() {
            output.push_str("=== USAGE (command structure) ===\n");
            output.push_str(&doc.usage);
            output.push_str("\n\n");
        }

        // EXAMPLES - concrete examples for learning
        if !doc.examples.is_empty() {
            output.push_str("=== EXAMPLES (learn from these) ===\n");
            output.push_str(doc.examples.trim());
            output.push_str("\n\n");
        }

        // COMMANDS - available subcommands
        if !doc.commands.is_empty() {
            output.push_str("=== SUBCOMMANDS ===\n");
            output.push_str(&doc.commands);
            output.push_str("\n\n");
        }

        // OPTIONS - compressed flags
        if !doc.options.is_empty() {
            output.push_str("=== OPTIONS/FLAGS ===\n");
            output.push_str(doc.options.trim());
            output.push_str("\n\n");
        }

        // Other useful info
        if !doc.other.is_empty() {
            output.push_str(doc.other.trim());
            output.push_str("\n\n");
        }

        // Quick reference flags
        if !doc.quick_flags.is_empty() {
            output.push_str("=== QUICK REFERENCE FLAGS ===\n");
            output.push_str(
                &doc.quick_flags
                    .iter()
                    .take(30)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(" "),
            );
        }

        output.trim().to_string()
    }

    /// Remove noise patterns from documentation
    fn remove_noise(&self, docs: &str) -> String {
        let mut cleaned = docs.to_string();

        // Apply noise patterns
        for pattern in &self.noise_patterns {
            cleaned = pattern.replace_all(&cleaned, "").to_string();
        }

        // Collapse multiple blank lines to double newline
        let blank_line_re = Regex::new(r"\n{3,}").unwrap();
        cleaned = blank_line_re.replace_all(&cleaned, "\n\n").to_string();

        cleaned.trim().to_string()
    }

    /// Extract sections from documentation
    fn extract_sections(&self, docs: &str) -> Vec<(String, String)> {
        let mut sections = Vec::new();
        let lines: Vec<&str> = docs.lines().collect();

        let mut current_section = String::new();
        let mut current_content = String::new();

        for line in &lines {
            let trimmed = line.trim();

            // Check if this is a section header
            if self.is_section_header(trimmed) {
                // Save previous section
                if !current_section.is_empty() && !current_content.is_empty() {
                    sections.push((current_section.clone(), current_content.clone()));
                }

                // Start new section
                current_section = trimmed.to_string();
                current_content = String::new();
            } else {
                // Add content to current section
                if !current_section.is_empty() {
                    current_content.push_str(line);
                    current_content.push('\n');
                }
            }
        }

        // Don't forget the last section
        if !current_section.is_empty() && !current_content.is_empty() {
            sections.push((current_section, current_content));
        }

        // If no sections found, treat entire doc as one section
        if sections.is_empty() {
            sections.push(("Documentation".to_string(), docs.to_string()));
        }

        sections
    }

    /// Check if a line is a section header
    fn is_section_header(&self, line: &str) -> bool {
        if line.is_empty() {
            return false;
        }

        // Common section header patterns
        let header_patterns = [
            "USAGE:",
            "Usage:",
            "OPTIONS:",
            "Options:",
            "ARGUMENTS:",
            "Arguments:",
            "EXAMPLES:",
            "Examples:",
            "PARAMETERS:",
            "Parameters:",
            "FLAGS:",
            "Flags:",
            "COMMANDS:",
            "Commands:",
            "DESCRIPTION:",
            "Description:",
            "SYNOPSIS:",
            "Synopsis:",
        ];

        // Check exact matches
        if header_patterns.iter().any(|p| line.starts_with(p)) {
            return true;
        }

        // Check for all-caps headers with colon
        if line.ends_with(':') && line.chars().filter(|c| c.is_uppercase()).count() > 3 {
            return true;
        }

        false
    }

    /// Compress OPTIONS section to essential flags
    fn compress_options(&self, content: &str) -> String {
        let mut compressed = String::new();
        let lines: Vec<&str> = content.lines().collect();

        for line in lines.iter().take(30) {
            let trimmed = line.trim();

            // Keep flag lines
            if trimmed.starts_with('-') {
                compressed.push_str(trimmed);
                compressed.push('\n');
            } else if trimmed.starts_with('<') || trimmed.starts_with('[') {
                // Keep placeholder descriptions
                compressed.push_str(trimmed);
                compressed.push('\n');
            }
        }

        compressed.trim().to_string()
    }

    /// Extract subcommands from COMMANDS section
    fn extract_subcommands(&self, content: &str) -> String {
        let mut commands = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for line in lines.iter().take(20) {
            let trimmed = line.trim();

            // Extract subcommand names (usually first word on line)
            if let Some(first_word) = trimmed.split_whitespace().next() {
                // Skip if it looks like a flag or placeholder
                if !first_word.starts_with('-')
                    && !first_word.starts_with('<')
                    && !first_word.starts_with('[')
                {
                    commands.push(first_word.to_string());
                }
            }
        }

        commands.sort();
        commands.dedup();
        commands.join(", ")
    }

    /// Extract all flags from documentation
    fn extract_all_flags(&self, docs: &str) -> Vec<String> {
        let mut flags = HashSet::new();
        let flag_re = Regex::new(r"(?:^|\s)(-{1,2}[a-zA-Z0-9_-]+)").unwrap();

        for cap in flag_re.captures_iter(docs) {
            if let Some(flag) = cap.get(1) {
                flags.insert(flag.as_str().to_string());
            }
        }

        let mut flags_vec: Vec<String> = flags.into_iter().collect();
        flags_vec.sort();
        flags_vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_noise() {
        let processor = DocProcessor::new();
        let noisy = "Usage: tool\n\nFor more information see https://example.com\nReport bugs to bugs@example.com";
        let cleaned = processor.remove_noise(noisy);
        assert!(!cleaned.contains("For more information"));
        assert!(!cleaned.contains("Report bugs"));
    }

    #[test]
    fn test_extract_sections() {
        let processor = DocProcessor::new();
        let doc =
            "USAGE:\n  tool [options]\n\nOPTIONS:\n  --help  Show help\n\nEXAMPLES:\n  tool --help";
        let sections = processor.extract_sections(doc);

        assert_eq!(sections.len(), 3);
        assert!(sections[0].0.contains("USAGE"));
        assert!(sections[1].0.contains("OPTIONS"));
        assert!(sections[2].0.contains("EXAMPLES"));
    }

    #[test]
    fn test_clean_and_structure() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  tool [options] <input>\n\nOPTIONS:\n  --help  Show help\n  --version  Show version\n\nEXAMPLES:\n  tool --help\n  tool input.txt";

        let structured = processor.clean_and_structure(doc);

        assert!(structured.usage.contains("tool [options]"));
        assert!(structured.examples.contains("tool --help"));
        assert!(structured.options.contains("--help"));
        assert!(!structured.quick_flags.is_empty());
    }

    #[test]
    fn test_process_for_llm() {
        let processor = DocProcessor::new();
        let doc =
            "USAGE:\n  tool [options]\n\nOPTIONS:\n  --help  Show help\n\nEXAMPLES:\n  tool --help";

        let formatted = processor.process_for_llm(doc);

        assert!(formatted.contains("=== USAGE"));
        assert!(formatted.contains("=== EXAMPLES"));
        assert!(formatted.contains("=== OPTIONS"));
    }

    #[test]
    fn test_compress_options() {
        let processor = DocProcessor::new();
        let options = "  --help     Show this help message\n  --version  Show version\n  --output   Output file\n     Description text that should be kept minimal";

        let compressed = processor.compress_options(options);

        assert!(compressed.contains("--help"));
        assert!(compressed.contains("--version"));
        assert!(compressed.contains("--output"));
        // Should be compact
        assert!(compressed.len() < options.len());
    }

    #[test]
    fn test_extract_subcommands() {
        let processor = DocProcessor::new();
        let commands =
            "  sort     Sort BAM file\n  view     View BAM file\n  index    Index BAM file";

        let extracted = processor.extract_subcommands(commands);

        assert!(extracted.contains("sort"));
        assert!(extracted.contains("view"));
        assert!(extracted.contains("index"));
    }
}
