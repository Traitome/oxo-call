//! Enhanced Subcommand Detector v2
//!
//! Provides intelligent subcommand detection and task-based selection
//! for CLI tools with subcommand patterns.

#![allow(dead_code)]

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Definition of a subcommand with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubcommandDef {
    /// Subcommand name
    pub name: String,
    /// Description of what the subcommand does
    pub description: String,
    /// Usage pattern for this subcommand
    pub usage_pattern: String,
    /// Flags specific to this subcommand
    pub flags: Vec<String>,
    /// Keywords extracted for matching
    pub keywords: Vec<String>,
}

impl SubcommandDef {
    /// Create a new subcommand definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            usage_pattern: String::new(),
            flags: Vec::new(),
            keywords: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self.extract_keywords();
        self
    }

    /// Set the usage pattern
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage_pattern = usage.into();
        self
    }

    /// Add a flag
    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.push(flag.into());
        self
    }

    /// Extract keywords from name and description
    fn extract_keywords(&mut self) {
        let text = format!("{} {}", self.name, self.description).to_lowercase();

        // Extract action verbs
        let verbs = [
            "sort",
            "index",
            "view",
            "filter",
            "merge",
            "convert",
            "extract",
            "align",
            "call",
            "stats",
            "coverage",
            "intersect",
            "subtract",
            "annotate",
            "validate",
            "compress",
            "decompress",
            "split",
            "join",
            "query",
            "search",
            "count",
            "summarize",
            "plot",
            "compute",
            "mark",
            "remove",
            "add",
            "get",
            "set",
            "list",
            "show",
        ];

        for verb in &verbs {
            if text.contains(verb) {
                self.keywords.push(verb.to_string());
            }
        }

        // Extract file format keywords
        let formats = [
            "bam", "sam", "cram", "vcf", "bcf", "bed", "gff", "gtf", "gff3", "fasta", "fastq",
            "sam", "cram", "bed", "vcf", "bcf",
        ];

        for fmt in &formats {
            if text.contains(fmt) {
                self.keywords.push(fmt.to_string());
            }
        }
    }

    /// Calculate keyword match score against a task
    pub fn keyword_match_score(&self, task: &str) -> f32 {
        let task_lower = task.to_lowercase();
        let mut score = 0.0f32;

        // Direct name match is strongest signal
        if task_lower.contains(&self.name.to_lowercase()) {
            score += 0.5;
        }

        // Check for action verbs in task
        let task_verbs = extract_verbs(&task_lower);
        let cmd_verbs: std::collections::HashSet<_> = self.keywords.iter().cloned().collect();

        for verb in &task_verbs {
            if cmd_verbs.contains(verb) {
                score += 0.25;
            }
        }

        // Check for format mentions
        let formats = ["bam", "sam", "vcf", "bcf", "bed", "gff", "fasta", "fastq"];
        for fmt in &formats {
            if task_lower.contains(fmt) && self.keywords.contains(&fmt.to_string()) {
                score += 0.15;
            }
        }

        score.min(1.0)
    }
}

/// Extract action verbs from text
fn extract_verbs(text: &str) -> Vec<String> {
    let verbs = [
        "sort",
        "index",
        "view",
        "filter",
        "merge",
        "convert",
        "extract",
        "align",
        "call",
        "stats",
        "coverage",
        "intersect",
        "subtract",
        "annotate",
        "validate",
        "compress",
        "decompress",
        "split",
        "join",
        "query",
        "search",
        "count",
        "summarize",
        "plot",
        "compute",
        "mark",
        "remove",
        "add",
        "get",
        "set",
        "list",
        "show",
        "trim",
        "quality",
        "map",
        "assemble",
        "variant",
        "duplicate",
    ];

    verbs
        .iter()
        .filter(|&&v| text.contains(v))
        .map(|&v| v.to_string())
        .collect()
}

/// Regex patterns for subcommand extraction
static SUBCOMMAND_LINE_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match lines like "  view    View SAM/BAM/CRAM" or "sort     Sort alignments"
    Regex::new(r"^\s{2,}([a-zA-Z][a-zA-Z0-9_-]*)\s{2,}(.+)$").unwrap()
});

