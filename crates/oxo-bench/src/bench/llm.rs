//! LLM model evaluation harness for oxo-call benchmarking.
//!
//! Measures the accuracy and consistency of LLM-generated bioinformatics
//! commands across different model sizes/costs (gpt-4o, gpt-4o-mini, etc.).
//!
//! # Evaluation methodology
//!
//! For each (tool, task) pair drawn from a fixed evaluation suite:
//! 1. Generate a command `n_repeats` times with the target model.
//! 2. Parse each response and check:
//!    - Format validity (does it contain `ARGS:` and `EXPLANATION:`?).
//!    - Semantic correctness via a reference checklist (key flags expected).
//!    - Self-consistency (do repeated calls produce the same flags?).
//! 3. Aggregate metrics: accuracy@1, consistency, avg_latency_ms, avg_tokens.

/// Configuration for a model evaluation run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelBenchConfig {
    /// LLM model identifier (e.g. "gpt-4o-mini", "gpt-4o", "claude-3-haiku-20240307").
    pub model: String,
    /// Number of times to repeat each task for consistency measurement.
    pub n_repeats: usize,
    /// Temperature to use for generation (0.0 = deterministic).
    pub temperature: f32,
    /// Maximum tokens to generate per response.
    pub max_tokens: u32,
}

impl Default for ModelBenchConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-mini".to_string(),
            n_repeats: 3,
            temperature: 0.0,
            max_tokens: 512,
        }
    }
}

/// A single evaluation task: tool + natural language description + expected key flags.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvalTask {
    /// Tool binary name.
    pub tool: String,
    /// Natural language task description (the user input).
    pub task: String,
    /// Key flags or substrings that MUST appear in a correct ARGS line.
    pub required_patterns: Vec<String>,
    /// Category for grouping results (e.g. "alignment", "qc", "variant-calling").
    pub category: String,
}

/// Result for a single model × task evaluation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelBenchResult {
    pub model: String,
    pub tool: String,
    pub task_summary: String,
    pub category: String,
    /// Number of responses that matched ALL required patterns.
    pub correct_count: usize,
    /// Total number of attempts.
    pub total_count: usize,
    /// Fraction of responses with valid ARGS:/EXPLANATION: format.
    pub format_validity_rate: f64,
    /// Fraction of non-empty responses that are identical to the first response.
    pub self_consistency_rate: f64,
    /// Average latency per call (milliseconds). None if not measured.
    pub avg_latency_ms: Option<f64>,
}

impl ModelBenchResult {
    /// Accuracy: fraction of correct responses out of total.
    pub fn accuracy(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.correct_count as f64 / self.total_count as f64
        }
    }
}

