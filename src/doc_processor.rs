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

/// Noise patterns: lines that carry no useful information for LLM consumption.
static NOISE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"For more information.*").expect("valid regex"),
        Regex::new(r"Report bugs to.*").expect("valid regex"),
        Regex::new(r"See the full documentation.*").expect("valid regex"),
        Regex::new(r"Homepage:.*").expect("valid regex"),
        Regex::new(r"^\s*Version:.*$").expect("valid regex"),
        Regex::new(r"^\s*$").expect("valid regex"),
    ]
});

/// Matches three or more consecutive newlines (for collapsing blank lines).
static BLANK_LINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").expect("valid regex"));

/// Matches CLI flags like `-o`, `--output`, `--output-file`.
static FLAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)(-{1,2}[a-zA-Z0-9_-]+)").expect("valid regex"));

/// Matches structured flag lines in OPTIONS sections (e.g. `  -o FILE   Output file name`).
static FLAG_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(-{1,2}[a-zA-Z0-9@_-]+(?:[,\s]+--?[a-zA-Z0-9_-]+)?(?:\s+\S+)?)\s{2,}(.+)")
        .expect("valid regex")
});

/// Matches the value-type metavar in a flag line (e.g. `INT`, `FILE`, `STR`, `N`, `PATH`).
static FLAG_TYPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(INT|FILE|STR|STRING|FLOAT|NUM|N|PATH|DIR|URL|FMT|FORMAT|NAME|KEY|VALUE|PATTERN)\b",
    )
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
    /// Each entry carries the flag, its type constraint (if detectable), and description.
    #[serde(default)]
    pub flag_catalog: Vec<FlagEntry>,
    /// Concrete command-line examples extracted from EXAMPLES / documentation.
    /// Each entry is a raw command string found in the help text.
    #[serde(default)]
    pub extracted_examples: Vec<String>,
    /// Documentation quality score (0.0–1.0) computed deterministically.
    #[serde(default)]
    pub quality_score: f32,
    /// Does this tool require a subcommand as the first token?
    /// E.g., samtools needs "sort", "view", etc. before flags.
    /// Tools like flye have NO subcommands - flags come first.
    #[serde(default)]
    pub has_subcommands: bool,
    /// List of detected subcommands (sort, view, index, etc.)
    #[serde(default)]
    pub subcommands: Vec<String>,
    /// Format hint extracted from USAGE line for LLM guidance.
    #[serde(default)]
    pub format_hint: Option<String>,
    /// Companion binaries that should be used as first token instead of subcommands.
    /// E.g., rsem-prepare-reference, bowtie2-build, bwa-mem2.
    #[serde(default)]
    pub companion_binaries: Vec<String>,
    /// Detailed USAGE pattern analysis for format validation.
    #[serde(default)]
    pub usage_pattern: UsagePattern,
    /// File type to flag mappings extracted from examples.
    #[serde(default)]
    pub file_type_mappings: Vec<FileTypeMapping>,
}

impl StructuredDoc {
    /// Extract USAGE patterns for a specific subcommand from the documentation.
    ///
    /// This is the key method for Phase 2: Mini-Skill USAGE Injection.
    /// It scans the USAGE section and examples to find patterns that match
    /// the given subcommand, returning a compact representation suitable
    /// for few-shot injection.
    ///
    /// # Arguments
    /// * `subcommand` - The subcommand to find USAGE for (e.g., "sort", "build")
    /// * `tool` - The tool name (e.g., "samtools", "bowtie2")
    ///
    /// # Returns
    /// * `Some(String)` - The extracted USAGE pattern if found
    /// * `None` - If no specific USAGE pattern is found for the subcommand
    pub fn extract_subcommand_usage(&self, subcommand: &str, tool: &str) -> Option<String> {
        let subcommand_lower = subcommand.to_ascii_lowercase();

        // Strategy 1: Look for explicit USAGE lines containing the subcommand
        for line in self.usage.lines() {
            let line_lower = line.to_ascii_lowercase();
            // Match patterns like "Usage: tool subcommand [options]" or "tool COMMAND [options]"
            if line_lower.contains(&subcommand_lower)
                || (line_lower.contains("<command>") && line_lower.contains(&subcommand_lower))
            {
                return Some(line.trim().to_string());
            }
        }

        // Strategy 2: Look in examples for patterns with this subcommand
        for line in self.examples.lines() {
            let line_lower = line.to_ascii_lowercase();
            // Match example lines that start with the tool and subcommand
            if line_lower.starts_with(&format!("{tool} {subcommand_lower}").to_ascii_lowercase())
                || line_lower
                    .starts_with(&format!("$ {tool} {subcommand_lower}").to_ascii_lowercase())
                || line_lower
                    .starts_with(&format!("% {tool} {subcommand_lower}").to_ascii_lowercase())
            {
                // Extract just the command pattern
                let cmd = line.trim_start_matches('$').trim_start_matches('%').trim();
                return Some(format!("Example: {cmd}"));
            }
        }

        // Strategy 3: Check extracted examples
        for ex in &self.extracted_examples {
            let ex_lower = ex.to_ascii_lowercase();
            if ex_lower.starts_with(&format!("{tool} {subcommand_lower}").to_ascii_lowercase())
                || ex_lower.starts_with(subcommand_lower.as_str())
            {
                return Some(format!("Example: {ex}"));
            }
        }

        // Strategy 4: For tools with companion binaries, check if subcommand matches a companion
        for companion in &self.companion_binaries {
            let companion_lower = companion.to_ascii_lowercase();
            if companion_lower.contains(&subcommand_lower)
                || subcommand_lower.contains(&companion_lower)
            {
                return Some(format!("Usage: {companion} [options] <args>"));
            }
        }

        None
    }

    /// Build a mini-skill injection string for compact prompts.
    ///
    /// This creates a focused few-shot example from the documentation
    /// that demonstrates the correct command structure for a specific task.
    pub fn build_mini_skill_injection(&self, tool: &str, task: &str) -> Option<String> {
        // Extract task keywords to match with subcommands
        let task_lower = task.to_ascii_lowercase();

        // Find the best matching subcommand
        let best_subcommand = self.subcommands.iter().find(|cmd| {
            let cmd_lower = cmd.to_ascii_lowercase();
            task_lower.contains(&cmd_lower)
                || cmd_lower.contains(
                    &task_lower
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
        });

        if let Some(subcommand) = best_subcommand {
            // Try to extract specific USAGE for this subcommand
            if let Some(usage) = self.extract_subcommand_usage(subcommand, tool) {
                // Look for a matching example
                let example = self
                    .extracted_examples
                    .iter()
                    .find(|ex| {
                        ex.to_ascii_lowercase()
                            .contains(&subcommand.to_ascii_lowercase())
                    })
                    .cloned()
                    .unwrap_or_else(|| format!("{tool} {subcommand} [options] <input>"));

                return Some(format!("USAGE: {usage}\nExample: {example}",));
            }
        }

        // Fallback: if no specific subcommand match, try companion binaries
        for companion in &self.companion_binaries {
            let companion_lower = companion.to_ascii_lowercase();
            if task_lower.contains(&companion_lower)
                || companion_lower.contains(
                    &task_lower
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                )
            {
                return Some(format!(
                    "USAGE: {companion} [options] <args>\nNote: Use companion binary '{companion}' instead of main tool"
                ));
            }
        }

        None
    }
}

/// A single flag/option entry extracted from the documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagEntry {
    /// The flag itself, e.g. `-o`, `--output`, `-@ INT`.
    pub flag: String,
    /// Value-type constraint inferred from the metavar, e.g. `INT`, `FILE`, `STR`.
    /// `None` when the flag is a boolean switch with no argument.
    #[serde(default)]
    pub value_type: Option<String>,
    /// Brief description extracted from the help text.
    pub description: String,
    /// Is this flag required? Detected from keywords like "required", "mandatory".
    #[serde(default)]
    pub required: bool,
    /// Default value if specified in docs (e.g., `[default: 4]`).
    #[serde(default)]
    pub default: Option<String>,
    /// Alternative form pairing (e.g., `-o` paired with `--output`).
    #[serde(default)]
    pub alt_form: Option<String>,
}

