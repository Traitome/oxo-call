#!/usr/bin/env nextflow
// =============================================================================
// ATAC-seq workflow — fastp QC → Bowtie2 alignment → MACS3 peak calling
//
// Usage:
//   nextflow run atacseq.nf --samplesheet samplesheet.csv \
//                           --bowtie2_index /path/to/bt2_index \
//                           --blacklist /path/to/blacklist.bed \
//                           -profile standard
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet   = "samplesheet.csv"
params.bowtie2_index = null
params.blacklist     = null
params.genome_size   = "hs"
params.outdir        = "results"
params.threads       = 8


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
        --length_required 20
    """
}

process BOWTIE2_ALIGN {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(r1), path(r2)
    val bt2_index

    output:
    tuple val(sample_id), path("${sample_id}.unsorted.bam")

    script:
    """
    bowtie2 \\
        -x ${bt2_index} \\
        -1 ${r1} -2 ${r2} \\
        -p ${params.threads} \\
        --no-mixed --no-discordant \\
        --no-unal \\
        | samtools view -bS - > ${sample_id}.unsorted.bam
    """
}

process SORT_AND_MARKDUP {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy', pattern: "*.txt"

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path("${sample_id}.markdup.bam"), emit: bam
    path "${sample_id}.markdup_metrics.txt", emit: metrics

    script:
    """
    samtools sort -@ ${params.threads} -o sorted.bam ${bam}
    picard MarkDuplicates \\
        I=sorted.bam \\
        O=${sample_id}.markdup.bam \\
        M=${sample_id}.markdup_metrics.txt \\
        REMOVE_DUPLICATES=false
    samtools index ${sample_id}.markdup.bam
    """
}

process FILTER_BLACKLIST {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(bam)
    path blacklist

    output:
    tuple val(sample_id), path("${sample_id}.filtered.bam")

    script:
    """
    samtools view -@ ${params.threads} -b -F 1804 -f 2 -q 30 ${bam} \\
        | bedtools intersect -v -abam stdin -b ${blacklist} \\
        > ${sample_id}.filtered.bam
    samtools index ${sample_id}.filtered.bam
    """
}

process MACS3_PEAKS {
    tag "${sample_id}"
    publishDir "${params.outdir}/peaks", mode: 'copy'

    input:
    tuple val(sample_id), path(bam)

    output:
    path "${sample_id}_peaks.narrowPeak"
    path "${sample_id}_summits.bed"

    script:
    """
    macs3 callpeak \\
        -t ${bam} \\
        -f BAMPE \\
        -n ${sample_id} \\
        --outdir . \\
        -g ${params.genome_size} \\
        --nomodel --shift -75 --extsize 150 \\
        -B --SPMR \\
        --keep-dup all
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
    blacklist_ch = file(params.blacklist)

    FASTP(reads_ch)
    BOWTIE2_ALIGN(FASTP.out.trimmed, params.bowtie2_index)
    SORT_AND_MARKDUP(BOWTIE2_ALIGN.out)
    FILTER_BLACKLIST(SORT_AND_MARKDUP.out.bam, blacklist_ch)
    MACS3_PEAKS(FILTER_BLACKLIST.out)

    qc_files = FASTP.out.json.mix(SORT_AND_MARKDUP.out.metrics).collect()
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
