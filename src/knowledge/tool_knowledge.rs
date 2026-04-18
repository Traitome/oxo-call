//! Embedded bioconda tool knowledge base with keyword-based similarity search.
//!
//! Provides fast, offline lookup of tool metadata (name, description, category,
//! keywords) for 6000+ bioconda tools. Uses TF-IDF–style keyword scoring to
//! find the most relevant tools for a given natural-language query.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw JSONL data embedded at compile time.
const BIOCONDA_JSONL: &str = include_str!("../../data/bioconda_tools_metadata.jsonl");

/// A single JSONL record from the bioconda metadata file.
#[derive(Deserialize)]
struct BiocondaRecord {
    name: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    description: String,
}

/// Boost factor applied to exact tool-name matches in the search index.
const TOOL_NAME_BOOST: f32 = 3.0;

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

    /// Load the full bioconda tool catalog from the embedded JSONL data.
    fn load_embedded_catalog() -> Vec<ToolEntry> {
        BIOCONDA_JSONL
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let record: BiocondaRecord = serde_json::from_str(line).ok()?;
                let description = if record.summary.is_empty() {
                    record.description.clone()
                } else {
                    record.summary.clone()
                };
                let category = infer_category(&record.name, &description);
                let keywords =
                    Self::tokenize(&format!("{} {} {}", record.name, description, category));
                Some(ToolEntry {
                    name: record.name,
                    description,
                    category,
                    keywords,
                })
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
                    TOOL_NAME_BOOST
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

/// Infer a tool category from its name and description using keyword matching.
fn infer_category(name: &str, description: &str) -> String {
    let combined = format!("{} {}", name, description).to_lowercase();

    // Order matters: more specific categories first.
    let rules: &[(&str, &[&str])] = &[
        (
            "single-cell",
            &[
                "single-cell",
                "scrna",
                "10x",
                "cellranger",
                "seurat",
                "scanpy",
                "velocyto",
            ],
        ),
        (
            "long-reads",
            &[
                "nanopore",
                "pacbio",
                "ont",
                "long-read",
                "hifi",
                "dorado",
                "medaka",
                "guppy",
            ],
        ),
        (
            "structural-variants",
            &[
                "structural variant",
                "cnv",
                "translocation",
                "deletion",
                "manta",
                "delly",
                "lumpy",
            ],
        ),
        (
            "epigenomics",
            &[
                "methyl",
                "bisulfite",
                "chip-seq",
                "chipseq",
                "atac",
                "histone",
                "epigenom",
                "macs",
                "deeptools",
            ],
        ),
        (
            "metagenomics",
            &[
                "metagenomic",
                "taxonom",
                "kraken",
                "metaphlan",
                "humann",
                "bracken",
                "amplicon",
                "16s",
                "microbiom",
            ],
        ),
        (
            "variant-calling",
            &[
                "variant",
                "snp",
                "indel",
                "haplotype",
                "genotype",
                "vcf",
                "mutect",
                "caller",
                "gatk",
            ],
        ),
        (
            "rna-seq",
            &[
                "rna",
                "transcript",
                "expression",
                "quantif",
                "salmon",
                "kallisto",
                "rsem",
                "deseq",
                "cufflinks",
            ],
        ),
        (
            "quality-control",
            &[
                "quality",
                "qc",
                "trim",
                "filter",
                "adapter",
                "fastqc",
                "fastp",
                "cutadapt",
                "trimmomatic",
            ],
        ),
        (
            "assembly",
            &[
                "assembl", "scaffold", "contig", "spades", "megahit", "flye", "canu", "hifiasm",
            ],
        ),
        (
            "alignment",
            &[
                "bam", "sam", "align", "map", "minimap", "bowtie", "hisat", "bwa", "star",
            ],
        ),
        (
            "annotation",
            &[
                "annot",
                "gene prediction",
                "functional",
                "snpeff",
                "vep",
                "ensembl",
                "interpro",
            ],
        ),
        (
            "file-formats",
            &[
                "convert", "htslib", "tabix", "bgzip", "cram", "bam2", "fasta2", "vcf2",
            ],
        ),
        (
            "phylogenetics",
            &["phylo", "tree", "evolution", "iqtree", "raxml", "beast"],
        ),
        (
            "proteomics",
            &[
                "proteo",
                "peptide",
                "mass spec",
                "protein identification",
                "maxquant",
            ],
        ),
        (
            "workflow",
            &[
                "workflow",
                "pipeline",
                "snakemake",
                "nextflow",
                "cromwell",
                "wdl",
            ],
        ),
    ];

    for (category, keywords) in rules {
        if keywords.iter().any(|kw| combined.contains(kw)) {
            return (*category).to_string();
        }
    }

    "bioinformatics".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_not_empty() {
        let kb = ToolKnowledgeBase::new();
        assert!(kb.len() > 5000, "Expected 5000+ tools, got {}", kb.len());
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
        let results = kb.search("RNA-seq quantification transcript", 20);
        assert!(!results.is_empty());
        // With 6103 tools, many have RNA/transcript in their description.
        // Verify that at least one result is in the rna-seq category.
        let has_rna_seq = results.iter().any(|r| r.entry.category == "rna-seq");
        assert!(
            has_rna_seq,
            "Expected at least one rna-seq category tool in results"
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
