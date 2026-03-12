# =============================================================================
# Long-read genome assembly workflow — NanoQ/NanoStat QC → Flye → Medaka → QUAST
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {reads: path/to/reads.fastq.gz}
#   genome_size:  5m       # estimated genome size
#   read_type:    nano-hq  # flye read type (nano-raw|nano-hq|pacbio-hifi|...)
#   medaka_model: r941_min_sup_g507
#   threads: 16
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/polished/{sample}/consensus.fasta", sample=SAMPLES),
        expand("results/qc/quast_{sample}/report.html", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── NanoQ read filtering ──────────────────────────────────────────────────────
rule nanoq_filter:
    input:
        reads = lambda wc: config["units"][wc.sample]["reads"],
    output:
        filtered = "results/filtered/{sample}.fastq.gz",
        report   = "results/qc/{sample}_nanoq_report.txt",
        stats    = "results/qc/{sample}_nanoq_stats.json",
    log: "logs/nanoq/{sample}.log"
    shell:
        "nanoq "
        "-i {input.reads} "
        "-r {output.report} "
        "-s {output.stats} "
        "--min-len 1000 "
        "--min-qual 8 "
        "-o {output.filtered} "
        "> {log} 2>&1"


# ── NanoStat summary statistics ───────────────────────────────────────────────
rule nanostat:
    input:
        reads = "results/filtered/{sample}.fastq.gz",
    output:
        stats = "results/qc/{sample}NanoStats.txt",
    threads: config.get("threads", 16)
    log: "logs/nanostat/{sample}.log"
    shell:
        "NanoStat "
        "--fastq {input.reads} "
        "--threads {threads} "
        "-n {wildcards.sample} "
        "-o results/qc/ "
        "--tsv "
        "> {log} 2>&1"


# ── Flye de novo assembly ─────────────────────────────────────────────────────
rule flye_assemble:
    input:
        reads = "results/filtered/{sample}.fastq.gz",
    output:
        assembly = "results/assembly/{sample}/assembly.fasta",
        info     = "results/assembly/{sample}/assembly_info.txt",
    threads: config.get("threads", 16)
    params:
        genome_size = config.get("genome_size", "5m"),
        read_type   = config.get("read_type", "nano-hq"),
    log: "logs/flye/{sample}.log"
    shell:
        "flye "
        "--{params.read_type} {input.reads} "
        "--genome-size {params.genome_size} "
        "--out-dir results/assembly/{wildcards.sample} "
        "--threads {threads} "
        "> {log} 2>&1"


# ── Medaka consensus polishing ────────────────────────────────────────────────
rule medaka_polish:
    input:
        reads    = "results/filtered/{sample}.fastq.gz",
        assembly = "results/assembly/{sample}/assembly.fasta",
    output:
        consensus = "results/polished/{sample}/consensus.fasta",
    threads: config.get("threads", 16)
    params:
        model = config.get("medaka_model", "r941_min_sup_g507"),
    log: "logs/medaka/{sample}.log"
    shell:
        "medaka_consensus "
        "-i {input.reads} "
        "-d {input.assembly} "
        "-o results/polished/{wildcards.sample} "
        "-t {threads} "
        "-m {params.model} "
        "> {log} 2>&1"


# ── QUAST assembly evaluation ─────────────────────────────────────────────────
rule quast_evaluate:
    input:
        assembly = "results/polished/{sample}/consensus.fasta",
    output:
        report_txt  = "results/qc/quast_{sample}/report.txt",
        report_html = "results/qc/quast_{sample}/report.html",
    threads: config.get("threads", 16)
    log: "logs/quast/{sample}.log"
    shell:
        "quast.py "
        "{input.assembly} "
        "-o results/qc/quast_{wildcards.sample} "
        "--threads {threads} "
        "--min-contig 500 "
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}NanoStats.txt", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ -o results/multiqc/ --force > {log} 2>&1"
