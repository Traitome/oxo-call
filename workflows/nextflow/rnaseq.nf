#!/usr/bin/env nextflow
// =============================================================================
// RNA-seq workflow — fastp QC → STAR alignment → featureCounts quantification
//
// Usage:
//   nextflow run rnaseq.nf --samplesheet samplesheet.csv \
//                          --genome /path/to/genome.fa \
//                          --star_index /path/to/star_index/ \
//                          --gtf /path/to/annotation.gtf \
//                          -profile standard
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet  = "samplesheet.csv"
params.genome       = null
params.star_index   = null
params.gtf          = null
params.outdir       = "results"
params.threads      = 8


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
    path "${sample_id}_fastp.html"

    script:
    """
    fastp \\
        --in1 ${r1} --in2 ${r2} \\
        --out1 ${sample_id}_R1.fastq.gz --out2 ${sample_id}_R2.fastq.gz \\
        --html ${sample_id}_fastp.html --json ${sample_id}_fastp.json \\
        --thread ${params.threads} \\
        --detect_adapter_for_pe \\
        --qualified_quality_phred 20 \\
        --length_required 30
    """
}

process STAR_ALIGN {
    tag "${sample_id}"
    publishDir "${params.outdir}/aligned", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)
    path star_index

    output:
    tuple val(sample_id), path("${sample_id}/Aligned.sortedByCoord.out.bam"), emit: bam
    path "${sample_id}/Log.final.out", emit: log

    script:
    """
    STAR \\
        --runMode alignReads \\
        --genomeDir ${star_index} \\
        --readFilesIn ${r1} ${r2} \\
        --readFilesCommand zcat \\
        --outSAMtype BAM SortedByCoordinate \\
        --outSAMattributes NH HI AS NM MD \\
        --outFileNamePrefix ${sample_id}/ \\
        --runThreadN ${params.threads} \\
        --outSAMstrandField intronMotif \\
        --outFilterIntronMotifs RemoveNoncanonical
    """
}

process SAMTOOLS_INDEX {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path(bam), path("${bam}.bai")

    script:
    """
    samtools index -@ ${params.threads} ${bam}
    """
}

process FEATURECOUNTS {
    tag "${sample_id}"
    publishDir "${params.outdir}/counts", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)
    path gtf

    output:
    path "${sample_id}_counts.txt"

    script:
    """
    featureCounts \\
        -T ${params.threads} \\
        -p \\
        -a ${gtf} \\
        -o ${sample_id}_counts.txt \\
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
    star_index_ch = file(params.star_index)
    gtf_ch        = file(params.gtf)

    FASTP(reads_ch)
    STAR_ALIGN(FASTP.out.trimmed, star_index_ch)
    SAMTOOLS_INDEX(STAR_ALIGN.out.bam)
    FEATURECOUNTS(SAMTOOLS_INDEX.out, gtf_ch)

    // Collect all QC files for MultiQC
    qc_files = FASTP.out.json.mix(STAR_ALIGN.out.log).collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config (save as nextflow.config in the same directory):
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
