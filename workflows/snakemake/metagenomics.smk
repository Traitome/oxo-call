# =============================================================================
# Shotgun metagenomics workflow — fastp QC → host removal → Kraken2 → Bracken
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   host_genome_index:  /path/to/bowtie2_host_index
#   kraken2_db:         /path/to/kraken2_db/
#   bracken_db:         /path/to/kraken2_db/   (same dir as kraken2_db)
#   bracken_read_len:   150
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/bracken/{sample}.bracken", sample=SAMPLES),
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
        "--length_required 50 "
        "> {log} 2>&1"


# ── Host read removal ─────────────────────────────────────────────────────────
rule remove_host:
    input:
        r1    = "results/trimmed/{sample}_R1.fastq.gz",
        r2    = "results/trimmed/{sample}_R2.fastq.gz",
        index = config["host_genome_index"],
    output:
        r1 = "results/dehost/{sample}_R1.fastq.gz",
        r2 = "results/dehost/{sample}_R2.fastq.gz",
    threads: config.get("threads", 8)
    log: "logs/dehost/{sample}.log"
    shell:
        "bowtie2 "
        "-x {input.index} "
        "-1 {input.r1} -2 {input.r2} "
        "-p {threads} "
        "--no-unal "
        "--un-conc-gz results/dehost/{wildcards.sample}_R%.fastq.gz "
        "-S /dev/null "
        "> {log} 2>&1"


# ── Kraken2 taxonomic classification ─────────────────────────────────────────
rule kraken2_classify:
    input:
        r1 = "results/dehost/{sample}_R1.fastq.gz",
        r2 = "results/dehost/{sample}_R2.fastq.gz",
        db = config["kraken2_db"],
    output:
        report = "results/kraken2/{sample}.report",
        out    = "results/kraken2/{sample}.kraken2.out",
    threads: config.get("threads", 8)
    log: "logs/kraken2/{sample}.log"
    shell:
        "kraken2 "
        "--db {input.db} "
        "--threads {threads} "
        "--paired "
        "--gzip-compressed "
        "--report {output.report} "
        "--output {output.out} "
        "{input.r1} {input.r2} "
        "> {log} 2>&1"


# ── Bracken abundance estimation ──────────────────────────────────────────────
rule bracken_abundance:
    input:
        report = "results/kraken2/{sample}.report",
        db     = config["bracken_db"],
    output:
        bracken = "results/bracken/{sample}.bracken",
        report  = "results/bracken/{sample}.bracken_report",
    params:
        read_len = config.get("bracken_read_len", 150),
    log: "logs/bracken/{sample}.log"
    shell:
        "bracken "
        "-d {input.db} "
        "-i {input.report} "
        "-o {output.bracken} "
        "-w {output.report} "
        "-r {params.read_len} "
        "-l S "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        expand("results/kraken2/{sample}.report", sample=SAMPLES),
        expand("results/bracken/{sample}.bracken", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ results/kraken2/ results/bracken/ -o results/multiqc/ --force > {log} 2>&1"
