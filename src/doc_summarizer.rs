//! Intelligent documentation summarization for LLM consumption.
//!
//! Extracts key information from tool documentation while keeping prompts concise.

use regex::Regex;
use std::collections::HashSet;

/// Maximum documentation length for different model tiers (in characters)
pub const MAX_DOC_LEN_SMALL_MODEL: usize = 3_000; // For 0.5B-1B models
pub const MAX_DOC_LEN_MEDIUM_MODEL: usize = 6_000; // For 7B models
pub const MAX_DOC_LEN_LARGE_MODEL: usize = 10_000; // For 16B+ models

/// Minimum length threshold for summarization (docs shorter than this are kept as-is)
pub const MIN_SUMMARIZE_LEN: usize = 2_000;

/// Key sections to prioritize when summarizing
const PRIORITY_SECTIONS: &[&str] = &[
    "usage",
    "options",
    "arguments",
    "examples",
    "parameters",
    "flags",
    "commands",
];

/// Patterns to filter out (noise in help text)
const NOISE_PATTERNS: &[&str] = &[
    r"For more information.*",
    r"Report bugs to.*",
    r"See the full documentation.*",
    r"Homepage:.*",
    r"Version:.*", // We add version separately
];

/// Summarize tool documentation for efficient LLM consumption.
///
/// This function:
/// 1. Extracts key sections (usage, options, examples)
/// 2. Filters out noise (bug reports, links, etc.)
/// 3. Limits total length while preserving critical information
/// 4. Structures output for LLM readability
pub fn summarize_docs(docs: &str, max_len: usize) -> String {
    // Don't summarize short docs
    if docs.len() <= MIN_SUMMARIZE_LEN {
        return docs.to_string();
    }

    // Step 1: Clean noise
    let cleaned = clean_noise(docs);

    // Step 2: Extract key sections
    let sections = extract_key_sections(&cleaned);

    // Step 3: Build structured summary
    let summary = build_structured_summary(&sections);

    // Step 4: Truncate if still too long
    if summary.len() > max_len {
        truncate_smart(&summary, max_len)
    } else {
        summary
    }
}

/// Remove noise patterns from documentation
fn clean_noise(docs: &str) -> String {
    let mut cleaned = docs.to_string();

    for pattern in NOISE_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            cleaned = re.replace_all(&cleaned, "").to_string();
        }
    }

    // Remove excessive blank lines
    let blank_line_re = Regex::new(r"\n{3,}").unwrap();
    let cleaned = blank_line_re.replace_all(&cleaned, "\n\n").to_string();

    cleaned.trim().to_string()
}

