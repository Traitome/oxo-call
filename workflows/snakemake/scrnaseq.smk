# =============================================================================
# Single-cell RNA-seq workflow — fastp QC → STARsolo (10x v3) → cell QC metrics
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   star_index:     /path/to/star_index
#   star_whitelist: /path/to/3M-february-2018.txt
#   threads: 16
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        expand("results/starsolo/{sample}/Solo.out/Gene/filtered/matrix.mtx.gz", sample=SAMPLES),
        expand("results/qc/{sample}_cell_stats.json", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── fastp QC (minimal: preserve CB+UMI read length) ──────────────────────────
rule fastp_qc:
    input:
        r1 = lambda wc: config["units"][wc.sample]["r1"],
        r2 = lambda wc: config["units"][wc.sample]["r2"],
    output:
        r1   = "results/trimmed/{sample}_R1.fastq.gz",
        r2   = "results/trimmed/{sample}_R2.fastq.gz",
        html = "results/qc/{sample}_fastp.html",
        json = "results/qc/{sample}_fastp.json",
    threads: config.get("threads", 16)
    log: "logs/fastp/{sample}.log"
    shell:
        "fastp "
        "--in1 {input.r1} --in2 {input.r2} "
        "--out1 {output.r1} --out2 {output.r2} "
        "--html {output.html} --json {output.json} "
        "--thread {threads} "
        "--disable_adapter_trimming "
        "--disable_quality_filtering "
        "--disable_length_filtering "
        "> {log} 2>&1"


# ── STARsolo — alignment + UMI/barcode demultiplexing ─────────────────────────
rule starsolo:
    input:
        r1        = "results/trimmed/{sample}_R1.fastq.gz",
        r2        = "results/trimmed/{sample}_R2.fastq.gz",
        index     = config["star_index"],
        whitelist = config["star_whitelist"],
    output:
        bam    = "results/starsolo/{sample}/Aligned.sortedByCoord.out.bam",
        matrix = "results/starsolo/{sample}/Solo.out/Gene/filtered/matrix.mtx.gz",
    threads: config.get("threads", 16)
    log: "logs/starsolo/{sample}.log"
    shell:
        "STAR "
        "--soloType CB_UMI_Simple "
        "--soloCBwhitelist {input.whitelist} "
        "--soloCBstart 1 --soloCBlen 16 "
        "--soloUMIstart 17 --soloUMIlen 12 "
        "--genomeDir {input.index} "
        "--readFilesIn {input.r2} {input.r1} "
        "--readFilesCommand zcat "
        "--outSAMtype BAM SortedByCoordinate "
        "--outSAMattributes NH HI nM AS CR UR CB UB GX GN sS sQ sM "
        "--outFileNamePrefix results/starsolo/{wildcards.sample}/ "
        "--runThreadN {threads} "
        "--soloCellFilter EmptyDrops_CR "
        "--soloFeatures Gene GeneFull "
        "--outSAMunmapped Within "
        "> {log} 2>&1"


# ── Cell QC statistics ────────────────────────────────────────────────────────
rule cell_qc:
    input:
        matrix = "results/starsolo/{sample}/Solo.out/Gene/filtered/matrix.mtx.gz",
    output:
        stats = "results/qc/{sample}_cell_stats.json",
    log: "logs/cell_qc/{sample}.log"
    shell:
        "python3 -c \""
        "import scipy.io, numpy as np, json, os, gzip; "
        "d='results/starsolo/{wildcards.sample}/Solo.out/Gene/filtered'; "
        "mat=scipy.io.mmread(d+'/matrix.mtx.gz').toarray(); "
        "s={{'sample':'{wildcards.sample}','n_cells':int(mat.shape[1]),"
        "'median_umi':float(np.median(mat.sum(axis=0))),"
        "'median_genes':float(np.median((mat>0).sum(axis=0)))}}; "
        "open('{output.stats}','w').write(json.dumps(s,indent=2)) "
        "\" > {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ -o results/multiqc/ --force > {log} 2>&1"
