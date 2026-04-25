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
- Two-step workflow: (1) rsem-prepare-reference to build index; (2) rsem-calculate-expression to quantify. Both are companion binaries.
- rsem-prepare-reference requires transcriptome FASTA or genome+GTF: --gtf genes.gtf genome.fa index_prefix.
- RSEM aligns internally using Bowtie2 (default) or STAR (--star flag) for the alignment step.
- Output files: <prefix>.genes.results and <prefix>.isoforms.results with TPM, FPKM, expected_count columns.
- Use --num-threads N for multi-threading; --paired-end for paired-end data.
- Use --estimate-rspd for read start position distribution correction (improves accuracy for non-uniform coverage).
- rsem-generate-data-matrix converts multiple RSEM results to a count matrix for DESeq2/edgeR.
- RSEM can quantify from pre-aligned BAM/SAM files using --alignments flag.
- --calc-ci computes 95% confidence intervals for expression estimates.
- --no-qualities allows processing FASTA format reads without quality scores.
- RSEM supports both Bowtie and Bowtie2 aligners (--bowtie or --bowtie2 flags).

## Pitfalls

- RSEM is a multi-binary suite. Each tool is a separate command: rsem-prepare-reference, rsem-calculate-expression, rsem-generate-data-matrix, rsem-generate-ngs-count-matrix. There is NO single 'rsem' subcommand pattern. ARGS for each tool start with their own flags.
- RSEM companion binaries (rsem-prepare-reference, rsem-calculate-expression, rsem-generate-data-matrix) must be used as the first ARGS token — the system detects them and uses them as the actual executables.
- The index_prefix must match between rsem-prepare-reference and rsem-calculate-expression.
- Without --paired-end flag, RSEM treats paired-end data as single-end, halving effective read count.
- RSEM is slower than Salmon/kallisto because it performs alignment internally.
- The expected_count column (not TPM/FPKM) should be used for DESeq2/edgeR — these tools need raw counts.
- When using --star, RSEM manages STAR internally — do NOT pre-align with STAR separately.
- --strandedness forward/reverse/none must match the library prep for accurate quantification.
- --alignments requires sorted BAM with adjacent paired-end mates; use samtools sort and fixmate first.
- --calc-ci significantly increases computation time; use only when confidence intervals are needed.
- Multiple input files should be comma-separated, not space-separated.

## Examples

### prepare RSEM reference from genome FASTA and GTF annotation
**Args:** `rsem-prepare-reference --gtf genes.gtf --num-threads 8 genome.fa rsem_index/genome`
**Explanation:** rsem-prepare-reference companion binary; --gtf genes.gtf extracts transcripts from genome; --num-threads 8 parallel processing; genome.fa input genome FASTA; rsem_index/genome output index prefix

### quantify paired-end RNA-seq reads using RSEM with Bowtie2
**Args:** `rsem-calculate-expression --paired-end --num-threads 8 --strandedness reverse R1.fastq.gz R2.fastq.gz rsem_index/genome sample_output`
**Explanation:** rsem-calculate-expression companion binary; --paired-end paired-end mode; --num-threads 8 parallel processing; --strandedness reverse for dUTP libraries; R1.fastq.gz R2.fastq.gz input FASTQs; rsem_index/genome index prefix; sample_output output prefix

### quantify RNA-seq using RSEM with STAR aligner
**Args:** `rsem-calculate-expression --paired-end --star --num-threads 8 R1.fastq.gz R2.fastq.gz rsem_index/genome sample_output`
**Explanation:** rsem-calculate-expression companion binary; --paired-end mode; --star uses STAR instead of Bowtie2 for alignment; --num-threads 8 parallel processing; R1.fastq.gz R2.fastq.gz input FASTQs; rsem_index/genome index prefix; sample_output output prefix; RSEM manages STAR internally

### prepare RSEM reference directly from transcriptome FASTA
**Args:** `rsem-prepare-reference --num-threads 4 transcriptome.fa rsem_transcript_index/transcripts`
**Explanation:** rsem-prepare-reference companion binary; --num-threads 4 parallel processing; transcriptome.fa input transcriptome FASTA; rsem_transcript_index/transcripts output index prefix; simpler than genome+GTF approach

### generate count matrix from multiple RSEM results files for DESeq2
**Args:** `rsem-generate-data-matrix sample1.genes.results sample2.genes.results sample3.genes.results > gene_count_matrix.txt`
**Explanation:** rsem-generate-data-matrix companion binary; sample1.genes.results sample2.genes.results sample3.genes.results input gene results files; > gene_count_matrix.txt output matrix; combines expected_count columns into matrix for DESeq2

### quantify single-end RNA-seq reads using RSEM
**Args:** `rsem-calculate-expression --num-threads 8 reads.fastq.gz rsem_index/genome sample_output`
**Explanation:** rsem-calculate-expression companion binary; --num-threads 8 parallel processing; reads.fastq.gz input single-end FASTQ; rsem_index/genome index prefix; sample_output output prefix; default uses Bowtie2; produces genes.results and isoforms.results

