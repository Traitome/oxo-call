#!/usr/bin/env nextflow
// =============================================================================
// WGBS methylation workflow — Trim Galore QC → Bismark alignment → dedup → methylation extraction
//
// Usage:
//   nextflow run methylseq.nf --samplesheet samplesheet.csv \
//                              --bismark_index /path/to/bismark_genome
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet  = "samplesheet.csv"
params.bismark_index = null
params.outdir       = "results"
params.threads      = 8


// ── Channel setup ──────────────────────────────────────────────────────────
Channel
    .fromPath(params.samplesheet)
    .splitCsv(header: true)
    .map { row -> tuple(row.sample_id, file(row.r1), file(row.r2)) }
    .set { reads_ch }


// ── Processes ─────────────────────────────────────────────────────────────

process TRIM_GALORE {
    tag "${sample_id}"
    publishDir "${params.outdir}/trimmed", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}_R1_val_1.fq.gz"), path("${sample_id}_R2_val_2.fq.gz"), emit: trimmed
    path "${sample_id}_R1_val_1_fastqc.zip", emit: fastqc

    script:
    """
    trim_galore \\
        --paired --cores ${params.threads} \\
        --fastqc --illumina \\
        --output_dir . \\
        ${r1} ${r2}
    mv \$(basename ${r1} .fastq.gz)_val_1.fq.gz ${sample_id}_R1_val_1.fq.gz
    mv \$(basename ${r2} .fastq.gz)_val_2.fq.gz ${sample_id}_R2_val_2.fq.gz
    """
}

process BISMARK_ALIGN {
    tag "${sample_id}"
    publishDir "${params.outdir}/aligned", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}_bismark_bt2_pe.bam"), emit: bam
    path "${sample_id}_bismark_bt2_PE_report.txt", emit: report

    script:
    """
    bismark \\
        --genome ${params.bismark_index} \\
        -1 ${r1} -2 ${r2} \\
        --output_dir . \\
        --prefix ${sample_id} \\
        --parallel ${params.threads} \\
        --non_directional
    """
}

process BISMARK_DEDUP {
    tag "${sample_id}"
    publishDir "${params.outdir}/deduped", mode: 'copy'

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path("${sample_id}_bismark_bt2_pe.deduplicated.bam"), emit: bam
    path "${sample_id}_bismark_bt2_pe.deduplication_report.txt", emit: report

    script:
    """
    deduplicate_bismark \\
        --paired \\
        --output_dir . \\
        ${bam}
    """
}

process SORT_INDEX {
    tag "${sample_id}"
    publishDir "${params.outdir}/deduped", mode: 'copy'

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path("${sample_id}.sorted.bam"), path("${sample_id}.sorted.bam.bai"), emit: bam

    script:
    """
    samtools sort -@ ${params.threads} -o ${sample_id}.sorted.bam ${bam}
    samtools index ${sample_id}.sorted.bam
    """
}

process METHYLATION_EXTRACT {
    tag "${sample_id}"
    publishDir "${params.outdir}/methyl", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)

    output:
    path "${sample_id}.sorted_CpG.bedGraph", emit: bedgraph
    path "${sample_id}.sorted.bismark.cov.gz"
    path "${sample_id}.sorted_splitting_report.txt", emit: report

    script:
    """
    bismark_methylation_extractor \\
        --paired-end --comprehensive --CX_context \\
        --cytosine_report \\
        --genome_folder ${params.bismark_index} \\
        --output . \\
        --parallel ${params.threads} \\
        ${bam}
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
    TRIM_GALORE(reads_ch)
    BISMARK_ALIGN(TRIM_GALORE.out.trimmed)
    BISMARK_DEDUP(BISMARK_ALIGN.out.bam)
    SORT_INDEX(BISMARK_DEDUP.out.bam)
    METHYLATION_EXTRACT(SORT_INDEX.out.bam)

    // MultiQC across all reports
    all_reports = TRIM_GALORE.out.fastqc
        .mix(BISMARK_ALIGN.out.report)
        .mix(BISMARK_DEDUP.out.report)
        .mix(METHYLATION_EXTRACT.out.report)
        .collect()
    MULTIQC(all_reports)
}

/*
// nextflow.config:
process {
    cpus   = 8
    memory = '64 GB'
    time   = '8h'
}
executor {
    name      = 'local'
    cpus      = 32
    memory    = '256 GB'
}
*/
