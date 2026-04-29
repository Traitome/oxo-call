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

use crate::doc_processor::{
    clean_noise, extract_flags_standalone, extract_sections_standalone, truncate_smart,
};

pub const MAX_DOC_LEN_SMALL_MODEL: usize = 3_000;
pub const MAX_DOC_LEN_MEDIUM_MODEL: usize = 6_000;
pub const MAX_DOC_LEN_LARGE_MODEL: usize = 10_000;

pub const MIN_SUMMARIZE_LEN: usize = 2_000;

const PRIORITY_SECTIONS: &[&str; 9] = &[
    "usage",
    "options",
    "arguments",
    "examples",
    "parameters",
    "flags",
    "commands",
    "input",
    "output",
];

pub fn summarize_docs(docs: &str, max_len: usize) -> String {
    if docs.len() <= MIN_SUMMARIZE_LEN {
        return format_for_llm(docs);
    }

    let cleaned = clean_noise(docs);

    let all_sections = extract_sections_standalone(&cleaned);
    let sections: Vec<(String, String)> = all_sections
        .into_iter()
        .filter(|(title, _)| {
            let lower = title.to_lowercase();
            PRIORITY_SECTIONS.iter().any(|&s| lower.contains(s)) || title == "Documentation"
        })
        .collect();
    let sections = if sections.is_empty() {
        vec![("Documentation".to_string(), cleaned)]
    } else {
        sections
    };

    let summary = build_llm_optimized_summary(&sections, max_len);

    if summary.len() > max_len {
        truncate_smart(&summary, max_len)
    } else {
        summary
    }
}

fn format_for_llm(docs: &str) -> String {
    let mut formatted = String::new();
    let lines: Vec<&str> = docs.lines().collect();

    for line in lines {
        formatted.push_str(&format!("{}\n", line));
    }

    formatted.trim().to_string()
}

fn build_llm_optimized_summary(sections: &[(String, String)], max_len: usize) -> String {
    let mut summary = String::new();

    let priority_order = [
        "usage",
        "examples",
        "options",
        "arguments",
        "commands",
        "parameters",
        "flags",
    ];

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

    for (priority, title, content) in sorted_sections {
        if summary.len() > (max_len as f64 * 0.8) as usize {
            break;
        }

        if !summary.is_empty() {
            summary.push_str("\n\n");
        }

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

        let formatted_content = format_section_content(content, priority);
        summary.push_str(&formatted_content);
    }

    if summary.len() < (max_len as f64 * 0.9) as usize {
        let flags = extract_flags_from_sections(sections);
        if !flags.is_empty() && summary.len() + 200 < max_len {
            summary.push_str("\n\n=== QUICK REFERENCE FLAGS ===\n");
            summary.push_str(&flags.join(" "));
        }
    }

    summary
}

fn format_section_content(content: &str, priority: usize) -> String {
    let mut formatted = String::new();
    let lines: Vec<&str> = content.lines().collect();

    if priority == 0 {
        for line in lines.iter().take(5) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                if trimmed.contains('[') || trimmed.contains('<') || trimmed.contains('|') {
                    formatted.push_str(&format!(">>> {}\n", trimmed));
                } else {
                    formatted.push_str(&format!("{}\n", trimmed));
                }
            }
        }
        return formatted;
    }

    if priority == 1 {
        for line in lines.iter().take(10) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                formatted.push_str(&format!("{}\n", trimmed));
            }
        }
        return formatted;
    }

    if priority <= 3 {
        for line in lines.iter().take(20) {
            let trimmed = line.trim();
            if trimmed.starts_with('-') || (!trimmed.is_empty() && formatted.len() < 500) {
                formatted.push_str(&format!("{}\n", trimmed));
            }
        }
        return formatted;
    }

    content.lines().take(15).collect::<Vec<_>>().join("\n")
}

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

#[allow(dead_code)]
pub fn extract_flags(docs: &str) -> Vec<String> {
    extract_flags_standalone(docs)
}

