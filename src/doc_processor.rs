//! Intelligent lossless documentation processing for LLM consumption.
//!
//! This module provides smart documentation cleaning and structuring without
//! destructive compression. It preserves complete USAGE, EXAMPLES, and key
//! sections while removing only noise and redundancy.
//!
//! ## Shared primitives
//!
//! The free functions [`clean_noise`], [`is_section_header`],
//! [`extract_flags_standalone`], and [`extract_sections_standalone`] are the
//! canonical implementations used by both this module and
//! [`crate::doc_summarizer`].  `doc_summarizer` re-uses them instead of
//! maintaining its own copies, eliminating code duplication.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::sync::LazyLock;

// ─── Pre-compiled regex patterns (compiled once, reused across all calls) ─────

/// Combined noise pattern: lines that carry no useful information for LLM consumption.
/// Single regex with alternation for all noise patterns to avoid multiple String allocations.
static NOISE_COMBINED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"For more information.*|Report bugs to.*|See the full documentation.*|Homepage:.*|^\s*Version:.*$"
    ).expect("valid regex")
});

/// Matches blank lines (used separately for collapsing).
static BLANK_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").expect("valid regex"));

/// Matches CLI flags like `-o`, `--output`, `--output-file`, and bracketed flags like `[-h]`, `[-dna]`.
/// Many bioinformatics tools (meme suite) use bracket notation for flags.
static FLAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s|\[)(-{1,2}[a-zA-Z0-9_-]+)").expect("valid regex"));

/// Matches structured flag lines in OPTIONS sections (e.g. `  -o FILE   Output file name`).
static FLAG_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(-{1,2}[a-zA-Z0-9@_-]+(?:[,\s]+--?[a-zA-Z0-9_-]+)?(?:\s+\S+)?)\s{2,}(.+)")
        .expect("valid regex")
});

/// Structured documentation with separated sections
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Structured flag catalog extracted from the documentation.
    /// Each entry is `(flag, brief_description)`.
    #[serde(default)]
    pub flag_catalog: Vec<FlagEntry>,
    /// Concrete command-line examples extracted from EXAMPLES / documentation.
    /// Each entry is a raw command string found in the help text.
    #[serde(default)]
    pub extracted_examples: Vec<String>,
    /// Documentation quality score (0.0–1.0) computed deterministically.
    #[serde(default)]
    pub quality_score: f32,
    /// Detected command pattern: "subcommand", "flags-first", or "positional"
    /// Critical for small models to understand argument structure
    #[serde(default)]
    pub command_pattern: String,
    /// Detected subcommand from USAGE line (e.g., "mem" for bwa mem)
    /// Used to prepend subcommand to generated commands
    #[serde(default)]
    pub detected_subcommand: Option<String>,
    /// All detected subcommands for multi-subcommand tools
    #[serde(default)]
    pub all_subcommands: Vec<String>,
}

/// A single flag/option entry extracted from the documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEntry {
    /// The flag itself, e.g. `-o`, `--output`, `-@ INT`.
    pub flag: String,
    /// Brief description extracted from the help text.
    pub description: String,
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
            let flags: Vec<&str> = self
                .quick_flags
                .iter()
                .take(30)
                .map(|s| s.as_str())
                .collect();
            output.push_str(&flags.join(" "));
        }

        write!(f, "{}", output.trim())
    }
}

/// Document processor for cleaning and structuring tool documentation.
///
/// Noise-pattern regexes are compiled once as module-level statics
/// (`NOISE_PATTERNS`, `BLANK_LINE_RE`, etc.) and shared across all instances.
/// Section headers are detected using the free-standing [`is_section_header`]
/// function.
#[derive(Debug, Clone)]
pub struct DocProcessor {
    // All state has been moved to module-level statics.
    // The struct is kept as a method namespace for documentation processing.
    _priv: (),
}

