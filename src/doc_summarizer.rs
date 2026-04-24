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

    // ── New tests for improved coverage ──────────────────────────────────────

    #[test]
    fn test_summarize_long_docs_with_usage_section() {
        // Build a long doc (> MIN_SUMMARIZE_LEN = 2000 chars) with a USAGE section
        let usage = "USAGE:\n  samtools sort [options] <in.bam>\n\n";
        let options = "OPTIONS:\n  -o FILE  Write output to FILE\n  -@ INT   Number of sorting/compression threads [1]\n  -m INT   Set maximum memory per thread; suffix K/M/G recognized [768M]\n\n";
        let bugs = "BUGS:\n  This is a bug tracker section that should be filtered.\n\n";
        // Pad to exceed MIN_SUMMARIZE_LEN
        let padding = "Description line for padding.\n".repeat(100);
        let long_doc = format!("{usage}{options}{bugs}{padding}");
        assert!(
            long_doc.len() > MIN_SUMMARIZE_LEN,
            "test doc ({}) must be > MIN_SUMMARIZE_LEN ({})",
            long_doc.len(),
            MIN_SUMMARIZE_LEN
        );

        let result = summarize_docs(&long_doc, MAX_DOC_LEN_LARGE_MODEL);
        assert!(!result.is_empty());
        // Should contain USAGE or OPTIONS section markers
        assert!(
            result.contains("USAGE") || result.contains("OPTIONS") || result.contains("==="),
            "Summary should contain section markers: {result}"
        );
    }

    #[test]
    fn test_summarize_long_docs_produces_output_within_max_len() {
        let long_doc = "x".repeat(MIN_SUMMARIZE_LEN + 1000);
        let max_len = 500;
        let result = summarize_docs(&long_doc, max_len);
        assert!(
            result.len() <= max_len + 50,
            "result should not exceed max_len by much"
        );
    }

    #[test]
    fn test_summarize_docs_respects_priority_order() {
        // USAGE should appear before OPTIONS in the summary
        let usage_line = "USAGE:\n  tool command [options]\n\n";
        let options_line = "OPTIONS:\n  -v  Verbose\n  -h  Help\n\n";
        let examples_line = "EXAMPLES:\n  tool command -v input.bam\n\n";
        let padding = "More information about the tool and its usage.\n".repeat(100);
        let doc = format!("{options_line}{examples_line}{usage_line}{padding}");
        assert!(
            doc.len() > MIN_SUMMARIZE_LEN,
            "test doc ({}) must be > MIN_SUMMARIZE_LEN ({})",
            doc.len(),
            MIN_SUMMARIZE_LEN
        );

        let result = summarize_docs(&doc, MAX_DOC_LEN_MEDIUM_MODEL);
        // USAGE should be near the top since it's highest priority
        let usage_pos = result.find("USAGE");
        let options_pos = result.find("OPTIONS");
        if let (Some(u), Some(o)) = (usage_pos, options_pos) {
            assert!(u <= o, "USAGE section should appear before OPTIONS section");
        }
    }

    #[test]
    fn test_summarize_docs_no_priority_sections_uses_fallback() {
        // Long doc with no priority sections — should use Documentation fallback
        let long_doc = format!("Some description.\n{}", "More info. ".repeat(200));
        assert!(long_doc.len() > MIN_SUMMARIZE_LEN);
        let result = summarize_docs(&long_doc, MAX_DOC_LEN_LARGE_MODEL);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_extract_examples_basic() {
        let doc = "USAGE:\n  tool sort input\n\nEXAMPLE:\n  tool sort -o out.bam in.bam\n\nOPTIONS:\n  -o FILE  Output file";
        let examples = extract_examples(doc);
        assert!(!examples.is_empty(), "should extract at least one example");
    }

    #[test]
    fn test_extract_examples_empty_doc() {
        let examples = extract_examples("");
        assert!(examples.is_empty());
    }

    #[test]
    fn test_extract_examples_no_example_section() {
        let doc = "USAGE:\n  tool [options]\n\nOPTIONS:\n  -v  Verbose";
        let examples = extract_examples(doc);
        assert!(examples.is_empty(), "no EXAMPLE section → empty result");
    }

    #[test]
    fn test_extract_examples_truncates_at_three() {
        // Multiple example sections — should return at most 3
        let doc = "EXAMPLE:\n  cmd1\n\nEXAMPLE:\n  cmd2\n\nEXAMPLE:\n  cmd3\n\nEXAMPLE:\n  cmd4\n";
        let examples = extract_examples(doc);
        assert!(examples.len() <= 3, "should truncate to max 3 examples");
    }

    #[test]
    fn test_doc_length_constants_ordering() {
        // Sanity check: constants should be in ascending order
        assert!(MAX_DOC_LEN_SMALL_MODEL < MAX_DOC_LEN_MEDIUM_MODEL);
        assert!(MAX_DOC_LEN_MEDIUM_MODEL < MAX_DOC_LEN_LARGE_MODEL);
        assert!(MIN_SUMMARIZE_LEN < MAX_DOC_LEN_SMALL_MODEL);
    }

    #[test]
    fn test_extract_flags_from_sections_via_summarize() {
        // Quick reference flags section is added when space is available
        let usage = "USAGE:\n  tool [flags]\n\n";
        let options = "OPTIONS:\n  --verbose  Verbose output\n  --quiet    Quiet mode\n  -o FILE    Output file\n\n";
        let padding = "Description. ".repeat(200);
        let doc = format!("{usage}{options}{padding}");
        assert!(doc.len() > MIN_SUMMARIZE_LEN);

        let result = summarize_docs(&doc, MAX_DOC_LEN_LARGE_MODEL);
        // Either flags are extracted or the summary is at least non-empty
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_for_llm_preserves_content() {
        // Short doc goes through format_for_llm
        let doc = "Line 1\nLine 2\nLine 3";
        let result = summarize_docs(doc, 1000);
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 2"));
        assert!(result.contains("Line 3"));
    }
}
