#!/usr/bin/env nextflow
// =============================================================================
// Single-cell RNA-seq workflow — fastp QC → STARsolo (10x Chromium v3) → cell QC
//
// Usage:
//   nextflow run scrnaseq.nf --samplesheet samplesheet.csv \
//                             --star_index /path/to/star_index \
//                             --star_whitelist /path/to/3M-february-2018.txt
//
// Samplesheet CSV format (with header row):
//   sample_id,r1,r2
//   sample1,/path/R1.fastq.gz,/path/R2.fastq.gz
//
// Notes:
//   R1 = cell barcode + UMI (28 bp for 10x v3)
//   R2 = cDNA read
// =============================================================================

nextflow.enable.dsl = 2

params.samplesheet    = "samplesheet.csv"
params.star_index     = null
params.star_whitelist = null
params.outdir         = "results"
params.threads        = 16


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
    // For 10x data: do NOT trim R1 (barcode+UMI), only pass-through
    """
    fastp \\
        --in1 ${r1} --in2 ${r2} \\
        --out1 ${sample_id}_R1.fastq.gz --out2 ${sample_id}_R2.fastq.gz \\
        --html ${sample_id}_fastp.html --json ${sample_id}_fastp.json \\
        --thread ${params.threads} \\
        --disable_adapter_trimming \\
        --disable_quality_filtering \\
        --disable_length_filtering
    """
}

process STARSOLO {
    tag "${sample_id}"
    publishDir "${params.outdir}/starsolo", mode: 'copy'

    input:
    tuple val(sample_id), path(r1), path(r2)

    output:
    tuple val(sample_id), path("${sample_id}/Aligned.sortedByCoord.out.bam"), emit: bam
    tuple val(sample_id), path("${sample_id}/Solo.out/Gene/filtered/matrix.mtx.gz"), emit: matrix

    script:
    """
    STAR \\
        --soloType CB_UMI_Simple \\
        --soloCBwhitelist ${params.star_whitelist} \\
        --soloCBstart 1 --soloCBlen 16 \\
        --soloUMIstart 17 --soloUMIlen 12 \\
        --genomeDir ${params.star_index} \\
        --readFilesIn ${r2} ${r1} \\
        --readFilesCommand zcat \\
        --outSAMtype BAM SortedByCoordinate \\
        --outSAMattributes NH HI nM AS CR UR CB UB GX GN sS sQ sM \\
        --outFileNamePrefix ${sample_id}/ \\
        --runThreadN ${params.threads} \\
        --soloCellFilter EmptyDrops_CR \\
        --soloFeatures Gene GeneFull \\
        --outSAMunmapped Within
    """
}

process CELL_QC {
    tag "${sample_id}"
    publishDir "${params.outdir}/qc", mode: 'copy'

    input:
    tuple val(sample_id), path(matrix)

    output:
    path "${sample_id}_cell_stats.json"

    script:
    """
    python3 -c "
import scipy.io, numpy as np, json, os, pathlib
d = pathlib.Path('${matrix}').parent
mat = scipy.io.mmread(str(d) + '/matrix.mtx.gz').toarray()
s = {
    'sample': '${sample_id}',
    'n_cells': int(mat.shape[1]),
    'median_umi': float(np.median(mat.sum(axis=0))),
    'median_genes': float(np.median((mat > 0).sum(axis=0)))
}
print(json.dumps(s, indent=2))
open('${sample_id}_cell_stats.json', 'w').write(json.dumps(s, indent=2))
"
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
    STARSOLO(FASTP.out.trimmed)
    CELL_QC(STARSOLO.out.matrix)

    qc_files = FASTP.out.json.collect()
    MULTIQC(qc_files)
}

/*
// nextflow.config:
process {
    cpus   = 16
    memory = '64 GB'
    time   = '8h'
}
executor {
    name      = 'local'
    cpus      = 64
    memory    = '256 GB'
}
*/
