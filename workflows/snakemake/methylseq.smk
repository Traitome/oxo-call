# =============================================================================
# WGBS / Methylation sequencing workflow — Trim Galore → Bismark → methylation extraction
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   bismark_index: /path/to/bismark_genome
#   genome_fasta:  /path/to/genome.fa
#   silva_db:      /path/to/silva_train_set.fa.gz  (not used here)
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/methyl/{sample}.sorted_CpG.bedGraph", sample=SAMPLES),
        expand("results/qc/{sample}_R1_val_1_fastqc.zip", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── Adapter/quality trimming with Trim Galore ─────────────────────────────────
rule trim_galore:
    input:
        r1 = lambda wc: config["units"][wc.sample]["r1"],
        r2 = lambda wc: config["units"][wc.sample]["r2"],
    output:
        r1      = "results/trimmed/{sample}_R1_val_1.fq.gz",
        r2      = "results/trimmed/{sample}_R2_val_2.fq.gz",
        fastqc1 = "results/qc/{sample}_R1_val_1_fastqc.zip",
        fastqc2 = "results/qc/{sample}_R2_val_2_fastqc.zip",
    threads: config.get("threads", 8)
    log: "logs/trim_galore/{sample}.log"
    shell:
        "trim_galore "
        "--paired --cores {threads} "
        "--fastqc --illumina "
        "--output_dir results/trimmed/ "
        "{input.r1} {input.r2} "
        "> {log} 2>&1 && "
        "mv results/trimmed/{wildcards.sample}_R1_val_1_fastqc.zip {output.fastqc1} && "
        "mv results/trimmed/{wildcards.sample}_R2_val_2_fastqc.zip {output.fastqc2}"


# ── Bismark alignment ─────────────────────────────────────────────────────────
rule bismark_align:
    input:
        r1    = "results/trimmed/{sample}_R1_val_1.fq.gz",
        r2    = "results/trimmed/{sample}_R2_val_2.fq.gz",
        index = config["bismark_index"],
    output:
        bam    = "results/aligned/{sample}_bismark_bt2_pe.bam",
        report = "results/aligned/{sample}_bismark_bt2_PE_report.txt",
    threads: config.get("threads", 8)
    log: "logs/bismark_align/{sample}.log"
    shell:
        "bismark "
        "--genome {input.index} "
        "-1 {input.r1} -2 {input.r2} "
        "--output_dir results/aligned/ "
        "--prefix {wildcards.sample} "
        "--parallel {threads} "
        "--non_directional "
        "> {log} 2>&1"


# ── Bismark deduplication ─────────────────────────────────────────────────────
rule bismark_dedup:
    input:
        bam = "results/aligned/{sample}_bismark_bt2_pe.bam",
    output:
        bam    = "results/deduped/{sample}_bismark_bt2_pe.deduplicated.bam",
        report = "results/deduped/{sample}_bismark_bt2_pe.deduplication_report.txt",
    log: "logs/bismark_dedup/{sample}.log"
    shell:
        "deduplicate_bismark "
        "--paired "
        "--output_dir results/deduped/ "
        "{input.bam} "
        "> {log} 2>&1"


# ── Sort and index ────────────────────────────────────────────────────────────
rule sort_index:
    input:
        bam = "results/deduped/{sample}_bismark_bt2_pe.deduplicated.bam",
    output:
        bam = "results/deduped/{sample}.sorted.bam",
        bai = "results/deduped/{sample}.sorted.bam.bai",
    threads: config.get("threads", 8)
    log: "logs/sort_index/{sample}.log"
    shell:
        "samtools sort -@ {threads} "
        "-o {output.bam} {input.bam} > {log} 2>&1 && "
        "samtools index {output.bam}"


# ── Bismark methylation extraction ────────────────────────────────────────────
rule methylation_extract:
    input:
        bam   = "results/deduped/{sample}.sorted.bam",
        index = config["bismark_index"],
    output:
        bedgraph = "results/methyl/{sample}.sorted_CpG.bedGraph",
        cov      = "results/methyl/{sample}.sorted.bismark.cov.gz",
        report   = "results/methyl/{sample}.sorted_splitting_report.txt",
    threads: config.get("threads", 8)
    log: "logs/methylation_extract/{sample}.log"
    shell:
        "bismark_methylation_extractor "
        "--paired-end --comprehensive --CX_context "
        "--cytosine_report "
        "--genome_folder {input.index} "
        "--output results/methyl/ "
        "--parallel {threads} "
        "{input.bam} "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_R1_val_1_fastqc.zip", sample=SAMPLES),
        expand("results/aligned/{sample}_bismark_bt2_PE_report.txt", sample=SAMPLES),
        expand("results/deduped/{sample}_bismark_bt2_pe.deduplication_report.txt", sample=SAMPLES),
        expand("results/methyl/{sample}.sorted_splitting_report.txt", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/trimmed/ results/aligned/ results/deduped/ results/methyl/ "
        "-o results/multiqc/ > {log} 2>&1"
