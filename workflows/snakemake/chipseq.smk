# =============================================================================
# ChIP-seq workflow — fastp QC → Bowtie2 → Picard MarkDup → filter → MACS3 → bigWig
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [H3K27ac_rep1, H3K27ac_rep2, ...]
#   units:
#     H3K27ac_rep1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   bowtie2_index: /path/to/bt2_index
#   blacklist:     /path/to/blacklist.bed
#   genome_size:   hs
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/peaks/{sample}_peaks.narrowPeak", sample=SAMPLES),
        expand("results/bigwig/{sample}.bw", sample=SAMPLES),
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
        "--length_required 20 "
        "> {log} 2>&1"


# ── Bowtie2 alignment ─────────────────────────────────────────────────────────
rule bowtie2_align:
    input:
        r1    = "results/trimmed/{sample}_R1.fastq.gz",
        r2    = "results/trimmed/{sample}_R2.fastq.gz",
        index = config["bowtie2_index"],
    output:
        bam = "results/aligned/{sample}.sorted.bam",
        bai = "results/aligned/{sample}.sorted.bam.bai",
    threads: config.get("threads", 8)
    log: "logs/bowtie2/{sample}.log"
    shell:
        "bowtie2 -x {input.index} "
        "-1 {input.r1} -2 {input.r2} "
        "-p {threads} --no-mixed --no-discordant "
        "2>{log} "
        "| samtools sort -@ 4 -o {output.bam} && "
        "samtools index {output.bam}"


# ── Picard MarkDuplicates ─────────────────────────────────────────────────────
rule mark_duplicates:
    input:
        bam = "results/aligned/{sample}.sorted.bam",
    output:
        bam     = "results/aligned/{sample}.markdup.bam",
        bai     = "results/aligned/{sample}.markdup.bam.bai",
        metrics = "results/qc/{sample}.markdup_metrics.txt",
    log: "logs/markdup/{sample}.log"
    shell:
        "picard MarkDuplicates "
        "I={input.bam} O={output.bam} M={output.metrics} "
        "REMOVE_DUPLICATES=true "
        "> {log} 2>&1 && "
        "samtools index {output.bam}"


# ── Blacklist filtering ────────────────────────────────────────────────────────
rule filter_reads:
    input:
        bam       = "results/aligned/{sample}.markdup.bam",
        blacklist = config["blacklist"],
    output:
        bam = "results/aligned/{sample}.filtered.bam",
        bai = "results/aligned/{sample}.filtered.bam.bai",
    threads: config.get("threads", 8)
    log: "logs/filter/{sample}.log"
    shell:
        "samtools view -@ {threads} -b -F 1804 -f 2 -q 30 {input.bam} "
        "| bedtools intersect -v -abam stdin -b {input.blacklist} "
        "> {output.bam} 2>{log} && "
        "samtools index {output.bam}"


# ── MACS3 peak calling ────────────────────────────────────────────────────────
rule macs3_callpeak:
    input:
        bam = "results/aligned/{sample}.filtered.bam",
    output:
        peaks   = "results/peaks/{sample}_peaks.narrowPeak",
        summits = "results/peaks/{sample}_summits.bed",
    params:
        genome_size = config.get("genome_size", "hs"),
    log: "logs/macs3/{sample}.log"
    shell:
        "macs3 callpeak "
        "-t {input.bam} -f BAMPE "
        "-n {wildcards.sample} "
        "--outdir results/peaks/ "
        "-g {params.genome_size} "
        "-B --SPMR --keep-dup all --call-summits "
        "> {log} 2>&1"


# ── deepTools bigWig ──────────────────────────────────────────────────────────
rule bamcoverage_bigwig:
    input:
        bam = "results/aligned/{sample}.filtered.bam",
        bai = "results/aligned/{sample}.filtered.bam.bai",
    output:
        bw = "results/bigwig/{sample}.bw",
    threads: config.get("threads", 8)
    log: "logs/bigwig/{sample}.log"
    shell:
        "bamCoverage "
        "-b {input.bam} -o {output.bw} "
        "--binSize 10 --normalizeUsing RPKM "
        "--ignoreDuplicates "
        "-p {threads} "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        expand("results/qc/{sample}.markdup_metrics.txt", sample=SAMPLES),
        expand("results/peaks/{sample}_peaks.narrowPeak", sample=SAMPLES),
        expand("results/bigwig/{sample}.bw", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ results/aligned/ results/peaks/ results/bigwig/ -o results/multiqc/ --force > {log} 2>&1"
