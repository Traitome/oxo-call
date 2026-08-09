//! Bioinformatics knowledge graph — embedded tool network.
#![allow(dead_code)] // library module; used by tests and bench
//!
//! Nodes: tools with categories, CLI types, input/output formats.
//! Edges: alternative, companion, pipeline predecessor, version break.
//!
//! Stored in SQLite (`knowledge_graph.db`) adjacent to the bioconda index.
//! The graph is pre-populated with curated domain knowledge and expanded
//! at runtime via category/type inference from bioconda + --help analysis.
//!
//! ## Edge types
//!
//! | Type              | Example                    | Meaning                       |
//! |-------------------|----------------------------|-------------------------------|
//! | ALTERNATIVE_TO    | bwa ↔ minimap2             | Both do alignment             |
//! | SUCCESSOR_OF      | bwa-mem2 → bwa             | Faster/better replacement     |
//! | COMPANION         | samtools → bgzip           | Used together                 |
//! | PIPELINE_PRE      | fastqc → trim_galore       | Upstream in typical workflow  |
//! | VERSION_BREAK     | iqtree2 → iqtree3          | Major version flag changes    |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node in the tool knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolNode {
    pub name: String,
    pub category: String,
    pub cli_type: String,          // "flags", "subcommands", "positional"
    pub input_types: Vec<String>,  // e.g. [".fastq", ".bam"]
    pub output_types: Vec<String>, // e.g. [".bam", ".vcf"]
    pub description: String,
}

/// Edge in the tool knowledge graph.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    AlternativeTo,
    SuccessorOf,
    Companion,
    PipelinePredecessor,
    VersionBreak,
}

impl EdgeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AlternativeTo => "alternative_to",
            Self::SuccessorOf => "successor_of",
            Self::Companion => "companion",
            Self::PipelinePredecessor => "pipeline_pre",
            Self::VersionBreak => "version_break",
        }
    }
}

/// Directed edge between two tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEdge {
    pub from_tool: String,
    pub to_tool: String,
    pub edge_type: EdgeType,
}

/// Knowledge graph with SQLite persistence.
pub struct KnowledgeGraph {
    nodes: HashMap<String, ToolNode>,
    /// from_tool → Vec<(to_tool, EdgeType)>
    edges: HashMap<String, Vec<(String, EdgeType)>>,
}

impl KnowledgeGraph {
    /// Build the knowledge graph, loading from SQLite if available,
    /// falling back to the embedded curated graph.
    pub fn load() -> Result<Self, String> {
        if let Ok(kg) = Self::load_from_sqlite()
            && !kg.nodes.is_empty()
        {
            return Ok(kg);
        }
        let kg = Self::build_curated_graph();
        let _ = kg.save_to_sqlite();
        Ok(kg)
    }

    /// Build the curated graph from embedded domain knowledge.
    pub fn build_curated_graph() -> Self {
        let mut kg = Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };

        // ── Alignment tools ────────────────────────────────────────────
        kg.add_node(
            "bwa",
            "alignment",
            "subcommands",
            &[".fastq", ".fq"],
            &[".sam"],
            "Burrows-Wheeler Aligner",
        );
        kg.add_node(
            "bwa-mem2",
            "alignment",
            "subcommands",
            &[".fastq", ".fq"],
            &[".sam"],
            "BWA-MEM2, optimized BWA-MEM",
        );
        kg.add_node(
            "bowtie2",
            "alignment",
            "subcommands",
            &[".fastq", ".fq"],
            &[".sam"],
            "Bowtie 2 short-read aligner",
        );
        kg.add_node(
            "minimap2",
            "alignment",
            "flags",
            &[".fastq", ".fa"],
            &[".sam", ".paf"],
            "Minimap2 long-read aligner",
        );
        kg.add_node(
            "hisat2",
            "alignment",
            "subcommands",
            &[".fastq", ".fq"],
            &[".sam"],
            "HISAT2 spliced aligner",
        );
        kg.add_node(
            "STAR",
            "alignment",
            "flags",
            &[".fastq", ".fq"],
            &[".sam", ".bam"],
            "STAR RNA-seq aligner",
        );
        kg.add_node(
            "samtools",
            "alignment",
            "subcommands",
            &[".sam", ".bam", ".cram"],
            &[".bam", ".sam"],
            "SAMtools",
        );
        kg.add_node(
            "picard",
            "alignment",
            "subcommands",
            &[".sam", ".bam"],
            &[".bam"],
            "Picard tools",
        );

