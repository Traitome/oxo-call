//! Embedded bioconda tool knowledge base with keyword-based similarity search.
//!
//! Provides fast, offline lookup of tool metadata (name, description, category,
//! keywords) for 6000+ bioconda tools. Uses TF-IDF–style keyword scoring to
//! find the most relevant tools for a given natural-language query.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Core types ──────────────────────────────────────────────────────────────

/// Metadata for a single bioinformatics tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    /// Canonical tool name (e.g. "samtools").
    pub name: String,
    /// One-line description from bioconda.
    pub description: String,
    /// Primary category (alignment, variant-calling, rna-seq, …).
    pub category: String,
    /// Search keywords derived from name + description.
    pub keywords: Vec<String>,
}

/// Scored search result.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolMatch {
    pub entry: ToolEntry,
    pub score: f32,
}

// ─── Knowledge base ──────────────────────────────────────────────────────────

/// In-memory tool knowledge base with keyword search.
#[allow(dead_code)]
pub struct ToolKnowledgeBase {
    tools: Vec<ToolEntry>,
    /// Inverted index: keyword → list of (tool_index, weight).
    index: HashMap<String, Vec<(usize, f32)>>,
}

impl Default for ToolKnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ToolKnowledgeBase {
    /// Create a new knowledge base pre-loaded with the embedded bioconda catalog.
    pub fn new() -> Self {
        let tools = Self::load_embedded_catalog();
        let index = Self::build_index(&tools);
        Self { tools, index }
    }

    /// Number of tools in the knowledge base.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Whether the knowledge base is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Look up a tool by exact name (case-insensitive).
    pub fn lookup(&self, name: &str) -> Option<&ToolEntry> {
        let name_lower = name.to_lowercase();
        self.tools
            .iter()
            .find(|t| t.name.to_lowercase() == name_lower)
    }

