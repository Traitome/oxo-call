//! Bioinformatics best practices knowledge base.
//!
//! Provides domain-specific best practices that can be injected into LLM prompts
//! to improve command generation quality, especially for small models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A best practice recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPractice {
    /// Category this practice applies to.
    pub category: String,
    /// Short title.
    pub title: String,
    /// Detailed recommendation.
    pub recommendation: String,
    /// Tools this applies to (empty = universal).
    pub tools: Vec<String>,
}

/// In-memory best practices database.
pub struct BestPracticesDb {
    practices: Vec<BestPractice>,
    /// Index: category → practice indices.
    #[allow(dead_code)]
    category_index: HashMap<String, Vec<usize>>,
    /// Index: tool name → practice indices.
    tool_index: HashMap<String, Vec<usize>>,
}

impl Default for BestPracticesDb {
    fn default() -> Self {
        Self::new()
    }
}

impl BestPracticesDb {
    /// Create a new DB loaded with embedded best practices.
    pub fn new() -> Self {
        let practices = Self::load_embedded();
        let mut category_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut tool_index: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, p) in practices.iter().enumerate() {
            category_index
                .entry(p.category.clone())
                .or_default()
                .push(idx);
            for tool in &p.tools {
                tool_index.entry(tool.to_lowercase()).or_default().push(idx);
            }
        }

