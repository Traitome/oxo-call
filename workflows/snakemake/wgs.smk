# =============================================================================
# WGS workflow — fastp QC → BWA-MEM2 alignment → GATK HaplotypeCaller
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   genome_fasta:  /path/to/reference.fa   (must be BWA-MEM2 indexed)
#   dbsnp:         /path/to/dbsnp.vcf.gz
#   known_indels:  /path/to/known_indels.vcf.gz
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/vcf/{sample}.g.vcf.gz", sample=SAMPLES),
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
        "> {log} 2>&1"


# ── BWA-MEM2 alignment ────────────────────────────────────────────────────────
rule bwa_mem2_align:
    input:
        r1  = "results/trimmed/{sample}_R1.fastq.gz",
        r2  = "results/trimmed/{sample}_R2.fastq.gz",
        ref = config["genome_fasta"],
    output:
        bam = temp("results/aligned/{sample}.unsorted.bam"),
    threads: config.get("threads", 8)
    log: "logs/bwa_mem2/{sample}.log"
    shell:
        "bwa-mem2 mem "
        "-t {threads} "
        "-R '@RG\\tID:{wildcards.sample}\\tSM:{wildcards.sample}\\tLB:{wildcards.sample}\\tPL:ILLUMINA' "
        "{input.ref} {input.r1} {input.r2} "
        "2> {log} "
        "| samtools view -bS - > {output.bam}"


# ── Sort & mark duplicates ────────────────────────────────────────────────────
rule sort_bam:
    input:
        bam = "results/aligned/{sample}.unsorted.bam",
    output:
        bam = "results/aligned/{sample}.sorted.bam",
    threads: 4
    log: "logs/samtools_sort/{sample}.log"
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
        "gatk MarkDuplicates "
        "-I {input.bam} "
        "-O {output.bam} "
        "-M {output.metrics} "
        "> {log} 2>&1"

rule index_bam:
    input:
        bam = "results/aligned/{sample}.markdup.bam",
    output:
        bai = "results/aligned/{sample}.markdup.bam.bai",
    threads: 4
    log: "logs/samtools_index/{sample}.log"
    shell:
        "samtools index -@ {threads} {input.bam} > {log} 2>&1"


# ── BQSR ─────────────────────────────────────────────────────────────────────
rule base_quality_score_recalibration:
    input:
        bam          = "results/aligned/{sample}.markdup.bam",
        bai          = "results/aligned/{sample}.markdup.bam.bai",
        ref          = config["genome_fasta"],
        dbsnp        = config["dbsnp"],
        known_indels = config["known_indels"],
    output:
        table = "results/aligned/{sample}.recal.table",
    log: "logs/bqsr/{sample}.log"
    shell:
        "gatk BaseRecalibrator "
        "-I {input.bam} "
        "-R {input.ref} "
        "--known-sites {input.dbsnp} "
        "--known-sites {input.known_indels} "
        "-O {output.table} "
        "> {log} 2>&1"

rule apply_bqsr:
    input:
        bam   = "results/aligned/{sample}.markdup.bam",
        bai   = "results/aligned/{sample}.markdup.bam.bai",
        ref   = config["genome_fasta"],
        table = "results/aligned/{sample}.recal.table",
    output:
        bam = "results/aligned/{sample}.recal.bam",
    log: "logs/apply_bqsr/{sample}.log"
    shell:
        "gatk ApplyBQSR "
        "-I {input.bam} "
        "-R {input.ref} "
        "--bqsr-recal-file {input.table} "
        "-O {output.bam} "
        "> {log} 2>&1"


# ── HaplotypeCaller variant calling ──────────────────────────────────────────
rule haplotype_caller:
    input:
        bam = "results/aligned/{sample}.recal.bam",
        ref = config["genome_fasta"],
    output:
        gvcf = "results/vcf/{sample}.g.vcf.gz",
    threads: config.get("threads", 8)
    log: "logs/haplotype_caller/{sample}.log"
    shell:
        "gatk HaplotypeCaller "
        "-R {input.ref} "
        "-I {input.bam} "
        "-O {output.gvcf} "
        "-ERC GVCF "
        "--native-pair-hmm-threads {threads} "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        expand("results/qc/{sample}.markdup_metrics.txt", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ results/aligned/ -o results/multiqc/ > {log} 2>&1"
