# =============================================================================
# ATAC-seq workflow — fastp QC → Bowtie2 alignment → MACS3 peak calling
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   genome_fasta:   /path/to/genome.fa    (must be Bowtie2 indexed)
#   bowtie2_index:  /path/to/bt2_index    (basename, e.g. /data/hg38/hg38)
#   blacklist:      /path/to/blacklist.bed
#   genome_size:    hs                    (macs3: hs/mm/ce/dm or integer)
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/peaks/{sample}_peaks.narrowPeak", sample=SAMPLES),
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── QC & adapter trimming with fastp ─────────────────────────────────────────
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
        "--length_required 20 "
        "> {log} 2>&1"


# ── Bowtie2 alignment ─────────────────────────────────────────────────────────
rule bowtie2_align:
    input:
        r1    = "results/trimmed/{sample}_R1.fastq.gz",
        r2    = "results/trimmed/{sample}_R2.fastq.gz",
        index = config["bowtie2_index"],
    output:
        bam = temp("results/aligned/{sample}.unsorted.bam"),
    threads: config.get("threads", 8)
    log: "logs/bowtie2/{sample}.log"
    shell:
        "bowtie2 "
        "-x {input.index} "
        "-1 {input.r1} -2 {input.r2} "
        "-p {threads} "
        "--no-mixed --no-discordant "
        "--no-unal "
        "2> {log} "
        "| samtools view -bS - > {output.bam}"


# ── Sort, mark duplicates, filter ─────────────────────────────────────────────
rule sort_bam:
    input:
        bam = "results/aligned/{sample}.unsorted.bam",
    output:
        bam = "results/aligned/{sample}.sorted.bam",
    threads: 4
    log: "logs/sort/{sample}.log"
    shell:
        "samtools sort -@ {threads} -o {output.bam} {input.bam} > {log} 2>&1"

rule mark_duplicates:
    input:
        bam = "results/aligned/{sample}.sorted.bam",
    output:
        bam     = "results/aligned/{sample}.markdup.bam",
        metrics = "results/qc/{sample}.markdup_metrics.txt",
    log: "logs/markdup/{sample}.log"
    shell:
        "picard MarkDuplicates "
        "I={input.bam} "
        "O={output.bam} "
        "M={output.metrics} "
        "REMOVE_DUPLICATES=false "
        "CREATE_INDEX=true "
        "> {log} 2>&1"

rule filter_and_blacklist:
    input:
        bam       = "results/aligned/{sample}.markdup.bam",
        blacklist = config["blacklist"],
    output:
        bam = "results/aligned/{sample}.filtered.bam",
        bai = "results/aligned/{sample}.filtered.bam.bai",
    threads: 4
    log: "logs/filter/{sample}.log"
    shell:
        "samtools view -@ {threads} -b -F 1804 -f 2 -q 30 {input.bam} "
        "| bedtools intersect -v -abam stdin -b {input.blacklist} "
        "> {output.bam} 2> {log} && "
        "samtools index -@ {threads} {output.bam} >> {log} 2>&1"


# ── MACS3 peak calling ────────────────────────────────────────────────────────
rule macs3_call_peaks:
    input:
        bam = "results/aligned/{sample}.filtered.bam",
    output:
        peaks = "results/peaks/{sample}_peaks.narrowPeak",
        summits = "results/peaks/{sample}_summits.bed",
    params:
        gsize  = config.get("genome_size", "hs"),
        outdir = "results/peaks",
    log: "logs/macs3/{sample}.log"
    shell:
        "macs3 callpeak "
        "-t {input.bam} "
        "-f BAMPE "
        "-n {wildcards.sample} "
        "--outdir {params.outdir} "
        "-g {params.gsize} "
        "--nomodel --shift -75 --extsize 150 "
        "-B --SPMR "
        "--keep-dup all "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ -o results/multiqc/ --force > {log} 2>&1"
