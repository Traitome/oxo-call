//! Experiment context inference for bioinformatics task understanding.
//!
//! When a user provides a high-level task description (e.g., "analyze RNA-seq data"),
//! the `ExperimentContext` module infers the assay type, organism, library layout,
//! and analysis stage from the task text and (optionally) input file extensions.
//! This context is then used to:
//!
//! - Recommend appropriate workflow templates
//! - Provide sensible default parameters (threads, reference paths)
//! - Enrich the LLM prompt with domain-specific context
//!
//! The inference is purely heuristic and keyword-based — no ML model is needed.

use std::collections::HashMap;

// ─── Assay types ──────────────────────────────────────────────────────────────

/// Recognised sequencing assay types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssayType {
    RnaSeq,
    Wgs,
    Wes,
    ChipSeq,
    AtacSeq,
    HiC,
    Bisulfite,
    ScRnaSeq,
    LongReads,
    Metagenomics,
    #[allow(dead_code)]
    Amplicon,
}

impl AssayType {
    /// Human-readable label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::RnaSeq => "RNA-seq",
            Self::Wgs => "WGS",
            Self::Wes => "WES/Exome",
            Self::ChipSeq => "ChIP-seq",
            Self::AtacSeq => "ATAC-seq",
            Self::HiC => "Hi-C",
            Self::Bisulfite => "Bisulfite-seq",
            Self::ScRnaSeq => "scRNA-seq",
            Self::LongReads => "Long-read sequencing",
            Self::Metagenomics => "Metagenomics",
            Self::Amplicon => "Amplicon sequencing",
        }
    }
}

// ─── Library layout ───────────────────────────────────────────────────────────

/// Paired-end vs. single-end library layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryType {
    PairedEnd,
    SingleEnd,
    Unknown,
}

// ─── Analysis stage ───────────────────────────────────────────────────────────

/// High-level analysis stage within a pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Qc,
    Trimming,
    Alignment,
    PostAlignment,
    Quantification,
    VariantCalling,
    DifferentialAnalysis,
    Annotation,
    Assembly,
    Unknown,
}

// ─── Experiment context ───────────────────────────────────────────────────────

/// Inferred experiment context from task description and file extensions.
#[derive(Debug, Clone)]
pub struct ExperimentContext {
    pub assay_type: Option<AssayType>,
    pub organism: Option<String>,
    pub library_type: LibraryType,
    pub analysis_stage: Stage,
}

impl ExperimentContext {
    /// Infer experiment context from a task description and optional input file names.
    pub fn infer(task: &str, input_files: &[&str]) -> Self {
        let lower = task.to_ascii_lowercase();

        ExperimentContext {
            assay_type: infer_assay_type(&lower, input_files),
            organism: infer_organism(&lower),
            library_type: infer_library_type(&lower, input_files),
            analysis_stage: infer_stage(&lower),
        }
    }

    /// Suggest a built-in workflow template name based on the inferred context.
    #[allow(dead_code)]
    pub fn recommended_workflow(&self) -> Option<&'static str> {
        self.assay_type.map(|a| match a {
            AssayType::RnaSeq => "rnaseq",
            AssayType::Wgs => "wgs",
            AssayType::Wes => "wes",
            AssayType::ChipSeq => "chipseq",
            AssayType::AtacSeq => "atacseq",
            AssayType::HiC => "hic",
            AssayType::Bisulfite => "methylseq",
            AssayType::ScRnaSeq => "scrnaseq",
            AssayType::LongReads => "longreads",
            AssayType::Metagenomics => "metagenomics",
            AssayType::Amplicon => "amplicon",
        })
    }

    /// Generate recommended default parameters based on the inferred context.
    #[allow(dead_code)]
    pub fn default_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        // Default thread count
        params.insert("threads".to_string(), "8".to_string());

        // Organism-based reference defaults
        if let Some(ref org) = self.organism {
            match org.as_str() {
                "hg38" | "human" | "grch38" => {
                    params.insert("reference".to_string(), "hg38".to_string());
                }
                "mm10" | "mouse" | "grcm38" => {
                    params.insert("reference".to_string(), "mm10".to_string());
                }
                "mm39" | "grcm39" => {
                    params.insert("reference".to_string(), "mm39".to_string());
                }
                _ => {
                    params.insert("reference".to_string(), org.clone());
                }
            }
        }

        // Library-type-specific defaults
        if self.library_type == LibraryType::PairedEnd {
            params.insert("layout".to_string(), "paired-end".to_string());
        }

        params
    }

    /// Generate a concise context summary for LLM prompt enrichment.
    pub fn to_prompt_hint(&self) -> String {
        let mut parts = Vec::new();

        if let Some(assay) = self.assay_type {
            parts.push(format!("Assay: {}", assay.label()));
        }
        if let Some(ref org) = self.organism {
            parts.push(format!("Organism: {org}"));
        }
        if self.library_type != LibraryType::Unknown {
            let layout = match self.library_type {
                LibraryType::PairedEnd => "paired-end",
                LibraryType::SingleEnd => "single-end",
                LibraryType::Unknown => "unknown",
            };
            parts.push(format!("Library: {layout}"));
        }
        if self.analysis_stage != Stage::Unknown {
            let stage = match self.analysis_stage {
                Stage::Qc => "QC",
                Stage::Trimming => "trimming",
                Stage::Alignment => "alignment",
                Stage::PostAlignment => "post-alignment",
                Stage::Quantification => "quantification",
                Stage::VariantCalling => "variant calling",
                Stage::DifferentialAnalysis => "differential analysis",
                Stage::Annotation => "annotation",
                Stage::Assembly => "assembly",
                Stage::Unknown => "unknown",
            };
            parts.push(format!("Stage: {stage}"));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("[Context: {}]", parts.join(", "))
        }
    }
}