        // Alternatives
        kg.add_edge("bwa", "minimap2", EdgeType::AlternativeTo);
        kg.add_edge("bwa-mem2", "bwa", EdgeType::SuccessorOf);
        kg.add_edge("bowtie2", "bwa", EdgeType::AlternativeTo);
        kg.add_edge("STAR", "hisat2", EdgeType::AlternativeTo);
        kg.add_edge("minimap2", "bwa", EdgeType::AlternativeTo);

        // ── Preprocessing ──────────────────────────────────────────────
        kg.add_node(
            "fastqc",
            "qc",
            "flags",
            &[".fastq", ".fq", ".bam"],
            &[".html", ".zip"],
            "FastQC quality control",
        );
        kg.add_node(
            "fastp",
            "preprocessing",
            "flags",
            &[".fastq", ".fq"],
            &[".fastq", ".fq"],
            "fastp all-in-one preprocessing",
        );
        kg.add_node(
            "trim_galore",
            "preprocessing",
            "flags",
            &[".fastq", ".fq"],
            &[".fastq", ".fq"],
            "Trim Galore adapter trimming",
        );
        kg.add_node(
            "cutadapt",
            "preprocessing",
            "flags",
            &[".fastq", ".fq"],
            &[".fastq", ".fq"],
            "Cutadapt adapter trimmer",
        );
        kg.add_node(
            "trimmomatic",
            "preprocessing",
            "flags",
            &[".fastq", ".fq"],
            &[".fastq", ".fq"],
            "Trimmomatic read trimmer",
        );
        kg.add_node(
            "bbtools",
            "sequence-utilities",
            "subcommands",
            &[".fastq", ".fq", ".bam", ".fa"],
            &[".fastq", ".bam", ".fa"],
            "BBTools suite",
        );

        // Alternatives
        kg.add_edge("fastp", "trim_galore", EdgeType::AlternativeTo);
        kg.add_edge("fastp", "cutadapt", EdgeType::AlternativeTo);
        kg.add_edge("fastp", "trimmomatic", EdgeType::AlternativeTo);

        // ── Pipeline relationships ─────────────────────────────────────
        kg.add_edge("fastqc", "trim_galore", EdgeType::PipelinePredecessor);
        kg.add_edge("fastqc", "fastp", EdgeType::PipelinePredecessor);
        kg.add_edge("trim_galore", "bwa", EdgeType::PipelinePredecessor);
        kg.add_edge("trim_galore", "bowtie2", EdgeType::PipelinePredecessor);
        kg.add_edge("trim_galore", "minimap2", EdgeType::PipelinePredecessor);
        kg.add_edge("bwa", "samtools", EdgeType::PipelinePredecessor);
        kg.add_edge("bowtie2", "samtools", EdgeType::PipelinePredecessor);
        kg.add_edge("minimap2", "samtools", EdgeType::PipelinePredecessor);
        kg.add_edge("samtools", "bcftools", EdgeType::PipelinePredecessor);
        kg.add_edge("samtools", "picard", EdgeType::PipelinePredecessor);

        // ── Variant calling ────────────────────────────────────────────
        kg.add_node(
            "bcftools",
            "variant-calling",
            "subcommands",
            &[".vcf", ".bcf", ".bam"],
            &[".vcf", ".bcf"],
            "BCFtools",
        );
        kg.add_node(
            "gatk",
            "variant-calling",
            "subcommands",
            &[".bam", ".vcf"],
            &[".vcf", ".bam"],
            "GATK",
        );
        kg.add_node(
            "freebayes",
            "variant-calling",
            "flags",
            &[".bam"],
            &[".vcf"],
            "FreeBayes variant caller",
        );
        kg.add_node(
            "deepvariant",
            "variant-calling",
            "flags",
            &[".bam"],
            &[".vcf"],
            "DeepVariant",
        );
        kg.add_node(
            "strelka",
            "variant-calling",
            "flags",
            &[".bam"],
            &[".vcf"],
            "Strelka2",
        );

