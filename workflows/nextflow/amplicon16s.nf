#!/usr/bin/env nextflow
// =============================================================================
// 16S amplicon sequencing workflow — cutadapt primer trim → fastp QC → DADA2
//
// Usage:
//   nextflow run amplicon16s.nf --samplesheet samplesheet.csv \
//                                --silva_db /path/to/silva_nr99_v138.1_train_set.fa.gz \
//                                --primer_f GTGYCAGCMGCCGCGGTAA \
//                                --primer_r GGACTACNVGGGTWTCTAAT
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet = "samplesheet.csv"
params.silva_db    = null
params.primer_f    = "GTGYCAGCMGCCGCGGTAA"    // 515F (V4)
params.primer_r    = "GGACTACNVGGGTWTCTAAT"   // 806R (V4)
params.outdir      = "results"
params.threads     = 8


// ── Channel setup ──────────────────────────────────────────────────────────
Channel
    .fromPath(params.samplesheet)
    .splitCsv(header: true)
    .map { row -> tuple(row.sample_id, file(row.r1), file(row.r2)) }
    .set { reads_ch }


// ── Processes ─────────────────────────────────────────────────────────────

process CUTADAPT {
    tag "${sample_id}"
    publishDir "${params.outdir}/trimmed", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}_R1.fastq.gz"), path("${sample_id}_R2.fastq.gz"), emit: trimmed
    path "${sample_id}_cutadapt.txt", emit: log

    script:
    """
    cutadapt \\
        -g ${params.primer_f} \\
        -G ${params.primer_r} \\
        --discard-untrimmed \\
        -j ${params.threads} \\
        --minimum-length 100 \\
        -o ${sample_id}_R1.fastq.gz \\
        -p ${sample_id}_R2.fastq.gz \\
        ${r1} ${r2} \\
        > ${sample_id}_cutadapt.txt
    """
}

process FASTP {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}_R1.fastq.gz"), path("${sample_id}_R2.fastq.gz"), emit: filtered
    path "${sample_id}_fastp.json", emit: json
    path "${sample_id}_fastp.html"

    script:
    """
    fastp \\
        --in1 ${r1} --in2 ${r2} \\
        --out1 ${sample_id}_R1.fastq.gz --out2 ${sample_id}_R2.fastq.gz \\
        --html ${sample_id}_fastp.html --json ${sample_id}_fastp.json \\
        --thread ${params.threads} \\
        --qualified_quality_phred 20 \\
        --length_required 100
    """
}

process DADA2 {
    publishDir "${params.outdir}/dada2", mode: 'copy'

    input:
    path "filtered/*"
    path silva_db

    output:
    path "asv_table.csv"
    path "taxonomy_table.csv"
    path "seqtab_nochim.rds"

    script:
    """
    Rscript - <<'REOF'
library(dada2)
path_filt <- "filtered"
samples   <- list.files(path_filt, pattern="_R1.fastq.gz", full.names=FALSE)
samples   <- sub("_R1.fastq.gz", "", samples)
fns_f <- file.path(path_filt, paste0(samples, "_R1.fastq.gz"))
fns_r <- file.path(path_filt, paste0(samples, "_R2.fastq.gz"))
names(fns_f) <- samples; names(fns_r) <- samples
err_f <- learnErrors(fns_f, multithread=TRUE)
err_r <- learnErrors(fns_r, multithread=TRUE)
dadaFs  <- dada(fns_f, err=err_f, multithread=TRUE)
dadaRs  <- dada(fns_r, err=err_r, multithread=TRUE)
mergers <- mergePairs(dadaFs, fns_f, dadaRs, fns_r, verbose=TRUE)
seqtab  <- makeSequenceTable(mergers)
seqtab_nochim <- removeBimeraDenovo(seqtab, method="consensus", multithread=TRUE)
taxa <- assignTaxonomy(seqtab_nochim, "${silva_db}", multithread=TRUE)
saveRDS(seqtab_nochim, "seqtab_nochim.rds")
write.csv(t(seqtab_nochim), "asv_table.csv")
write.csv(taxa,              "taxonomy_table.csv")
cat("ASVs:", ncol(seqtab_nochim), "\n")
REOF
    """
}

process MULTIQC {
    publishDir "${params.outdir}/multiqc", mode: 'copy'

    input:
    path "*"

    output:
    path "multiqc_report.html"

    script:
    """
    multiqc .
    """
}


// ── Workflow ──────────────────────────────────────────────────────────────

workflow {
    silva_ch = file(params.silva_db)

    CUTADAPT(reads_ch)
    FASTP(CUTADAPT.out.trimmed)

    // Collect all filtered FASTQ files for joint DADA2 analysis
    all_filtered = FASTP.out.filtered
        .map { sample_id, r1, r2 -> [r1, r2] }
        .collect()
        .flatten()
    DADA2(all_filtered, silva_ch)

    // QC aggregation — runs in parallel with DADA2
    qc_files = FASTP.out.json.mix(CUTADAPT.out.log).collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config:
process {
    cpus   = 8
    memory = '32 GB'
    time   = '4h'
}
executor {
    name      = 'local'
    cpus      = 32
    memory    = '128 GB'
}
*/