#[allow(dead_code)]
static USAGE_SUBCOMMAND_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match "Usage: tool <command>" or "Usage: tool <subcommand>"
    Regex::new(r"(?i)usage:\s*\S+\s+<(?:command|subcommand|cmd)>").unwrap()
});

static COMMANDS_SECTION_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(commands|available\s+commands|subcommands):\s*$").unwrap()
});

/// Enhanced Subcommand Detector v2
#[derive(Debug, Clone, Default)]
pub struct SubcommandDetectorV2;

impl SubcommandDetectorV2 {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    /// Detect all subcommands from documentation
    pub fn detect(&self, raw_doc: &str, tool_name: &str) -> Vec<SubcommandDef> {
        let mut subcommands = Vec::new();

        // Strategy 1: Extract from COMMANDS section
        subcommands.extend(self.extract_from_commands_section(raw_doc));

        // Strategy 2: Extract from USAGE patterns
        subcommands.extend(self.extract_from_usage(raw_doc, tool_name));

        // Strategy 3: Extract from OPTIONS section preamble (for some tools)
        subcommands.extend(self.extract_from_preamble(raw_doc));

        // Strategy 4: Tool-specific extraction
        subcommands.extend(self.tool_specific_extraction(raw_doc, tool_name));

        // Deduplicate and merge
        self.deduplicate_and_merge(subcommands)
    }