// ─── Inference helpers ────────────────────────────────────────────────────────

/// Keywords mapped to assay types, checked in order.  First match wins.
const ASSAY_KEYWORDS: &[(&[&str], AssayType)] = &[
    (
        &[
            "scrna",
            "scrnaseq",
            "sc-rna",
            "single-cell rna",
            "10x",
            "cellranger",
        ],
        AssayType::ScRnaSeq,
    ),
    (
        &[
            "rnaseq",
            "rna-seq",
            "rna seq",
            "transcriptom",
            "mrna",
            "gene expression",
        ],
        AssayType::RnaSeq,
    ),
    (
        &["wgs", "whole genome sequencing", "whole-genome"],
        AssayType::Wgs,
    ),
    (&["wes", "exome", "whole exome"], AssayType::Wes),
    (
        &[
            "chipseq",
            "chip-seq",
            "chip seq",
            "chromatin immunoprecipitation",
        ],
        AssayType::ChipSeq,
    ),
    (
        &["atacseq", "atac-seq", "atac seq", "chromatin accessibility"],
        AssayType::AtacSeq,
    ),
    (&["hi-c", "hic", "chromosome conformation"], AssayType::HiC),
    (
        &[
            "bisulfite",
            "methylation",
            "methylseq",
            "bismark",
            "wgbs",
            "rrbs",
        ],
        AssayType::Bisulfite,
    ),
    (
        &[
            "nanopore",
            "pacbio",
            "long-read",
            "long read",
            "ont",
            "minion",
            "hifi",
        ],
        AssayType::LongReads,
    ),
    (
        &["metagenom", "16s", "amplicon", "kraken", "metaphlan"],
        AssayType::Metagenomics,
    ),
];

fn infer_assay_type(lower_task: &str, input_files: &[&str]) -> Option<AssayType> {
    // Check task keywords
    for (keywords, assay) in ASSAY_KEYWORDS {
        for kw in *keywords {
            if lower_task.contains(kw) {
                return Some(*assay);
            }
        }
    }

    // Fallback: infer from file extensions
    let all_files: String = input_files.join(" ").to_ascii_lowercase();
    if all_files.contains(".bam") || all_files.contains(".sam") || all_files.contains(".cram") {
        // Alignment files — could be any assay, don't guess
        return None;
    }
    if all_files.contains(".fastq") || all_files.contains(".fq") {
        // Raw reads — can't distinguish assay from extension alone
        return None;
    }

    None
}

/// Organism / reference genome keywords.
const ORGANISM_KEYWORDS: &[(&[&str], &str)] = &[
    (&["hg38", "grch38", "human"], "hg38"),
    (&["hg19", "grch37"], "hg19"),
    (&["mm10", "grcm38", "mouse"], "mm10"),
    (&["mm39", "grcm39"], "mm39"),
    (&["rn6", "rn7", "rat", "rattus"], "rn7"),
    (&["dm6", "drosophila", "fruit fly"], "dm6"),
    (&["danrer11", "zebrafish", "danio"], "danrer11"),
    (&["saccer3", "yeast", "saccharomyces"], "saccer3"),
    (&["tair10", "arabidopsis"], "tair10"),
    (&["ce11", "c.elegans", "c. elegans", "worm"], "ce11"),
    (&["susscr11", "pig", "swine", "sus scrofa"], "susScr11"),
    (&["galgal6", "chicken", "gallus"], "galGal6"),
    (&["bostau9", "cow", "bovine", "bos taurus"], "bosTau9"),
    (&["canfam3", "dog", "canine"], "canFam3"),
];