        kg.add_edge("bcftools", "gatk", EdgeType::AlternativeTo);
        kg.add_edge("freebayes", "gatk", EdgeType::AlternativeTo);
        kg.add_edge("gatk", "samtools", EdgeType::PipelinePredecessor);

        // ── Phylogenetics ──────────────────────────────────────────────
        kg.add_node(
            "iqtree2",
            "phylogenetics",
            "flags",
            &[".phy", ".nex", ".fasta", ".aln"],
            &[".treefile", ".iqtree"],
            "IQ-TREE 2",
        );
        kg.add_node(
            "raxml-ng",
            "phylogenetics",
            "flags",
            &[".phy", ".fasta"],
            &[".raxml"],
            "RAxML-NG",
        );
        kg.add_node(
            "mrbayes",
            "phylogenetics",
            "flags",
            &[".nex"],
            &[".nex"],
            "MrBayes",
        );
        kg.add_node(
            "beast",
            "phylogenetics",
            "flags",
            &[".xml"],
            &[".log", ".trees"],
            "BEAST",
        );

        kg.add_edge("iqtree2", "raxml-ng", EdgeType::AlternativeTo);
        kg.add_edge("iqtree2", "iqtree3", EdgeType::VersionBreak);

        // ── Assembly ───────────────────────────────────────────────────
        kg.add_node(
            "spades",
            "assembly",
            "flags",
            &[".fastq", ".fq"],
            &[".fasta", ".gfa"],
            "SPAdes assembler",
        );
        kg.add_node(
            "canu",
            "assembly",
            "flags",
            &[".fastq", ".fq", ".fa"],
            &[".fasta"],
            "Canu long-read assembler",
        );
        kg.add_node(
            "flye",
            "assembly",
            "flags",
            &[".fastq", ".fa"],
            &[".fasta", ".gfa"],
            "Flye assembler",
        );
        kg.add_node(
            "hifiasm",
            "assembly",
            "flags",
            &[".fastq", ".fa"],
            &[".fasta", ".gfa"],
            "Hifiasm HiFi assembler",
        );

        kg.add_edge("canu", "flye", EdgeType::AlternativeTo);
        kg.add_edge("flye", "hifiasm", EdgeType::AlternativeTo);

        // ── Metagenomics ───────────────────────────────────────────────
        kg.add_node(
            "kraken2",
            "metagenomics",
            "subcommands",
            &[".fastq", ".fq"],
            &[".txt", ".report"],
            "Kraken 2 classifier",
        );
        kg.add_node(
            "bracken",
            "metagenomics",
            "flags",
            &[".report"],
            &[".txt"],
            "Bracken abundance estimator",
        );
        kg.add_node(
            "metaphlan",
            "metagenomics",
            "flags",
            &[".fastq", ".fq"],
            &[".txt"],
            "MetaPhlAn profiler",
        );
        kg.add_node(
            "humann3",
            "functional-annotation",
            "flags",
            &[".fastq", ".fq"],
            &[".tsv"],
            "HUMAnN 3",
        );

        kg.add_edge("kraken2", "bracken", EdgeType::Companion);
        kg.add_edge("kraken2", "metaphlan", EdgeType::AlternativeTo);

        // ── Annotation ─────────────────────────────────────────────────
        kg.add_node(
            "prokka",
            "annotation",
            "flags",
            &[".fasta", ".fa"],
            &[".gff", ".gbk"],
            "Prokka prokaryotic annotation",
        );
        kg.add_node(
            "bakta",
            "annotation",
            "flags",
            &[".fasta", ".fa"],
            &[".gff3", ".gbff"],
            "Bakta annotation",
        );
        kg.add_node(
            "augustus",
            "annotation",
            "flags",
            &[".fasta", ".fa"],
            &[".gff", ".gtf"],
            "AUGUSTUS gene predictor",
        );