    /// Select the best subcommand for a given task
    pub fn select_for_task<'a>(
        &self,
        task: &str,
        subcommands: &'a [SubcommandDef],
    ) -> Option<&'a SubcommandDef> {
        if subcommands.is_empty() {
            return None;
        }

        // Score each subcommand
        let mut scored: Vec<_> = subcommands
            .iter()
            .map(|sc| {
                let score = sc.keyword_match_score(task);
                (sc, score)
            })
            .filter(|(_, score)| *score > 0.1) // Minimum threshold
            .collect();

        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return the best match if score is high enough
        scored
            .first()
            .filter(|(_, score)| *score >= 0.2)
            .map(|(sc, _)| *sc)
    }

    /// Extract subcommands from COMMANDS section
    fn extract_from_commands_section(&self, raw_doc: &str) -> Vec<SubcommandDef> {
        let mut subcommands = Vec::new();
        let lines: Vec<_> = raw_doc.lines().collect();

        let mut in_commands_section = false;
        let mut blank_line_count = 0;

        for line in &lines {
            // Check for section header
            if COMMANDS_SECTION_HEADER.is_match(line) {
                in_commands_section = true;
                continue;
            }

            // Check for section end (next header or too many blank lines)
            if in_commands_section {
                if line.trim().is_empty() {
                    blank_line_count += 1;
                    if blank_line_count > 2 {
                        in_commands_section = false;
                    }
                    continue;
                }

                // Check if this is another section header
                if self.is_section_header(line) && !line.to_lowercase().contains("command") {
                    in_commands_section = false;
                    continue;
                }

                blank_line_count = 0;

                // Try to parse subcommand line
                if let Some(sc) = self.parse_subcommand_line(line) {
                    subcommands.push(sc);
                }
            }
        }

        subcommands
    }

    /// Extract subcommands from USAGE patterns
    fn extract_from_usage(&self, raw_doc: &str, tool_name: &str) -> Vec<SubcommandDef> {
        let mut subcommands = Vec::new();

        // Look for usage lines that include subcommands
        let usage_re = Regex::new(&format!(
            r"(?i)usage:\s*{}\s+(\w+)",
            regex::escape(tool_name)
        ))
        .unwrap();

        for cap in usage_re.captures_iter(raw_doc) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str().to_string();
                // Skip if it looks like an option, not a subcommand
                if !name.starts_with('-') && name.len() < 30 {
                    subcommands.push(SubcommandDef::new(name));
                }
            }
        }

        subcommands
    }

    /// Extract from preamble (text before OPTIONS)
    fn extract_from_preamble(&self, raw_doc: &str) -> Vec<SubcommandDef> {
        let mut subcommands = Vec::new();

        // Some tools describe subcommands in the description
        // Look for patterns like "The sort command..." or "Use 'view' to..."
        let cmd_desc_re = Regex::new(
            "(?i)(?:the|use)\\s+['\x22]?([a-zA-Z][a-zA-Z0-9_-]*)['\x22]?\\s+(?:command|to|will)",
        )
        .unwrap();

        for cap in cmd_desc_re.captures_iter(raw_doc) {
            if let Some(m) = cap.get(1) {
                let name = m.as_str().to_string();
                if name.len() > 2 && !name.starts_with('-') {
                    subcommands.push(SubcommandDef::new(name));
                }
            }
        }

        subcommands
    }

    /// Tool-specific extraction for known patterns
    fn tool_specific_extraction(&self, raw_doc: &str, tool_name: &str) -> Vec<SubcommandDef> {
        let mut subcommands = Vec::new();
        let name_lower = tool_name.to_lowercase();

        // GATK-style: look for tool names in the documentation
        if name_lower == "gatk" || raw_doc.contains("Genome Analysis Toolkit") {
            let gatk_tool_re = Regex::new(r"(?i)(\w+):\s*(?:Tool|Walker|Module)").unwrap();
            for cap in gatk_tool_re.captures_iter(raw_doc) {
                if let Some(m) = cap.get(1) {
                    subcommands.push(SubcommandDef::new(m.as_str()));
                }
            }
        }

        // samtools-style: look for commands in the initial description
        if name_lower == "samtools" {
            let cmds = ["view", "sort", "index", "flagstat", "depth", "merge"];
            for cmd in &cmds {
                if raw_doc.to_lowercase().contains(&format!(" {} ", cmd)) {
                    subcommands.push(SubcommandDef::new(*cmd));
                }
            }
        }

        subcommands
    }

    /// Parse a single subcommand line
    fn parse_subcommand_line(&self, line: &str) -> Option<SubcommandDef> {
        // Try standard format: "  name    description"
        if let Some(caps) = SUBCOMMAND_LINE_RE.captures(line) {
            let name = caps.get(1)?.as_str().to_string();
            let desc = caps.get(2)?.as_str().to_string();

            // Filter out common false positives
            let false_positives = [
                "usage",
                "options",
                "arguments",
                "examples",
                "description",
                "note",
                "see",
                "copyright",
                "license",
                "author",
                "version",
                "help",
            ];
            if false_positives.contains(&name.to_lowercase().as_str()) {
                return None;
            }

            return Some(SubcommandDef::new(name).with_description(desc));
        }

        None
    }

    /// Check if a line is a section header
    fn is_section_header(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // All caps with colon
        if trimmed.len() > 3
            && trimmed
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace() || c == ':')
        {
            return true;
        }

        // Underlined headers (=== or ---)
        if trimmed.starts_with("===") || trimmed.starts_with("---") {
            return true;
        }

        false
    }

    /// Deduplicate subcommands and merge information
    fn deduplicate_and_merge(&self, subcommands: Vec<SubcommandDef>) -> Vec<SubcommandDef> {
        let mut map: HashMap<String, SubcommandDef> = HashMap::new();

        for sc in subcommands {
            let key = sc.name.to_lowercase();
            if let Some(existing) = map.get_mut(&key) {
                // Merge information
                if existing.description.is_empty() && !sc.description.is_empty() {
                    existing.description = sc.description;
                }
                if existing.usage_pattern.is_empty() && !sc.usage_pattern.is_empty() {
                    existing.usage_pattern = sc.usage_pattern;
                }
                for flag in sc.flags {
                    if !existing.flags.contains(&flag) {
                        existing.flags.push(flag);
                    }
                }
            } else {
                map.insert(key, sc);
            }
        }

        let mut result: Vec<_> = map.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Get common subcommands for a tool (fallback)
    pub fn get_common_subcommands(&self, tool_name: &str) -> Vec<SubcommandDef> {
        let name_lower = tool_name.to_lowercase();

        match name_lower.as_str() {
            "samtools" => vec![
                SubcommandDef::new("view").with_description("View SAM/BAM/CRAM files"),
                SubcommandDef::new("sort").with_description("Sort alignments"),
                SubcommandDef::new("index").with_description("Index sorted alignments"),
                SubcommandDef::new("flagstat").with_description("Show alignment statistics"),
            ],
            "bcftools" => vec![
                SubcommandDef::new("view").with_description("View VCF/BCF files"),
                SubcommandDef::new("filter").with_description("Filter variants"),
                SubcommandDef::new("merge").with_description("Merge VCF/BCF files"),
                SubcommandDef::new("stats").with_description("Calculate statistics"),
            ],
            "bedtools" => vec![
                SubcommandDef::new("intersect").with_description("Find overlapping intervals"),
                SubcommandDef::new("merge").with_description("Merge overlapping intervals"),
                SubcommandDef::new("sort").with_description("Sort interval files"),
                SubcommandDef::new("coverage").with_description("Compute coverage"),
            ],
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching() {
        let sc = SubcommandDef::new("sort").with_description("Sort BAM files by coordinate");

        let score1 = sc.keyword_match_score("sort BAM file");
        assert!(score1 > 0.5, "Should match 'sort' keyword");

        let score2 = sc.keyword_match_score("align reads to genome");
        assert!(score2 < 0.3, "Should not match unrelated task");
    }

    #[test]
    fn test_extract_from_commands_section() {
        let detector = SubcommandDetectorV2::new();

        let doc = r#"
Some description here.

Commands:
  view     View SAM/BAM/CRAM files
  sort     Sort alignment files
  index    Index sorted alignments

Options:
  -h       Show help
"#;

        let cmds = detector.extract_from_commands_section(doc);
        assert!(!cmds.is_empty());
        assert!(cmds.iter().any(|c| c.name == "view"));
        assert!(cmds.iter().any(|c| c.name == "sort"));
    }

    #[test]
    fn test_select_for_task() {
        let detector = SubcommandDetectorV2::new();

        let subcommands = vec![
            SubcommandDef::new("sort").with_description("Sort alignments"),
            SubcommandDef::new("view").with_description("View BAM files"),
            SubcommandDef::new("index").with_description("Create index"),
        ];

        let selected = detector.select_for_task("sort the BAM file", &subcommands);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "sort");
    }

    #[test]
    fn test_subcommand_def_builder() {
        let sc = SubcommandDef::new("view")
            .with_description("View SAM/BAM/CRAM files")
            .with_usage("samtools view [options] <in.bam>")
            .with_flag("-b")
            .with_flag("-h");

        assert_eq!(sc.name, "view");
        assert_eq!(sc.description, "View SAM/BAM/CRAM files");
        assert_eq!(sc.usage_pattern, "samtools view [options] <in.bam>");
        assert!(sc.flags.contains(&"-b".to_string()));
        assert!(sc.flags.contains(&"-h".to_string()));
        assert!(!sc.keywords.is_empty());
    }

    #[test]
    fn test_keyword_match_score_name_match() {
        let sc = SubcommandDef::new("sort").with_description("Sort BAM files by coordinate");
        let score = sc.keyword_match_score("sort the BAM file");
        assert!(score > 0.5);
    }

    #[test]
    fn test_keyword_match_score_format_match() {
        let sc = SubcommandDef::new("view").with_description("View BAM files");
        let score = sc.keyword_match_score("view the bam file");
        assert!(score > 0.3);
    }

    #[test]
    fn test_keyword_match_score_no_match() {
        let sc = SubcommandDef::new("sort").with_description("Sort alignments");
        let score = sc.keyword_match_score("align reads to genome");
        assert!(score < 0.3);
    }

    #[test]
    fn test_keyword_match_score_max_one() {
        let sc = SubcommandDef::new("sort").with_description("Sort BAM files by coordinate sort");
        let score = sc.keyword_match_score("sort BAM file sort");
        assert!(score <= 1.0);
    }

    #[test]
    fn test_extract_verbs() {
        let verbs = extract_verbs("sort and filter the bam file then index it");
        assert!(verbs.contains(&"sort".to_string()));
        assert!(verbs.contains(&"filter".to_string()));
        assert!(verbs.contains(&"index".to_string()));
    }

    #[test]
    fn test_extract_verbs_empty() {
        let verbs = extract_verbs("hello world");
        assert!(verbs.is_empty());
    }

    #[test]
    fn test_detect_full_pipeline() {
        let detector = SubcommandDetectorV2::new();
        let doc = r#"
samtools - Tools for dealing with SAM, BAM and CRAM

Commands:
  view     View SAM/BAM/CRAM files
  sort     Sort alignment files
  index    Index sorted alignments
  flagstat Show alignment statistics
  depth    Compute depth coverage

Options:
  -h       Show help
  --version Show version
"#;
        let subcmds = detector.detect(doc, "samtools");
        assert!(!subcmds.is_empty());
        assert!(subcmds.iter().any(|c| c.name == "view"));
        assert!(subcmds.iter().any(|c| c.name == "sort"));
    }

    #[test]
    fn test_detect_from_usage_patterns() {
        let detector = SubcommandDetectorV2::new();
        let doc = r#"Usage: git commit [options]
Usage: git push [options]
Usage: git pull [options]"#;
        let subcmds = detector.extract_from_usage(doc, "git");
        assert!(subcmds.iter().any(|c| c.name == "commit"));
        assert!(subcmds.iter().any(|c| c.name == "push"));
        assert!(subcmds.iter().any(|c| c.name == "pull"));
    }

    #[test]
    fn test_detect_from_preamble() {
        let detector = SubcommandDetectorV2::new();
        let doc = "Use 'sort' to sort alignments. The view command displays BAM files.";
        let subcmds = detector.extract_from_preamble(doc);
        assert!(subcmds.iter().any(|c| c.name == "sort"));
    }

    #[test]
    fn test_tool_specific_extraction_gatk() {
        let detector = SubcommandDetectorV2::new();
        let doc = "Genome Analysis Toolkit\nHaplotypeCaller: Tool for calling variants\nBaseRecalibrator: Walker for base quality";
        let subcmds = detector.tool_specific_extraction(doc, "gatk");
        assert!(!subcmds.is_empty());
    }

    #[test]
    fn test_tool_specific_extraction_samtools() {
        let detector = SubcommandDetectorV2::new();
        let doc = "samtools - Tools for alignments\n view sort index flagstat depth merge";
        let subcmds = detector.tool_specific_extraction(doc, "samtools");
        assert!(!subcmds.is_empty());
        assert!(subcmds.iter().any(|c| c.name == "view"));
        assert!(subcmds.iter().any(|c| c.name == "sort"));
    }

    #[test]
    fn test_tool_specific_extraction_unknown() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.tool_specific_extraction("some doc", "unknown_tool");
        assert!(subcmds.is_empty());
    }

    #[test]
    fn test_parse_subcommand_line_valid() {
        let detector = SubcommandDetectorV2::new();
        let result = detector.parse_subcommand_line("  sort     Sort alignment files");
        assert!(result.is_some());
        let sc = result.unwrap();
        assert_eq!(sc.name, "sort");
    }

    #[test]
    fn test_parse_subcommand_line_invalid_false_positive() {
        let detector = SubcommandDetectorV2::new();
        let result = detector.parse_subcommand_line("  usage     Show usage information");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_subcommand_line_options() {
        let detector = SubcommandDetectorV2::new();
        let result = detector.parse_subcommand_line("  options     Show options");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_subcommand_line_no_match() {
        let detector = SubcommandDetectorV2::new();
        let result = detector.parse_subcommand_line("not a subcommand line");
        assert!(result.is_none());
    }

    #[test]
    fn test_is_section_header_all_caps() {
        let detector = SubcommandDetectorV2::new();
        assert!(detector.is_section_header("OPTIONS:"));
        assert!(detector.is_section_header("COMMANDS"));
        assert!(!detector.is_section_header("Options"));
    }

    #[test]
    fn test_is_section_header_underline() {
        let detector = SubcommandDetectorV2::new();
        assert!(detector.is_section_header("==="));
        assert!(detector.is_section_header("---"));
        assert!(!detector.is_section_header("normal text"));
    }

    #[test]
    fn test_deduplicate_and_merge() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = vec![
            SubcommandDef::new("sort"),
            SubcommandDef::new("sort").with_description("Sort alignments"),
            SubcommandDef::new("view").with_description("View files"),
        ];
        let result = detector.deduplicate_and_merge(subcmds);
        assert_eq!(result.len(), 2);
        let sort = result.iter().find(|c| c.name == "sort").unwrap();
        assert_eq!(sort.description, "Sort alignments");
    }

    #[test]
    fn test_deduplicate_and_merge_flags() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = vec![
            SubcommandDef::new("sort").with_flag("-o"),
            SubcommandDef::new("sort").with_flag("-@"),
        ];
        let result = detector.deduplicate_and_merge(subcmds);
        assert_eq!(result.len(), 1);
        assert!(result[0].flags.contains(&"-o".to_string()));
        assert!(result[0].flags.contains(&"-@".to_string()));
    }

    #[test]
    fn test_deduplicate_sorted_output() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = vec![
            SubcommandDef::new("view"),
            SubcommandDef::new("sort"),
            SubcommandDef::new("index"),
        ];
        let result = detector.deduplicate_and_merge(subcmds);
        let names: Vec<&str> = result.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["index", "sort", "view"]);
    }

    #[test]
    fn test_select_for_task_empty() {
        let detector = SubcommandDetectorV2::new();
        let result = detector.select_for_task("sort", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_select_for_task_low_score() {
        let detector = SubcommandDetectorV2::new();
        let subcommands = vec![SubcommandDef::new("sort").with_description("Sort alignments")];
        let result = detector.select_for_task("completely unrelated task xyz", &subcommands);
        assert!(result.is_none());
    }

    #[test]
    fn test_get_common_subcommands_samtools() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.get_common_subcommands("samtools");
        assert!(!subcmds.is_empty());
        assert!(subcmds.iter().any(|c| c.name == "sort"));
        assert!(subcmds.iter().any(|c| c.name == "view"));
    }

    #[test]
    fn test_get_common_subcommands_bcftools() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.get_common_subcommands("bcftools");
        assert!(!subcmds.is_empty());
        assert!(subcmds.iter().any(|c| c.name == "view"));
    }

    #[test]
    fn test_get_common_subcommands_bedtools() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.get_common_subcommands("bedtools");
        assert!(!subcmds.is_empty());
        assert!(subcmds.iter().any(|c| c.name == "intersect"));
    }

    #[test]
    fn test_get_common_subcommands_unknown() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.get_common_subcommands("unknown_tool");
        assert!(subcmds.is_empty());
    }

    #[test]
    fn test_extract_from_commands_section_with_blank_lines() {
        let detector = SubcommandDetectorV2::new();
        let doc = r#"
Commands:
  view     View files


  sort     Sort files


Options:
  -h       Help
"#;
        let subcmds = detector.extract_from_commands_section(doc);
        assert!(subcmds.iter().any(|c| c.name == "view"));
        assert!(subcmds.iter().any(|c| c.name == "sort"));
    }

    #[test]
    fn test_extract_from_usage_skips_options() {
        let detector = SubcommandDetectorV2::new();
        let doc = "Usage: mytool --help\nUsage: mytool sort [options]";
        let subcmds = detector.extract_from_usage(doc, "mytool");
        assert!(subcmds.iter().any(|c| c.name == "sort"));
        assert!(!subcmds.iter().any(|c| c.name == "--help"));
    }

    #[test]
    fn test_subcommand_def_equality() {
        let a = SubcommandDef::new("sort").with_description("Sort");
        let b = SubcommandDef::new("sort").with_description("Sort");
        assert_eq!(a, b);
    }

    #[test]
    fn test_subcommand_def_inequality() {
        let a = SubcommandDef::new("sort");
        let b = SubcommandDef::new("view");
        assert_ne!(a, b);
    }

    #[test]
    fn test_detect_empty_doc() {
        let detector = SubcommandDetectorV2::new();
        let subcmds = detector.detect("", "tool");
        assert!(subcmds.is_empty());
    }
}