fn infer_organism(lower_task: &str) -> Option<String> {
    for (keywords, org) in ORGANISM_KEYWORDS {
        for kw in *keywords {
            if lower_task.contains(kw) {
                return Some(org.to_string());
            }
        }
    }
    None
}

fn infer_library_type(lower_task: &str, input_files: &[&str]) -> LibraryType {
    // Explicit mentions
    if lower_task.contains("paired")
        || lower_task.contains("pair-end")
        || lower_task.contains("pe ")
    {
        return LibraryType::PairedEnd;
    }
    if lower_task.contains("single-end")
        || lower_task.contains("single end")
        || lower_task.contains(" se ")
    {
        return LibraryType::SingleEnd;
    }

    // Infer from file naming convention (_R1/_R2, _1/_2)
    let files: String = input_files.join(" ");
    if files.contains("_R1") && files.contains("_R2") {
        return LibraryType::PairedEnd;
    }
    if files.contains("_1.") && files.contains("_2.") {
        return LibraryType::PairedEnd;
    }

    LibraryType::Unknown
}

/// Stage keywords, checked in order.
const STAGE_KEYWORDS: &[(&[&str], Stage)] = &[
    (&["quality control", "qc", "fastqc", "multiqc"], Stage::Qc),
    (
        &[
            "trim",
            "adapter",
            "fastp",
            "cutadapt",
            "trimmomatic",
            "bbduk",
        ],
        Stage::Trimming,
    ),
    (
        &[
            "align",
            "mapping",
            "map reads",
            "bwa mem",
            "bowtie2",
            "hisat2",
            "star ",
        ],
        Stage::Alignment,
    ),
    (
        &["sort", "index", "markdup", "dedup", "merge bam", "fixmate"],
        Stage::PostAlignment,
    ),
    (
        &[
            "quantif",
            "count",
            "featurecounts",
            "htseq",
            "salmon",
            "kallisto",
            "stringtie",
            "rsem",
        ],
        Stage::Quantification,
    ),
    (
        &[
            "variant",
            "call variant",
            "gatk",
            "haplotypecaller",
            "freebayes",
            "bcftools call",
            "mutect",
            "strelka",
        ],
        Stage::VariantCalling,
    ),
    (
        &["differential", "deseq2", "edger", "limma", "deg"],
        Stage::DifferentialAnalysis,
    ),
    (
        &["annotat", "vep", "annovar", "snpeff", "funcotator"],
        Stage::Annotation,
    ),
    (
        &["assembl", "spades", "megahit", "trinity", "flye", "canu"],
        Stage::Assembly,
    ),
];

