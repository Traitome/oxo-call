//! Intelligent documentation summarization for LLM consumption.
//!
//! Extracts key information from tool documentation while keeping prompts concise.
//!
//! ## Shared primitives
//!
//! Noise removal, section-header detection, section extraction, and flag
//! extraction are implemented once in [`crate::doc_processor`] and re-used
//! here to avoid code duplication.

use std::collections::HashSet;

// Re-use canonical implementations from doc_processor.
use crate::doc_processor::{
    clean_noise, extract_flags_standalone, extract_sections_standalone, truncate_smart,
};

/// Maximum documentation length for different model tiers (in characters)
pub const MAX_DOC_LEN_SMALL_MODEL: usize = 3_000; // For 0.5B-1B models
pub const MAX_DOC_LEN_MEDIUM_MODEL: usize = 6_000; // For 7B models
pub const MAX_DOC_LEN_LARGE_MODEL: usize = 10_000; // For 16B+ models

/// Minimum length threshold for summarization (docs shorter than this are kept as-is)
pub const MIN_SUMMARIZE_LEN: usize = 2_000;

/// Key sections to prioritize when summarizing
const PRIORITY_SECTIONS: &[&str; 7] = &[
    "usage",
    "options",
    "arguments",
    "examples",
    "parameters",
    "flags",
    "commands",
];

/// Summarize tool documentation for efficient LLM consumption.
///
/// This function:
/// 1. Extracts key sections (usage, options, examples)
/// 2. Filters out noise (bug reports, links, etc.)
/// 3. Limits total length while preserving critical information
/// 4. Structures output for LLM readability
/// 5. Highlights USAGE and critical flags
pub fn summarize_docs(docs: &str, max_len: usize) -> String {
    // Don't summarize short docs
    if docs.len() <= MIN_SUMMARIZE_LEN {
        return format_for_llm(docs);
    }

    // Step 1: Clean noise (shared primitive from doc_processor)
    let cleaned = clean_noise(docs);

    // Step 2: Extract key sections — use shared primitive, then filter to
    // priority sections only (the summarizer is stricter than doc_processor).
    let all_sections = extract_sections_standalone(&cleaned);
    let sections: Vec<(String, String)> = all_sections
        .into_iter()
        .filter(|(title, _)| {
            let lower = title.to_lowercase();
            PRIORITY_SECTIONS.iter().any(|&s| lower.contains(s)) || title == "Documentation" // keep the fallback section
        })
        .collect();
    let sections = if sections.is_empty() {
        vec![("Documentation".to_string(), cleaned)]
    } else {
        sections
    };

    // Step 3: Build structured summary optimized for LLM
    let summary = build_llm_optimized_summary(&sections, max_len);

    // Step 4: Truncate if still too long (shared primitive)
    if summary.len() > max_len {
        truncate_smart(&summary, max_len)
    } else {
        summary
    }
}

/// Format documentation for LLM consumption, highlighting key patterns
fn format_for_llm(docs: &str) -> String {
    let mut formatted = String::new();
    let lines: Vec<&str> = docs.lines().collect();

    for line in lines {
        let _trimmed = line.trim();

        // Keep other lines
        formatted.push_str(&format!("{}\n", line));
    }

    formatted.trim().to_string()
}

// NOTE: clean_noise, is_section_header, extract_sections, extract_flags, and
// truncate_smart were formerly defined here.  They are now canonical in
// `crate::doc_processor` and imported at the top of this file.