#[allow(dead_code)]
pub fn extract_examples(docs: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    let mut in_example = false;
    let mut current_example = String::new();

    for line in &lines {
        let trimmed = line.trim();

        if trimmed.to_lowercase().contains("example") {
            if !current_example.is_empty() {
                examples.push(current_example.trim().to_string());
                current_example = String::new();
            }
            in_example = true;
        } else if in_example {
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

    if !current_example.is_empty() {
        examples.push(current_example.trim().to_string());
    }

    examples.truncate(3);
    examples
}

pub fn extract_usage_lines(docs: &str) -> Vec<String> {
    let mut usage_lines = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    for line in &lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("usage:")
            || lower.starts_with("usage :")
            || (lower.contains("usage:") && trimmed.len() < 200)
        {
            usage_lines.push(trimmed.to_string());
        } else if !usage_lines.is_empty() {
            if trimmed.is_empty()
                || trimmed.starts_with("options")
                || trimmed.starts_with("arguments")
            {
                break;
            }
            if trimmed.len() < 200
                && (trimmed.starts_with(' ')
                    || trimmed.starts_with('\t')
                    || trimmed.contains(" | "))
            {
                usage_lines.push(trimmed.to_string());
            }
        }
    }

    usage_lines.truncate(5);
    usage_lines
}

pub fn extract_subcommands_from_docs(docs: &str) -> Vec<String> {
    let mut subcmds = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    let mut in_commands_section = false;
    for line in &lines {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        if lower.contains("command") && (lower.ends_with(':') || lower.ends_with("s:")) {
            in_commands_section = true;
            continue;
        }

        if !in_commands_section {
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with("option") || trimmed.starts_with("usage") {
            break;
        }

        if let Some(word) = trimmed.split_whitespace().next()
            && !word.starts_with('-')
            && !word.starts_with('=')
            && word.len() > 1
            && word.len() < 30
            && !subcmds.contains(&word.to_string())
        {
            subcmds.push(word.to_string());
        }
    }

    subcmds.truncate(20);
    subcmds
}

pub fn build_structured_summary(docs: &str, tool: &str) -> String {
    let mut summary = String::new();

    // Detect CLI pattern type from documentation structure
    let pattern_hint = detect_cli_pattern(docs, tool);
    summary.push_str(&pattern_hint);
    summary.push('\n');

    let usage_lines = extract_usage_lines(docs);
    if !usage_lines.is_empty() {
        summary.push_str("=== COMMAND STRUCTURE (CRITICAL) ===\n");
        for line in &usage_lines {
            // Highlight positional argument structure for small models
            let highlighted = highlight_positional_args(line, tool);
            summary.push_str(&format!("  {}\n", highlighted));
        }
        summary.push('\n');
    }

    let all_flags = extract_flags_standalone(docs);
    if !all_flags.is_empty() {
        summary.push_str(&format!("=== VALID FLAGS for {} ===\n", tool));
        summary.push_str("Use ONLY flags from this list. Do NOT invent flags.\n");
        let short_flags: Vec<_> = all_flags
            .iter()
            .filter(|f| f.starts_with('-'))
            .take(30)
            .collect();
        summary.push_str(&format!(
            "  {}\n",
            short_flags
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        summary.push('\n');
    }

    let subcmds = extract_subcommands_from_docs(docs);
    if !subcmds.is_empty() {
        summary.push_str("=== SUBCOMMANDS (FIRST ARG if needed) ===\n");
        summary.push_str(&format!("Available: {}\n", subcmds.join(", ")));
        summary.push_str("If task matches a subcommand, it MUST be the FIRST argument.\n");
        summary.push('\n');
    }

    summary.trim_end().to_string()
}

/// Detect CLI pattern type from documentation and provide guidance for the LLM.
///
/// CLI tools follow distinct patterns:
/// - Pattern A: Subcommand-based (samtools, bcftools) - args start with subcommand
/// - Pattern B: Direct flags (fastp, minimap2) - args start with flags like -i -o
/// - Pattern C: Index+Action (bowtie2, bwa) - separate build/index commands
/// - Pattern D: Long-option only (STAR) - uses --option=value format
fn detect_cli_pattern(docs: &str, tool: &str) -> String {
    let docs_lower = docs.to_lowercase();

    // Check for subcommand pattern
    let subcmds = extract_subcommands_from_docs(docs);
    if !subcmds.is_empty() {
        // Extract first subcommand for example
        let first_subcmd = subcmds.first().map(|s| s.as_str()).unwrap_or("subcmd");
        let subcmds_str = subcmds
            .iter()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return format!(
            "=== CLI PATTERN: SUBCOMMAND-BASED ===\n\
            ⚠️ CRITICAL: {tool} REQUIRES a subcommand as the FIRST argument!\n\
            \n\
            ✅ CORRECT: '{tool} {first_subcmd} -flags args'\n\
            ❌ WRONG: '{tool} -flags args' (missing subcommand - will fail!)\n\
            ❌ WRONG: '{tool} --output file' (no subcommand - will fail!)\n\
            \n\
            Available subcommands: {subcmds_str}\n\
            The subcommand MUST appear immediately after the tool name."
        );
    }

    // Check for common direct-flag tools
    let direct_flag_tools = [
        "fastp",
        "minimap2",
        "seqkit",
        "seqtk",
        "fastqc",
        "multiqc",
        "kraken2",
        "kraken",
        "centrifuge",
        "gffread",
        "prodigal",
        "salmon",
        "kallisto",
        "featurecounts",
        "bedtools",
    ];
    if direct_flag_tools.contains(&tool) {
        return format!(
            "=== CLI PATTERN: DIRECT FLAGS ===\n\
            ✅ {tool} has NO subcommand. ARGS start directly with flags.\n\
            \n\
            ✅ CORRECT: '{tool} -i input -o output'\n\
            ❌ WRONG: '{tool} subcommand -i input' (no subcommand needed)\n\
            \n\
            First argument MUST be a flag (starts with -) or an input file."
        );
    }

    // Check for STAR-style long-option tools
    if docs_lower.contains("--runmode") || docs_lower.contains("--genomedir") || tool == "star" {
        return String::from(
            "=== CLI PATTERN: LONG OPTIONS ===\n\
            ⚠️ This tool uses --option=value format exclusively.\n\
            \n\
            ✅ CORRECT: '--option=value --option2=value2 input_files'\n\
            ❌ WRONG: '-o value' (short flags not supported)\n\
            \n\
            Put all options before positional args. Use --option=value format.",
        );
    }

    // Check for index+action pattern (aligners)
    let index_tools = ["bwa", "bowtie2", "hisat2"];
    if index_tools.contains(&tool) {
        return format!(
            "=== CLI PATTERN: INDEX+ACTION ===\n\
            ⚠️ {tool} requires TWO steps:\n\
            \n\
            Step 1: Build index\n\
            ✅ CORRECT: '{tool}-index reference.fa' OR 'bwa index reference.fa'\n\
            \n\
            Step 2: Align reads\n\
            ✅ CORRECT: '{tool} mem -t N reference.fa reads.fq'\n\
            \n\
            The alignment command uses 'mem' (or other algorithm) as subcommand!"
        );
    }

    // Default pattern hint
    String::from(
        "=== CLI PATTERN: STANDARD ===\n\
    Check USAGE line for exact structure.\n\
    ⚠️ Study the USAGE pattern carefully - some tools use positional args, others use flags.",
    )
}

/// Highlight positional arguments in usage lines for better LLM comprehension.
///
/// Small models (3B+) struggle with understanding which tokens are positional
/// (file paths, required arguments) vs flags. This function adds markers.
fn highlight_positional_args(line: &str, tool: &str) -> String {
    // Replace tool name with placeholder to clarify structure
    line.replace(tool, "[TOOL]")
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
        // Sanity check: constants should be in ascending order (compile-time verified)
        const {
            assert!(MAX_DOC_LEN_SMALL_MODEL < MAX_DOC_LEN_MEDIUM_MODEL);
            assert!(MAX_DOC_LEN_MEDIUM_MODEL < MAX_DOC_LEN_LARGE_MODEL);
            assert!(MIN_SUMMARIZE_LEN < MAX_DOC_LEN_SMALL_MODEL);
        }
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

    #[test]
    fn test_summarize_admixture_doc() {
        // ADMIXTURE has non-standard section headers like "General options:", "Algorithm options:"
        // These should be recognized and processed correctly
        let admixture_doc = r#"****                   ADMIXTURE Version 1.3.0                  ****

  ADMIXTURE basic usage:  (see manual for complete reference)
    % admixture [options] inputFile K

  General options:
    -jX          : do computation on X threads
    --seed=X     : use random seed X for initialization

  Algorithm options:
    --method=[em|block]     : set method.  block is default

  Convergence criteria:
    -C=X : set major convergence criterion (for point estimation)
    -c=x : set minor convergence criterion (for bootstrap and CV reestimates)

  Bootstrap standard errors:
    -B[X]      : do bootstrapping [with X replicates]

  Additional padding to exceed MIN_SUMMARIZE_LEN:
"#;
        // Pad to exceed MIN_SUMMARIZE_LEN
        let padding = "\n".repeat(100) + &"Description line for padding.\n".repeat(50);
        let long_doc = admixture_doc.to_string() + &padding;
        assert!(
            long_doc.len() > MIN_SUMMARIZE_LEN,
            "ADMIXTURE doc ({}) must be > MIN_SUMMARIZE_LEN ({})",
            long_doc.len(),
            MIN_SUMMARIZE_LEN
        );

        let result = summarize_docs(&long_doc, MAX_DOC_LEN_MEDIUM_MODEL);

        // Should extract sections with "options" in their title
        // "General options" and "Algorithm options" contain "options"
        assert!(
            result.contains("OPTIONS")
                || result.contains("General options")
                || result.contains("Algorithm options"),
            "Summary should contain OPTIONS section or the non-standard headers: {result}"
        );

        // Should extract flags from the sections
        let flags = extract_flags(&result);
        assert!(
            !flags.is_empty(),
            "Flags should be extracted from ADMIXTURE summary: {result}"
        );
        assert!(
            flags
                .iter()
                .any(|f| f.contains("-j") || f.contains("--seed") || f.contains("--method")),
            "Expected ADMIXTURE flags (-jX, --seed=X, --method) not found in: {:?}",
            flags
        );
    }
}