/// File type to flag mapping extracted from documentation examples.
/// Maps file extensions (e.g., "fastq", "bam") to the flags used for those inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeMapping {
    /// File extension without dot (e.g., "fastq", "bam", "vcf")
    pub extension: String,
    /// Flag(s) typically used with this file type (e.g., "-i", "--input")
    pub flags: Vec<String>,
    /// Whether this is an input or output file type
    pub io_type: FileIOType,
}

/// Whether a file type is used for input or output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileIOType {
    Input,
    Output,
    Both,
}

impl Default for FileIOType {
    fn default() -> Self {
        FileIOType::Input
    }
}

/// Detailed USAGE pattern analysis for format constraint extraction.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsagePattern {
    /// The raw USAGE line(s) extracted from docs
    pub raw_usage: String,
    /// Pattern type: subcommand-required, flag-first, positional-args, etc.
    pub pattern_type: UsagePatternType,
    /// Arguments position: subcommand-first, flags-first, files-first
    pub arg_order: Vec<ArgPosition>,
    /// Whether the tool uses companion binaries (e.g., bowtie2-build)
    pub uses_companion_binaries: bool,
    /// Detected positional argument patterns (e.g., "INPUT", "OUTPUT")
    pub positional_args: Vec<String>,
}

/// Types of USAGE patterns found in bioinformatics tools.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsagePatternType {
    /// Tool requires subcommand: `tool COMMAND [options]` (e.g., samtools, bcftools)
    SubcommandRequired,
    /// Tool uses flags directly: `tool [options] <input>` (e.g., flye, metaphlan)
    FlagFirst,
    /// Tool uses positional arguments: `tool <input> <output>` (e.g., admixture)
    PositionalArgs,
    /// Tool has companion binaries: `tool-build [options]` (e.g., bowtie2-build)
    CompanionBinary,
    /// Mixed or unclear pattern
    Mixed,
}

impl Default for UsagePatternType {
    fn default() -> Self {
        UsagePatternType::Mixed
    }
}