### prepare RSEM reference with Bowtie2 and poly-A trimming for scRNA-seq
**Args:** `rsem-prepare-reference --gtf genes.gtf --num-threads 8 --polyA genome.fa rsem_polyA_index/genome`
**Explanation:** rsem-prepare-reference companion binary; --gtf genes.gtf annotation file; --num-threads 8 parallel processing; --polyA adds poly-A tails to all transcripts; genome.fa input genome FASTA; rsem_polyA_index/genome output index prefix; useful for 3' end scRNA-seq quantification

### quantify with RSEM and estimate read start position distribution
**Args:** `rsem-calculate-expression --paired-end --num-threads 8 --estimate-rspd --strandedness none R1.fq.gz R2.fq.gz rsem_index/genome sample`
**Explanation:** rsem-calculate-expression companion binary; --paired-end mode; --num-threads 8 parallel processing; --estimate-rspd corrects for non-uniform read start positions; --strandedness none for unstranded libraries; R1.fq.gz R2.fq.gz input FASTQs; rsem_index/genome index prefix; sample output prefix

### extract TPM column from RSEM gene results for cross-sample comparison
**Args:** `rsem-generate-data-matrix sample1.genes.results sample2.genes.results sample3.genes.results > count_matrix.txt`
**Explanation:** rsem-generate-data-matrix companion binary; sample1.genes.results sample2.genes.results sample3.genes.results input gene results files; > count_matrix.txt output matrix; extracts expected_count columns; for TPM use 'cut -f1,6' on individual .genes.results files

### calculate expression with confidence intervals for uncertainty estimation
**Args:** `rsem-calculate-expression --paired-end --num-threads 8 --calc-ci R1.fastq.gz R2.fastq.gz rsem_index/genome sample_ci`
**Explanation:** rsem-calculate-expression companion binary; --paired-end mode; --num-threads 8 parallel processing; --calc-ci computes 95% confidence intervals for expression estimates; R1.fastq.gz R2.fastq.gz input FASTQs; rsem_index/genome index prefix; sample_ci output prefix; useful for downstream uncertainty-aware DE analysis

### quantify from pre-aligned BAM file
**Args:** `rsem-calculate-expression --alignments --paired-end --num-threads 8 aligned.bam rsem_index/genome sample_from_bam`
**Explanation:** rsem-calculate-expression companion binary; --alignments uses pre-aligned BAM; --paired-end mode; --num-threads 8 parallel processing; aligned.bam input BAM; rsem_index/genome index prefix; sample_from_bam output prefix; BAM must have adjacent paired-end mates; faster if alignment already exists

### quantify multiple FASTQ files (technical replicates)
**Args:** `rsem-calculate-expression --paired-end --num-threads 8 R1a.fq.gz,R1b.fq.gz R2a.fq.gz,R2b.fq.gz rsem_index/genome sample_merged`
**Explanation:** rsem-calculate-expression companion binary; --paired-end mode; --num-threads 8 parallel processing; R1a.fq.gz,R1b.fq.gz comma-separated R1 files; R2a.fq.gz,R2b.fq.gz comma-separated R2 files; rsem_index/genome index prefix; sample_merged output prefix; comma-separated files for technical replicates; RSEM merges them before quantification

### prepare reference with Bowtie instead of Bowtie2
**Args:** `rsem-prepare-reference --gtf genes.gtf --bowtie --num-threads 8 genome.fa rsem_bowtie_index/genome`
**Explanation:** rsem-prepare-reference companion binary; --gtf genes.gtf annotation file; --bowtie builds Bowtie index instead of Bowtie2; --num-threads 8 parallel processing; genome.fa input genome FASTA; rsem_bowtie_index/genome output index prefix; useful for compatibility with older workflows

### run RSEM with STAR and output transcriptome BAM
**Args:** `rsem-calculate-expression --paired-end --star --star-output-genome-bam --num-threads 8 R1.fq.gz R2.fq.gz rsem_index/genome sample_star`
**Explanation:** rsem-calculate-expression companion binary; --paired-end mode; --star uses STAR aligner; --star-output-genome-bam outputs genome-coordinate BAM; --num-threads 8 parallel processing; R1.fq.gz R2.fq.gz input FASTQs; rsem_index/genome index prefix; sample_star output prefix; useful for visualization

### generate count matrix from multiple samples for DESeq2
**Args:** `rsem-generate-data-matrix sample1.genes.results sample2.genes.results sample3.genes.results > gene_counts.matrix`
**Explanation:** rsem-generate-data-matrix companion binary; sample1.genes.results sample2.genes.results sample3.genes.results input gene results files; > gene_counts.matrix output matrix; extracts expected_count columns from multiple samples; output is a count matrix suitable for DESeq2/edgeR

### extract TPM values for cross-sample comparison
**Args:** `awk 'NR==1 {print "gene\tTPM"} NR>1 {print $1"\t"$6}' sample.genes.results > sample.tpm.txt`
**Explanation:** awk command; 'NR==1 {print "gene\tTPM"} NR>1 {print $1"\t"$6}' expression; sample.genes.results input gene results file; > sample.tpm.txt output TPM file; extracts gene IDs and TPM values; TPM is normalized for cross-sample comparison