fn infer_stage(lower_task: &str) -> Stage {
    for (keywords, stage) in STAGE_KEYWORDS {
        for kw in *keywords {
            if lower_task.contains(kw) {
                return *stage;
            }
        }
    }
    Stage::Unknown
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_rnaseq_from_task() {
        let ctx = ExperimentContext::infer("analyze RNA-seq data", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::RnaSeq));
    }

    #[test]
    fn test_infer_wgs_from_task() {
        let ctx = ExperimentContext::infer("whole genome sequencing analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Wgs));
    }

    #[test]
    fn test_infer_chipseq_from_task() {
        let ctx = ExperimentContext::infer("ChIP-seq peak calling", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::ChipSeq));
    }

    #[test]
    fn test_infer_scrnaseq_from_task() {
        let ctx = ExperimentContext::infer("10x single-cell RNA analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::ScRnaSeq));
    }

    #[test]
    fn test_infer_organism_human() {
        let ctx = ExperimentContext::infer("align to hg38 reference", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("hg38"));
    }

    #[test]
    fn test_infer_organism_mouse() {
        let ctx = ExperimentContext::infer("mouse mm10 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("mm10"));
    }

    #[test]
    fn test_infer_paired_end_from_task() {
        let ctx = ExperimentContext::infer("align paired-end reads", &[]);
        assert_eq!(ctx.library_type, LibraryType::PairedEnd);
    }

    #[test]
    fn test_infer_paired_end_from_files() {
        let ctx =
            ExperimentContext::infer("align reads", &["sample_R1.fastq.gz", "sample_R2.fastq.gz"]);
        assert_eq!(ctx.library_type, LibraryType::PairedEnd);
    }

    #[test]
    fn test_infer_alignment_stage() {
        let ctx = ExperimentContext::infer("align reads to reference", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Alignment);
    }

    #[test]
    fn test_infer_qc_stage() {
        let ctx = ExperimentContext::infer("run fastqc quality control", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Qc);
    }

    #[test]
    fn test_infer_variant_calling_stage() {
        let ctx = ExperimentContext::infer("call variants with GATK HaplotypeCaller", &[]);
        assert_eq!(ctx.analysis_stage, Stage::VariantCalling);
    }

    #[test]
    fn test_recommended_workflow_rnaseq() {
        let ctx = ExperimentContext::infer("RNA-seq analysis", &[]);
        assert_eq!(ctx.recommended_workflow(), Some("rnaseq"));
    }

    #[test]
    fn test_recommended_workflow_none_for_generic() {
        let ctx = ExperimentContext::infer("process some files", &[]);
        assert_eq!(ctx.recommended_workflow(), None);
    }

    #[test]
    fn test_default_params_include_threads() {
        let ctx = ExperimentContext::infer("anything", &[]);
        assert_eq!(ctx.default_params().get("threads").unwrap(), "8");
    }

    #[test]
    fn test_default_params_include_reference_for_organism() {
        let ctx = ExperimentContext::infer("align to hg38", &[]);
        assert_eq!(ctx.default_params().get("reference").unwrap(), "hg38");
    }

    #[test]
    fn test_prompt_hint_nonempty_for_rnaseq() {
        let ctx = ExperimentContext::infer("RNA-seq hg38 alignment", &[]);
        let hint = ctx.to_prompt_hint();
        assert!(hint.contains("RNA-seq"));
        assert!(hint.contains("hg38"));
    }

    #[test]
    fn test_prompt_hint_empty_for_generic() {
        let ctx = ExperimentContext::infer("do something", &[]);
        let hint = ctx.to_prompt_hint();
        assert!(hint.is_empty());
    }

    #[test]
    fn test_infer_bisulfite_from_bismark() {
        let ctx = ExperimentContext::infer("run bismark alignment", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Bisulfite));
    }

    #[test]
    fn test_infer_metagenomics_from_kraken() {
        let ctx = ExperimentContext::infer("classify reads with kraken", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Metagenomics));
    }

    #[test]
    fn test_infer_assembly_stage() {
        let ctx = ExperimentContext::infer("assemble genome with spades", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Assembly);
    }

    #[test]
    fn test_infer_quantification_stage() {
        let ctx = ExperimentContext::infer("quantify expression with salmon", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Quantification);
    }

    #[test]
    fn test_infer_wes_from_task() {
        let ctx = ExperimentContext::infer("whole exome sequencing", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Wes));
    }

    #[test]
    fn test_infer_atacseq_from_task() {
        let ctx = ExperimentContext::infer("ATAC-seq analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::AtacSeq));
    }

    #[test]
    fn test_infer_hic_from_task() {
        let ctx = ExperimentContext::infer("Hi-C chromosome conformation", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::HiC));
    }

    #[test]
    fn test_infer_long_reads_nanopore() {
        let ctx = ExperimentContext::infer("nanopore sequencing", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::LongReads));
    }

    #[test]
    fn test_infer_long_reads_pacbio() {
        let ctx = ExperimentContext::infer("pacbio hifi reads", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::LongReads));
    }

    #[test]
    fn test_infer_assay_no_match() {
        let ctx = ExperimentContext::infer("process some data", &[]);
        assert_eq!(ctx.assay_type, None);
    }

    #[test]
    fn test_infer_assay_from_bam_files() {
        let ctx = ExperimentContext::infer("process data", &["input.bam"]);
        assert_eq!(ctx.assay_type, None);
    }

    #[test]
    fn test_infer_assay_from_fastq_files() {
        let ctx = ExperimentContext::infer("process data", &["input.fastq"]);
        assert_eq!(ctx.assay_type, None);
    }

    #[test]
    fn test_infer_organism_hg19() {
        let ctx = ExperimentContext::infer("align to hg19 reference", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("hg19"));
    }

    #[test]
    fn test_infer_organism_mm39() {
        let ctx = ExperimentContext::infer("mm39 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("mm39"));
    }

    #[test]
    fn test_infer_organism_rat() {
        let ctx = ExperimentContext::infer("rat rn6 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("rn7"));
    }

    #[test]
    fn test_infer_organism_drosophila() {
        let ctx = ExperimentContext::infer("drosophila dm6 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("dm6"));
    }

    #[test]
    fn test_infer_organism_zebrafish() {
        let ctx = ExperimentContext::infer("zebrafish genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("danrer11"));
    }

    #[test]
    fn test_infer_organism_yeast() {
        let ctx = ExperimentContext::infer("yeast sacCer3 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("saccer3"));
    }

    #[test]
    fn test_infer_organism_arabidopsis() {
        let ctx = ExperimentContext::infer("arabidopsis tair10", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("tair10"));
    }

    #[test]
    fn test_infer_organism_celegans() {
        let ctx = ExperimentContext::infer("c. elegans ce11", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("ce11"));
    }

    #[test]
    fn test_infer_organism_pig() {
        let ctx = ExperimentContext::infer("pig susScr11 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("susScr11"));
    }

    #[test]
    fn test_infer_organism_chicken() {
        let ctx = ExperimentContext::infer("chicken galGal6", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("galGal6"));
    }

    #[test]
    fn test_infer_organism_cow() {
        let ctx = ExperimentContext::infer("cow bosTau9 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("bosTau9"));
    }

    #[test]
    fn test_infer_organism_dog() {
        let ctx = ExperimentContext::infer("dog canFam3 genome", &[]);
        assert_eq!(ctx.organism.as_deref(), Some("canFam3"));
    }

    #[test]
    fn test_infer_organism_unknown() {
        let ctx = ExperimentContext::infer("process some data", &[]);
        assert_eq!(ctx.organism, None);
    }

    #[test]
    fn test_infer_single_end_from_task() {
        let ctx = ExperimentContext::infer("align single-end reads", &[]);
        assert_eq!(ctx.library_type, LibraryType::SingleEnd);
    }

    #[test]
    fn test_infer_paired_end_from_underscore_files() {
        let ctx = ExperimentContext::infer("align reads", &["sample_1.fastq", "sample_2.fastq"]);
        assert_eq!(ctx.library_type, LibraryType::PairedEnd);
    }

    #[test]
    fn test_infer_library_type_unknown() {
        let ctx = ExperimentContext::infer("process data", &["input.txt"]);
        assert_eq!(ctx.library_type, LibraryType::Unknown);
    }

    #[test]
    fn test_infer_trimming_stage() {
        let ctx = ExperimentContext::infer("trim adapters with fastp", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Trimming);
    }

    #[test]
    fn test_infer_post_alignment_stage() {
        let ctx = ExperimentContext::infer("sort and index bam", &[]);
        assert_eq!(ctx.analysis_stage, Stage::PostAlignment);
    }

    #[test]
    fn test_infer_differential_stage() {
        let ctx = ExperimentContext::infer("differential expression with deseq2", &[]);
        assert_eq!(ctx.analysis_stage, Stage::DifferentialAnalysis);
    }

    #[test]
    fn test_infer_annotation_stage() {
        let ctx = ExperimentContext::infer("annotate with snpeff", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Annotation);
    }

    #[test]
    fn test_infer_unknown_stage() {
        let ctx = ExperimentContext::infer("process some data", &[]);
        assert_eq!(ctx.analysis_stage, Stage::Unknown);
    }

    #[test]
    fn test_assay_type_labels() {
        assert_eq!(AssayType::RnaSeq.label(), "RNA-seq");
        assert_eq!(AssayType::Wgs.label(), "WGS");
        assert_eq!(AssayType::Wes.label(), "WES/Exome");
        assert_eq!(AssayType::ChipSeq.label(), "ChIP-seq");
        assert_eq!(AssayType::AtacSeq.label(), "ATAC-seq");
        assert_eq!(AssayType::HiC.label(), "Hi-C");
        assert_eq!(AssayType::Bisulfite.label(), "Bisulfite-seq");
        assert_eq!(AssayType::ScRnaSeq.label(), "scRNA-seq");
        assert_eq!(AssayType::LongReads.label(), "Long-read sequencing");
        assert_eq!(AssayType::Metagenomics.label(), "Metagenomics");
        assert_eq!(AssayType::Amplicon.label(), "Amplicon sequencing");
    }

    #[test]
    fn test_recommended_workflow_all_types() {
        let types_and_workflows = [
            (AssayType::RnaSeq, "rnaseq"),
            (AssayType::Wgs, "wgs"),
            (AssayType::Wes, "wes"),
            (AssayType::ChipSeq, "chipseq"),
            (AssayType::AtacSeq, "atacseq"),
            (AssayType::HiC, "hic"),
            (AssayType::Bisulfite, "methylseq"),
            (AssayType::ScRnaSeq, "scrnaseq"),
            (AssayType::LongReads, "longreads"),
            (AssayType::Metagenomics, "metagenomics"),
            (AssayType::Amplicon, "amplicon"),
        ];
        for (assay, expected) in types_and_workflows {
            let ctx = ExperimentContext {
                assay_type: Some(assay),
                organism: None,
                library_type: LibraryType::Unknown,
                analysis_stage: Stage::Unknown,
            };
            assert_eq!(ctx.recommended_workflow(), Some(expected));
        }
    }

    #[test]
    fn test_default_params_mouse_reference() {
        let ctx = ExperimentContext::infer("mouse mm10 RNA-seq", &[]);
        let params = ctx.default_params();
        assert_eq!(params.get("reference").unwrap(), "mm10");
    }

    #[test]
    fn test_default_params_mm39_reference() {
        let ctx = ExperimentContext::infer("mm39 genome", &[]);
        let params = ctx.default_params();
        assert_eq!(params.get("reference").unwrap(), "mm39");
    }

    #[test]
    fn test_default_params_unknown_organism() {
        let ctx = ExperimentContext::infer("zebrafish genome", &[]);
        let params = ctx.default_params();
        assert_eq!(params.get("reference").unwrap(), "danrer11");
    }

    #[test]
    fn test_default_params_paired_end_layout() {
        let ctx = ExperimentContext::infer("paired-end RNA-seq", &[]);
        let params = ctx.default_params();
        assert_eq!(params.get("layout").unwrap(), "paired-end");
    }

    #[test]
    fn test_prompt_hint_with_library() {
        let ctx = ExperimentContext::infer("paired-end RNA-seq alignment", &[]);
        let hint = ctx.to_prompt_hint();
        assert!(hint.contains("paired-end"));
    }

    #[test]
    fn test_prompt_hint_with_stage() {
        let ctx = ExperimentContext::infer("align RNA-seq reads", &[]);
        let hint = ctx.to_prompt_hint();
        assert!(hint.contains("alignment"));
    }

    #[test]
    fn test_prompt_hint_all_fields() {
        let ctx = ExperimentContext::infer("paired-end RNA-seq hg38 alignment", &[]);
        let hint = ctx.to_prompt_hint();
        assert!(hint.contains("RNA-seq"));
        assert!(hint.contains("hg38"));
        assert!(hint.contains("paired-end"));
        assert!(hint.contains("alignment"));
    }

    #[test]
    fn test_infer_rnaseq_from_transcriptom() {
        let ctx = ExperimentContext::infer("transcriptomics analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::RnaSeq));
    }

    #[test]
    fn test_infer_scrnaseq_from_cellranger() {
        let ctx = ExperimentContext::infer("run cellranger count", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::ScRnaSeq));
    }

    #[test]
    fn test_infer_bisulfite_from_wgbs() {
        let ctx = ExperimentContext::infer("WGBS methylation analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Bisulfite));
    }

    #[test]
    fn test_infer_metagenomics_from_16s() {
        let ctx = ExperimentContext::infer("16s sequencing analysis", &[]);
        assert_eq!(ctx.assay_type, Some(AssayType::Metagenomics));
    }

    #[test]
    fn test_infer_assay_from_sam_files() {
        let ctx = ExperimentContext::infer("process data", &["input.sam"]);
        assert_eq!(ctx.assay_type, None);
    }

    #[test]
    fn test_infer_assay_from_cram_files() {
        let ctx = ExperimentContext::infer("process data", &["input.cram"]);
        assert_eq!(ctx.assay_type, None);
    }

    #[test]
    fn test_infer_assay_from_fq_files() {
        let ctx = ExperimentContext::infer("process data", &["input.fq"]);
        assert_eq!(ctx.assay_type, None);
    }
}