/// Position of arguments in command structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArgPosition {
    Subcommand,
    Flag,
    InputFile,
    OutputFile,
    Positional,
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
            // Add "Subcommands:" prefix so extract_subcommands can parse it correctly
            output.push_str("Subcommands: ");
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
    #[allow(dead_code)] // Public API
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
            has_subcommands: false,
            subcommands: Vec::new(),
            format_hint: None,
            companion_binaries: Vec::new(),
            usage_pattern: UsagePattern::default(),
            file_type_mappings: Vec::new(),
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

        // Step 5: Build structured flag catalog from options section
        structured.flag_catalog = self.extract_flag_catalog(&structured.options);

        // Step 6: Extract concrete command examples from EXAMPLES section & raw text
        structured.extracted_examples = self.extract_command_examples(&cleaned);

        // Step 7: Detect format constraints (subcommand requirements, companion binaries)
        let (has_subcommands, detected_subcommands, format_hint) =
            self.detect_format_constraints(&structured.usage, &structured.commands, &cleaned);
        structured.has_subcommands = has_subcommands;
        structured.subcommands = detected_subcommands;
        structured.format_hint = format_hint;

        // Step 8: Detect companion binaries from documentation
        structured.companion_binaries =
            self.detect_companion_binaries(&cleaned, &structured.examples);

        // Step 9: Enhance flag catalog with required/default detection and alt_form pairing
        self.enhance_flag_catalog(&mut structured.flag_catalog);

        // Step 10: Extract detailed USAGE pattern analysis
        structured.usage_pattern = self.extract_usage_pattern(
            &structured.usage,
            &structured.examples,
            structured.has_subcommands,
        );

        // Step 11: Extract file type to flag mappings from examples
        structured.file_type_mappings =
            self.extract_file_type_mappings(&structured.examples, &structured.flag_catalog);

        // Step 12: Compute documentation quality score
        structured.quality_score = self.compute_quality_score(&structured);

        structured
    }

    /// Detect format constraints from USAGE section and examples.
    ///
    /// Returns (has_subcommands, subcommands_list, format_hint)
    fn detect_format_constraints(
        &self,
        usage: &str,
        commands: &str,
        full_doc: &str,
    ) -> (bool, Vec<String>, Option<String>) {
        let mut has_subcommands = false;
        let mut subcommands = Vec::new();
        let mut format_hint = None;

        // Parse USAGE line patterns
        // Pattern 1: "Usage: tool COMMAND" or "Usage: tool <command>" -> has subcommands
        // Pattern 2: "Usage: tool [options] <input>" -> no subcommands
        // Pattern 3: "Usage: tool [subcommand]" -> optional subcommands

        let usage_lower = usage.to_lowercase();

        // Check for explicit subcommand indicators in USAGE
        if usage_lower.contains(" command")
            || usage_lower.contains(" <command>")
            || usage_lower.contains(" [command]")
            || usage_lower.contains(" command ")
        {
            has_subcommands = true;
            format_hint = Some("First token must be a subcommand".to_string());
        }

        // Check for "COMMAND" or "SUBCOMMAND" placeholder in usage
        if usage.contains("COMMAND") || usage.contains("SUBCOMMAND") {
            has_subcommands = true;
        }

        // Check for positional argument patterns (input files) without subcommands
        // Pattern: "Usage: tool [options] <input>" suggests no subcommand
        if (usage_lower.contains("<input>")
            || usage_lower.contains("<file>")
            || usage_lower.contains("<path>"))
            && !has_subcommands
        {
            format_hint = Some("First token is a flag or input file".to_string());
        }

        // Extract subcommands from COMMANDS section ONLY if USAGE indicates subcommands
        // Some tools (e.g., canu) have a "Commands:" section describing pipeline stages/modes
        // that are selected via flags, NOT positional subcommands. Check USAGE first.
        let usage_indicates_subcommands = has_subcommands; // set by USAGE patterns above

        if !commands.is_empty() && usage_indicates_subcommands {
            has_subcommands = true;
            // Split comma-separated list and clean up
            for cmd in commands.split(',') {
                let cmd = cmd.trim();
                if !cmd.is_empty() && !cmd.contains(' ') {
                    subcommands.push(cmd.to_string());
                }
            }
        } else if !commands.is_empty() {
            // Commands section exists but USAGE doesn't indicate subcommands
            // Store the commands but don't mark as has_subcommands
            // This handles tools like canu where "Commands:" are pipeline stages
            for cmd in commands.split(',') {
                let cmd = cmd.trim();
                if !cmd.is_empty() && !cmd.contains(' ') {
                    subcommands.push(cmd.to_string());
                }
            }
            // has_subcommands remains false - first token is a flag
        }

        // Look for common subcommand patterns in examples
        // Tools like samtools show usage patterns like "samtools sort [options]"
        let common_subcommands = [
            "sort", "view", "index", "merge", "cat", "faidx", "dict", "sort", "view", "index",
            "merge", "mpileup", "fasta", "fastq", "call", "filter", "norm", "annotate", "merge",
            "concat", "align", "index", "build", "extract", "stat",
        ];

        // Analyze examples to infer subcommands
        for line in full_doc.lines() {
            let trimmed = line.trim();
            // Look for "tool subcommand" patterns in examples
            for word in trimmed.split_whitespace() {
                let word = word.trim_start_matches('$').trim_start_matches('%').trim();
                if common_subcommands.contains(&word) && word.len() > 2 {
                    if !subcommands.contains(&word.to_string()) {
                        subcommands.push(word.to_string());
                    }
                    has_subcommands = true;
                }
            }
        }

        // Sort and deduplicate subcommands
        subcommands.sort();
        subcommands.dedup();

        // Limit to reasonable number (allow more for multi-command tools like samtools)
        if subcommands.len() > 50 {
            subcommands.truncate(50);
        }

        // Final inference from examples if still unclear
        if !has_subcommands && subcommands.is_empty() {
            // Check if examples show subcommand patterns
            let example_subcommands = self.infer_subcommands_from_examples(full_doc);
            if !example_subcommands.is_empty() {
                has_subcommands = true;
                subcommands = example_subcommands;
            }
        }

        (has_subcommands, subcommands, format_hint)
    }

    /// Infer subcommands from example usage patterns.
    fn infer_subcommands_from_examples(&self, docs: &str) -> Vec<String> {
        let mut subcommands = HashSet::new();
        let mut in_examples = false;

        for line in docs.lines() {
            let trimmed = line.trim();

            // Track examples section
            if self.is_section_header(trimmed) && trimmed.to_lowercase().contains("example") {
                in_examples = true;
                continue;
            } else if self.is_section_header(trimmed) {
                in_examples = false;
            }

            if !in_examples {
                continue;
            }

            // Look for command patterns
            // e.g., "$ tool sort ..." -> "sort" is the subcommand
            // e.g., "tool view ..." -> "view" is the subcommand
            if trimmed.starts_with('$') || trimmed.starts_with('%') {
                let parts: Vec<&str> = trimmed
                    .trim_start_matches('$')
                    .trim_start_matches('%')
                    .split_whitespace()
                    .collect();

                if parts.len() >= 2 {
                    // First part after $ is usually the tool name
                    // Second part is potentially the subcommand if it doesn't start with -
                    let potential_subcommand = parts[1];
                    if !potential_subcommand.starts_with('-')
                        && !potential_subcommand.starts_with('<')
                        && !potential_subcommand.starts_with('[')
                        && potential_subcommand.len() > 1
                        && potential_subcommand
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    {
                        subcommands.insert(potential_subcommand.to_string());
                    }
                }
            }
        }

        let mut result: Vec<String> = subcommands.into_iter().collect();
        result.sort();
        result.into_iter().take(10).collect()
    }

    /// Detect companion binaries from documentation.
    ///
    /// Companion binaries are separate executables that share the tool name prefix,
    /// e.g., rsem-prepare-reference, bowtie2-build, bwa-mem2.
    fn detect_companion_binaries(&self, docs: &str, examples: &str) -> Vec<String> {
        let mut binaries = HashSet::new();

        // Common companion binary suffixes/prefixes
        let companion_patterns = [
            "-build",
            "-index",
            "-prepare-reference",
            "-calculate-expression",
            "-generate-data-matrix",
            "-generate-library-type",
            "-sort",
            "-view",
        ];

        // Scan for companion binary patterns
        for line in docs.lines() {
            let trimmed = line.trim();

            // Look for binary names in text
            for pattern in &companion_patterns {
                if let Some(pos) = trimmed.to_lowercase().find(pattern) {
                    // Extract the full binary name
                    let start = trimmed[..pos]
                        .rfind(|c: char| c.is_whitespace() || c == '`' || c == '[')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let end = trimmed[pos..]
                        .find(|c: char| c.is_whitespace() || c == '`' || c == ']')
                        .map(|i| pos + i)
                        .unwrap_or(trimmed.len());

                    let binary = trimmed[start..end].trim().to_string();
                    if !binary.is_empty() && binary.len() > 3 {
                        binaries.insert(binary);
                    }
                }
            }
        }

        // Also check examples for companion binary invocations
        for line in examples.lines() {
            let trimmed = line.trim();
            if let Some(first_word) = trimmed.split_whitespace().next() {
                // If first word contains a hyphen and looks like a companion binary
                if first_word.contains('-') && first_word.len() > 5 {
                    binaries.insert(first_word.to_string());
                }
            }
        }

        let mut result: Vec<String> = binaries.into_iter().collect();
        result.sort();
        result.dedup();
        result.into_iter().take(5).collect()
    }

    /// Enhance flag catalog with required/default detection and alt_form pairing.
    fn enhance_flag_catalog(&self, catalog: &mut [FlagEntry]) {
        // Detect required flags from descriptions
        for entry in catalog.iter_mut() {
            let desc_lower = entry.description.to_lowercase();

            // Check for required keywords
            if desc_lower.contains("required")
                || desc_lower.contains("mandatory")
                || desc_lower.contains("must be")
                || desc_lower.contains("must specify")
            {
                entry.required = true;
            }

            // Extract default value
            // Patterns: [default: X], (default: X), default=X, default: X
            let default_patterns = [
                r"\[default:\s*([^\]]+)\]",
                r"\(default:\s*([^)]+)\)",
                r"default[=:]\s*(\S+)",
            ];

            for pattern in &default_patterns {
                if let Ok(re) = regex::Regex::new(pattern)
                    && let Some(cap) = re.captures(&entry.description)
                    && let Some(m) = cap.get(1)
                {
                    entry.default = Some(m.as_str().trim().to_string());
                    break;
                }
            }
        }

        // Pair short and long forms
        let mut short_forms: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut long_forms: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (idx, entry) in catalog.iter().enumerate() {
            if entry.flag.starts_with("--") && entry.flag.len() > 2 {
                // Long form - store without dashes
                let key = entry.flag.trim_start_matches('-').to_string();
                long_forms.insert(key, idx);
            } else if entry.flag.starts_with('-') && !entry.flag.starts_with("--") {
                // Short form - store the single char
                if let Some(ch) = entry.flag.chars().nth(1) {
                    short_forms.insert(ch.to_string(), idx);
                }
            }
        }

        // Look for semantic matches between short and long forms
        // e.g., -o and --output, -t and --threads
        let semantic_pairs = [
            ('o', "output"),
            ('i', "input"),
            ('t', "threads"),
            ('n', "name"),
            ('f', "file"),
            ('d', "dir"),
            ('p', "prefix"),
            ('r', "reference"),
            ('g', "genome"),
            ('b', "bam"),
            ('v', "vcf"),
            ('h', "help"),
        ];

        for (short_ch, long_str) in &semantic_pairs {
            if let (Some(&short_idx), Some(&long_idx)) = (
                short_forms.get(&short_ch.to_string()),
                long_forms.get(*long_str),
            ) {
                let short_flag = catalog[short_idx].flag.clone();
                let long_flag = catalog[long_idx].flag.clone();

                // Set alt_form on both entries
                catalog[short_idx].alt_form = Some(long_flag);
                catalog[long_idx].alt_form = Some(short_flag);
            }
        }

        // Heuristic: Mark critical flags as required based on semantics
        for entry in catalog.iter_mut() {
            let flag_lower = entry.flag.to_lowercase();
            let desc_lower = entry.description.to_lowercase();

            // Skip if already marked required
            if entry.required {
                continue;
            }

            // Heuristic 1: Database/index flags for specific tools
            if (flag_lower.contains("--index") || flag_lower.contains("--db") || flag_lower.contains("--database"))
                && (desc_lower.contains("database") || desc_lower.contains("index"))
            {
                entry.required = true;
            }

            // Heuristic 2: Input type flags when they're critical
            if flag_lower.contains("--input_type") || flag_lower.contains("--input-type") {
                entry.required = true;
            }

            // Heuristic 3: Output directory flags (most tools need output)
            if (flag_lower.contains("-d") || flag_lower.contains("--outdir") || flag_lower.contains("--output-dir"))
                && desc_lower.contains("output")
                && (desc_lower.contains("directory") || desc_lower.contains("dir"))
            {
                entry.required = true;
            }

            // Heuristic 4: Thread/CPU flags - set default if not present
            if (flag_lower.contains("-t") || flag_lower.contains("-@") || flag_lower.contains("--thread") || flag_lower.contains("--nproc"))
                && entry.default.is_none()
            {
                entry.default = Some("4".to_string());
            }
        }
    }

    /// Extract detailed USAGE pattern analysis from documentation.
    ///
    /// This analyzes the USAGE section and examples to determine:
    /// - Pattern type (subcommand-required, flag-first, positional-args, etc.)
    /// - Argument order (subcommand, flags, files)
    /// - Positional argument patterns
    fn extract_usage_pattern(&self, usage: &str, examples: &str, has_subcommands: bool) -> UsagePattern {
        let mut pattern = UsagePattern {
            raw_usage: usage.to_string(),
            pattern_type: UsagePatternType::Mixed,
            arg_order: Vec::new(),
            uses_companion_binaries: false,
            positional_args: Vec::new(),
        };

        let usage_lower = usage.to_lowercase();

        // Determine pattern type from USAGE line
        if has_subcommands {
            pattern.pattern_type = UsagePatternType::SubcommandRequired;
            pattern.arg_order.push(ArgPosition::Subcommand);
        } else if usage_lower.contains("<input>") || usage_lower.contains("<file>") {
            // Check if positional args come before flags
            if usage_lower.contains("<input>") && !usage_lower.contains("[options]") {
                pattern.pattern_type = UsagePatternType::PositionalArgs;
                pattern.arg_order.push(ArgPosition::Positional);
            } else {
                pattern.pattern_type = UsagePatternType::FlagFirst;
                pattern.arg_order.push(ArgPosition::Flag);
            }
        } else {
            pattern.pattern_type = UsagePatternType::FlagFirst;
            pattern.arg_order.push(ArgPosition::Flag);
        }

        // Check for companion binaries in examples
        for line in examples.lines() {
            let trimmed = line.trim();
            if trimmed.contains("-build") || trimmed.contains("-prepare") || trimmed.contains("-index") {
                if let Some(first_word) = trimmed.split_whitespace().next() {
                    if first_word.contains('-') && first_word.len() > 5 {
                        pattern.uses_companion_binaries = true;
                        break;
                    }
                }
            }
        }

        // Extract positional argument patterns from USAGE
        for line in usage.lines() {
            // Look for patterns like <INPUT>, <OUTPUT>, <FILE>, etc.
            let re = regex::Regex::new(r"<([A-Z_]+)>").unwrap();
            for cap in re.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    let arg = m.as_str().to_string();
                    if !pattern.positional_args.contains(&arg) {
                        pattern.positional_args.push(arg);
                    }
                }
            }
        }

        pattern
    }

    /// Extract file type to flag mappings from documentation examples.
    ///
    /// Analyzes example commands to map file extensions to the flags used with them.
    /// e.g., "-i input.fastq" -> FileTypeMapping { extension: "fastq", flags: ["-i"], io_type: Input }
    #[allow(dead_code)] // flag_catalog reserved for future semantic analysis
    fn extract_file_type_mappings(&self, examples: &str, _flag_catalog: &[FlagEntry]) -> Vec<FileTypeMapping> {
        let mut mappings: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        let mut output_mappings: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        // Common file extensions in bioinformatics
        let file_extensions = [
            "fastq", "fq", "fasta", "fa", "fna", "bam", "sam", "cram",
            "vcf", "bcf", "bed", "gtf", "gff", "bam.bai", "sam.gz",
            "txt", "tsv", "csv", "json", "html", "pdf", "png",
            "gz", "bgz", "zip", "tar", "tar.gz",
        ];

        for line in examples.lines() {
            let trimmed = line.trim();

            // Look for flag + file patterns: "-i file.fastq", "--input file.fasta"
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                // Check if this word is a flag
                if word.starts_with('-') && i + 1 < words.len() {
                    let next_word = words[i + 1];

                    // Check if next word is a file path with extension
                    for ext in &file_extensions {
                        if next_word.ends_with(&format!(".{}", ext)) ||
                           next_word.ends_with(&format!(".{}.{}", ext, "gz")) ||
                           next_word.ends_with(&format!(".{}.{}", ext, "bgz")) {
                            let flag = word.to_string();
                            let ext_key = ext.to_string();

                            // Determine if input or output based on flag semantics
                            let is_output = flag.contains("-o") ||
                                           flag.contains("--output") ||
                                           flag.contains("--out") ||
                                           flag.contains("-O");

                            if is_output {
                                output_mappings.entry(ext_key).or_default().push(flag);
                            } else {
                                mappings.entry(ext_key).or_default().push(flag);
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Build FileTypeMapping results
        let mut results = Vec::new();

        for (ext, flags) in mappings {
            let unique_flags: Vec<String> = flags.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
            results.push(FileTypeMapping {
                extension: ext,
                flags: unique_flags,
                io_type: FileIOType::Input,
            });
        }

        for (ext, flags) in output_mappings {
            let unique_flags: Vec<String> = flags.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect();
            results.push(FileTypeMapping {
                extension: ext,
                flags: unique_flags,
                io_type: FileIOType::Output,
            });
        }

        results
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
            // Add "Subcommands:" prefix so extract_subcommands can parse it correctly
            output.push_str("Subcommands: ");
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

        // Apply statically-compiled noise patterns
        for pattern in NOISE_PATTERNS.iter() {
            cleaned = pattern.replace_all(&cleaned, "").to_string();
        }

        // Collapse multiple blank lines to double newline
        cleaned = BLANK_LINE_RE.replace_all(&cleaned, "\n\n").to_string();

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
            "COMMAND:",
            "Command:",
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

        // Check for "=== SECTION ===" format (from doc_summarizer)
        if line.starts_with("=== ") && line.ends_with(" ===") {
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


        // Check for formatted subcommand list (e.g., "Subcommands: cmd1, cmd2, cmd3")
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.to_lowercase().starts_with("subcommands:") {
                // Parse comma-separated subcommands from formatted line
                let after_colon = trimmed.splitn(2, ':').nth(1).unwrap_or("").trim();
                for cmd in after_colon.split(',') {
                    let cmd = cmd.trim();
                    if !cmd.is_empty()
                        && cmd.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_'
                        })
                    {
                        commands.push(cmd.to_string());
                    }
                }
                // Return immediately if we found formatted subcommands
                if !commands.is_empty() {
                    commands.sort();
                    commands.dedup();
                    return commands.join(", ");
                }
            }
        }

        // Fall back to parsing raw help text format
        // Handle formats like:
        //   samtools: "  sort     Sort BAM file" (indented with spaces)
        //   bwa: "         mem           BWA-MEM algorithm" (indented after "Command:")
        let lines: Vec<&str> = content.lines().collect();

        for line in lines.iter() {
            let trimmed = line.trim();

            // Skip the "Command:" header line itself (not the subcommands listed after it)
            if trimmed.to_lowercase().starts_with("command:") {
                // Extract the subcommand from the same line if present (e.g., "Command: index")
                if let Some(rest) = trimmed.strip_prefix("Command:").or_else(|| trimmed.strip_prefix("command:")) {
                    let rest_trimmed = rest.trim();
                    if let Some(first_word) = rest_trimmed.split_whitespace().next() {
                        if first_word.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
                            commands.push(first_word.to_string());
                        }
                    }
                }
                continue;
            }

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip category headers (lines starting with "--")
            if trimmed.starts_with("--") {
                continue;
            }

            // Extract subcommand names (usually first word on line)
            if let Some(first_word) = trimmed.split_whitespace().next() {
                // Skip if it looks like a flag, placeholder, or description text
                if first_word.starts_with('-')
                    || first_word.starts_with('<')
                    || first_word.starts_with('[')
                {
                    continue;
                }

                // Skip common non-subcommand words that might appear in descriptions
                let non_command_words = [
                    "and", "or", "the", "a", "an", "to", "of", "for", "in", "on", "with",
                    "from", "by", "at", "as", "into", "through", "during", "before", "after",
                    "above", "below", "between", "under", "again", "further", "then", "once",
                    "here", "there", "when", "where", "why", "how", "all", "each", "few",
                    "more", "most", "other", "some", "such", "only", "own", "same", "so",
                    "than", "too", "very", "can", "will", "just", "should", "now", "use",
                    "using", "used", "using", "see", "also", "e.g.", "i.e.", "etc.", "note",
                    "this", "that", "these", "those", "am", "is", "are", "was", "were",
                    "be", "been", "being", "have", "has", "had", "do", "does", "did",
                    "but", "if", "because", "until", "while", "although", "though",
                ];
                if non_command_words.contains(&first_word.to_lowercase().as_str()) {
                    continue;
                }

                // Valid subcommand names are typically lowercase alphanumeric with hyphens/underscores
                // and don't contain sentence punctuation
                if first_word.chars().all(|c| {
                    c.is_ascii_lowercase()
                        || c.is_ascii_digit()
                        || c == '-'
                        || c == '_'
                }) {
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
    /// Parses lines that start with `-` and captures the flag name, type
    /// constraint (INT/FILE/STR/…), and description.  Handles common layouts:
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
                let flag_with_meta = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let desc = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                if !flag_with_meta.is_empty() {
                    // Extract value type from the metavar token in the flag field
                    // (e.g., `-@ INT` → `INT`, `--output FILE` → `FILE`).
                    let value_type = FLAG_TYPE_RE
                        .find(flag_with_meta)
                        .map(|m| m.as_str().to_uppercase());

                    // Strip the metavar from the flag token to keep it clean.
                    let flag_clean = flag_with_meta
                        .split_whitespace()
                        .next()
                        .unwrap_or(flag_with_meta)
                        .trim_end_matches(',');

                    catalog.push(FlagEntry {
                        flag: flag_clean.to_string(),
                        value_type,
                        description: desc.to_string(),
                        required: false,
                        default: None,
                        alt_form: None,
                    });
                }
            } else {
                // Simpler pattern: just the flag token (boolean switch).
                let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
                if !parts.is_empty() && parts[0].starts_with('-') {
                    let rest = parts.get(1).unwrap_or(&"").trim();
                    // Check if the next word looks like a metavar type.
                    let (value_type, description) = if let Some(first_word) = rest
                        .split_whitespace()
                        .next()
                        .filter(|w| FLAG_TYPE_RE.is_match(w))
                    {
                        let desc = rest[first_word.len()..].trim();
                        (Some(first_word.to_uppercase()), desc.to_string())
                    } else {
                        (None, rest.to_string())
                    };
                    catalog.push(FlagEntry {
                        flag: parts[0].trim_end_matches(',').to_string(),
                        value_type,
                        description,
                        required: false,
                        default: None,
                        alt_form: None,
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
            score += 0.20;
        }

        // Has EXAMPLES section (very valuable for few-shot)
        if !doc.examples.is_empty() {
            score += 0.20;
        }

        // Has extracted command examples (directly usable as few-shot)
        let example_count = doc.extracted_examples.len();
        score += (example_count.min(5) as f32) * 0.04; // up to 0.20

        // Has OPTIONS / flag catalog (prevents flag hallucination)
        let flag_count = doc.flag_catalog.len();
        score += (flag_count.min(10) as f32) * 0.015; // up to 0.15

        // Has subcommands detected (important for format constraints)
        if !doc.subcommands.is_empty() || doc.has_subcommands {
            score += 0.10;
        }

        // Has format hint (critical for correct command structure)
        if doc.format_hint.is_some() {
            score += 0.05;
        }

        // Has companion binaries detected
        if !doc.companion_binaries.is_empty() {
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
    /// including the value-type constraint when known.
    #[allow(dead_code)] // Public API; used in tests
    pub fn flag_catalog_compact(&self, catalog: &[FlagEntry]) -> String {
        catalog
            .iter()
            .take(30)
            .map(|f| match &f.value_type {
                Some(t) => format!("{} {}", f.flag, t),
                None => f.flag.clone(),
            })
            .collect::<Vec<_>>()
            .join("  ")
    }
}

/// LLM-based intelligent document processor (reserved for future use).
///
/// This is an advanced processor that uses LLM to understand and extract
/// key information from documentation, rather than simple pattern matching.
/// Currently only uses the rule-based fast path; the LLM path is a placeholder.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IntelligentDocProcessor {
    /// Rule-based processor for fast path
    rule_processor: DocProcessor,
    /// Cache for processed documents
    cache: std::collections::HashMap<String, ProcessedDoc>,
}

impl Default for IntelligentDocProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl IntelligentDocProcessor {
    pub(crate) fn new() -> Self {
        Self {
            rule_processor: DocProcessor::new(),
            cache: std::collections::HashMap::new(),
        }
    }

    /// Process documentation using hybrid approach (rules + optional LLM)
    ///
    /// # Arguments
    /// * `doc` - Raw documentation text
    /// * `tool` - Tool name
    /// * `use_llm` - Whether to use LLM for intelligent processing
    ///
    /// # Returns
    /// Processed documentation with key information extracted
    pub(crate) async fn process(&self, doc: &str, tool: &str, use_llm: bool) -> ProcessedDoc {
        // Step 1: Calculate document hash for caching
        let doc_hash = self.calculate_hash(doc);

        // Step 2: Check cache
        if let Some(cached) = self.cache.get(&doc_hash) {
            return cached.clone();
        }

        // Step 3: Use rule-based processor first
        let structured = self.rule_processor.clean_and_structure(doc);

        // Step 4: Extract common pitfalls from documentation patterns
        let pitfalls = self.extract_pitfalls(tool, &structured);

        // Step 5: Extract common usage patterns
        let common_patterns = self.extract_common_patterns(&structured);

        if use_llm {
            // Build the LLM prompt (ready for integration once LlmClient
            // is injected into IntelligentDocProcessor).
            let _llm_prompt = self.build_llm_prompt(&structured, tool);
            // TODO(llm): Wire LlmClient here to refine structured doc via LLM.
            // The prompt is prepared so wiring a real LlmClient is a one-line change.
            tracing::debug!(
                "LLM doc processing requested for '{}' — prompt ready ({} chars)",
                tool,
                _llm_prompt.len()
            );
        }

        ProcessedDoc {
            core_usage: structured.usage.clone(),
            key_parameters: self.extract_key_parameters(&structured),
            common_patterns,
            pitfalls,
            examples: self.extract_examples(&structured),
            quality_score: self.assess_quality(&structured),
        }
    }

    /// Extract common pitfalls from documentation structure.
    fn extract_pitfalls(&self, tool: &str, structured: &StructuredDoc) -> Vec<String> {
        let mut pitfalls = Vec::new();
        if structured.usage.is_empty() {
            pitfalls.push(format!(
                "No clear usage pattern found in {tool} docs — verify command syntax manually"
            ));
        }
        if structured.options.is_empty() {
            pitfalls.push(
                "No options/flags section detected — the tool may use positional arguments only"
                    .to_string(),
            );
        }
        if !structured.commands.is_empty() && !structured.commands.contains(' ') {
            pitfalls.push(
                "Tool requires a subcommand — ensure the first argument is a valid subcommand"
                    .to_string(),
            );
        }
        pitfalls
    }

    /// Extract common usage patterns from documentation.
    fn extract_common_patterns(&self, structured: &StructuredDoc) -> Vec<String> {
        let mut patterns = Vec::new();
        if !structured.commands.is_empty() {
            for cmd in structured
                .commands
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(5)
            {
                patterns.push(cmd.trim().to_string());
            }
        }
        patterns
    }

    /// Calculate document hash for caching
    fn calculate_hash(&self, doc: &str) -> String {
        use sha2::{Digest, Sha256};
        hex::encode(Sha256::digest(doc.as_bytes()))
    }

    /// Extract key parameters from structured documentation
    fn extract_key_parameters(&self, structured: &StructuredDoc) -> Vec<KeyParameter> {
        let mut params = Vec::new();

        // Parse options section
        for line in structured.options.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Extract flag and description
            if let Some(param) = self.parse_parameter_line(line) {
                params.push(param);
            }
        }

        // Limit to top 20 parameters
        params.into_iter().take(20).collect()
    }

    /// Parse a single parameter line
    fn parse_parameter_line(&self, line: &str) -> Option<KeyParameter> {
        let line = line.trim();

        // Pattern: --flag <type> description
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() >= 2 && parts[0].starts_with('-') {
            Some(KeyParameter {
                name: parts[0].to_string(),
                description: parts.get(2).unwrap_or(&"").to_string(),
                default: None,
                common_use_case: None,
            })
        } else {
            None
        }
    }

    /// Extract examples from structured documentation
    fn extract_examples(&self, structured: &StructuredDoc) -> Vec<DocExample> {
        let mut examples = Vec::new();

        for line in structured.examples.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Simple heuristic: lines starting with tool name are examples
            if line
                .trim()
                .starts_with(structured.usage.split_whitespace().next().unwrap_or(""))
            {
                examples.push(DocExample {
                    command: line.trim().to_string(),
                    description: "Example from documentation".to_string(),
                });
            }
        }

        examples.into_iter().take(5).collect()
    }

    /// Assess documentation quality (0.0-1.0)
    fn assess_quality(&self, structured: &StructuredDoc) -> f32 {
        let mut score = 0.0;

        // Check for essential sections
        if !structured.usage.is_empty() {
            score += 0.3;
        }
        if !structured.examples.is_empty() {
            score += 0.3;
        }
        if !structured.options.is_empty() {
            score += 0.2;
        }

        // Check for quick flags
        if !structured.quick_flags.is_empty() {
            score += 0.1;
        }

        // Check for commands
        if !structured.commands.is_empty() {
            score += 0.1;
        }

        score
    }

    /// Build LLM prompt for intelligent document processing
    fn build_llm_prompt(&self, structured: &StructuredDoc, tool: &str) -> String {
        format!(
            r#"You are a bioinformatics documentation expert. Extract and organize the most critical information from this {tool} documentation.

Documentation:
{structured}

Output JSON format:
{{
  "core_usage": "The most common usage pattern (one line)",
  "key_parameters": [
    {{
      "name": "parameter name",
      "description": "brief description",
      "default": "default value if any",
      "common_use_case": "when to use this parameter"
    }}
  ],
  "common_patterns": [
    "pattern 1: description",
    "pattern 2: description"
  ],
  "pitfalls": [
    "common mistake 1",
    "common mistake 2"
  ],
  "examples": [
    {{
      "command": "actual command",
      "description": "what this command does"
    }}
  ]
}}

Focus on information that helps generate correct commands. Remove noise and redundancy."#
        )
    }
}

/// Processed documentation with extracted key information (reserved for future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProcessedDoc {
    /// Core usage pattern (most common)
    pub core_usage: String,
    /// Key parameters extracted from documentation
    pub key_parameters: Vec<KeyParameter>,
    /// Common usage patterns
    pub common_patterns: Vec<String>,
    /// Common pitfalls to avoid
    pub pitfalls: Vec<String>,
    /// Examples extracted from documentation
    pub examples: Vec<DocExample>,
    /// Quality score (0.0-1.0)
    pub quality_score: f32,
}

impl Default for ProcessedDoc {
    fn default() -> Self {
        Self {
            core_usage: String::new(),
            key_parameters: Vec::new(),
            common_patterns: Vec::new(),
            pitfalls: Vec::new(),
            examples: Vec::new(),
            quality_score: 0.0,
        }
    }
}

/// Key parameter extracted from documentation (reserved for future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeyParameter {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
    pub common_use_case: Option<String>,
}

/// Example extracted from documentation (reserved for future use).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DocExample {
    pub command: String,
    pub description: String,
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
    let mut cleaned = docs.to_string();

    for re in NOISE_PATTERNS.iter() {
        cleaned = re.replace_all(&cleaned, "").to_string();
    }

    // Collapse multiple blank lines to double newline
    cleaned = BLANK_LINE_RE.replace_all(&cleaned, "\n\n").to_string();

    cleaned.trim().to_string()
}

/// Check whether a line looks like a section header (e.g. `USAGE:`, `Options:`).
///
/// Canonical implementation shared with [`crate::doc_summarizer`].
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

    if header_patterns.iter().any(|p| line.starts_with(p)) {
        return true;
    }

    // All-caps header with trailing colon
    if line.ends_with(':') && line.chars().filter(|c| c.is_uppercase()).count() > 3 {
        return true;
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
                flag: "-o".to_string(),
                value_type: Some("FILE".to_string()),
                description: "Output".to_string(),
                required: false,
                default: None,
                alt_form: None,
            },
            FlagEntry {
                flag: "-@".to_string(),
                value_type: Some("INT".to_string()),
                description: "Threads".to_string(),
                required: false,
                default: None,
                alt_form: None,
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

    #[test]
    fn test_samtools_style_extraction() {
        let processor = DocProcessor::new();
        let samtools_doc = r#"Program: samtools (Tools for alignments in the SAM format)
Version: 1.23.1 (using htslib 1.23.1)

Usage:   samtools <command> [options]

Commands:
  -- Indexing
     dict           create a sequence dictionary file
     faidx          index/extract FASTA
     fqidx          index/extract FASTQ
     index          index alignment

  -- Editing
     calmd          recalculate MD/NM tags and '=' bases
     fixmate        fix mate information

  -- File operations
     sort           sort alignment file
     view           SAM<->BAM<->CRAM conversion

  -- Statistics
     flagstat       simple stats
     idxstats       BAM index stats

  -- Viewing
     flags          explain BAM flags
     tview          text alignment viewer

  -- Misc
     help [cmd]     display this help message
     version        detailed version information"#;

        let structured = processor.clean_and_structure(samtools_doc);

        // Check that subcommands were extracted
        println!("Subcommands: {:?}", structured.subcommands);
        println!("Has subcommands: {}", structured.has_subcommands);
        println!("Usage pattern type: {:?}", structured.usage_pattern.pattern_type);

        // Should NOT contain "--" (category headers)
        assert!(
            !structured.subcommands.iter().any(|s| s == "--"),
            "Should not contain category headers like '--', got: {:?}",
            structured.subcommands
        );

        // Should have detected subcommands are required
        assert!(
            structured.has_subcommands,
            "Should detect that subcommands are required"
        );

        // Should contain actual subcommands
        assert!(
            structured.subcommands.iter().any(|s| s == "sort"),
            "Should contain 'sort' subcommand, got: {:?}",
            structured.subcommands
        );
        assert!(
            structured.subcommands.iter().any(|s| s == "view"),
            "Should contain 'view' subcommand, got: {:?}",
            structured.subcommands
        );
        assert!(
            structured.subcommands.iter().any(|s| s == "index"),
            "Should contain 'index' subcommand, got: {:?}",
            structured.subcommands
        );
        assert!(
            structured.subcommands.iter().any(|s| s == "dict"),
            "Should contain 'dict' subcommand, got: {:?}",
            structured.subcommands
        );

        // Usage pattern should be SubcommandRequired
        assert!(
            matches!(
                structured.usage_pattern.pattern_type,
                UsagePatternType::SubcommandRequired
            ),
            "Usage pattern should be SubcommandRequired, got: {:?}",
            structured.usage_pattern.pattern_type
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

    // ─── Tests for Phase 2: Mini-Skill USAGE Injection ─────────────────────────

    #[test]
    fn test_extract_subcommand_usage() {
        let processor = DocProcessor::new();
        let doc = r#"USAGE:
  bowtie2 [options]* -x <idx> {-1 <m1> -2 <m2> | -U <r>} [-S <sam>]
  bowtie2-build [options]* <reference_in> <bt2_index_base>

OPTIONS:
        // Test extracting usage for build (should find bowtie2-build via companion binary)

EXAMPLES:

        // Test extracting usage for align (should find bowtie2 USAGE)
        let align_usage = sdoc.extract_subcommand_usage("align", "bowtie2");
        assert!(align_usage.is_some(), "Should find usage for \'align\'");
        let usage = align_usage.unwrap();
        assert!(
            usage.contains("bowtie2") || usage.contains("Example:"),
            "Usage should contain bowtie2 pattern or example: {}",
            usage
        );

        // Test extracting usage for build (should find bowtie2-build)
        let build_usage = sdoc.extract_subcommand_usage("build", "bowtie2");
        assert!(build_usage.is_some(), "Should find usage for \'build\'");
        let usage = build_usage.unwrap();
        assert!(
            usage.contains("bowtie2-build") || usage.contains("Example:"),
            "Usage should contain bowtie2-build pattern: {}",
            usage
        );
    }

    #[test]
    fn test_build_mini_skill_injection() {
        let processor = DocProcessor::new();
        let doc = r#"USAGE:
  samtools sort [options] <in.bam>
  samtools view [options] <in.bam>

OPTIONS:
  -o FILE  Output file
  -@ INT   Threads

EXAMPLES:
  $ samtools sort -o sorted.bam input.bam
  $ samtools view -b input.sam > output.bam
"#;
        let sdoc = processor.clean_and_structure(doc);

        // Should detect subcommands
        assert!(sdoc.has_subcommands, "Should detect subcommands");
        assert!(
            sdoc.subcommands.contains(&"sort".to_string()),
            "Should detect \'sort\' subcommand"
        );
        assert!(
            sdoc.subcommands.contains(&"view".to_string()),
            "Should detect \'view\' subcommand"
        );

        // Test mini-skill for sort task
        let mini_skill = sdoc.build_mini_skill_injection("samtools", "sort the bam file");
        assert!(
            mini_skill.is_some(),
            "Should build mini-skill for \'sort\' task"
        );
        let skill = mini_skill.unwrap();
        assert!(
            skill.contains("sort") || skill.contains("Example:"),
            "Mini-skill should contain sort: {}",
            skill
        );

        // Test mini-skill for view task
        let mini_skill_view = sdoc.build_mini_skill_injection("samtools", "view the bam file");
        assert!(
            mini_skill_view.is_some(),
            "Should build mini-skill for \'view\' task"
        );
        let skill_view = mini_skill_view.unwrap();
        assert!(
            skill_view.contains("view") || skill_view.contains("Example:"),
            "Mini-skill should contain view: {}",
            skill_view
        );
    }

    #[test]
    fn test_extract_subcommand_usage_with_companion_binary() {
        let processor = DocProcessor::new();
        let doc = r#"USAGE:
  rsem-calculate-expression [options] <input> <index> <output>
  rsem-prepare-reference [options] <reference_fasta> <index_name>

EXAMPLES:
  rsem-prepare-reference reference.fa reference_index
"#;
        let sdoc = processor.clean_and_structure(doc);

        // Should detect companion binaries
        assert!(
            !sdoc.companion_binaries.is_empty(),
            "Should detect companion binaries"
        );
        let has_prepare_ref = sdoc
            .companion_binaries
            .iter()
            .any(|b| b.contains("prepare-reference"));
        assert!(has_prepare_ref, "Should detect rsem-prepare-reference");

        // Test extracting usage for prepare-reference task
        let prepare_usage = sdoc.extract_subcommand_usage("prepare", "rsem");
        assert!(
            prepare_usage.is_some(),
            "Should find usage for \'prepare\' task"
        );
    }

    #[test]
    fn test_bwa_style_extraction() {
        let processor = DocProcessor::new();
        let bwa_doc = r#"Program: bwa (alignment via Burrows-Wheeler transformation)
Version: 0.7.19-r1273

Usage:   bwa <command> [options]

Command: index         index sequences in the FASTA format
         mem           BWA-MEM algorithm
         fastmap       identify super-maximal exact matches
         aln           gapped/ungapped alignment
         samse         generate alignment (single ended)
         sampe         generate alignment (paired ended)
         bwasw         BWA-SW for long queries

Note: To use BWA, you need to first index the genome with `bwa index'.
      There are three alignment algorithms in BWA: `mem', `bwasw', and
      `aln/samse/sampe'."#;

        let structured = processor.clean_and_structure(bwa_doc);

        println!("BWA Subcommands: {:?}", structured.subcommands);
        println!("Has subcommands: {}", structured.has_subcommands);

        // Should contain all major subcommands
        assert!(structured.subcommands.iter().any(|s| s == "index"), "Should contain 'index'");
        assert!(structured.subcommands.iter().any(|s| s == "mem"), "Should contain 'mem'");
        assert!(structured.subcommands.iter().any(|s| s == "aln"), "Should contain 'aln'");
        assert!(structured.subcommands.iter().any(|s| s == "samse"), "Should contain 'samse'");
        assert!(structured.subcommands.iter().any(|s| s == "sampe"), "Should contain 'sampe'");
        assert!(structured.subcommands.iter().any(|s| s == "bwasw"), "Should contain 'bwasw'");
    }

    #[test]
    fn test_bcftools_style_extraction() {
        let processor = DocProcessor::new();
        let bcftools_doc = r#"Program: bcftools (Tools for variant calling and manipulating VCFs and BCFs)
Version: 1.21.1 (using htslib 1.21.1)

Usage:   bcftools [--version|--version-only] [--help] <command> <argument>
Commands:
   -- Indexing
      index        index VCF/BCF files
   -- VCF/BCF manipulation
      annotate     annotate and edit VCF/BCF files
      concat       concatenate VCF/BCF files from the same set of samples
      convert      convert VCF/BCF files to different formats and back
      isec         intersections of VCF/BCF files
      merge        merge VCF/BCF files files from non-overlapping sample sets
      norm         left-align and normalize indels
      plugin       user-defined plugins
      query        transform VCF/BCF into user-defined formats
      reheader     modify VCF/BCF header, change sample names
      sort         sort VCF/BCF files
      view         VCF/BCF conversion, view, subset and filter VCF/BCF files
   -- VCF/BCF analysis
      call         SNP/indel calling
      consensus    create consensus sequence by applying VCF variants
      csq          call variation consequences
      filter       filter VCF/BCF files using fixed thresholds
      gtcheck      check sample concordance
      roh          identify runs of autozygosity
      stats        produce VCF/BCF stats

Use "bcftools <command>" to see command-specific help."#;

        let structured = processor.clean_and_structure(bcftools_doc);

        println!("BCFTOOLS Subcommands: {:?}", structured.subcommands);
        println!("Has subcommands: {}", structured.has_subcommands);
        println!("Commands field: '{}'", structured.commands);

        // Should detect subcommands are required
        assert!(structured.has_subcommands, "Should detect that subcommands are required");

        // Should contain major subcommands
        assert!(structured.subcommands.iter().any(|s| s == "view"), "Should contain 'view', got: {:?}", structured.subcommands);
        assert!(structured.subcommands.iter().any(|s| s == "index"), "Should contain 'index', got: {:?}", structured.subcommands);
        assert!(structured.subcommands.iter().any(|s| s == "sort"), "Should contain 'sort', got: {:?}", structured.subcommands);
        assert!(structured.subcommands.iter().any(|s| s == "merge"), "Should contain 'merge', got: {:?}", structured.subcommands);
        assert!(structured.subcommands.iter().any(|s| s == "call"), "Should contain 'call', got: {:?}", structured.subcommands);
    }

    #[test]
    fn test_summarized_bcftools_extraction() {
        use crate::doc_summarizer::summarize_docs;

        let processor = DocProcessor::new();
        let bcftools_doc = r#"Program: bcftools (Tools for variant calling and manipulating VCFs and BCFs)
Version: 1.21.1 (using htslib 1.21.1)

Usage:   bcftools [--version|--version-only] [--help] <command> <argument>
Commands:
   -- Indexing
      index        index VCF/BCF files
   -- VCF/BCF manipulation
      annotate     annotate and edit VCF/BCF files
      concat       concatenate VCF/BCF files from the same set of samples
      convert      convert VCF/BCF files to different formats and back
      isec         intersections of VCF/BCF files
      merge        merge VCF/BCF files files from non-overlapping sample sets
      norm         left-align and normalize indels
      plugin       user-defined plugins
      query        transform VCF/BCF into user-defined formats
      reheader     modify VCF/BCF header, change sample names
      sort         sort VCF/BCF files
      view         VCF/BCF conversion, view, subset and filter VCF/BCF files
   -- VCF/BCF analysis
      call         SNP/indel calling
      consensus    create consensus sequence by applying VCF variants
      csq          call variation consequences
      filter       filter VCF/BCF files using fixed thresholds
      gtcheck      check sample concordance
      roh          identify runs of autozygosity
      stats        produce VCF/BCF stats

Use "bcftools <command>" to see command-specific help."#;

        // First, summarize the docs (like the LLM pipeline does)
        let summarized = summarize_docs(bcftools_doc, 6000);
        println!("Summarized docs:\n{}", summarized);
        println!("--- End summarized docs ---");

        // Then process the summarized docs
        let structured = processor.clean_and_structure(&summarized);

        println!("From summarized - Subcommands: {:?}", structured.subcommands);
        println!("From summarized - Has subcommands: {}", structured.has_subcommands);
        println!("From summarized - Commands field: '{}'", structured.commands);

        // Should still detect subcommands even after summarization
        assert!(structured.has_subcommands, "Should detect subcommands after summarization");
        assert!(!structured.subcommands.is_empty(), "Should have non-empty subcommands after summarization, got: {:?}", structured.subcommands);

        // Test what happens when we format the structured doc and re-parse it
        // (this is what happens in the actual pipeline)
        let formatted = format!("{}", structured);
        println!("Formatted structured doc:\n{}", formatted);
        println!("--- End formatted ---");

        let reparsed = processor.clean_and_structure(&formatted);
        println!("From reparsed - Commands field: '{}' (len={})", reparsed.commands, reparsed.commands.len());
        println!("From reparsed - Subcommands: {:?}", reparsed.subcommands);
    }

    #[test]
    fn test_canu_style_extraction() {
        let processor = DocProcessor::new();
        // Canu help text format (simplified) - NOTE: canu does NOT have subcommands!
        // The "Commands:" section describes MODES/PIPELINE STAGES, not subcommands
        let canu_doc = r#"USAGE:
  canu [-haplotype] [-options] [-help] \
       [-version]

DESCRIPTION:
  canu is a next-generation sequencing read assembler.

COMMANDS:
  denovo          Assemble reads de novo.
  genome          Assemble a genome.
  meta-assembly   Assemble a metagenome.
  assembly        Run just the assembly step.
  correct         Run just the correction step.
  trim            Run just the trimming step.

OPTIONS:
  -p <name>       Assembly name (required)
  -d <dir>        Output directory (required)
  genomeSize=<X>  Estimated genome size (required)
  -nanopore-raw   Input is raw ONT data
  -pacbio-hifi    Input is PacBio HiFi data
  maxMemory=<X>   Maximum memory to use
  maxThreads=<N>  Maximum threads to use

EXAMPLES:
  canu -p ecoli -d ecoli_asm genomeSize=5m -nanopore-raw reads.fastq.gz
  canu -p asm -d out genomeSize=3g -pacbio-hifi reads.bam maxThreads=8"#;

        let structured = processor.clean_and_structure(canu_doc);

        println!("Canu subcommands: {:?}", structured.subcommands);
        println!("Has subcommands: {}", structured.has_subcommands);
        println!("Commands field: '{}'", structured.commands);

        // IMPORTANT: canu does NOT have subcommands - the "Commands:" section
        // describes pipeline stages that are selected via OPTIONS (-assemble, -correct, -trim)
        // NOT positional subcommands. First token is always a flag.
        assert!(!structured.has_subcommands, "Canu should NOT have subcommands - first token is always a flag");
    }
}