        kg.add_edge("bakta", "prokka", EdgeType::SuccessorOf);

        // ── RNA-seq ────────────────────────────────────────────────────
        kg.add_node(
            "featureCounts",
            "rna-seq",
            "flags",
            &[".bam", ".sam"],
            &[".txt", ".tsv"],
            "featureCounts",
        );
        kg.add_node(
            "stringtie",
            "rna-seq",
            "flags",
            &[".bam"],
            &[".gtf"],
            "StringTie assembler",
        );
        kg.add_node(
            "salmon",
            "rna-seq",
            "flags",
            &[".fastq", ".fq"],
            &[".sf"],
            "Salmon quantifier",
        );
        kg.add_node(
            "kallisto",
            "rna-seq",
            "flags",
            &[".fastq", ".fq"],
            &[".h5", ".tsv"],
            "kallisto quantifier",
        );

        kg.add_edge("salmon", "kallisto", EdgeType::AlternativeTo);
        kg.add_edge("STAR", "featureCounts", EdgeType::PipelinePredecessor);
        kg.add_edge("STAR", "stringtie", EdgeType::PipelinePredecessor);

        // ── Format converters / companions ─────────────────────────────
        kg.add_node(
            "tabix",
            "genomic-intervals",
            "flags",
            &[".vcf", ".bed", ".gff", ".sam"],
            &[".tbi", ".csi"],
            "Tabix indexer",
        );
        kg.add_node(
            "bgzip",
            "genomic-intervals",
            "flags",
            &[".vcf", ".sam", ".bed"],
            &[".gz"],
            "Bgzip compressor",
        );
        kg.add_node(
            "bedtools",
            "genomic-intervals",
            "subcommands",
            &[".bed", ".gff", ".vcf", ".bam"],
            &[".bed", ".txt"],
            "BEDTools",
        );

        kg.add_edge("bgzip", "tabix", EdgeType::Companion);
        kg.add_edge("bedtools", "samtools", EdgeType::Companion);

        // ── Container / HPC ────────────────────────────────────────────
        kg.add_node(
            "docker",
            "containerization",
            "subcommands",
            &[],
            &[],
            "Docker",
        );
        kg.add_node(
            "singularity",
            "containerization",
            "subcommands",
            &[],
            &[],
            "Singularity/Apptainer",
        );

        kg.add_edge("singularity", "docker", EdgeType::AlternativeTo);

        // ── BLAST / search ─────────────────────────────────────────────
        kg.add_node(
            "blast",
            "sequence-utilities",
            "subcommands",
            &[".fasta", ".fa"],
            &[".txt", ".tsv"],
            "BLAST+",
        );
        kg.add_node(
            "diamond",
            "sequence-utilities",
            "flags",
            &[".fasta", ".fa"],
            &[".txt", ".daa"],
            "DIAMOND aligner",
        );

        kg.add_edge("diamond", "blast", EdgeType::AlternativeTo);

        // ── Version breaks ─────────────────────────────────────────────
        kg.add_edge("kraken2", "kraken", EdgeType::VersionBreak);
        kg.add_edge("humann3", "humann2", EdgeType::VersionBreak);
        kg.add_edge("gffcompare", "cuffcompare", EdgeType::SuccessorOf);

