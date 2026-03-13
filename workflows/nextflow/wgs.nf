#!/usr/bin/env nextflow
// =============================================================================
// WGS workflow — fastp QC → BWA-MEM2 alignment → GATK HaplotypeCaller
//
// Usage:
//   nextflow run wgs.nf --samplesheet samplesheet.csv \
//                       --genome /path/to/reference.fa \
//                       --dbsnp /path/to/dbsnp.vcf.gz \
//                       --known_indels /path/to/known_indels.vcf.gz \
//                       -profile standard
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet   = "samplesheet.csv"
params.genome        = null
params.dbsnp         = null
params.known_indels  = null
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
        --qualified_quality_phred 20
    """
}

process BWA_MEM2 {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(r1), path(r2)
    path genome

    output:
    tuple val(sample_id), path("${sample_id}.unsorted.bam")

    script:
    """
    bwa-mem2 mem \\
        -t ${params.threads} \\
        -R '@RG\\tID:${sample_id}\\tSM:${sample_id}\\tLB:${sample_id}\\tPL:ILLUMINA' \\
        ${genome} ${r1} ${r2} \\
        | samtools view -bS - > ${sample_id}.unsorted.bam
    """
}

process SORT_BAM {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path("${sample_id}.sorted.bam")

    script:
    """
    samtools sort -@ ${params.threads} -o ${sample_id}.sorted.bam ${bam}
    """
}

process MARK_DUPLICATES {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy', pattern: "*.metrics"

    input:
    tuple val(sample_id), path(bam)

    output:
    tuple val(sample_id), path("${sample_id}.markdup.bam"), emit: bam
    path "${sample_id}.markdup.metrics", emit: metrics

    script:
    """
    gatk MarkDuplicates \\
        -I ${bam} \\
        -O ${sample_id}.markdup.bam \\
        -M ${sample_id}.markdup.metrics
    samtools index ${sample_id}.markdup.bam
    """
}

process BQSR {
    tag "${sample_id}"

    input:
    tuple val(sample_id), path(bam)
    path genome
    path dbsnp
    path known_indels

    output:
    tuple val(sample_id), path("${sample_id}.recal.bam")

    script:
    """
    gatk BaseRecalibrator \\
        -I ${bam} \\
        -R ${genome} \\
        --known-sites ${dbsnp} \\
        --known-sites ${known_indels} \\
        -O recal.table

    gatk ApplyBQSR \\
        -I ${bam} \\
        -R ${genome} \\
        --bqsr-recal-file recal.table \\
        -O ${sample_id}.recal.bam
    """
}

process HAPLOTYPE_CALLER {
    tag "${sample_id}"
    publishDir "${params.outdir}/vcf", mode: 'copy'

    input:
    tuple val(sample_id), path(bam)
    path genome

    output:
    path "${sample_id}.g.vcf.gz"

    script:
    """
    samtools index ${bam}
    gatk HaplotypeCaller \\
        -R ${genome} \\
        -I ${bam} \\
        -O ${sample_id}.g.vcf.gz \\
        -ERC GVCF \\
        --native-pair-hmm-threads ${params.threads}
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
    genome_ch       = file(params.genome)
    dbsnp_ch        = file(params.dbsnp)
    known_indels_ch = file(params.known_indels)

    FASTP(reads_ch)
    BWA_MEM2(FASTP.out.trimmed, genome_ch)
    SORT_BAM(BWA_MEM2.out)
    MARK_DUPLICATES(SORT_BAM.out)
    BQSR(MARK_DUPLICATES.out.bam, genome_ch, dbsnp_ch, known_indels_ch)
    HAPLOTYPE_CALLER(BQSR.out, genome_ch)

    qc_files = FASTP.out.json.collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config:
process {
    cpus   = 8
    memory = '64 GB'
    time   = '8h'
}
*/
