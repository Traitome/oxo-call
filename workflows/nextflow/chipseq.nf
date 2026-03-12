#!/usr/bin/env nextflow
// =============================================================================
// ChIP-seq workflow — fastp QC → Bowtie2 → Picard MarkDup → filter → MACS3 → bigWig
//
// Usage:
//   nextflow run chipseq.nf --samplesheet samplesheet.csv \
//                           --bowtie2_index /path/to/bt2_index \
//                           --blacklist /path/to/blacklist.bed \
//                           --genome_size hs
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   H3K27ac_rep1,/path/R1.fastq.gz,/path/R2.fastq.gz
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet  = "samplesheet.csv"
params.bowtie2_index = null
params.blacklist    = null
params.genome_size  = "hs"
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
        --length_required 20
    """
}

process BOWTIE2_ALIGN {
    tag "${sample_id}"
    publishDir "${params.outdir}/aligned", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}.sorted.bam"), path("${sample_id}.sorted.bam.bai"), emit: bam

    script:
    """
    bowtie2 \\
        -x ${params.bowtie2_index} \\
        -1 ${r1} -2 ${r2} \\
        -p ${params.threads} \\
        --no-mixed --no-discordant \\
        | samtools sort -@ 4 -o ${sample_id}.sorted.bam
    samtools index ${sample_id}.sorted.bam
    """
}

process MARK_DUPLICATES {
    tag "${sample_id}"
    publishDir "${params.outdir}/aligned", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)

    output:
    tuple val(sample_id), path("${sample_id}.markdup.bam"), path("${sample_id}.markdup.bam.bai"), emit: bam
    path "${sample_id}.markdup_metrics.txt", emit: metrics

    script:
    """
    picard MarkDuplicates \\
        I=${bam} O=${sample_id}.markdup.bam \\
        M=${sample_id}.markdup_metrics.txt \\
        REMOVE_DUPLICATES=true
    samtools index ${sample_id}.markdup.bam
    """
}

process FILTER_READS {
    tag "${sample_id}"
    publishDir "${params.outdir}/aligned", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)

    output:
    tuple val(sample_id), path("${sample_id}.filtered.bam"), path("${sample_id}.filtered.bam.bai"), emit: bam

    script:
    """
    samtools view -@ ${params.threads} -b -F 1804 -f 2 -q 30 ${bam} \\
        | bedtools intersect -v -abam stdin -b ${params.blacklist} \\
        > ${sample_id}.filtered.bam
    samtools index ${sample_id}.filtered.bam
    """
}

process MACS3_CALLPEAK {
    tag "${sample_id}"
    publishDir "${params.outdir}/peaks", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)

    output:
    path "${sample_id}_peaks.narrowPeak"
    path "${sample_id}_summits.bed"

    script:
    """
    macs3 callpeak \\
        -t ${bam} -f BAMPE \\
        -n ${sample_id} \\
        --outdir . \\
        -g ${params.genome_size} \\
        -B --SPMR --keep-dup all --call-summits
    """
}

process BAMCOVERAGE {
    tag "${sample_id}"
    publishDir "${params.outdir}/bigwig", mode: 'copy'

    input:
    tuple val(sample_id), path(bam), path(bai)

    output:
    path "${sample_id}.bw"

    script:
    """
    bamCoverage \\
        -b ${bam} -o ${sample_id}.bw \\
        --binSize 10 --normalizeUsing RPKM \\
        --ignoreDuplicates \\
        -p ${params.threads}
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
    FASTP(reads_ch)
    BOWTIE2_ALIGN(FASTP.out.trimmed)
    MARK_DUPLICATES(BOWTIE2_ALIGN.out.bam)
    FILTER_READS(MARK_DUPLICATES.out.bam)
    MACS3_CALLPEAK(FILTER_READS.out.bam)
    BAMCOVERAGE(FILTER_READS.out.bam)

    // Collect all QC files for MultiQC
    qc_files = FASTP.out.json.mix(MARK_DUPLICATES.out.metrics).collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config (save alongside this .nf file):
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
