#!/usr/bin/env nextflow
// =============================================================================
// Long-read genome assembly workflow — NanoQ QC → Flye assembly → Medaka polishing → QUAST
//
// Usage:
//   nextflow run longreads.nf --samplesheet samplesheet.csv \
//                              --genome_size 5m \
//                              --read_type nano-hq \
//                              --medaka_model r941_min_sup_g507
//
// Samplesheet CSV format (with header row):
//   sample_id,reads
//   sample1,/path/reads.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet   = "samplesheet.csv"
params.genome_size   = "5m"
params.read_type     = "nano-hq"
params.medaka_model  = "r941_min_sup_g507"
params.outdir        = "results"
params.threads       = 16


// ── Channel setup ──────────────────────────────────────────────────────────
Channel
    .fromPath(params.samplesheet)
    .splitCsv(header: true)
    .map { row -> tuple(row.sample_id, file(row.reads)) }
    .set { reads_ch }


// ── Processes ─────────────────────────────────────────────────────────────

process NANOQ_FILTER {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy', pattern: "*.{txt,json}"

    input:
    tuple val(sample_id), path(reads)

    output:
    tuple val(sample_id), path("${sample_id}.filtered.fastq.gz"), emit: filtered
    path "${sample_id}_nanoq_report.txt"
    path "${sample_id}_nanoq_stats.json", emit: stats

    script:
    """
    nanoq \\
        -i ${reads} \\
        -r ${sample_id}_nanoq_report.txt \\
        -s ${sample_id}_nanoq_stats.json \\
        --min-len 1000 \\
        --min-qual 8 \\
        -o ${sample_id}.filtered.fastq.gz
    """
}

process NANOSTAT {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy'

    input:
    tuple val(sample_id), path(reads)

    output:
    path "${sample_id}NanoStats.txt", emit: stats

    script:
    """
    NanoStat \\
        --fastq ${reads} \\
        --threads ${params.threads} \\
        -n ${sample_id} \\
        -o . \\
        --tsv
    """
}

process FLYE_ASSEMBLE {
    tag "${sample_id}"
    publishDir "${params.outdir}/assembly", mode: 'copy'

    input:
    tuple val(sample_id), path(reads)

    output:
    tuple val(sample_id), path("${sample_id}/assembly.fasta"), emit: assembly
    path "${sample_id}/assembly_info.txt"

    script:
    """
    flye \\
        --${params.read_type} ${reads} \\
        --genome-size ${params.genome_size} \\
        --out-dir ${sample_id} \\
        --threads ${params.threads}
    """
}

process MEDAKA_POLISH {
    tag "${sample_id}"
    publishDir "${params.outdir}/polished", mode: 'copy'

    input:
    tuple val(sample_id), path(reads), path(assembly)

    output:
    tuple val(sample_id), path("${sample_id}/consensus.fasta"), emit: consensus

    script:
    """
    medaka_consensus \\
        -i ${reads} \\
        -d ${assembly} \\
        -o ${sample_id} \\
        -t ${params.threads} \\
        -m ${params.medaka_model}
    """
}

process QUAST_EVALUATE {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy'

    input:
    tuple val(sample_id), path(assembly)

    output:
    path "quast_${sample_id}/report.txt", emit: report
    path "quast_${sample_id}/report.html"

    script:
    """
    quast.py \\
        ${assembly} \\
        -o quast_${sample_id} \\
        --threads ${params.threads} \\
        --min-contig 500
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
    NANOQ_FILTER(reads_ch)
    NANOSTAT(NANOQ_FILTER.out.filtered)
    FLYE_ASSEMBLE(NANOQ_FILTER.out.filtered)

    // Join filtered reads + assembly for Medaka polishing
    polish_input = NANOQ_FILTER.out.filtered.join(FLYE_ASSEMBLE.out.assembly)
    MEDAKA_POLISH(polish_input)
    QUAST_EVALUATE(MEDAKA_POLISH.out.consensus)

    qc_files = NANOSTAT.out.stats.mix(QUAST_EVALUATE.out.report).collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config:
process {
    cpus   = 16
    memory = '64 GB'
    time   = '24h'
}
executor {
    name      = 'local'
    cpus      = 64
    memory    = '256 GB'
}
*/
