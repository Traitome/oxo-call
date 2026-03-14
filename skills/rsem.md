---
name: rsem
category: rna-seq
description: RNA-seq expression estimation using expectation-maximization for isoform quantification with uncertainty
tags: [rna-seq, quantification, isoform, expression, tpm, fpkm, em]
author: oxo-call built-in
source_url: "https://deweylab.github.io/RSEM/"
---

## Concepts

- RSEM uses EM algorithm to estimate expression at transcript and gene level, handling multi-mapping reads probabilistically.
- Two-step workflow: (1) rsem-prepare-reference to build index; (2) rsem-calculate-expression to quantify.
- rsem-prepare-reference requires transcriptome FASTA or genome+GTF: --gtf genes.gtf genome.fa index_prefix.
- RSEM aligns internally using Bowtie2 (default) or STAR (--star flag) for the alignment step.
- Output files: <prefix>.genes.results and <prefix>.isoforms.results with TPM, FPKM, expected_count columns.
- Use --num-threads N for multi-threading; --paired-end for paired-end data.
- Use --estimate-rspd for read start position distribution correction (improves accuracy for non-uniform coverage).
- rsem-generate-data-matrix converts multiple RSEM results to a count matrix for DESeq2/edgeR.

## Pitfalls

- The index_prefix must match between rsem-prepare-reference and rsem-calculate-expression.
- Without --paired-end flag, RSEM treats paired-end data as single-end, halving effective read count.
- RSEM is slower than Salmon/kallisto because it performs alignment internally.
- The expected_count column (not TPM/FPKM) should be used for DESeq2/edgeR — these tools need raw counts.
- When using --star, RSEM manages STAR internally — do NOT pre-align with STAR separately.
- --strandedness forward/reverse/none must match the library prep for accurate quantification.

## Examples

### prepare RSEM reference from genome FASTA and GTF annotation
**Args:** `--gtf genes.gtf --num-threads 8 genome.fa rsem_index/genome`
**Explanation:** rsem-prepare-reference command; --gtf extracts transcripts from genome; index prefix rsem_index/genome

### quantify paired-end RNA-seq reads using RSEM with Bowtie2
**Args:** `--paired-end --num-threads 8 --strandedness reverse R1.fastq.gz R2.fastq.gz rsem_index/genome sample_output`
**Explanation:** rsem-calculate-expression; --paired-end; --strandedness reverse for dUTP libraries; creates sample_output.genes.results

### quantify RNA-seq using RSEM with STAR aligner
**Args:** `--paired-end --star --num-threads 8 R1.fastq.gz R2.fastq.gz rsem_index/genome sample_output`
**Explanation:** rsem-calculate-expression --star uses STAR instead of Bowtie2 for alignment; RSEM manages STAR internally

### prepare RSEM reference directly from transcriptome FASTA
**Args:** `--num-threads 4 transcriptome.fa rsem_transcript_index/transcripts`
**Explanation:** rsem-prepare-reference from transcriptome FASTA; simpler than genome+GTF approach

### generate count matrix from multiple RSEM results files for DESeq2
**Args:** `sample1.genes.results sample2.genes.results sample3.genes.results > gene_count_matrix.txt`
**Explanation:** rsem-generate-data-matrix combines expected_count columns; output is a matrix for DESeq2