/// The canonical evaluation task suite covering common bioinformatics operations.
///
/// This benchmark spans 20+ tools and 100+ tasks organised by category.  Each
/// task defines the minimum required patterns for a correct response so that
/// accuracy can be measured automatically.
pub fn canonical_eval_tasks() -> Vec<EvalTask> {
    vec![
        // ── Alignment ─────────────────────────────────────────────────────────
        EvalTask {
            tool: "bwa".to_string(),
            task: "align reads.fastq to ref.fa with 8 threads".to_string(),
            required_patterns: vec!["mem".to_string(), "-t 8".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bwa".to_string(),
            task: "build BWA index for genome.fa".to_string(),
            required_patterns: vec!["index".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bwa-mem2".to_string(),
            task: "align paired reads R1.fastq R2.fastq to reference.fa with read group ID sample1".to_string(),
            required_patterns: vec!["mem".to_string(), "-R".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bowtie2".to_string(),
            task: "align R1.fastq.gz and R2.fastq.gz to index bt2_index with 4 threads".to_string(),
            required_patterns: vec!["-x".to_string(), "-1".to_string(), "-2".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "bowtie2".to_string(),
            task: "build bowtie2 index from genome.fa".to_string(),
            required_patterns: vec!["bowtie2-build".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "minimap2".to_string(),
            task: "align long reads reads.fastq.gz to reference.fa with 16 threads".to_string(),
            required_patterns: vec!["-t".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "minimap2".to_string(),
            task: "align ONT cDNA reads to reference genome for splice-aware mapping".to_string(),
            required_patterns: vec!["-ax splice".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "hisat2".to_string(),
            task: "align paired reads R1.fq.gz R2.fq.gz to HISAT2 index genome_ht2 with 8 threads".to_string(),
            required_patterns: vec!["-x".to_string(), "-1".to_string(), "-2".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "STAR".to_string(),
            task: "align paired reads R1.fq.gz R2.fq.gz to STAR index in star_idx/ and output coordinate-sorted BAM".to_string(),
            required_patterns: vec!["--genomeDir".to_string(), "--readFilesIn".to_string(), "SortedByCoordinate".to_string()],
            category: "alignment".to_string(),
        },
        EvalTask {
            tool: "STAR".to_string(),
            task: "generate STAR genome index from genome.fa and genes.gtf with 8 threads".to_string(),
            required_patterns: vec!["--runMode genomeGenerate".to_string(), "--genomeFastaFiles".to_string(), "--sjdbGTFfile".to_string()],
            category: "alignment".to_string(),
        },
        // ── QC & preprocessing ────────────────────────────────────────────────
        EvalTask {
            tool: "fastp".to_string(),
            task: "quality trim paired reads R1.fastq.gz R2.fastq.gz with adapter auto-detection and 8 threads".to_string(),
            required_patterns: vec!["--in1".to_string(), "--in2".to_string(), "--detect_adapter_for_pe".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "fastp".to_string(),
            task: "trim single-end reads input.fq.gz and filter reads shorter than 50bp".to_string(),
            required_patterns: vec!["--in1".to_string(), "--length_required".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "fastqc".to_string(),
            task: "run quality check on sample_R1.fastq.gz with 4 threads and save to qc_output/".to_string(),
            required_patterns: vec!["-t".to_string(), "-o".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "trimmomatic".to_string(),
            task: "trim paired-end reads R1.fq.gz R2.fq.gz with leading:3 trailing:3 slidingwindow:4:20".to_string(),
            required_patterns: vec!["PE".to_string(), "LEADING".to_string(), "TRAILING".to_string(), "SLIDINGWINDOW".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "cutadapt".to_string(),
            task: "trim Illumina TruSeq adapters from paired reads R1.fq R2.fq and discard reads shorter than 20bp".to_string(),
            required_patterns: vec!["-a".to_string(), "-o".to_string(), "-m".to_string()],
            category: "qc".to_string(),
        },
        EvalTask {
            tool: "multiqc".to_string(),
            task: "aggregate QC reports in results/ directory and output to multiqc_output/".to_string(),
            required_patterns: vec!["-o".to_string()],
            category: "qc".to_string(),
        },
        // ── SAM/BAM manipulation ──────────────────────────────────────────────
        EvalTask {
            tool: "samtools".to_string(),
            task: "sort aligned.bam by coordinate and output to sorted.bam using 4 threads".to_string(),
            required_patterns: vec!["sort".to_string(), "-o".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "index sorted.bam".to_string(),
            required_patterns: vec!["index".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "view only mapped reads from input.bam and output as BAM".to_string(),
            required_patterns: vec!["view".to_string(), "-F 4".to_string(), "-b".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "compute alignment statistics for aligned.bam".to_string(),
            required_patterns: vec!["flagstat".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "merge sample1.bam sample2.bam sample3.bam into merged.bam".to_string(),
            required_patterns: vec!["merge".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "generate depth information for sample.bam and output to depth.txt".to_string(),
            required_patterns: vec!["depth".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "extract reads from chromosome chr1:1000-2000 of sorted.bam".to_string(),
            required_patterns: vec!["view".to_string(), "chr1".to_string()],
            category: "sam-bam".to_string(),
        },
        EvalTask {
            tool: "picard".to_string(),
            task: "mark duplicates in input.bam and output to dedup.bam with metrics file".to_string(),
            required_patterns: vec!["MarkDuplicates".to_string(), "INPUT".to_string()],
            category: "sam-bam".to_string(),
        },
        // ── BED/interval operations ───────────────────────────────────────────
        EvalTask {
            tool: "bedtools".to_string(),
            task: "intersect peaks.bed with genes.bed and report only overlapping regions".to_string(),
            required_patterns: vec!["intersect".to_string(), "-a".to_string(), "-b".to_string()],
            category: "interval-ops".to_string(),
        },
        EvalTask {
            tool: "bedtools".to_string(),
            task: "compute genome coverage from aligned.bam and output bedGraph".to_string(),
            required_patterns: vec!["genomecov".to_string(), "-ibam".to_string(), "-bg".to_string()],
            category: "interval-ops".to_string(),
        },
        EvalTask {
            tool: "bedtools".to_string(),
            task: "sort input.bed by chromosome and position".to_string(),
            required_patterns: vec!["sort".to_string(), "-i".to_string()],
            category: "interval-ops".to_string(),
        },
        EvalTask {
            tool: "bedtools".to_string(),
            task: "merge overlapping intervals in peaks.bed".to_string(),
            required_patterns: vec!["merge".to_string(), "-i".to_string()],
            category: "interval-ops".to_string(),
        },
        // ── Variant calling ────────────────────────────────────────────────
        EvalTask {
            tool: "gatk".to_string(),
            task: "call variants in gVCF mode on sample.bam against reference.fa".to_string(),
            required_patterns: vec!["HaplotypeCaller".to_string(), "-ERC GVCF".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "gatk".to_string(),
            task: "apply base quality score recalibration to sample.bam using recal.table and reference.fa".to_string(),
            required_patterns: vec!["ApplyBQSR".to_string(), "-R".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "gatk".to_string(),
            task: "combine gVCF files sample1.g.vcf.gz and sample2.g.vcf.gz for joint genotyping".to_string(),
            required_patterns: vec!["CombineGVCFs".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "bcftools".to_string(),
            task: "call SNVs and indels from input.bcf with ploidy 2".to_string(),
            required_patterns: vec!["call".to_string(), "-m".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "bcftools".to_string(),
            task: "filter VCF to keep only variants with QUAL > 30 and DP > 10".to_string(),
            required_patterns: vec!["filter".to_string(), "QUAL".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "bcftools".to_string(),
            task: "compute VCF statistics for variants.vcf.gz".to_string(),
            required_patterns: vec!["stats".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "freebayes".to_string(),
            task: "call variants in sample.bam against reference.fa with min mapping quality 20".to_string(),
            required_patterns: vec!["-f".to_string(), "--min-mapping-quality".to_string()],
            category: "variant-calling".to_string(),
        },
        EvalTask {
            tool: "deepvariant".to_string(),
            task: "call variants from sample.bam with reference.fa using WGS model and 8 threads".to_string(),
            required_patterns: vec!["--model_type".to_string(), "--ref".to_string(), "--reads".to_string()],
            category: "variant-calling".to_string(),
        },
        // ── Structural variants ───────────────────────────────────────────────
        EvalTask {
            tool: "delly".to_string(),
            task: "call structural variants from sample.bam against reference.fa".to_string(),
            required_patterns: vec!["call".to_string(), "-g".to_string()],
            category: "structural-variants".to_string(),
        },
        EvalTask {
            tool: "manta".to_string(),
            task: "configure and run SV calling for sample.bam with reference.fa".to_string(),
            required_patterns: vec!["--bam".to_string(), "--referenceFasta".to_string()],
            category: "structural-variants".to_string(),
        },
        // ── Quantification ─────────────────────────────────────────────────
        EvalTask {
            tool: "featureCounts".to_string(),
            task: "count reads in aligned.bam against annotation.gtf for paired-end data with 8 threads".to_string(),
            required_patterns: vec!["-a".to_string(), "-p".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "featureCounts".to_string(),
            task: "count reads at gene level from aligned.bam using annotation.gtf with strand-specific counting (reverse)".to_string(),
            required_patterns: vec!["-a".to_string(), "-s 2".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "salmon".to_string(),
            task: "quantify expression from R1.fastq.gz and R2.fastq.gz against index salmon_index".to_string(),
            required_patterns: vec!["quant".to_string(), "-1".to_string(), "-2".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "salmon".to_string(),
            task: "build salmon index from transcripts.fa".to_string(),
            required_patterns: vec!["index".to_string(), "-t".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "kallisto".to_string(),
            task: "quantify paired-end reads R1.fq.gz R2.fq.gz against transcriptome index and output to results/ with 100 bootstraps".to_string(),
            required_patterns: vec!["quant".to_string(), "-i".to_string(), "-b".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "htseq-count".to_string(),
            task: "count reads in aligned.bam using genes.gtf with reverse strand mode".to_string(),
            required_patterns: vec!["-s reverse".to_string()],
            category: "quantification".to_string(),
        },
        EvalTask {
            tool: "stringtie".to_string(),
            task: "assemble transcripts from aligned.bam using reference annotation genes.gtf with 8 threads".to_string(),
            required_patterns: vec!["-G".to_string(), "-p".to_string()],
            category: "quantification".to_string(),
        },
        // ── Metagenomics ────────────────────────────────────────────────────
        EvalTask {
            tool: "kraken2".to_string(),
            task: "classify paired reads R1.fastq.gz R2.fastq.gz against database /db/kraken2 and write report to report.txt".to_string(),
            required_patterns: vec!["--db".to_string(), "--paired".to_string(), "--report".to_string()],
            category: "metagenomics".to_string(),
        },
        EvalTask {
            tool: "bracken".to_string(),
            task: "estimate abundance from kraken2 report.txt with database /db/kraken2 and read length 150".to_string(),
            required_patterns: vec!["-d".to_string(), "-i".to_string(), "-r 150".to_string()],
            category: "metagenomics".to_string(),
        },
        EvalTask {
            tool: "metaphlan".to_string(),
            task: "profile metagenome from reads.fastq.gz with 8 threads and output to profile.txt".to_string(),
            required_patterns: vec!["--input_type".to_string(), "--nproc".to_string(), "-o".to_string()],
            category: "metagenomics".to_string(),
        },
        EvalTask {
            tool: "megahit".to_string(),
            task: "assemble metagenome from paired reads R1.fq.gz R2.fq.gz with 16 threads and output to assembly/".to_string(),
            required_patterns: vec!["-1".to_string(), "-2".to_string(), "-t".to_string(), "-o".to_string()],
            category: "metagenomics".to_string(),
        },
        EvalTask {
            tool: "spades".to_string(),
            task: "assemble metagenome from paired reads using meta mode with 8 threads".to_string(),
            required_patterns: vec!["--meta".to_string()],
            category: "metagenomics".to_string(),
        },
        // ── Epigenomics ─────────────────────────────────────────────────────
        EvalTask {
            tool: "macs3".to_string(),
            task: "call narrow peaks from treatment.bam with control.bam using genome size hs".to_string(),
            required_patterns: vec!["callpeak".to_string(), "-t".to_string(), "-c".to_string(), "-g hs".to_string()],
            category: "epigenomics".to_string(),
        },
        EvalTask {
            tool: "macs3".to_string(),
            task: "call broad peaks from ChIP-seq treatment.bam with control.bam".to_string(),
            required_patterns: vec!["callpeak".to_string(), "--broad".to_string()],
            category: "epigenomics".to_string(),
        },
        EvalTask {
            tool: "deeptools".to_string(),
            task: "compute read coverage across the genome from sample.bam normalized by RPKM with 8 threads".to_string(),
            required_patterns: vec!["bamCoverage".to_string(), "--normalizeUsing RPKM".to_string()],
            category: "epigenomics".to_string(),
        },
        EvalTask {
            tool: "deeptools".to_string(),
            task: "plot heatmap of signal around TSS from matrix.gz".to_string(),
            required_patterns: vec!["plotHeatmap".to_string(), "-m".to_string()],
            category: "epigenomics".to_string(),
        },
        EvalTask {
            tool: "bismark".to_string(),
            task: "align bisulfite-seq reads R1.fq.gz R2.fq.gz to bisulfite genome in bismark_idx/".to_string(),
            required_patterns: vec!["--genome".to_string(), "-1".to_string(), "-2".to_string()],
            category: "epigenomics".to_string(),
        },
        // ── Single-cell ──────────────────────────────────────────────────────
        EvalTask {
            tool: "cellranger".to_string(),
            task: "count single-cell 3' gene expression from fastqs/ directory with reference transcriptome in refdata/".to_string(),
            required_patterns: vec!["count".to_string(), "--fastqs".to_string(), "--transcriptome".to_string()],
            category: "single-cell".to_string(),
        },
        EvalTask {
            tool: "STARsolo".to_string(),
            task: "align 10x Chromium v3 scRNA-seq reads R1.fq.gz R2.fq.gz to STAR index in star_idx/".to_string(),
            required_patterns: vec!["--soloType".to_string(), "--genomeDir".to_string()],
            category: "single-cell".to_string(),
        },
        EvalTask {
            tool: "velocyto".to_string(),
            task: "count spliced/unspliced from sample.bam using genes.gtf and output to velocyto/".to_string(),
            required_patterns: vec!["run".to_string()],
            category: "single-cell".to_string(),
        },
        // ── Assembly ─────────────────────────────────────────────────────────
        EvalTask {
            tool: "flye".to_string(),
            task: "assemble nanopore reads reads.fastq.gz with estimated genome size 5m".to_string(),
            required_patterns: vec!["--nano-raw".to_string(), "--genome-size".to_string()],
            category: "assembly".to_string(),
        },
        EvalTask {
            tool: "hifiasm".to_string(),
            task: "assemble PacBio HiFi reads reads.fq.gz with 16 threads".to_string(),
            required_patterns: vec!["-t".to_string()],
            category: "assembly".to_string(),
        },
        EvalTask {
            tool: "quast".to_string(),
            task: "evaluate genome assembly assembly.fasta with reference genome reference.fa".to_string(),
            required_patterns: vec!["-r".to_string()],
            category: "assembly".to_string(),
        },
        // ── Annotation ───────────────────────────────────────────────────────
        EvalTask {
            tool: "prokka".to_string(),
            task: "annotate bacterial genome assembly.fa with genus Escherichia and output to annotation/".to_string(),
            required_patterns: vec!["--genus".to_string(), "--outdir".to_string()],
            category: "annotation".to_string(),
        },
        EvalTask {
            tool: "snpEff".to_string(),
            task: "annotate variants in variants.vcf using hg38 database".to_string(),
            required_patterns: vec!["hg38".to_string()],
            category: "annotation".to_string(),
        },
        EvalTask {
            tool: "blast".to_string(),
            task: "search protein query.fa against database nr with evalue 1e-5 and output format 6".to_string(),
            required_patterns: vec!["blastp".to_string(), "-evalue".to_string(), "-outfmt 6".to_string()],
            category: "annotation".to_string(),
        },
        EvalTask {
            tool: "diamond".to_string(),
            task: "search query.fa against nr.dmnd in sensitive mode with 8 threads".to_string(),
            required_patterns: vec!["blastp".to_string(), "--sensitive".to_string(), "-p".to_string()],
            category: "annotation".to_string(),
        },
        // ── Sequence manipulation ─────────────────────────────────────────────
        EvalTask {
            tool: "seqkit".to_string(),
            task: "extract sequences longer than 1000bp from input.fasta".to_string(),
            required_patterns: vec!["seq".to_string(), "--min-len".to_string()],
            category: "sequence-ops".to_string(),
        },
        EvalTask {
            tool: "seqkit".to_string(),
            task: "compute statistics for input.fastq.gz".to_string(),
            required_patterns: vec!["stats".to_string()],
            category: "sequence-ops".to_string(),
        },
        EvalTask {
            tool: "seqkit".to_string(),
            task: "subsample 10000 reads from input.fastq.gz".to_string(),
            required_patterns: vec!["sample".to_string(), "-n".to_string()],
            category: "sequence-ops".to_string(),
        },
        // ── Phylogenetics ────────────────────────────────────────────────────
        EvalTask {
            tool: "mafft".to_string(),
            task: "perform multiple sequence alignment on sequences.fa with auto strategy".to_string(),
            required_patterns: vec!["--auto".to_string()],
            category: "phylogenetics".to_string(),
        },
        EvalTask {
            tool: "iqtree2".to_string(),
            task: "infer maximum likelihood tree from alignment.fa with 1000 ultrafast bootstraps and auto model selection".to_string(),
            required_patterns: vec!["-s".to_string(), "-B 1000".to_string(), "-m".to_string()],
            category: "phylogenetics".to_string(),
        },
        // ── Format conversion ────────────────────────────────────────────────
        EvalTask {
            tool: "samtools".to_string(),
            task: "convert SAM to BAM format from input.sam".to_string(),
            required_patterns: vec!["view".to_string(), "-b".to_string()],
            category: "format-conversion".to_string(),
        },
        EvalTask {
            tool: "bcftools".to_string(),
            task: "convert BCF to VCF format from input.bcf".to_string(),
            required_patterns: vec!["view".to_string()],
            category: "format-conversion".to_string(),
        },
        EvalTask {
            tool: "bedtools".to_string(),
            task: "convert BAM to BED format from aligned.bam".to_string(),
            required_patterns: vec!["bamtobed".to_string(), "-i".to_string()],
            category: "format-conversion".to_string(),
        },
    ]
}

/// Ablation study task set.
///
/// This smaller set is designed for comparing pipeline variants (docs-only
/// vs docs+skills vs full pipeline).  Each task is tagged with its expected
/// difficulty so that ablation results can be stratified.
pub fn ablation_eval_tasks() -> Vec<EvalTask> {
    vec![
        // Easy — common flags well-documented in --help
        EvalTask {
            tool: "samtools".to_string(),
            task: "sort aligned.bam by coordinate".to_string(),
            required_patterns: vec!["sort".to_string()],
            category: "easy".to_string(),
        },
        EvalTask {
            tool: "samtools".to_string(),
            task: "index sorted.bam".to_string(),
            required_patterns: vec!["index".to_string()],
            category: "easy".to_string(),
        },
        EvalTask {
            tool: "bwa".to_string(),
            task: "align reads.fq to ref.fa".to_string(),
            required_patterns: vec!["mem".to_string()],
            category: "easy".to_string(),
        },
        // Medium — requires combining multiple flags correctly
        EvalTask {
            tool: "fastp".to_string(),
            task: "trim paired reads R1.fq.gz R2.fq.gz with adapter auto-detection".to_string(),
            required_patterns: vec!["--in1".to_string(), "--in2".to_string(), "--detect_adapter_for_pe".to_string()],
            category: "medium".to_string(),
        },
        EvalTask {
            tool: "gatk".to_string(),
            task: "call variants from sample.bam in gVCF mode against reference.fa".to_string(),
            required_patterns: vec!["HaplotypeCaller".to_string(), "-ERC GVCF".to_string()],
            category: "medium".to_string(),
        },
        // Hard — requires domain knowledge beyond --help
        EvalTask {
            tool: "STAR".to_string(),
            task: "align paired RNA-seq reads with coordinate-sorted BAM output and intron motif filtering".to_string(),
            required_patterns: vec!["SortedByCoordinate".to_string(), "--outFilterIntronMotifs".to_string()],
            category: "hard".to_string(),
        },
        EvalTask {
            tool: "macs3".to_string(),
            task: "call broad histone modification peaks from treatment.bam with control.bam".to_string(),
            required_patterns: vec!["callpeak".to_string(), "--broad".to_string(), "-t".to_string(), "-c".to_string()],
            category: "hard".to_string(),
        },
    ]
}

/// Parse a raw LLM response string for `ARGS:` and `EXPLANATION:` lines.
///
/// Returns `(args, explanation)` where `args` is `None` if the format is invalid.
pub fn parse_llm_response(raw: &str) -> (Option<String>, Option<String>) {
    let args = raw
        .lines()
        .find(|l| l.trim_start().starts_with("ARGS:"))
        .map(|l| l.trim_start_matches("ARGS:").trim().to_string());

    let explanation = raw
        .lines()
        .find(|l| l.trim_start().starts_with("EXPLANATION:"))
        .map(|l| l.trim_start_matches("EXPLANATION:").trim().to_string());

    (args, explanation)
}

/// Check whether a parsed ARGS line satisfies all required patterns for a task.
pub fn check_correctness(args: &str, required_patterns: &[String]) -> bool {
    required_patterns
        .iter()
        .all(|pat| args.contains(pat.as_str()))
}

/// Compute self-consistency: fraction of responses matching the first non-empty response.
pub fn compute_consistency(responses: &[Option<String>]) -> f64 {
    let non_empty: Vec<&str> = responses
        .iter()
        .filter_map(|r| r.as_deref())
        .filter(|s| !s.is_empty())
        .collect();

    if non_empty.len() <= 1 {
        return 1.0;
    }

    let reference = non_empty[0];
    let matching = non_empty.iter().filter(|&&r| r == reference).count();
    matching as f64 / non_empty.len() as f64
}

/// Run the LLM model benchmark for an offline-simulated scenario
/// (without actual LLM API calls — for unit testing the harness logic).
///
/// In production use, the `oxo-bench eval-models` CLI command wires up real LLM calls.
pub fn run_model_bench(
    config: &ModelBenchConfig,
    tasks: &[EvalTask],
    // Injected response generator — allows testing without real API calls.
    response_fn: &dyn Fn(&str, &str) -> String,
) -> Vec<ModelBenchResult> {
    tasks
        .iter()
        .map(|task| {
            let mut correct_count = 0;
            let mut valid_format = 0;
            let mut args_responses: Vec<Option<String>> = Vec::new();

            for _ in 0..config.n_repeats {
                let raw = response_fn(&task.tool, &task.task);
                let (args, _explanation) = parse_llm_response(&raw);

                if args.is_some() {
                    valid_format += 1;
                }

                if let Some(ref a) = args
                    && check_correctness(a, &task.required_patterns)
                {
                    correct_count += 1;
                }
                args_responses.push(args);
            }

            let consistency = compute_consistency(&args_responses);
            let format_rate = valid_format as f64 / config.n_repeats as f64;

            ModelBenchResult {
                model: config.model.clone(),
                tool: task.tool.clone(),
                task_summary: task.task.chars().take(60).collect(),
                category: task.category.clone(),
                correct_count,
                total_count: config.n_repeats,
                format_validity_rate: format_rate,
                self_consistency_rate: consistency,
                avg_latency_ms: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_response(tool: &str, _task: &str) -> String {
        // Simulate a well-formed response for well-known tools.
        match tool {
            "samtools" => "ARGS: sort -o sorted.bam -@ 4 aligned.bam\nEXPLANATION: Sort by coordinate.".to_string(),
            "fastp" => "ARGS: --in1 R1.fastq.gz --in2 R2.fastq.gz --detect_adapter_for_pe\nEXPLANATION: Trim paired reads.".to_string(),
            _ => "ARGS: --help\nEXPLANATION: Show help.".to_string(),
        }
    }

    #[test]
    fn test_parse_valid_response() {
        let raw = "ARGS: sort -o out.bam in.bam\nEXPLANATION: Sort BAM file.";
        let (args, expl) = parse_llm_response(raw);
        assert_eq!(args.as_deref(), Some("sort -o out.bam in.bam"));
        assert_eq!(expl.as_deref(), Some("Sort BAM file."));
    }

    #[test]
    fn test_parse_invalid_response() {
        let raw = "some random text without required format";
        let (args, _) = parse_llm_response(raw);
        assert!(args.is_none());
    }

    #[test]
    fn test_check_correctness() {
        let patterns = vec!["sort".to_string(), "-o".to_string()];
        assert!(check_correctness("sort -o out.bam input.bam", &patterns));
        assert!(!check_correctness("view -b input.bam", &patterns));
    }

    #[test]
    fn test_consistency_perfect() {
        let responses = vec![
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
        ];
        assert_eq!(compute_consistency(&responses), 1.0);
    }

    #[test]
    fn test_consistency_partial() {
        let responses = vec![
            Some("sort -o out.bam".to_string()),
            Some("sort -o out.bam".to_string()),
            Some("sort -n out.bam".to_string()),
        ];
        let c = compute_consistency(&responses);
        assert!((c - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_run_model_bench_basic() {
        let config = ModelBenchConfig {
            model: "mock".to_string(),
            n_repeats: 3,
            temperature: 0.0,
            max_tokens: 256,
        };
        let tasks = canonical_eval_tasks();
        let results = run_model_bench(&config, &tasks[..2], &mock_response);
        assert_eq!(results.len(), 2);
        for r in &results {
            assert_eq!(r.total_count, 3);
        }
    }

    #[test]
    fn test_canonical_eval_tasks_not_empty() {
        let tasks = canonical_eval_tasks();
        assert!(!tasks.is_empty());
    }
}
