//! Test Cases for RAG Benchmark
//!
//! Defines comprehensive test cases covering different CLI patterns
//! and bioinformatics tools.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// CLI pattern types for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CliPatternType {
    /// Simple single-command tools
    Simple,
    /// Tools with subcommands (samtools, bcftools, etc.)
    Subcommand,
    /// Meta-tools (deepTools, etc.)
    MetaTool,
    /// Multi-entry tools
    MultiEntry,
}

impl CliPatternType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliPatternType::Simple => "simple",
            CliPatternType::Subcommand => "subcommand",
            CliPatternType::MetaTool => "metatool",
            CliPatternType::MultiEntry => "multientry",
        }
    }
}

/// A single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Tool name
    pub tool: String,
    /// CLI pattern type
    pub pattern_type: CliPatternType,
    /// Task description
    pub task: String,
    /// Expected correct command (reference)
    pub expected_args: String,
    /// Category (alignment, variant_calling, etc.)
    pub category: String,
    /// Difficulty level (easy, medium, hard)
    pub difficulty: String,
    /// Key capabilities being tested
    pub tags: Vec<String>,
}

impl TestCase {
    pub fn new(
        tool: impl Into<String>,
        pattern_type: CliPatternType,
        task: impl Into<String>,
        expected_args: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            tool: tool.into(),
            pattern_type,
            task: task.into(),
            expected_args: expected_args.into(),
            category: category.into(),
            difficulty: "medium".to_string(),
            tags: Vec::new(),
        }
    }

    pub fn with_difficulty(mut self, difficulty: impl Into<String>) -> Self {
        self.difficulty = difficulty.into();
        self
    }

    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }
}

/// Complete test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub name: String,
    pub description: String,
    pub cases: Vec<TestCase>,
}

impl TestSuite {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            cases: Vec::new(),
        }
    }

    pub fn add_case(mut self, case: TestCase) -> Self {
        self.cases.push(case);
        self
    }

    /// Get test cases by pattern type
    pub fn by_pattern(&self, pattern: CliPatternType) -> Vec<&TestCase> {
        self.cases
            .iter()
            .filter(|c| c.pattern_type == pattern)
            .collect()
    }

    /// Get test cases by tool
    pub fn by_tool(&self, tool: &str) -> Vec<&TestCase> {
        self.cases.iter().filter(|c| c.tool == tool).collect()
    }

    /// Get test cases by difficulty
    pub fn by_difficulty(&self, difficulty: &str) -> Vec<&TestCase> {
        self.cases
            .iter()
            .filter(|c| c.difficulty == difficulty)
            .collect()
    }
}