/// Build LLM-optimized summary with clear structure and highlighted patterns
fn build_llm_optimized_summary(sections: &[(String, String)], max_len: usize) -> String {
    let mut summary = String::new();

    // Priority order for LLM understanding
    let priority_order = [
        "usage",      // Most critical - shows command structure
        "examples",   // Concrete examples for LLM to learn from
        "options",    // Available flags
        "arguments",  // Required inputs
        "commands",   // Subcommands
        "parameters", // Additional parameters
        "flags",      // Alternative to options
    ];

    // Sort sections by priority
    let mut sorted_sections: Vec<(usize, &String, &String)> = sections
        .iter()
        .map(|(title, content)| {
            let priority = priority_order
                .iter()
                .position(|p| title.to_lowercase().contains(p))
                .unwrap_or(999);
            (priority, title, content)
        })
        .collect();

    sorted_sections.sort_by_key(|(p, _, _)| *p);

    // Build summary with clear markers
    for (priority, title, content) in sorted_sections {
        if summary.len() > (max_len as f64 * 0.8) as usize {
            break; // Stop before exceeding limit
        }

        if !summary.is_empty() {
            summary.push_str("\n\n");
        }

        // Add clear section marker for LLM
        let title_lower = title.to_lowercase();
        if title_lower.contains("usage") {
            summary.push_str("=== USAGE (command structure) ===\n");
        } else if title_lower.contains("example") {
            summary.push_str("=== EXAMPLES (learn from these) ===\n");
        } else if title_lower.contains("option") || title_lower.contains("flag") {
            summary.push_str("=== OPTIONS/FLAGS ===\n");
        } else {
            summary.push_str(&format!("=== {} ===\n", title));
        }

        // Format content for LLM readability
        let formatted_content = format_section_content(content, priority);
        summary.push_str(&formatted_content);
    }

    // Add quick reference if space available
    if summary.len() < (max_len as f64 * 0.9) as usize {
        let flags = extract_flags_from_sections(sections);
        if !flags.is_empty() && summary.len() + 200 < max_len {
            summary.push_str("\n\n=== QUICK REFERENCE FLAGS ===\n");
            summary.push_str(&flags.join(" "));
        }
    }

    summary
}

/// Format section content for better LLM understanding
fn format_section_content(content: &str, priority: usize) -> String {
    let mut formatted = String::new();
    let lines: Vec<&str> = content.lines().collect();

    // For USAGE (highest priority), highlight the pattern
    if priority == 0 {
        for line in lines.iter().take(5) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // Highlight command patterns
                if trimmed.contains('[') || trimmed.contains('<') || trimmed.contains('|') {
                    formatted.push_str(&format!(">>> {}\n", trimmed));
                } else {
                    formatted.push_str(&format!("{}\n", trimmed));
                }
            }
        }
        return formatted;
    }

    // For EXAMPLES, show concrete commands
    if priority == 1 {
        for line in lines.iter().take(10) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                // Highlight actual command lines (often start with $ or tool name)
                if trimmed.starts_with('$') || trimmed.starts_with('#') {
                    formatted.push_str(&format!("{}\n", trimmed));
                } else if trimmed.contains("  ") && !trimmed.starts_with('-') {
                    // Likely a command description
                    formatted.push_str(&format!("{}\n", trimmed));
                } else {
                    formatted.push_str(&format!("{}\n", trimmed));
                }
            }
        }
        return formatted;
    }

    // For OPTIONS/FLAGS, format as compact list
    if priority <= 3 {
        for line in lines.iter().take(20) {
            let trimmed = line.trim();
            if trimmed.starts_with('-') {
                // Flag line - keep it
                formatted.push_str(&format!("{}\n", trimmed));
            } else if !trimmed.is_empty() && formatted.len() < 500 {
                // Description - keep brief
                formatted.push_str(&format!("{}\n", trimmed));
            }
        }
        return formatted;
    }

    // Default: just trim and limit
    content.lines().take(15).collect::<Vec<_>>().join("\n")
}

/// Extract flags from all sections for quick reference.
///
/// Uses the shared [`extract_flags_standalone`] from `doc_processor`.
fn extract_flags_from_sections(sections: &[(String, String)]) -> Vec<String> {
    let mut all_flags = HashSet::new();

    for (_, content) in sections {
        let flags = extract_flags_standalone(content);
        all_flags.extend(flags);
    }

    let mut sorted: Vec<String> = all_flags.into_iter().collect();
    sorted.sort();
    sorted.truncate(30);
    sorted
}

/// Re-export of the canonical flag extractor from `doc_processor`.
#[allow(dead_code)]
pub fn extract_flags(docs: &str) -> Vec<String> {
    extract_flags_standalone(docs)
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
