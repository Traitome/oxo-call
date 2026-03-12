# =============================================================================
# RNA-seq workflow — fastp QC → STAR alignment → featureCounts quantification
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   genome_fasta: /path/to/genome.fa
#   star_index:   /path/to/star_index/
#   gtf:          /path/to/annotation.gtf
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/counts/{sample}_counts.txt", sample=SAMPLES),
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── QC with fastp ─────────────────────────────────────────────────────────────
rule fastp_qc:
    input:
        r1 = lambda wc: config["units"][wc.sample]["r1"],
        r2 = lambda wc: config["units"][wc.sample]["r2"],
    output:
        r1   = "results/trimmed/{sample}_R1.fastq.gz",
        r2   = "results/trimmed/{sample}_R2.fastq.gz",
        html = "results/qc/{sample}_fastp.html",
        json = "results/qc/{sample}_fastp.json",
    threads: config.get("threads", 8)
    log: "logs/fastp/{sample}.log"
    shell:
        "fastp "
        "--in1 {input.r1} --in2 {input.r2} "
        "--out1 {output.r1} --out2 {output.r2} "
        "--html {output.html} --json {output.json} "
        "--thread {threads} "
        "--detect_adapter_for_pe "
        "--qualified_quality_phred 20 "
        "--length_required 30 "
        "> {log} 2>&1"


# ── STAR alignment ────────────────────────────────────────────────────────────
rule star_align:
    input:
        r1    = "results/trimmed/{sample}_R1.fastq.gz",
        r2    = "results/trimmed/{sample}_R2.fastq.gz",
        index = config["star_index"],
    output:
        bam = "results/aligned/{sample}/Aligned.sortedByCoord.out.bam",
        log = "results/aligned/{sample}/Log.final.out",
    threads: config.get("threads", 8)
    log: "logs/star/{sample}.log"
    shell:
        "STAR "
        "--runMode alignReads "
        "--genomeDir {input.index} "
        "--readFilesIn {input.r1} {input.r2} "
        "--readFilesCommand zcat "
        "--outSAMtype BAM SortedByCoordinate "
        "--outSAMattributes NH HI AS NM MD "
        "--outFileNamePrefix results/aligned/{wildcards.sample}/ "
        "--runThreadN {threads} "
        "--outSAMstrandField intronMotif "
        "--outFilterIntronMotifs RemoveNoncanonical "
        "> {log} 2>&1"


# ── Index BAM ─────────────────────────────────────────────────────────────────
rule samtools_index:
    input:
        bam = "results/aligned/{sample}/Aligned.sortedByCoord.out.bam",
    output:
        bai = "results/aligned/{sample}/Aligned.sortedByCoord.out.bam.bai",
    threads: 4
    log: "logs/samtools_index/{sample}.log"
    shell:
        "samtools index -@ {threads} {input.bam} > {log} 2>&1"


# ── featureCounts quantification ──────────────────────────────────────────────
rule featurecounts:
    input:
        bam = "results/aligned/{sample}/Aligned.sortedByCoord.out.bam",
        gtf = config["gtf"],
    output:
        counts = "results/counts/{sample}_counts.txt",
    threads: config.get("threads", 8)
    log: "logs/featurecounts/{sample}.log"
    shell:
        "featureCounts "
        "-T {threads} "
        "-p "
        "-a {input.gtf} "
        "-o {output.counts} "
        "{input.bam} "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        expand("results/aligned/{sample}/Log.final.out", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ results/aligned/ -o results/multiqc/ > {log} 2>&1"