impl Default for DocProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl DocProcessor {
    /// Create a new document processor with default patterns
    pub fn new() -> Self {
        DocProcessor { _priv: () }
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
            flag_catalog: Vec::new(),
            extracted_examples: Vec::new(),
            quality_score: 0.0,
            command_pattern: String::new(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };

        for (section_name, content) in sections {
            let name_lower = section_name.to_lowercase();

            if name_lower.contains("usage") {
                structured.usage = content.clone();
            } else if name_lower.contains("example") {
                structured.examples.push_str(&content);
                structured.examples.push('\n');
            } else if name_lower.contains("option")
                || name_lower.contains("flag")
                || name_lower.contains("input")
                || name_lower.contains("output")
                || name_lower.contains("preset")
                || name_lower.contains("alignment")
                || name_lower.contains("scoring")
                || name_lower.contains("setting")
            {
                // Many tools (bowtie2, bwa) use non-standard section headers like
                // "Input:", "Alignment:", "Scoring:" that contain flag definitions.
                // Treat these as options sections for flag extraction.
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

        // Step 5: Build structured flag catalog from options section
        structured.flag_catalog = self.extract_flag_catalog(&structured.options);

        // Step 6: Extract concrete command examples from EXAMPLES section & raw text
        structured.extracted_examples = self.extract_command_examples(&cleaned);

        // Step 7: Compute documentation quality score
        structured.quality_score = self.compute_quality_score(&structured);

        // Step 8: Detect command pattern and subcommands (critical for small models)
        // This determines if the tool uses: subcommand-first, flags-first, or positional args
        structured = self.detect_command_pattern(structured, docs);

        structured
    }

    /// Detect command pattern from USAGE and examples.
    ///
    /// Determines if the tool uses:
    /// - "subcommand": ARGS must start with subcommand (e.g., samtools sort, bwa mem)
    /// - "flags-first": ARGS start with flags (e.g., fastp -i input -o output)
    /// - "positional": ARGS are positional (e.g., admixture input.bed K)
    ///
    /// This is CRITICAL for small models to generate correct commands.
    fn detect_command_pattern(&self, mut structured: StructuredDoc, _docs: &str) -> StructuredDoc {
        // Known subcommands (short verbs, not file paths)
        const KNOWN_SUBCOMMANDS: &[&str] = &[
            "sort",
            "view",
            "index",
            "merge",
            "extract",
            "filter",
            "call",
            "depth",
            "mem",
            "bwt2se",
            "fastq2bwt",
            "color",
            "sam2bwt",
            "realign",
            "flagstat",
            "mpileup",
            "markdup",
            "collate",
            "fixmate",
            "reheader",
            "cat",
            "stats",
            "bedcov",
            "isec",
            "norm",
            "annotate",
            "predict",
            "classify_wf",
            "identify",
            "align",
            "quant",
            "quantmerge",
            "refine",
            "rsem-calculate-expression",
            "rsem-prepare-reference",
            "discover",
            "gff-cache",
            "mbias",
            "HaplotypeCaller",
            "Mutect2",
            "BaseRecalibrator",
            "ApplyBQSR",
            "SplitNCigarReads",
            "CollectHsMetrics",
            "MarkDuplicates",
            "SortSam",
            "ValidateSamFile",
            "AddOrReplaceReadGroups",
            "CollectAlignmentSummaryMetrics",
            "CollectInsertSizeMetrics",
            "MergeSamFiles",
            "SamToFastq",
            "CreateSequenceDictionary",
            "blastn",
            "blastp",
            "blastx",
            "tblastn",
            "tblastx",
            "build",
            "quast",
            "metaquast",
            "count",
            "version",
            "help",
        ];

        // Check extracted examples first - they're the most reliable
        if let Some(first_example) = structured.extracted_examples.first() {
            let first_token = first_example.split_whitespace().next().unwrap_or("");
            let looks_like_file = first_token.contains('.') || first_token.contains('/');

            if KNOWN_SUBCOMMANDS.contains(&first_token) && !looks_like_file {
                structured.command_pattern = "subcommand".to_string();
                structured.detected_subcommand = Some(first_token.to_string());
            } else if first_token.starts_with('-') {
                structured.command_pattern = "flags-first".to_string();
            } else {
                structured.command_pattern = "positional".to_string();
            }
        }

        // Fallback: parse USAGE line
        if structured.command_pattern.is_empty() && !structured.usage.is_empty() {
            let usage_first_line = structured.usage.lines().next().unwrap_or("");
            let parts: Vec<&str> = usage_first_line.split_whitespace().collect();

            // Pattern: "tool subcmd [options]" or "Usage: tool subcmd [options]"
            // Find the tool name, then check if next token is a subcommand
            for (i, part) in parts.iter().enumerate() {
                let part_lower = part.to_lowercase();
                // Skip "usage:", "usage", and tool name placeholders
                if part_lower == "usage" || part_lower == "usage:" || part.contains("[") {
                    continue;
                }

                // Check if this looks like the tool name (no flags, no brackets)
                // Then the next token might be a subcommand
                if i + 1 < parts.len() {
                    let next = parts[i + 1];
                    let next_lower = next.to_lowercase();

                    // Check if next token is a known subcommand
                    if KNOWN_SUBCOMMANDS.contains(&next_lower.as_str())
                        && !next.contains('.')
                        && !next.contains('/')
                        && !next.starts_with('-')
                        && !next.starts_with('[')
                    {
                        structured.command_pattern = "subcommand".to_string();
                        structured.detected_subcommand = Some(next_lower);
                        break;
                    }
                }
            }

            // If no subcommand detected, check if USAGE shows flags-first or positional
            if structured.command_pattern.is_empty() {
                // Look for flags in USAGE
                if parts.iter().any(|p| p.starts_with('-')) {
                    structured.command_pattern = "flags-first".to_string();
                } else {
                    structured.command_pattern = "positional".to_string();
                }
            }
        }

        // Extract all subcommands from commands section
        if !structured.commands.is_empty() {
            structured.all_subcommands = structured
                .commands
                .split(',')
                .filter_map(|s| s.split_whitespace().next())
                .filter(|s| {
                    KNOWN_SUBCOMMANDS.contains(s)
                        || s.len() >= 2 && !s.contains('.') && !s.contains('/')
                })
                .map(|s| s.to_string())
                .collect();
        }

        // Log pattern detection for debugging
        if !structured.command_pattern.is_empty() {
            tracing::debug!(
                "Detected command pattern: {} (subcmd: {:?})",
                structured.command_pattern,
                structured.detected_subcommand
            );
        }

        structured
    }

    /// Process documentation for LLM with intelligent formatting
    ///
    /// This produces a LLM-ready string with clear section markers,
    /// preserving complete USAGE and EXAMPLES while compressing OPTIONS.
    #[allow(dead_code)] // Public API; used in tests and by downstream consumers
    pub fn process_for_llm(&self, docs: &str) -> String {
        let structured = self.clean_and_structure(docs);
        self.format_structured_doc(&structured)
    }

    /// Format structured documentation for LLM consumption
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
        // Apply combined noise pattern (single regex, single allocation)
        let cleaned = NOISE_COMBINED.replace_all(docs, "");

        // Collapse multiple blank lines to double newline
        let cleaned = BLANK_LINE_RE.replace_all(&cleaned, "\n\n");

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
        // Delegate to the standalone function (canonical implementation)
        is_section_header(line)
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

        // Process all lines - some tools (samtools) have many subcommands spread across
        // multiple sections, and critical ones like "sort" may appear late in the list.
        for line in lines.iter() {
            let trimmed = line.trim();

            // Extract subcommand names (usually first word on line)
            if let Some(first_word) = trimmed.split_whitespace().next() {
                // Skip if it looks like a flag, placeholder, or section header
                if !(first_word.starts_with('-')
                    || first_word.starts_with('<')
                    || first_word.starts_with('[')
                    || first_word.starts_with('=')
                    // Skip ALL_CAPS words that are likely section headers (like "USAGE:", "OPTIONS:")
                    || first_word.len() > 3 && first_word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()))
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

        for cap in FLAG_RE.captures_iter(docs) {
            if let Some(flag) = cap.get(1) {
                flags.insert(flag.as_str().to_string());
            }
        }

        let mut flags_vec: Vec<String> = flags.into_iter().collect();
        flags_vec.sort();
        flags_vec
    }

    /// Extract a structured flag catalog from the OPTIONS section.
    ///
    /// Parses lines that start with `-` and captures the flag name plus its
    /// description.  Handles common help-text layouts:
    ///   -o FILE         Output file name
    ///   --threads INT   Number of threads [4]
    ///   -@ INT          Number of threads (samtools style)
    fn extract_flag_catalog(&self, options: &str) -> Vec<FlagEntry> {
        let mut catalog = Vec::new();

        for line in options.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('-') {
                continue;
            }

            if let Some(caps) = FLAG_LINE_RE.captures(line) {
                let flag = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let desc = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                if !flag.is_empty() {
                    catalog.push(FlagEntry {
                        flag: flag.to_string(),
                        description: desc.to_string(),
                    });
                }
            } else {
                // Simpler pattern: just the flag token
                let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
                if !parts.is_empty() && parts[0].starts_with('-') {
                    catalog.push(FlagEntry {
                        flag: parts[0].to_string(),
                        description: parts.get(1).unwrap_or(&"").trim().to_string(),
                    });
                }
            }
        }

        catalog
    }

    /// Extract concrete command-line examples from the raw documentation.
    ///
    /// Looks for lines that look like actual CLI invocations:
    /// - Lines starting with `$` or `%` (shell prompts)
    /// - Lines inside EXAMPLES sections that start with a tool/subcommand name
    /// - Lines that contain multiple flag patterns (e.g., `-o out.bam`)
    fn extract_command_examples(&self, docs: &str) -> Vec<String> {
        let mut examples = Vec::new();
        let mut in_example_section = false;

        for line in docs.lines() {
            let trimmed = line.trim();

            // Track EXAMPLES section
            if self.is_section_header(trimmed) && trimmed.to_lowercase().contains("example") {
                in_example_section = true;
                continue;
            } else if self.is_section_header(trimmed) {
                in_example_section = false;
            }

            // Skip empty lines and pure prose
            if trimmed.is_empty() || trimmed.len() < 5 {
                continue;
            }

            // Skip prose-like lines (start with uppercase and no flags)
            if trimmed.starts_with(|c: char| c.is_uppercase())
                && !trimmed.contains(" -")
                && !trimmed.starts_with('$')
            {
                continue;
            }

            // Detect command-like lines
            let is_command_line = trimmed.starts_with('$')
                || trimmed.starts_with('%')
                || (in_example_section
                    && (trimmed.contains(" -")
                        || trimmed.contains(" --")
                        || trimmed.contains(" |")));

            if is_command_line {
                // Strip shell prompt prefix
                let cmd = trimmed
                    .strip_prefix("$ ")
                    .or_else(|| trimmed.strip_prefix("% "))
                    .unwrap_or(trimmed);

                // Skip very short or comment-only lines
                if cmd.len() >= 5 && !cmd.starts_with('#') {
                    examples.push(cmd.to_string());
                }
            }
        }

        // Deduplicate and limit
        examples.dedup();
        examples.into_iter().take(10).collect()
    }

    /// Compute a deterministic documentation quality score (0.0–1.0).
    ///
    /// Higher scores indicate documentation that is more likely to produce
    /// accurate LLM-generated commands.
    fn compute_quality_score(&self, doc: &StructuredDoc) -> f32 {
        let mut score: f32 = 0.0;

        // Has USAGE section (essential)
        if !doc.usage.is_empty() {
            score += 0.25;
        }

        // Has EXAMPLES section (very valuable for few-shot)
        if !doc.examples.is_empty() {
            score += 0.25;
        }

        // Has extracted command examples (directly usable as few-shot)
        let example_count = doc.extracted_examples.len();
        score += (example_count.min(5) as f32) * 0.05; // up to 0.25

        // Has OPTIONS / flag catalog (prevents flag hallucination)
        let flag_count = doc.flag_catalog.len();
        score += (flag_count.min(10) as f32) * 0.015; // up to 0.15

        // Has subcommands
        if !doc.commands.is_empty() {
            score += 0.05;
        }

        // Has quick flags
        if !doc.quick_flags.is_empty() {
            score += 0.05;
        }

        score.min(1.0)
    }

    /// Build a compact flag list suitable for injection into LLM prompts.
    ///
    /// Returns a string like: `-o FILE  --threads INT  -@ INT  --output-fmt FMT`
    #[allow(dead_code)] // Public API; used in tests
    pub fn flag_catalog_compact(&self, catalog: &[FlagEntry]) -> String {
        catalog
            .iter()
            .take(30)
            .map(|f| f.flag.clone())
            .collect::<Vec<_>>()
            .join("  ")
    }
}

// ─── Shared free-standing primitives ──────────────────────────────────────────
//
// These functions are the canonical implementations of noise removal, section
// header detection, section extraction, and flag extraction.  They are used
// both within `DocProcessor` methods and by `crate::doc_summarizer`.

/// Remove noise patterns from documentation and collapse excessive blank lines.
///
/// This is the shared implementation used by both [`DocProcessor::remove_noise`]
/// and [`crate::doc_summarizer`].  Uses the module-level `NOISE_PATTERNS` and
/// `BLANK_LINE_RE` statics so the regexes are compiled only once.
pub fn clean_noise(docs: &str) -> String {
    // Apply combined noise pattern (single regex, single allocation)
    let cleaned = NOISE_COMBINED.replace_all(docs, "");

    // Collapse multiple blank lines to double newline
    let cleaned = BLANK_LINE_RE.replace_all(&cleaned, "\n\n");

    cleaned.trim().to_string()
}

/// Check whether a line looks like a section header (e.g. `USAGE:`, `Options:`).
///
/// Canonical implementation shared with [`crate::doc_summarizer`].
/// Handles both standard headers and non-standard patterns like
/// "General options:", "Algorithm options:", "Convergence criteria:"
pub fn is_section_header(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

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

    // Exact match for standard headers
    if header_patterns.iter().any(|p| line.starts_with(p)) {
        return true;
    }

    // All-caps header with trailing colon (e.g. "ADDITIONAL SETTINGS:")
    if line.ends_with(':') && line.chars().filter(|c| c.is_uppercase()).count() > 3 {
        return true;
    }

    // Non-standard headers: line ending with colon that contains keywords
    // e.g. "General options:", "Algorithm options:", "Convergence criteria:"
    if line.ends_with(':') {
        let line_lower = line.to_lowercase();
        let keyword_patterns = [
            "option", "flag", "argument", "param", "setting", "criteria", "input", "output",
            "example", "command", "usage",
        ];
        if keyword_patterns.iter().any(|kw| line_lower.contains(kw)) {
            return true;
        }
    }

    false
}

/// Extract all `-` or `--` flags from a documentation string.
///
/// Canonical implementation shared with [`crate::doc_summarizer`].
/// Uses the module-level `FLAG_RE` static.
pub fn extract_flags_standalone(docs: &str) -> Vec<String> {
    let mut flags = HashSet::new();

    for cap in FLAG_RE.captures_iter(docs) {
        if let Some(flag) = cap.get(1) {
            flags.insert(flag.as_str().to_string());
        }
    }

    let mut flags_vec: Vec<String> = flags.into_iter().collect();
    flags_vec.sort();
    flags_vec
}

/// Extract key sections from documentation text.
///
/// Returns `(header, content)` pairs.  If no sections are found, the entire
/// text is returned as a single `"Documentation"` section.
///
/// Canonical implementation shared with [`crate::doc_summarizer`].
pub fn extract_sections_standalone(docs: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let lines: Vec<&str> = docs.lines().collect();

    let mut current_section = String::new();
    let mut current_content = String::new();

    for line in &lines {
        let trimmed = line.trim();

        if is_section_header(trimmed) {
            if !current_section.is_empty() && !current_content.is_empty() {
                sections.push((current_section.clone(), current_content.clone()));
            }
            current_section = trimmed.to_string();
            current_content = String::new();
        } else if !current_section.is_empty() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if !current_section.is_empty() && !current_content.is_empty() {
        sections.push((current_section, current_content));
    }

    if sections.is_empty() {
        sections.push(("Documentation".to_string(), docs.to_string()));
    }

    sections
}

/// Smart truncation that preserves complete lines and sections.
pub fn truncate_smart(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let truncate_at = max_len.saturating_sub(50);

    if let Some(pos) = text[..truncate_at].rfind("\n\n") {
        let truncated = &text[..pos];
        return format!("{}\n\n... [documentation truncated for brevity]", truncated);
    }

    if let Some(pos) = text[..truncate_at].rfind('\n') {
        let truncated = &text[..pos];
        return format!("{}\n\n... [documentation truncated for brevity]", truncated);
    }

    format!(
        "{}... [documentation truncated for brevity]",
        &text[..truncate_at]
    )
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

    #[test]
    fn test_extract_flag_catalog() {
        let processor = DocProcessor::new();
        let options = "-o FILE          Output file name\n-@ INT           Number of threads\n--threads INT    Number of threads (alias)";

        let catalog = processor.extract_flag_catalog(options);
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|f| f.flag.contains("-o")));
        assert!(catalog.iter().any(|f| f.flag.contains("-@")));
    }