/// Extract key sections from documentation
fn extract_key_sections(docs: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    let mut current_section = String::new();
    let mut current_content = String::new();
    let mut in_priority_section = false;

    for line in &lines {
        let trimmed = line.trim();

        // Detect section headers (lines starting with capital letters or common patterns)
        let is_header = is_section_header(trimmed);

        if is_header {
            // Save previous section if it exists
            if !current_section.is_empty() && !current_content.is_empty() {
                let priority = PRIORITY_SECTIONS
                    .iter()
                    .any(|&s| current_section.to_lowercase().contains(s));

                if priority || in_priority_section {
                    sections.push((current_section.clone(), current_content.clone()));
                }
            }

            // Start new section
            current_section = trimmed.to_string();
            current_content = String::new();
            in_priority_section = PRIORITY_SECTIONS
                .iter()
                .any(|&s| trimmed.to_lowercase().contains(s));
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

/// Check if a line is likely a section header
fn is_section_header(line: &str) -> bool {
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

    // Check for underlined headers (next line is all dashes or equals)
    false
}

/// Build a structured summary from extracted sections
fn build_structured_summary(sections: &[(String, String)]) -> String {
    let mut summary = String::new();

    // Group sections by priority
    let mut priority_sections: Vec<(String, String)> = Vec::new();
    let mut other_sections: Vec<(String, String)> = Vec::new();

    for (title, content) in sections {
        let is_priority = PRIORITY_SECTIONS
            .iter()
            .any(|&s| title.to_lowercase().contains(s));

        if is_priority {
            priority_sections.push((title.clone(), content.clone()));
        } else {
            other_sections.push((title.clone(), content.clone()));
        }
    }

    // Add priority sections first
    for (title, content) in priority_sections {
        if !summary.is_empty() {
            summary.push_str("\n\n");
        }
        summary.push_str(&format!("## {}\n", title));
        summary.push_str(content.trim());
    }

    // Add other sections if we have space
    for (title, content) in other_sections {
        if !summary.is_empty() {
            summary.push_str("\n\n");
        }
        summary.push_str(&format!("## {}\n", title));
        summary.push_str(content.trim());
    }

    summary
}

/// Smart truncation that preserves complete lines and sections
fn truncate_smart(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    // Find a good break point near max_len
    let truncate_at = max_len.saturating_sub(50); // Leave room for "..."

    // Try to break at a paragraph boundary
    if let Some(pos) = text[..truncate_at].rfind("\n\n") {
        let truncated = &text[..pos];
        return format!("{}\n\n... [documentation truncated for brevity]", truncated);
    }

    // Try to break at a line boundary
    if let Some(pos) = text[..truncate_at].rfind('\n') {
        let truncated = &text[..pos];
        return format!("{}\n\n... [documentation truncated for brevity]", truncated);
    }

    // Last resort: hard truncate
    format!(
        "{}... [documentation truncated for brevity]",
        &text[..truncate_at]
    )
}

/// Extract flags/options from documentation for quick reference
#[allow(dead_code)]
pub fn extract_flags(docs: &str) -> Vec<String> {
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

/// Extract examples from documentation
#[allow(dead_code)]
pub fn extract_examples(docs: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    let mut in_example = false;
    let mut current_example = String::new();

    for line in &lines {
        let trimmed = line.trim();

        // Detect example start
        if trimmed.to_lowercase().contains("example") {
            if !current_example.is_empty() {
                examples.push(current_example.trim().to_string());
                current_example = String::new();
            }
            in_example = true;
        } else if in_example {
            // Check if we've left the example section
            if trimmed.is_empty() && !current_example.is_empty() {
                examples.push(current_example.trim().to_string());
                current_example = String::new();
                in_example = false;
            } else {
                current_example.push_str(line);
                current_example.push('\n');
            }
        }
    }

    // Don't forget the last example
    if !current_example.is_empty() {
        examples.push(current_example.trim().to_string());
    }

    // Limit to top 3 examples
    examples.truncate(3);
    examples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_short_docs() {
        let short_doc = "Usage: tool [options]\n\nThis is a short doc.";
        let result = summarize_docs(short_doc, 1000);
        assert_eq!(result, short_doc);
    }

    #[test]
    fn test_clean_noise() {
        let noisy = "Usage: tool\n\nFor more information see https://example.com\nReport bugs to bugs@example.com";
        let cleaned = clean_noise(noisy);
        assert!(!cleaned.contains("For more information"));
        assert!(!cleaned.contains("Report bugs"));
    }

    #[test]
    fn test_extract_flags() {
        let doc = "Usage: tool --help --version -v -q";
        let flags = extract_flags(doc);
        assert!(flags.contains(&"--help".to_string()));
        assert!(flags.contains(&"--version".to_string()));
        assert!(flags.contains(&"-v".to_string()));
        assert!(flags.contains(&"-q".to_string()));
    }

    #[test]
    fn test_truncate_smart() {
        let text = "Line 1\n\nLine 2\n\nLine 3\n\nLine 4";
        let truncated = truncate_smart(text, 20);
        assert!(truncated.contains("[documentation truncated"));
        assert!(truncated.ends_with("]"));
    }
}