/// Create comprehensive test suite for bioinformatics tools
pub fn create_bioinformatics_suite() -> TestSuite {
    let mut suite = TestSuite::new(
        "Bioinformatics CLI Benchmark",
        "Comprehensive test suite for bioinformatics command-line tools",
    );

    // === Simple Tools ===
    suite = suite
        .add_case(
            TestCase::new(
                "awk",
                CliPatternType::Simple,
                "print the first column of a TSV file",
                "-F'\t' '{print $1}'",
                "text_processing",
            )
            .with_difficulty("easy")
            .with_tags(vec!["column_extraction", "tsv"]),
        )
        .add_case(
            TestCase::new(
                "grep",
                CliPatternType::Simple,
                "find lines containing 'ERROR' in log file",
                "'ERROR' log.txt",
                "text_processing",
            )
            .with_difficulty("easy")
            .with_tags(vec!["pattern_matching", "filtering"]),
        );

    // === Subcommand Tools: samtools ===
    suite = suite
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "sort BAM file by coordinates",
                "sort -o sorted.bam input.bam",
                "alignment",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "output_flag", "sorting"]),
        )
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "index a sorted BAM file",
                "index sorted.bam",
                "alignment",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "indexing"]),
        )
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "view BAM header only",
                "view -H input.bam",
                "alignment",
            )
            .with_difficulty("medium")
            .with_tags(vec!["subcommand_required", "header", "view"]),
        )
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "filter alignments with mapping quality >= 30",
                "view -q 30 -b input.bam",
                "alignment",
            )
            .with_difficulty("medium")
            .with_tags(vec!["subcommand_required", "filtering", "quality"]),
        )
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "count reads in BAM file with flagstat",
                "flagstat input.bam",
                "alignment",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "statistics"]),
        );

    // === Subcommand Tools: bcftools ===
    suite = suite
        .add_case(
            TestCase::new(
                "bcftools",
                CliPatternType::Subcommand,
                "view VCF header",
                "view -h input.vcf",
                "variant_calling",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "header", "vcf"]),
        )
        .add_case(
            TestCase::new(
                "bcftools",
                CliPatternType::Subcommand,
                "filter variants with quality > 30",
                "filter -i 'QUAL>30' input.vcf",
                "variant_calling",
            )
            .with_difficulty("medium")
            .with_tags(vec!["subcommand_required", "filtering", "expression"]),
        )
        .add_case(
            TestCase::new(
                "bcftools",
                CliPatternType::Subcommand,
                "query sample names from VCF",
                "query -l input.vcf",
                "variant_calling",
            )
            .with_difficulty("medium")
            .with_tags(vec!["subcommand_required", "query", "samples"]),
        );

    // === Subcommand Tools: bedtools ===
    suite = suite
        .add_case(
            TestCase::new(
                "bedtools",
                CliPatternType::Subcommand,
                "intersect two BED files",
                "intersect -a file1.bed -b file2.bed",
                "interval_analysis",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "intersection", "bed"]),
        )
        .add_case(
            TestCase::new(
                "bedtools",
                CliPatternType::Subcommand,
                "merge overlapping regions in BED file",
                "merge -i input.bed",
                "interval_analysis",
            )
            .with_difficulty("easy")
            .with_tags(vec!["subcommand_required", "merge", "bed"]),
        )
        .add_case(
            TestCase::new(
                "bedtools",
                CliPatternType::Subcommand,
                "get coverage of BED intervals in BAM",
                "coverage -a intervals.bed -b alignments.bam",
                "interval_analysis",
            )
            .with_difficulty("medium")
            .with_tags(vec!["subcommand_required", "coverage", "bed", "bam"]),
        );

    // === Hard Cases: Complex subcommands ===
    suite = suite
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "convert SAM to BAM with 8 threads and sort by read name",
                "sort -n -@ 8 -o output.bam input.sam",
                "alignment",
            )
            .with_difficulty("hard")
            .with_tags(vec![
                "subcommand_required",
                "multithread",
                "name_sort",
                "conversion",
            ]),
        )
        .add_case(
            TestCase::new(
                "bcftools",
                CliPatternType::Subcommand,
                "normalize indels and left-align variants using reference",
                "norm -f reference.fa -o output.vcf input.vcf",
                "variant_calling",
            )
            .with_difficulty("hard")
            .with_tags(vec!["subcommand_required", "normalization", "reference"]),
        );

    suite
}

/// Create minimal quick test suite
pub fn create_quick_suite() -> TestSuite {
    let mut suite = TestSuite::new(
        "Quick RAG Benchmark",
        "Minimal test suite for quick RAG validation",
    );

    suite = suite
        .add_case(
            TestCase::new(
                "samtools",
                CliPatternType::Subcommand,
                "sort BAM file",
                "sort -o sorted.bam input.bam",
                "alignment",
            )
            .with_difficulty("easy"),
        )
        .add_case(
            TestCase::new(
                "bedtools",
                CliPatternType::Subcommand,
                "intersect BED files",
                "intersect -a a.bed -b b.bed",
                "interval_analysis",
            )
            .with_difficulty("easy"),
        )
        .add_case(
            TestCase::new(
                "bcftools",
                CliPatternType::Subcommand,
                "filter by quality",
                "filter -i 'QUAL>30' input.vcf",
                "variant_calling",
            )
            .with_difficulty("medium"),
        );

    suite
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bioinformatics_suite() {
        let suite = create_bioinformatics_suite();
        assert!(!suite.cases.is_empty());

        let subcommand_cases = suite.by_pattern(CliPatternType::Subcommand);
        assert!(!subcommand_cases.is_empty());
    }

    #[test]
    fn test_filter_by_tool() {
        let suite = create_bioinformatics_suite();
        let samtools_cases = suite.by_tool("samtools");
        assert!(!samtools_cases.is_empty());
    }

    #[test]
    fn test_quick_suite() {
        let suite = create_quick_suite();
        assert_eq!(suite.cases.len(), 3);
    }
}