        kg
    }

    fn add_node(
        &mut self,
        name: &str,
        category: &str,
        cli_type: &str,
        inputs: &[&str],
        outputs: &[&str],
        desc: &str,
    ) {
        self.nodes.insert(
            name.to_string(),
            ToolNode {
                name: name.to_string(),
                category: category.to_string(),
                cli_type: cli_type.to_string(),
                input_types: inputs.iter().map(|s| s.to_string()).collect(),
                output_types: outputs.iter().map(|s| s.to_string()).collect(),
                description: desc.to_string(),
            },
        );
    }

    fn add_edge(&mut self, from: &str, to: &str, etype: EdgeType) {
        self.edges
            .entry(from.to_string())
            .or_default()
            .push((to.to_string(), etype));
        // Alternative is symmetric
        if etype == EdgeType::AlternativeTo || etype == EdgeType::Companion {
            self.edges
                .entry(to.to_string())
                .or_default()
                .push((from.to_string(), etype));
        }
    }

    /// Infer category and I/O types for an unknown tool.
    ///
    /// Uses: bioconda summary/description keywords, tool name heuristics,
    /// flag name patterns from --help.
    pub fn infer_tool_info(
        &self,
        name: &str,
        help_text: &str,
        bioconda_summary: Option<&str>,
    ) -> ToolNode {
        let category = self.infer_category(name, help_text, bioconda_summary);
        let (inputs, outputs) = self.infer_io_types(help_text);
        let cli_type = if help_text.contains("Commands:")
            || help_text.contains("COMMANDS:")
            || help_text.contains("Subcommands:")
        {
            "subcommands"
        } else {
            "flags"
        };
        ToolNode {
            name: name.to_string(),
            category,
            cli_type: cli_type.to_string(),
            input_types: inputs,
            output_types: outputs,
            description: bioconda_summary.unwrap_or("").to_string(),
        }
    }

    /// Infer category from bioconda metadata keywords + tool name heuristics.
    fn infer_category(
        &self,
        name: &str,
        help_text: &str,
        bioconda_summary: Option<&str>,
    ) -> String {
        let text = format!(
            "{} {} {}",
            name.to_lowercase(),
            bioconda_summary.unwrap_or("").to_lowercase(),
            help_text.to_lowercase()
        );

        let patterns: &[(&str, &[&str])] = &[
            (
                "alignment",
                &["align", "bam", "sam", "cram", "bwa", "bowtie"],
            ),
            (
                "variant-calling",
                &["variant", "vcf", "bcf", "call", "snp", "indel", "gatk"],
            ),
            (
                "assembly",
                &["assemble", "contig", "scaffold", "spades", "canu", "flye"],
            ),
            (
                "preprocessing",
                &["trim", "adapter", "quality", "filter", "preprocess"],
            ),
            (
                "qc",
                &["quality control", "fastqc", "qc", "report", "statistics"],
            ),
            (
                "rna-seq",
                &[
                    "rna",
                    "transcript",
                    "splice",
                    "expression",
                    "count",
                    "stringtie",
                ],
            ),
            (
                "phylogenetics",
                &[
                    "phylogen", "tree", "ml", "iqtree", "raxml", "beast", "mrbayes",
                ],
            ),
            (
                "metagenomics",
                &["metagenom", "kraken", "microbiome", "taxonom", "16s"],
            ),
            (
                "annotation",
                &[
                    "annot", "gene", "protein", "gff", "gtf", "prokka", "augustus",
                ],
            ),
            (
                "epigenomics",
                &["methyl", "chip", "atac", "epigen", "bismark", "histone"],
            ),
            (
                "functional-annotation",
                &["pathway", "kegg", "go term", "humann", "eggnog"],
            ),
            (
                "genomic-intervals",
                &["bed", "interval", "overlap", "intersect", "merge bed"],
            ),
            (
                "long-reads",
                &["long read", "pacbio", "nanopore", "ont", "hifi"],
            ),
            (
                "single-cell",
                &["single cell", "scrna", "10x", "cell ranger"],
            ),
            (
                "structural-variants",
                &["sv ", "structural variant", "cnv", "manta", "delly"],
            ),
            (
                "population-genomics",
                &["populat", "plink", "admixture", "pca", "gwas"],
            ),
            (
                "sequence-utilities",
                &["fasta", "fastq", "sequence", "extract", "format convert"],
            ),
            (
                "system-tools",
                &["shell", "bash", "linux", "system", "docker", "singularity"],
            ),
            (
                "containerization",
                &["docker", "singularity", "container", "apptainer"],
            ),
            (
                "hpc",
                &["slurm", "sge", "pbs", "hpc", "grid", "cluster", "job"],
            ),
            (
                "genome-evaluation",
                &["busco", "quast", "completeness", "genome evalu"],
            ),
            (
                "data-download",
                &["download", "fetch", "sra", "ena", "geo", "ncbi"],
            ),
            (
                "workflow",
                &["workflow", "pipeline", "nextflow", "snakemake", "cromwell"],
            ),
            (
                "comparative-genomics",
                &["ortholog", "synteny", "comparative", "homolog", "phylogen"],
            ),
            (
                "package-management",
                &["conda", "mamba", "pip", "cran", "bioconda", "package"],
            ),
        ];

        let mut best = "unknown".to_string();
        let mut best_score = 0;
        for (cat, keywords) in patterns {
            let score = keywords.iter().filter(|kw| text.contains(*kw)).count();
            if score > best_score {
                best_score = score;
                best = cat.to_string();
            }
        }
        if best_score == 0 {
            "unknown".to_string()
        } else {
            best
        }
    }

    /// Infer input/output file types from flag patterns in --help.
    fn infer_io_types(&self, help_text: &str) -> (Vec<String>, Vec<String>) {
        let lower = help_text.to_lowercase();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        // Input patterns
        let input_exts = &[
            (
                ".fastq",
                &["fastq", "fq", ".fq", "fastq.gz", "fq.gz"] as &[&str],
            ),
            (".bam", &["bam", ".bam"]),
            (".sam", &["sam", ".sam"]),
            (".vcf", &["vcf", ".vcf", "vcf.gz"]),
            (".fa", &["fasta", "fa", ".fa", "fna", "fasta.gz", "fa.gz"]),
            (".bed", &["bed", ".bed"]),
            (".gff", &["gff", ".gff", "gtf", ".gtf", "gff3"]),
            (".nex", &["nexus", "nex", ".nex"]),
            (".phy", &["phylip", "phy", ".phy"]),
        ];
        for (ext, keywords) in input_exts {
            if keywords.iter().any(|kw| lower.contains(kw)) && !inputs.contains(&ext.to_string()) {
                inputs.push(ext.to_string());
            }
        }

        // Output patterns — look for input files that are also output (BAM→BAM, etc.)
        let output_keywords = &[
            "-o ",
            "--output",
            "output file",
            "outfile",
            "write to",
            "> ",
        ];
        if output_keywords.iter().any(|kw| lower.contains(kw)) {
            // Re-use input types as possible outputs, add format-specific ones
            outputs.extend(inputs.clone());
            if lower.contains("bam") && !outputs.contains(&".bam".to_string()) {
                outputs.push(".bam".to_string());
            }
            if lower.contains("vcf") && !outputs.contains(&".vcf".to_string()) {
                outputs.push(".vcf".to_string());
            }
        }
        // Sort and deduplicate
        inputs.sort();
        inputs.dedup();
        outputs.sort();
        outputs.dedup();
        (inputs, outputs)
    }

    /// Look up a tool node by name.
    pub fn get_node(&self, name: &str) -> Option<&ToolNode> {
        self.nodes.get(name)
    }

    /// Get related tools (neighbors in the graph).
    pub fn related(&self, name: &str) -> Vec<(ToolNode, EdgeType)> {
        let edges = match self.edges.get(name) {
            Some(e) => e.clone(),
            None => return Vec::new(),
        };
        edges
            .iter()
            .filter_map(|(to, etype)| self.nodes.get(to).map(|n| (n.clone(), *etype)))
            .collect()
    }

    /// Get pipeline context: predecessors + successors for a tool.
    pub fn pipeline_context(&self, name: &str) -> (Vec<&ToolNode>, Vec<&ToolNode>) {
        let mut upstream = Vec::new();
        let mut downstream = Vec::new();
        if let Some(edges) = self.edges.get(name) {
            for (to, etype) in edges {
                if let Some(node) = self.nodes.get(to) {
                    match etype {
                        EdgeType::PipelinePredecessor => upstream.push(node),
                        EdgeType::Companion => downstream.push(node),
                        _ => {}
                    }
                }
            }
        }
        // Also find tools where this tool appears as the TARGET of a
        // PipelinePredecessor edge — those are upstream predecessors.
        for (from, edges) in &self.edges {
            for (to, etype) in edges {
                if to == name
                    && *etype == EdgeType::PipelinePredecessor
                    && let Some(node) = self.nodes.get(from)
                {
                    upstream.push(node);
                }
            }
        }
        (upstream, downstream)
    }

    /// Get alternatives for a tool (and successor/predecessor relationships).
    pub fn alternatives(&self, name: &str) -> Vec<&ToolNode> {
        self.edges
            .get(name)
            .map(|edges| {
                edges
                    .iter()
                    .filter(|(_, e)| *e == EdgeType::AlternativeTo || *e == EdgeType::SuccessorOf)
                    .filter_map(|(to, _)| self.nodes.get(to))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Build a prompt hint with related tools and pipeline context.
    pub fn to_prompt_hint(&self, name: &str) -> String {
        let alts = self.alternatives(name);
        let related = self.related(name);
        let (up, down) = self.pipeline_context(name);

        if alts.is_empty() && related.is_empty() && up.is_empty() && down.is_empty() {
            return String::new();
        }

        let mut hint = String::from("<!-- L4: Knowledge Graph Context -->\n");
        if !alts.is_empty() {
            let names: Vec<&str> = alts.iter().map(|n| n.name.as_str()).collect();
            hint.push_str(&format!("Alternatives: {}\n", names.join(", ")));
        }
        if !up.is_empty() {
            let names: Vec<&str> = up.iter().map(|n| n.name.as_str()).collect();
            hint.push_str(&format!("Typical upstream: {}\n", names.join(" → ")));
        }
        if !down.is_empty() {
            let names: Vec<&str> = down.iter().map(|n| n.name.as_str()).collect();
            hint.push_str(&format!("Typical downstream: {}\n", names.join(" → ")));
        }
        hint.push_str("<!-- END L4 -->\n");
        hint
    }

    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.edges.values().map(|v| v.len()).sum()
    }

    /// Index a new tool into the graph via inference.
    pub fn index_tool(&mut self, name: &str, help_text: &str, bioconda_summary: Option<&str>) {
        if self.nodes.contains_key(name) {
            return;
        }
        let node = self.infer_tool_info(name, help_text, bioconda_summary);
        self.nodes.insert(name.to_string(), node);
    }

    // ── SQLite persistence ───────────────────────────────────────────

    fn db_path() -> Result<std::path::PathBuf, String> {
        let dir = crate::config::Config::data_dir().map_err(|e| format!("data dir: {e}"))?;
        std::fs::create_dir_all(&dir).ok();
        Ok(dir.join("knowledge_graph.db"))
    }

    fn load_from_sqlite() -> Result<Self, String> {
        let path = Self::db_path()?;
        if !path.exists() {
            return Err("no kg db".into());
        }
        let conn = rusqlite::Connection::open(&path).map_err(|e| format!("sqlite open: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (
                name TEXT PRIMARY KEY, category TEXT, cli_type TEXT,
                input_types TEXT, output_types TEXT, description TEXT);
             CREATE TABLE IF NOT EXISTS edges (
                from_tool TEXT, to_tool TEXT, edge_type TEXT,
                PRIMARY KEY (from_tool, to_tool, edge_type))",
        )
        .map_err(|e| format!("schema: {e}"))?;

        let mut nodes = HashMap::new();
        let mut stmt = conn.prepare("SELECT name, category, cli_type, input_types, output_types, description FROM nodes")
            .map_err(|e| format!("prepare: {e}"))?;
        for row in stmt
            .query_map([], |r| {
                Ok(ToolNode {
                    name: r.get(0)?,
                    category: r.get(1)?,
                    cli_type: r.get(2)?,
                    input_types: r
                        .get::<_, String>(3)
                        .unwrap_or_default()
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                    output_types: r
                        .get::<_, String>(4)
                        .unwrap_or_default()
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                    description: r.get(5).unwrap_or_default(),
                })
            })
            .map_err(|e| format!("nodes: {e}"))?
            .flatten()
        {
            nodes.insert(row.name.clone(), row);
        }

        let mut edges: HashMap<String, Vec<(String, EdgeType)>> = HashMap::new();
        let mut stmt = conn
            .prepare("SELECT from_tool, to_tool, edge_type FROM edges")
            .map_err(|e| format!("edges: {e}"))?;
        for row in stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| format!("edges query: {e}"))?
            .flatten()
        {
            let (from, to, etype_str) = row;
            let etype = match etype_str.as_str() {
                "alternative_to" => EdgeType::AlternativeTo,
                "successor_of" => EdgeType::SuccessorOf,
                "companion" => EdgeType::Companion,
                "pipeline_pre" => EdgeType::PipelinePredecessor,
                "version_break" => EdgeType::VersionBreak,
                _ => continue,
            };
            edges.entry(from).or_default().push((to, etype));
        }

        Ok(Self { nodes, edges })
    }

    pub fn save_to_sqlite(&self) -> Result<(), String> {
        let path = Self::db_path()?;
        let conn = rusqlite::Connection::open(&path).map_err(|e| format!("sqlite open: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nodes (
                name TEXT PRIMARY KEY, category TEXT, cli_type TEXT,
                input_types TEXT, output_types TEXT, description TEXT);
             CREATE TABLE IF NOT EXISTS edges (
                from_tool TEXT, to_tool TEXT, edge_type TEXT,
                PRIMARY KEY (from_tool, to_tool, edge_type));
             DELETE FROM edges; DELETE FROM nodes",
        )
        .map_err(|e| format!("schema: {e}"))?;

        let mut ns = conn
            .prepare("INSERT OR REPLACE INTO nodes VALUES (?1,?2,?3,?4,?5,?6)")
            .map_err(|e| format!("nodes prep: {e}"))?;
        for node in self.nodes.values() {
            ns.execute(rusqlite::params![
                node.name,
                node.category,
                node.cli_type,
                node.input_types.join(","),
                node.output_types.join(","),
                node.description
            ])
            .map_err(|e| format!("nodes insert: {e}"))?;
        }

        let mut es = conn
            .prepare("INSERT OR REPLACE INTO edges VALUES (?1,?2,?3)")
            .map_err(|e| format!("edges prep: {e}"))?;
        for (from, edges) in &self.edges {
            for (to, etype) in edges {
                es.execute(rusqlite::params![from, to, etype.as_str()])
                    .map_err(|e| format!("edges insert: {e}"))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_curated_graph() {
        let kg = KnowledgeGraph::build_curated_graph();
        assert!(kg.node_count() > 30);
        assert!(kg.edge_count() > 20);
    }

    #[test]
    fn test_alternatives() {
        let kg = KnowledgeGraph::build_curated_graph();
        let alts = kg.alternatives("bwa");
        assert!(alts.iter().any(|n| n.name == "minimap2"));
        assert!(alts.iter().any(|n| n.name == "bowtie2"));
    }

    #[test]
    fn test_pipeline_context() {
        let kg = KnowledgeGraph::build_curated_graph();
        let (up, down) = kg.pipeline_context("samtools");
        assert!(up.iter().any(|n| n.name == "bwa"));
    }

    #[test]
    fn test_infer_category() {
        let kg = KnowledgeGraph::build_curated_graph();
        let cat = kg.infer_category(
            "unknown_aligner",
            "",
            Some("Fast and accurate short-read alignment tool"),
        );
        assert_eq!(cat, "alignment");
    }

    #[test]
    fn test_infer_io_types() {
        let kg = KnowledgeGraph::build_curated_graph();
        let (inputs, outputs) =
            kg.infer_io_types("Usage: tool --input in.bam --output out.vcf\n  -o FILE  output VCF");
        assert!(inputs.contains(&".bam".to_string()));
        assert!(outputs.contains(&".vcf".to_string()));
    }

    #[test]
    fn test_prompt_hint() {
        let kg = KnowledgeGraph::build_curated_graph();
        let hint = kg.to_prompt_hint("bwa");
        assert!(hint.contains("Alternatives"));
        assert!(hint.contains("L4: Knowledge Graph Context"));
    }

    #[test]
    fn test_version_break() {
        let kg = KnowledgeGraph::build_curated_graph();
        let related = kg.related("iqtree2");
        assert!(related.iter().any(|(n, _)| n.name == "raxml-ng"));
    }
}
