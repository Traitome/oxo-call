#!/usr/bin/env nextflow
// =============================================================================
// Shotgun metagenomics — fastp QC → host removal → Kraken2 → Bracken
//
// Usage:
//   nextflow run metagenomics.nf --samplesheet samplesheet.csv \
//                                --host_index /path/to/bt2_host_index \
//                                --kraken2_db /path/to/kraken2_db/ \
//                                -profile standard
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet    = "samplesheet.csv"
params.host_index     = null
params.kraken2_db     = null
params.bracken_db     = null   // defaults to kraken2_db
params.bracken_rlen   = 150
params.outdir         = "results"
params.threads        = 8


// ── Channel setup ──────────────────────────────────────────────────────────
Channel
    .fromPath(params.samplesheet)
    .splitCsv(header: true)
    .map { row -> tuple(row.sample_id, file(row.r1), file(row.r2)) }
    .set { reads_ch }


// ── Processes ─────────────────────────────────────────────────────────────

process FASTP {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}_R1.fastq.gz"), path("${sample_id}_R2.fastq.gz"), emit: trimmed
    path "${sample_id}_fastp.json", emit: json

    script:
    """
    fastp \\
        --in1 ${r1} --in2 ${r2} \\
        --out1 ${sample_id}_R1.fastq.gz --out2 ${sample_id}_R2.fastq.gz \\
        --html ${sample_id}_fastp.html --json ${sample_id}_fastp.json \\
        --thread ${params.threads} \\
        --detect_adapter_for_pe \\
        --qualified_quality_phred 20 \\
        --length_required 50
    """
}

process REMOVE_HOST {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(r1), path(r2)
    val host_index

    output:
    tuple val(sample_id), path("${sample_id}_dehost_R1.fastq.gz"), path("${sample_id}_dehost_R2.fastq.gz")

    script:
    """
    bowtie2 \\
        -x ${host_index} \\
        -1 ${r1} -2 ${r2} \\
        -p ${params.threads} \\
        --no-unal \\
        --un-conc-gz ${sample_id}_dehost_R%.fastq.gz \\
        -S /dev/null
    """
}

process KRAKEN2 {
    tag "${sample_id}"
    publishDir "${params.outdir}/kraken2", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)
    path kraken2_db

    output:
    tuple val(sample_id), path("${sample_id}.report"), emit: report
    path "${sample_id}.kraken2.out"

    script:
    """
    kraken2 \\
        --db ${kraken2_db} \\
        --threads ${params.threads} \\
        --paired \\
        --gzip-compressed \\
        --report ${sample_id}.report \\
        --output ${sample_id}.kraken2.out \\
        ${r1} ${r2}
    """
}

process BRACKEN {
    tag "${sample_id}"
    publishDir "${params.outdir}/bracken", mode: 'copy'

    input:
    tuple val(sample_id), path(report)
    path bracken_db

    output:
    path "${sample_id}.bracken"
    path "${sample_id}.bracken_report"

    script:
    def db = params.bracken_db ?: params.kraken2_db
    """
    bracken \\
        -d ${bracken_db} \\
        -i ${report} \\
        -o ${sample_id}.bracken \\
        -w ${sample_id}.bracken_report \\
        -r ${params.bracken_rlen} \\
        -l S
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
    kraken2_db_ch  = file(params.kraken2_db)
    bracken_db_ch  = params.bracken_db ? file(params.bracken_db) : kraken2_db_ch

    FASTP(reads_ch)
    REMOVE_HOST(FASTP.out.trimmed, params.host_index)
    KRAKEN2(REMOVE_HOST.out, kraken2_db_ch)
    BRACKEN(KRAKEN2.out.report, bracken_db_ch)

    qc_files = FASTP.out.json
        .mix(KRAKEN2.out.report)
        .mix(BRACKEN.out[0])
        .collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config:
process {
    cpus   = 8
    memory = '32 GB'
    time   = '4h'
}
*/
