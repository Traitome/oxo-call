# =============================================================================
# 16S rRNA amplicon sequencing workflow — cutadapt primer trim → fastp → DADA2
#
# Usage:
#   snakemake --cores 16 --use-conda
#
# Required config (config.yaml):
#   samples: [sample1, sample2, ...]
#   units:
#     sample1: {r1: path/to/R1.fastq.gz, r2: path/to/R2.fastq.gz}
#   primer_f:      GTGYCAGCMGCCGCGGTAA   # 515F
#   primer_r:      GGACTACNVGGGTWTCTAAT  # 806R
#   silva_db:      /path/to/silva_nr99_v138.1_train_set.fa.gz
#   threads: 8
# =============================================================================

configfile: "config.yaml"

SAMPLES = config["samples"]

rule all:
    input:
        "results/dada2/asv_table.csv",
        "results/dada2/taxonomy_table.csv",
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        "results/multiqc/multiqc_report.html",


# ── Primer removal with cutadapt ──────────────────────────────────────────────
rule cutadapt_primers:
    input:
        r1 = lambda wc: config["units"][wc.sample]["r1"],
        r2 = lambda wc: config["units"][wc.sample]["r2"],
    output:
        r1  = "results/trimmed/{sample}_R1.fastq.gz",
        r2  = "results/trimmed/{sample}_R2.fastq.gz",
        log = "results/qc/{sample}_cutadapt.txt",
    threads: config.get("threads", 8)
    params:
        primer_f = config.get("primer_f", "GTGYCAGCMGCCGCGGTAA"),
        primer_r = config.get("primer_r", "GGACTACNVGGGTWTCTAAT"),
    log: "logs/cutadapt/{sample}.log"
    shell:
        "cutadapt "
        "-g {params.primer_f} -G {params.primer_r} "
        "--discard-untrimmed "
        "-j {threads} "
        "--minimum-length 100 "
        "-o {output.r1} -p {output.r2} "
        "{input.r1} {input.r2} "
        "> {output.log} 2>{log}"


# ── Quality filtering with fastp ──────────────────────────────────────────────
rule fastp_qc:
    input:
        r1 = "results/trimmed/{sample}_R1.fastq.gz",
        r2 = "results/trimmed/{sample}_R2.fastq.gz",
    output:
        r1   = "results/filtered/{sample}_R1.fastq.gz",
        r2   = "results/filtered/{sample}_R2.fastq.gz",
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
        "--qualified_quality_phred 20 "
        "--length_required 100 "
        "> {log} 2>&1"


# ── DADA2 ASV inference (runs once on all samples together) ───────────────────
rule dada2_denoise:
    input:
        expand("results/filtered/{sample}_R1.fastq.gz", sample=SAMPLES),
        expand("results/filtered/{sample}_R2.fastq.gz", sample=SAMPLES),
        silva_db = config["silva_db"],
    output:
        asv_table  = "results/dada2/asv_table.csv",
        tax_table  = "results/dada2/taxonomy_table.csv",
        seqtab_rds = "results/dada2/seqtab_nochim.rds",
    threads: config.get("threads", 8)
    log: "logs/dada2/dada2.log"
    shell:
        "Rscript - <<'REOF'\n"
        "library(dada2)\n"
        "path_filt <- 'results/filtered'\n"
        "samples   <- list.files(path_filt, pattern='_R1.fastq.gz', full.names=FALSE)\n"
        "samples   <- sub('_R1.fastq.gz', '', samples)\n"
        "fns_f <- file.path(path_filt, paste0(samples, '_R1.fastq.gz'))\n"
        "fns_r <- file.path(path_filt, paste0(samples, '_R2.fastq.gz'))\n"
        "names(fns_f) <- samples\n"
        "names(fns_r) <- samples\n"
        "err_f <- learnErrors(fns_f, multithread=TRUE)\n"
        "err_r <- learnErrors(fns_r, multithread=TRUE)\n"
        "dadaFs  <- dada(fns_f, err=err_f, multithread=TRUE)\n"
        "dadaRs  <- dada(fns_r, err=err_r, multithread=TRUE)\n"
        "mergers <- mergePairs(dadaFs, fns_f, dadaRs, fns_r, verbose=TRUE)\n"
        "seqtab  <- makeSequenceTable(mergers)\n"
        "seqtab_nochim <- removeBimeraDenovo(seqtab, method='consensus', multithread=TRUE)\n"
        "taxa <- assignTaxonomy(seqtab_nochim, '{input.silva_db}', multithread=TRUE)\n"
        "dir.create('results/dada2', showWarnings=FALSE)\n"
        "saveRDS(seqtab_nochim, '{output.seqtab_rds}')\n"
        "write.csv(t(seqtab_nochim), '{output.asv_table}')\n"
        "write.csv(taxa,             '{output.tax_table}')\n"
        "REOF\n"
        "> {log} 2>&1"


# ── MultiQC aggregated report ─────────────────────────────────────────────────
rule multiqc:
    input:
        expand("results/qc/{sample}_fastp.json", sample=SAMPLES),
        expand("results/qc/{sample}_cutadapt.txt", sample=SAMPLES),
        "results/dada2/asv_table.csv",
    output:
        "results/multiqc/multiqc_report.html",
    log: "logs/multiqc.log"
    shell:
        "multiqc results/qc/ results/dada2/ -o results/multiqc/ --force > {log} 2>&1"