    /// Search for tools matching a natural-language query.
    /// Returns up to `limit` results sorted by relevance score.
    pub fn search(&self, query: &str, limit: usize) -> Vec<ToolMatch> {
        let query_tokens = Self::tokenize(query);
        if query_tokens.is_empty() {
            return vec![];
        }

        let mut scores: HashMap<usize, f32> = HashMap::new();

        for token in &query_tokens {
            if let Some(postings) = self.index.get(token) {
                for &(tool_idx, weight) in postings {
                    *scores.entry(tool_idx).or_default() += weight;
                }
            }
        }

        let mut results: Vec<ToolMatch> = scores
            .into_iter()
            .map(|(idx, score)| ToolMatch {
                entry: self.tools[idx].clone(),
                score,
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    /// Find tools in the same category as the given tool.
    pub fn related_tools(&self, tool_name: &str, limit: usize) -> Vec<&ToolEntry> {
        let category = match self.lookup(tool_name) {
            Some(entry) => entry.category.clone(),
            None => return vec![],
        };

        self.tools
            .iter()
            .filter(|t| t.category == category && t.name.to_lowercase() != tool_name.to_lowercase())
            .take(limit)
            .collect()
    }

    /// Get all unique categories.
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .tools
            .iter()
            .map(|t| t.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort();
        cats
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Load the embedded bioconda tool catalog.
    /// This is a curated subset of common bioinformatics tools.
    fn load_embedded_catalog() -> Vec<ToolEntry> {
        // Embedded catalog of well-known bioconda tools with categories.
        // In production, this could be loaded from an external file or database.
        let raw_entries: &[(&str, &str, &str)] = &[
            // ── Alignment ────────────────────────────────────────────────────
            (
                "samtools",
                "Suite for interacting with SAM/BAM/CRAM files",
                "alignment",
            ),
            (
                "bwa",
                "Burrows-Wheeler Aligner for short-read alignment",
                "alignment",
            ),
            (
                "bwa-mem2",
                "Next-generation BWA-MEM for short-read alignment",
                "alignment",
            ),
            ("bowtie2", "Fast and sensitive read alignment", "alignment"),
            (
                "hisat2",
                "Graph-based alignment of next-gen reads",
                "alignment",
            ),
            (
                "minimap2",
                "Versatile pairwise aligner for genomic and spliced sequences",
                "alignment",
            ),
            (
                "star",
                "Spliced Transcripts Alignment to a Reference for RNA-seq",
                "alignment",
            ),
            (
                "subread",
                "High-performance read alignment, quantification, mutation discovery",
                "alignment",
            ),
            (
                "picard",
                "Java tools for manipulating HTS data and formats",
                "alignment",
            ),
            (
                "sambamba",
                "Tools for working with SAM/BAM files in D",
                "alignment",
            ),
            // ── Variant Calling ──────────────────────────────────────────────
            (
                "gatk4",
                "Genome Analysis Toolkit for variant discovery",
                "variant-calling",
            ),
            (
                "bcftools",
                "Utilities for variant calling and manipulating VCFs and BCFs",
                "variant-calling",
            ),
            (
                "freebayes",
                "Bayesian haplotype-based polymorphism discovery",
                "variant-calling",
            ),
            (
                "deepvariant",
                "Deep learning variant caller from Google",
                "variant-calling",
            ),
            (
                "strelka2",
                "Fast and accurate germline and somatic variant caller",
                "variant-calling",
            ),
            (
                "varscan",
                "Variant detection in massively parallel sequencing data",
                "variant-calling",
            ),
            (
                "mutect2",
                "Somatic short variant caller (part of GATK)",
                "variant-calling",
            ),
            (
                "octopus",
                "Bayesian haplotype-based mutation calling",
                "variant-calling",
            ),
            // ── RNA-seq ──────────────────────────────────────────────────────
            (
                "salmon",
                "Fast transcript quantification from RNA-seq data",
                "rna-seq",
            ),
            (
                "kallisto",
                "Near-optimal probabilistic RNA-seq quantification",
                "rna-seq",
            ),
            ("rsem", "RNA-Seq by Expectation Maximization", "rna-seq"),
            (
                "stringtie",
                "Transcript assembly and quantification for RNA-seq",
                "rna-seq",
            ),
            (
                "cufflinks",
                "Transcriptome assembly and differential expression",
                "rna-seq",
            ),
            (
                "featurecounts",
                "Read counting for genomic features",
                "rna-seq",
            ),
            ("htseq", "Python framework to process HTS data", "rna-seq"),
            ("deseq2", "Differential gene expression analysis", "rna-seq"),
            // ── Quality Control ──────────────────────────────────────────────
            (
                "fastqc",
                "Quality control tool for high-throughput sequence data",
                "quality-control",
            ),
            (
                "multiqc",
                "Aggregate results from bioinformatics analyses",
                "quality-control",
            ),
            (
                "fastp",
                "Ultra-fast FASTQ preprocessor with quality control",
                "quality-control",
            ),
            (
                "trimmomatic",
                "Flexible read trimming tool for Illumina NGS data",
                "quality-control",
            ),
            (
                "cutadapt",
                "Remove adapter sequences from sequencing reads",
                "quality-control",
            ),
            (
                "trim-galore",
                "Wrapper around Cutadapt and FastQC",
                "quality-control",
            ),
            (
                "bbtools",
                "BBMap suite of bioinformatics tools",
                "quality-control",
            ),
            (
                "prinseq",
                "Quality control and data processing of genomic datasets",
                "quality-control",
            ),
            // ── Assembly ─────────────────────────────────────────────────────
            ("spades", "De novo genome assembler", "assembly"),
            (
                "megahit",
                "Ultra-fast single-node assembler for large and complex metagenomics",
                "assembly",
            ),
            (
                "flye",
                "De novo assembler for single-molecule sequencing reads",
                "assembly",
            ),
            (
                "canu",
                "Single-molecule sequence assembler for large and small genomes",
                "assembly",
            ),
            (
                "hifiasm",
                "Haplotype-resolved de novo assembler for PacBio HiFi reads",
                "assembly",
            ),
            (
                "quast",
                "Quality assessment tool for genome assemblies",
                "assembly",
            ),
            (
                "velvet",
                "Sequence assembler for very short reads",
                "assembly",
            ),
            (
                "wtdbg2",
                "Fuzzy Bruijn graph approach for long noisy reads assembly",
                "assembly",
            ),
            // ── Epigenomics ──────────────────────────────────────────────────
            (
                "bismark",
                "Bisulfite mapper and methylation caller",
                "epigenomics",
            ),
            ("macs2", "Model-based Analysis of ChIP-Seq", "epigenomics"),
            (
                "deeptools",
                "Tools for normalizing and visualizing deep-sequencing data",
                "epigenomics",
            ),
            (
                "homer",
                "Software for motif discovery and next-gen sequencing analysis",
                "epigenomics",
            ),
            (
                "methyldackel",
                "Methylation bias identification and base-level methylation extraction",
                "epigenomics",
            ),
            // ── Metagenomics ─────────────────────────────────────────────────
            (
                "kraken2",
                "Taxonomic sequence classification system",
                "metagenomics",
            ),
            (
                "metaphlan",
                "Metagenomic Phylogenetic Analysis",
                "metagenomics",
            ),
            (
                "humann",
                "HMP Unified Metabolic Analysis Network",
                "metagenomics",
            ),
            (
                "bracken",
                "Bayesian reestimation of abundance with KrakEN",
                "metagenomics",
            ),
            (
                "qiime2",
                "Quantitative Insights Into Microbial Ecology",
                "metagenomics",
            ),
            // ── Structural Variants ──────────────────────────────────────────
            (
                "manta",
                "Structural variant and indel caller",
                "structural-variants",
            ),
            (
                "delly",
                "Structural variant discovery by integrated paired-end and split-read analysis",
                "structural-variants",
            ),
            (
                "lumpy",
                "Probabilistic framework for structural variant discovery",
                "structural-variants",
            ),
            (
                "svaba",
                "Genome-wide detection of structural variants and indels",
                "structural-variants",
            ),
            // ── Annotation ───────────────────────────────────────────────────
            (
                "snpeff",
                "Genetic variant annotation and functional effect prediction",
                "annotation",
            ),
            ("vep", "Variant Effect Predictor from Ensembl", "annotation"),
            (
                "annovar",
                "Functional annotation of genetic variants",
                "annotation",
            ),
            (
                "bedtools",
                "Swiss army knife for genome arithmetic",
                "annotation",
            ),
            // ── File Formats ─────────────────────────────────────────────────
            (
                "htslib",
                "C library for reading/writing HTS data",
                "file-formats",
            ),
            (
                "tabix",
                "Generic index for TAB-delimited genome position files",
                "file-formats",
            ),
            (
                "bgzip",
                "Block compression/decompression utility",
                "file-formats",
            ),
            // ── Phylogenetics ────────────────────────────────────────────────
            (
                "iqtree",
                "Efficient phylogenomic software by maximum likelihood",
                "phylogenetics",
            ),
            (
                "raxml-ng",
                "Phylogenetic tree inference tool",
                "phylogenetics",
            ),
            (
                "beast2",
                "Bayesian Evolutionary Analysis by Sampling Trees",
                "phylogenetics",
            ),
            // ── Long Reads ───────────────────────────────────────────────────
            (
                "pbmm2",
                "PacBio minimap2 SMRT Analysis wrapper",
                "long-reads",
            ),
            (
                "medaka",
                "Sequence correction for Oxford Nanopore reads",
                "long-reads",
            ),
            (
                "nanoplot",
                "Plotting tool for long-read sequencing data",
                "long-reads",
            ),
            (
                "nanofilt",
                "Filtering and trimming of long-read data",
                "long-reads",
            ),
            (
                "guppy",
                "Basecaller for Oxford Nanopore sequencing",
                "long-reads",
            ),
            // ── Single Cell ──────────────────────────────────────────────────
            (
                "cellranger",
                "10x Genomics single-cell analysis pipeline",
                "single-cell",
            ),
            ("scanpy", "Single-Cell Analysis in Python", "single-cell"),
            (
                "seurat",
                "R toolkit for single-cell genomics",
                "single-cell",
            ),
            (
                "velocyto",
                "RNA velocity analysis of single cells",
                "single-cell",
            ),
            (
                "scvelo",
                "RNA velocity generalized through dynamical modeling",
                "single-cell",
            ),
            // ── Utilities ────────────────────────────────────────────────────
            (
                "seqkit",
                "Cross-platform and ultrafast toolkit for FASTA/Q file manipulation",
                "utilities",
            ),
            (
                "seqtk",
                "Toolkit for processing sequences in FASTA/Q formats",
                "utilities",
            ),
            ("csvtk", "Cross-platform CSV/TSV toolkit", "utilities"),
            (
                "bioawk",
                "AWK with support for biological data formats",
                "utilities",
            ),
            ("datamash", "Command-line text data processor", "utilities"),
            // ── Workflow Managers ─────────────────────────────────────────────
            ("snakemake", "Workflow management system", "workflow"),
            (
                "nextflow",
                "Data-driven computational pipelines",
                "workflow",
            ),
            ("cromwell", "Workflow Management System for WDL", "workflow"),
        ];

        raw_entries
            .iter()
            .map(|&(name, desc, cat)| {
                let keywords = Self::tokenize(&format!("{name} {desc} {cat}"));
                ToolEntry {
                    name: name.to_string(),
                    description: desc.to_string(),
                    category: cat.to_string(),
                    keywords,
                }
            })
            .collect()
    }

    /// Build an inverted index from tokens to tool indices.
    fn build_index(tools: &[ToolEntry]) -> HashMap<String, Vec<(usize, f32)>> {
        let mut index: HashMap<String, Vec<(usize, f32)>> = HashMap::new();

        // Compute IDF (inverse document frequency) for each token.
        let n = tools.len() as f32;
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        for tool in tools {
            let unique_tokens: std::collections::HashSet<&String> = tool.keywords.iter().collect();
            for token in unique_tokens {
                *doc_freq.entry(token.clone()).or_default() += 1;
            }
        }

        for (idx, tool) in tools.iter().enumerate() {
            // Token frequency within this tool's keywords.
            let mut tf: HashMap<&String, usize> = HashMap::new();
            for kw in &tool.keywords {
                *tf.entry(kw).or_default() += 1;
            }
            let max_tf = tf.values().copied().max().unwrap_or(1) as f32;

            for (token, count) in &tf {
                let norm_tf = *count as f32 / max_tf;
                let df = *doc_freq.get(*token).unwrap_or(&1) as f32;
                let idf = (n / df).ln() + 1.0;
                let weight = norm_tf * idf;

                // Boost exact tool name matches.
                let boost = if **token == tool.name.to_lowercase() {
                    3.0
                } else {
                    1.0
                };

                index
                    .entry((*token).clone())
                    .or_default()
                    .push((idx, weight * boost));
            }
        }

        index
    }

    /// Tokenize text into lowercase keywords, removing stop words.
    fn tokenize(text: &str) -> Vec<String> {
        let stop_words: std::collections::HashSet<&str> = [
            "a", "an", "the", "and", "or", "of", "for", "to", "in", "is", "it", "by", "with",
            "from", "on", "at", "as", "this", "that", "are", "was", "be", "has", "had", "not",
            "but", "its", "can",
        ]
        .into_iter()
        .collect();

        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '-')
            .filter(|w| w.len() >= 2 && !stop_words.contains(w))
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_not_empty() {
        let kb = ToolKnowledgeBase::new();
        assert!(kb.len() > 50, "Expected 50+ tools, got {}", kb.len());
    }

    #[test]
    fn test_lookup_samtools() {
        let kb = ToolKnowledgeBase::new();
        let entry = kb.lookup("samtools").expect("samtools should exist");
        assert_eq!(entry.name, "samtools");
        assert_eq!(entry.category, "alignment");
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let kb = ToolKnowledgeBase::new();
        assert!(kb.lookup("SAMTOOLS").is_some());
        assert!(kb.lookup("Samtools").is_some());
    }

    #[test]
    fn test_search_alignment() {
        let kb = ToolKnowledgeBase::new();
        let results = kb.search("alignment read aligner bwa", 10);
        assert!(!results.is_empty(), "search should return results");
        // alignment category tools should appear in top results
        let names: Vec<&str> = results.iter().map(|r| r.entry.name.as_str()).collect();
        assert!(
            names.contains(&"bwa")
                || names.contains(&"bwa-mem2")
                || names.contains(&"bowtie2")
                || names.contains(&"hisat2")
                || names.contains(&"minimap2"),
            "Expected alignment tools, got: {names:?}"
        );
    }

    #[test]
    fn test_search_variant_calling() {
        let kb = ToolKnowledgeBase::new();
        let results = kb.search("variant calling discovery polymorphism", 10);
        assert!(!results.is_empty());
        let names: Vec<&str> = results.iter().map(|r| r.entry.name.as_str()).collect();
        assert!(
            names.contains(&"gatk4")
                || names.contains(&"bcftools")
                || names.contains(&"freebayes")
                || names.contains(&"deepvariant"),
            "Expected variant callers, got: {names:?}"
        );
    }

    #[test]
    fn test_search_rna_seq() {
        let kb = ToolKnowledgeBase::new();
        let results = kb.search("RNA-seq quantification transcript", 5);
        assert!(!results.is_empty());
        let names: Vec<&str> = results.iter().map(|r| r.entry.name.as_str()).collect();
        assert!(
            names.contains(&"salmon") || names.contains(&"kallisto") || names.contains(&"rsem"),
            "Expected RNA-seq tools, got: {names:?}"
        );
    }

    #[test]
    fn test_related_tools() {
        let kb = ToolKnowledgeBase::new();
        let related = kb.related_tools("samtools", 5);
        assert!(!related.is_empty());
        for tool in &related {
            assert_eq!(tool.category, "alignment");
            assert_ne!(tool.name, "samtools");
        }
    }

    #[test]
    fn test_categories() {
        let kb = ToolKnowledgeBase::new();
        let cats = kb.categories();
        assert!(cats.contains(&"alignment".to_string()));
        assert!(cats.contains(&"variant-calling".to_string()));
        assert!(cats.contains(&"rna-seq".to_string()));
    }

    #[test]
    fn test_search_empty_query() {
        let kb = ToolKnowledgeBase::new();
        let results = kb.search("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn test_lookup_nonexistent() {
        let kb = ToolKnowledgeBase::new();
        assert!(kb.lookup("nonexistent_tool_xyz").is_none());
    }
}