    #[test]
    fn test_extract_command_examples() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  samtools sort [options]\n\nEXAMPLES:\n  samtools sort -o sorted.bam input.bam\n  samtools sort -@ 4 -o out.bam in.bam\n  samtools view -b input.sam > output.bam";

        let examples = processor.extract_command_examples(doc);
        assert!(!examples.is_empty());
    }

    #[test]
    fn test_extract_command_examples_with_prompt() {
        let processor = DocProcessor::new();
        let doc =
            "EXAMPLES:\n  $ samtools sort -o sorted.bam in.bam\n  $ samtools index sorted.bam";

        let examples = processor.extract_command_examples(doc);
        assert!(!examples.is_empty());
        // Should strip the $ prefix
        assert!(!examples[0].starts_with('$'));
    }

    #[test]
    fn test_quality_score() {
        let processor = DocProcessor::new();

        // Good documentation
        let good_doc = "USAGE:\n  tool [options] <input>\n\nOPTIONS:\n  -o FILE  Output\n  -@ INT  Threads\n\nEXAMPLES:\n  $ tool -o out.bam in.bam\n  $ tool -@ 4 in.bam";
        let structured = processor.clean_and_structure(good_doc);
        assert!(
            structured.quality_score > 0.4,
            "Good doc should score > 0.4, got {}",
            structured.quality_score
        );

        // Minimal documentation
        let minimal_doc = "tool - does something";
        let structured = processor.clean_and_structure(minimal_doc);
        assert!(
            structured.quality_score < 0.3,
            "Minimal doc should score < 0.3, got {}",
            structured.quality_score
        );
    }

    #[test]
    fn test_flag_catalog_compact() {
        let processor = DocProcessor::new();
        let catalog = vec![
            FlagEntry {
                flag: "-o FILE".to_string(),
                description: "Output".to_string(),
            },
            FlagEntry {
                flag: "-@ INT".to_string(),
                description: "Threads".to_string(),
            },
        ];
        let compact = processor.flag_catalog_compact(&catalog);
        assert!(compact.contains("-o FILE"));
        assert!(compact.contains("-@ INT"));
    }

    #[test]
    fn test_structured_doc_has_new_fields() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  tool [options]\n\nOPTIONS:\n  -o FILE  Output\n  -h       Help\n\nEXAMPLES:\n  $ tool -o out.txt input.txt";
        let structured = processor.clean_and_structure(doc);

        // flag_catalog should be populated
        assert!(
            !structured.flag_catalog.is_empty(),
            "flag_catalog should not be empty"
        );
        // extracted_examples should be populated
        assert!(
            !structured.extracted_examples.is_empty(),
            "extracted_examples should not be empty"
        );
        // quality_score should be > 0
        assert!(
            structured.quality_score > 0.0,
            "quality_score should be > 0"
        );
    }

    // ── Tests for shared free-standing primitives ─────────────────────────────

    #[test]
    fn test_shared_clean_noise() {
        let noisy = "Usage: tool\n\nFor more information see https://example.com\nReport bugs to bugs@example.com\nHomepage: https://example.com";
        let cleaned = clean_noise(noisy);
        assert!(!cleaned.contains("For more information"));
        assert!(!cleaned.contains("Report bugs"));
        assert!(!cleaned.contains("Homepage:"));
    }

    #[test]
    fn test_shared_clean_noise_collapses_blank_lines() {
        let text = "line1\n\n\n\n\nline2";
        let cleaned = clean_noise(text);
        assert_eq!(cleaned, "line1\n\nline2");
    }

    #[test]
    fn test_shared_is_section_header_standard() {
        assert!(is_section_header("USAGE:"));
        assert!(is_section_header("Options:"));
        assert!(is_section_header("EXAMPLES:"));
        assert!(is_section_header("SYNOPSIS:"));
    }

    #[test]
    fn test_shared_is_section_header_allcaps_colon() {
        assert!(is_section_header("ADDITIONAL SETTINGS:"));
        assert!(!is_section_header("just a line"));
        assert!(!is_section_header(""));
    }

    #[test]
    fn test_shared_is_section_header_non_standard() {
        // ADMIXTURE-style headers
        assert!(is_section_header("General options:"));
        assert!(is_section_header("Algorithm options:"));
        assert!(is_section_header("Convergence criteria:"));
        assert!(is_section_header("Input/Output options:"));
        // Other non-standard patterns
        assert!(is_section_header("Optional flags:"));
        assert!(is_section_header("Basic settings:"));
        assert!(is_section_header("Advanced parameters:"));
        // Should not match regular lines
        assert!(!is_section_header("This is just a sentence"));
        assert!(!is_section_header("some text without keywords"));
    }

    #[test]
    fn test_shared_extract_flags_standalone() {
        let doc = "Usage: tool --help --version -v -q --output FILE";
        let flags = extract_flags_standalone(doc);
        assert!(flags.contains(&"--help".to_string()));
        assert!(flags.contains(&"--version".to_string()));
        assert!(flags.contains(&"-v".to_string()));
        assert!(flags.contains(&"-q".to_string()));
        assert!(flags.contains(&"--output".to_string()));
    }

    #[test]
    fn test_shared_extract_flags_standalone_empty() {
        let flags = extract_flags_standalone("no flags here");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_shared_extract_sections_standalone() {
        let doc =
            "USAGE:\n  tool [options]\n\nOPTIONS:\n  --help  Show help\n\nEXAMPLES:\n  tool --help";
        let sections = extract_sections_standalone(doc);
        assert_eq!(sections.len(), 3);
        assert!(sections[0].0.contains("USAGE"));
        assert!(sections[1].0.contains("OPTIONS"));
        assert!(sections[2].0.contains("EXAMPLES"));
    }

    #[test]
    fn test_shared_extract_sections_standalone_no_sections() {
        let doc = "just some text without headers";
        let sections = extract_sections_standalone(doc);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].0, "Documentation");
    }

    #[test]
    fn test_shared_truncate_smart_short() {
        let text = "short text";
        assert_eq!(truncate_smart(text, 100), text);
    }

    #[test]
    fn test_shared_truncate_smart_at_paragraph() {
        let text = "Line 1\n\nLine 2\n\nLine 3 which is longer text to push past the limit";
        let result = truncate_smart(text, 25);
        assert!(result.contains("[documentation truncated"));
    }

    #[test]
    fn test_shared_truncate_smart_at_line() {
        let text = "Line 1\nLine 2\nLine 3 which is a bit longer to go past the truncation point";
        let result = truncate_smart(text, 30);
        assert!(result.contains("[documentation truncated"));
    }

    #[test]
    fn test_admixture_doc_processing() {
        // ADMIXTURE has non-standard section headers like "General options:", "Algorithm options:"
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
    -B[X]      : do bootstrapping [with X replicates]"#;

        let processor = DocProcessor::new();
        let structured = processor.clean_and_structure(admixture_doc);

        // Should recognize non-standard section headers
        assert!(
            !structured.options.is_empty() || !structured.quick_flags.is_empty(),
            "ADMIXTURE flags should be extracted: options='{}', quick_flags={:?}",
            structured.options,
            structured.quick_flags
        );

        // Should extract flags from the documentation
        assert!(
            !structured.quick_flags.is_empty(),
            "quick_flags should contain ADMIXTURE flags like -jX, --seed, --method, -C, -c, -B"
        );

        // Verify specific flags are extracted
        let flags_str = structured.quick_flags.join(" ");
        assert!(
            flags_str.contains("-j")
                || flags_str.contains("--seed")
                || flags_str.contains("--method"),
            "Expected flags not found in: {}",
            flags_str
        );
    }

    #[test]
    fn test_detect_command_pattern_subcommand() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  samtools sort [options]\n\nEXAMPLES:\n  sort -o out.bam in.bam\n";
        let structured = processor.clean_and_structure(doc);
        assert_eq!(structured.command_pattern, "subcommand");
    }

    #[test]
    fn test_detect_command_pattern_flags_first() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  fastp [options]\n\nOPTIONS:\n  -i FILE  Input\n  -o FILE  Output\n\nEXAMPLES:\n  -i in.fq -o out.fq\n";
        let structured = processor.clean_and_structure(doc);
        assert_eq!(structured.command_pattern, "flags-first");
    }

    #[test]
    fn test_detect_command_pattern_positional() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  tool input.bed K\n\nEXAMPLES:\n  input.bed 5\n";
        let structured = processor.clean_and_structure(doc);
        assert_eq!(structured.command_pattern, "positional");
    }

    #[test]
    fn test_detect_command_pattern_from_usage() {
        let processor = DocProcessor::new();
        let doc = "Usage: bwa mem [options] <ref.fa> <reads.fq>\n\nEXAMPLES:\n  mem -t 8 ref.fa reads.fq\n";
        let structured = processor.clean_and_structure(doc);
        assert_eq!(structured.command_pattern, "subcommand");
        assert_eq!(structured.detected_subcommand, Some("mem".to_string()));
    }

    #[test]
    fn test_structured_doc_display() {
        let doc = StructuredDoc {
            usage: "tool [options]".to_string(),
            examples: "tool --help".to_string(),
            options: "--help  Show help".to_string(),
            commands: "sort, view".to_string(),
            other: "Description text".to_string(),
            quick_flags: vec!["--help".to_string(), "--version".to_string()],
            flag_catalog: Vec::new(),
            extracted_examples: Vec::new(),
            quality_score: 0.8,
            command_pattern: "flags-first".to_string(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };
        let display = format!("{doc}");
        assert!(display.contains("=== USAGE ==="));
        assert!(display.contains("=== EXAMPLES ==="));
        assert!(display.contains("=== SUBCOMMANDS ==="));
        assert!(display.contains("=== OPTIONS/FLAGS ==="));
        assert!(display.contains("=== QUICK REFERENCE FLAGS ==="));
    }

    #[test]
    fn test_structured_doc_display_empty() {
        let doc = StructuredDoc::default();
        let display = format!("{doc}");
        assert!(display.is_empty() || display.trim().is_empty());
    }

    #[test]
    fn test_extract_sections_no_header() {
        let processor = DocProcessor::new();
        let sections = processor.extract_sections("just some text\nno headers");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].0, "Documentation");
    }

    #[test]
    fn test_extract_sections_multiple_same_type() {
        let processor = DocProcessor::new();
        let doc = "OPTIONS:\n  -h  Help\n\nEXAMPLES:\n  tool -h\n\nEXAMPLES:\n  tool -v\n";
        let sections = processor.extract_sections(doc);
        assert!(sections.len() >= 2);
    }

    #[test]
    fn test_compress_options_with_placeholders() {
        let processor = DocProcessor::new();
        let options =
            "  -o FILE  Output\n  <input>  Input file\n  [options]  Optional\n     Just text";
        let compressed = processor.compress_options(options);
        assert!(compressed.contains("-o"));
        assert!(compressed.contains("<input>"));
        assert!(compressed.contains("[options]"));
    }

    #[test]
    fn test_extract_subcommands_filters_flags() {
        let processor = DocProcessor::new();
        let content =
            "  sort    Sort data\n  --help  Show help\n  <input>  Input\n  merge   Merge data";
        let extracted = processor.extract_subcommands(content);
        assert!(extracted.contains("sort"));
        assert!(extracted.contains("merge"));
        assert!(!extracted.contains("--help"));
    }

    #[test]
    fn test_extract_all_flags_bracketed() {
        let processor = DocProcessor::new();
        let doc = "Usage: tool [-h] [-v] [-dna]";
        let flags = processor.extract_all_flags(doc);
        assert!(flags.iter().any(|f| f == "-h" || f == "-v"));
    }

    #[test]
    fn test_extract_flag_catalog_simple() {
        let processor = DocProcessor::new();
        let options = "--help\n--version\n--output";
        let catalog = processor.extract_flag_catalog(options);
        assert!(catalog.len() >= 2);
    }

    #[test]
    fn test_extract_command_examples_pipe() {
        let processor = DocProcessor::new();
        let doc = "EXAMPLES:\n  samtools view -b input.sam | samtools sort -o out.bam\n";
        let examples = processor.extract_command_examples(doc);
        assert!(!examples.is_empty());
    }

    #[test]
    fn test_extract_command_examples_skips_prose() {
        let processor = DocProcessor::new();
        let doc = "EXAMPLES:\n  This section shows examples\n  tool -o out.bam in.bam\n";
        let examples = processor.extract_command_examples(doc);
        assert!(examples.iter().all(|e| !e.starts_with("This")));
    }

    #[test]
    fn test_compute_quality_score_max() {
        let processor = DocProcessor::new();
        let doc = StructuredDoc {
            usage: "tool [options]".to_string(),
            examples: "tool -h".to_string(),
            options: "--help".to_string(),
            commands: "sort, view".to_string(),
            other: String::new(),
            quick_flags: vec!["--help".to_string()],
            flag_catalog: vec![FlagEntry {
                flag: "--help".to_string(),
                description: "Show help".to_string(),
            }],
            extracted_examples: vec!["tool -h".to_string()],
            quality_score: 0.0,
            command_pattern: String::new(),
            detected_subcommand: None,
            all_subcommands: Vec::new(),
        };
        let score = processor.compute_quality_score(&doc);
        assert!(score > 0.5);
    }

    #[test]
    fn test_compute_quality_score_empty() {
        let processor = DocProcessor::new();
        let doc = StructuredDoc::default();
        let score = processor.compute_quality_score(&doc);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_clean_noise_version_line() {
        let noisy = "For more information see https://example.com\nUsage: tool [options]";
        let cleaned = clean_noise(noisy);
        assert!(!cleaned.contains("For more information"));
        assert!(cleaned.contains("Usage:"));
    }

    #[test]
    fn test_is_section_header_empty() {
        assert!(!is_section_header(""));
    }

    #[test]
    fn test_is_section_header_mixed_case() {
        assert!(is_section_header("Input/Output options:"));
        assert!(is_section_header("Advanced parameters:"));
        assert!(is_section_header("Basic settings:"));
    }

    #[test]
    fn test_truncate_smart_short_text() {
        let text = "short";
        assert_eq!(truncate_smart(text, 100), "short");
    }

    #[test]
    fn test_truncate_smart_no_paragraph_break() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5 which is longer to exceed the limit";
        let result = truncate_smart(text, 30);
        assert!(result.contains("[documentation truncated"));
    }

    #[test]
    fn test_doc_processor_default() {
        let processor = DocProcessor::default();
        let doc = "USAGE:\n  tool [options]\n";
        let structured = processor.process(doc);
        assert!(!structured.usage.is_empty());
    }

    #[test]
    fn test_process_alias() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  tool [options]\n";
        let s1 = processor.process(doc);
        let s2 = processor.clean_and_structure(doc);
        assert_eq!(s1.usage, s2.usage);
    }

    #[test]
    fn test_detect_command_pattern_usage_flags_first() {
        let processor = DocProcessor::new();
        let doc = "Usage: fastp -i in.fq -o out.fq\n\nOPTIONS:\n  -i FILE  Input\n";
        let structured = processor.clean_and_structure(doc);
        assert!(!structured.command_pattern.is_empty());
    }

    #[test]
    fn test_all_subcommands_extracted() {
        let processor = DocProcessor::new();
        let doc = "COMMANDS:\n  sort  Sort data\n  view  View data\n  index  Index data\n";
        let structured = processor.clean_and_structure(doc);
        assert!(!structured.all_subcommands.is_empty() || !structured.commands.is_empty());
    }

    #[test]
    fn test_format_structured_doc() {
        let processor = DocProcessor::new();
        let doc = "USAGE:\n  tool [options]\n\nOPTIONS:\n  -h  Help\n\nEXAMPLES:\n  tool -h";
        let formatted = processor.process_for_llm(doc);
        assert!(formatted.contains("USAGE"));
        assert!(formatted.contains("EXAMPLES"));
    }
}