        Self {
            practices,
            category_index,
            tool_index,
        }
    }

    /// Get all practices relevant to a specific tool.
    pub fn for_tool(&self, tool: &str) -> Vec<&BestPractice> {
        let tool_lower = tool.to_lowercase();
        let mut indices: Vec<usize> = self
            .tool_index
            .get(&tool_lower)
            .map(|v| v.to_vec())
            .unwrap_or_default();

        // Also include universal practices (empty tools list).
        for (idx, p) in self.practices.iter().enumerate() {
            if p.tools.is_empty() && !indices.contains(&idx) {
                indices.push(idx);
            }
        }

        indices.into_iter().map(|i| &self.practices[i]).collect()
    }

    /// Get practices for a category.
    #[allow(dead_code)]
    pub fn for_category(&self, category: &str) -> Vec<&BestPractice> {
        self.category_index
            .get(category)
            .map(|indices| indices.iter().map(|&i| &self.practices[i]).collect())
            .unwrap_or_default()
    }

    /// Format relevant best practices as a prompt injection string.
    pub fn to_prompt_hint(&self, tool: &str) -> String {
        let practices = self.for_tool(tool);
        if practices.is_empty() {
            return String::new();
        }

        let mut lines = vec!["[Best Practices]".to_string()];
        for p in practices.iter().take(5) {
            lines.push(format!("• {}: {}", p.title, p.recommendation));
        }
        lines.join("\n")
    }

    /// Total number of practices.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.practices.len()
    }

    /// Whether the DB is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.practices.is_empty()
    }

    // ── Embedded data ────────────────────────────────────────────────────────

    fn load_embedded() -> Vec<BestPractice> {
        vec![
            // ── Threading practices (conditional - only when task mentions threads) ──────────────────────────────────────────
            BestPractice {
                category: "performance".to_string(),
                title: "Use threads when explicitly requested".to_string(),
                recommendation: "When the task mentions threads/cores, use -@ (samtools), -t (bwa), --threads (many tools). Do NOT add thread counts unless the task explicitly requests them.".to_string(),
                tools: vec!["samtools".to_string(), "bwa".to_string(), "bcftools".to_string(), "gatk4".to_string(), "star".to_string(), "hisat2".to_string(), "bowtie2".to_string()],
            },
            BestPractice {
                category: "general".to_string(),
                title: "Always specify output files explicitly".to_string(),
                recommendation: "Use output flags (e.g., -o or --output) to specify output files rather than relying on stdout redirection. This prevents partial writes on failure and makes commands more readable.".to_string(),
                tools: vec!["samtools".to_string(), "bcftools".to_string(), "gatk4".to_string(), "bwa".to_string(), "hisat2".to_string(), "star".to_string(), "bowtie2".to_string()],
            },
            BestPractice {
                category: "general".to_string(),
                title: "Pipe-friendly processing".to_string(),
                recommendation: "For large files, prefer piping between tools (e.g., samtools view | samtools sort) to avoid writing intermediate files to disk.".to_string(),
                tools: vec!["samtools".to_string(), "bwa".to_string(), "picard".to_string(), "bcftools".to_string()],
            },
            // ── Alignment ────────────────────────────────────────────────────
            BestPractice {
                category: "alignment".to_string(),
                title: "Sort BAM by coordinate after alignment".to_string(),
                recommendation: "Always coordinate-sort BAM files after alignment. Most downstream tools (variant callers, viewers) require coordinate-sorted BAM.".to_string(),
                tools: vec!["samtools".to_string(), "bwa".to_string(), "hisat2".to_string()],
            },
            BestPractice {
                category: "alignment".to_string(),
                title: "Mark duplicates before variant calling".to_string(),
                recommendation: "Run duplicate marking (samtools markdup or Picard MarkDuplicates) before variant calling to avoid PCR artefact bias.".to_string(),
                tools: vec!["samtools".to_string(), "picard".to_string(), "gatk4".to_string()],
            },
            BestPractice {
                category: "alignment".to_string(),
                title: "Index BAM files after sorting".to_string(),
                recommendation: "Always create a .bai index after sorting: `samtools index sorted.bam`. Most tools require indexed BAM.".to_string(),
                tools: vec!["samtools".to_string()],
            },
            // ── Variant Calling ──────────────────────────────────────────────
            BestPractice {
                category: "variant-calling".to_string(),
                title: "Use BQSR for GATK pipelines".to_string(),
                recommendation: "Apply Base Quality Score Recalibration (BQSR) before calling variants with GATK HaplotypeCaller for improved accuracy.".to_string(),
                tools: vec!["gatk4".to_string()],
            },
            BestPractice {
                category: "variant-calling".to_string(),
                title: "Filter variants after calling".to_string(),
                recommendation: "Always apply quality filters (QUAL, DP, MQ) after variant calling. Use bcftools filter or GATK VariantFiltration.".to_string(),
                tools: vec!["bcftools".to_string(), "gatk4".to_string()],
            },
            // ── RNA-seq ──────────────────────────────────────────────────────
            BestPractice {
                category: "rna-seq".to_string(),
                title: "Use splice-aware aligner for RNA-seq".to_string(),
                recommendation: "For RNA-seq data, use a splice-aware aligner (STAR, HISAT2) rather than a DNA aligner (bwa). This correctly handles reads spanning exon junctions.".to_string(),
                tools: vec!["star".to_string(), "hisat2".to_string()],
            },
            BestPractice {
                category: "rna-seq".to_string(),
                title: "Prefer Salmon/Kallisto for transcript quantification".to_string(),
                recommendation: "For transcript-level quantification, salmon and kallisto are faster and more accurate than alignment-based counting (featureCounts/HTSeq).".to_string(),
                tools: vec!["salmon".to_string(), "kallisto".to_string()],
            },
            // ── Quality Control ──────────────────────────────────────────────
            BestPractice {
                category: "quality-control".to_string(),
                title: "Run QC before and after processing".to_string(),
                recommendation: "Run FastQC/MultiQC on raw reads AND after trimming/alignment to verify each step improved data quality.".to_string(),
                tools: vec!["fastqc".to_string(), "multiqc".to_string()],
            },
            BestPractice {
                category: "quality-control".to_string(),
                title: "Trim adapters before alignment".to_string(),
                recommendation: "Use fastp or trim-galore to remove adapters and low-quality bases before alignment. Default adapter detection usually works well.".to_string(),
                tools: vec!["fastp".to_string(), "trimmomatic".to_string(), "trim-galore".to_string()],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_best_practices_not_empty() {
        let db = BestPracticesDb::new();
        assert!(db.len() > 5, "Expected 5+ practices, got {}", db.len());
    }

    #[test]
    fn test_for_tool_samtools() {
        let db = BestPracticesDb::new();
        let practices = db.for_tool("samtools");
        assert!(
            practices.len() >= 3,
            "samtools should have universal + specific practices"
        );
    }

    #[test]
    fn test_for_tool_universal() {
        let db = BestPracticesDb::new();
        // An unknown tool should get no practices (all are tool-specific now).
        let practices = db.for_tool("unknown_tool");
        assert!(
            practices.is_empty(),
            "unknown tool should get no practices when all are tool-specific"
        );
        // But a known tool should get practices.
        let samtools_practices = db.for_tool("samtools");
        assert!(
            !samtools_practices.is_empty(),
            "known tool should get practices"
        );
    }

    #[test]
    fn test_for_category() {
        let db = BestPracticesDb::new();
        let alignment = db.for_category("alignment");
        assert!(!alignment.is_empty());
    }

    #[test]
    fn test_prompt_hint_format() {
        let db = BestPracticesDb::new();
        let hint = db.to_prompt_hint("samtools");
        assert!(hint.contains("[Best Practices]"));
        assert!(hint.contains("•"));
    }

    #[test]
    fn test_prompt_hint_unknown_tool() {
        let db = BestPracticesDb::new();
        let hint = db.to_prompt_hint("unknown_tool");
        // No universal practices exist now, so unknown tools get no hint.
        assert!(hint.is_empty(), "unknown tool should get no hint");
    }
}
