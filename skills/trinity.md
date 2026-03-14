---
name: trinity
category: rna-seq
description: De novo reconstruction of full-length transcripts from RNA-seq data without a reference genome
tags: [rna-seq, assembly, de-novo, transcriptome, isoform, transcript, no-reference]
author: oxo-call built-in
source_url: "https://github.com/trinityrnaseq/trinityrnaseq"
---

## Concepts

- Trinity assembles transcripts de novo without a reference genome using three modules: Inchworm, Chrysalis, Butterfly.
- Use --seqType fq for FASTQ input; --left and --right for paired-end reads; --single for single-end.
- Trinity requires significant RAM: ~1 GB per 1M reads; use --max_memory to cap.
- --CPU for parallelism; --output for output directory.
- Trinity output: Trinity.fasta with all assembled transcripts in the output directory.
- Use TransDecoder (bundled) for ORF prediction; kallisto/salmon for quantification against Trinity assembly.
- For genome-guided assembly, use --genome_guided_bam with STAR-aligned BAM for better results.
- Component names: TRINITY_DN[cluster]_c[component]_g[gene]_i[isoform].

## Pitfalls

- Trinity is memory-intensive — always specify --max_memory to prevent OOM errors.
- Trinity is slow for large datasets — use Trinity on subsampled reads for initial validation.
- --CPU doesn't linearly improve speed — 16-32 CPUs is usually the practical limit.
- Trinity output transcripts may include redundant isoforms — cluster with CD-HIT-EST for non-redundant sets.
- For eukaryotic organisms with a reference, genome-guided assembly (--genome_guided_bam) is more accurate.
- Trinity requires all Inchworm, Chrysalis, Butterfly dependencies in PATH.

## Examples

### de novo transcriptome assembly from paired-end RNA-seq reads
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --max_memory 50G --CPU 16 --output trinity_output/`
**Explanation:** --seqType fq FASTQ; --left/--right PE reads; --max_memory 50G RAM limit; --CPU 16 threads

### genome-guided Trinity assembly using STAR alignments
**Args:** `--genome_guided_bam star_aligned.bam --genome_guided_max_intron 10000 --max_memory 50G --CPU 16 --output genome_guided_trinity/`
**Explanation:** --genome_guided_bam STAR BAM; --genome_guided_max_intron max intron size; more accurate than de novo

### de novo assembly from single-end RNA-seq reads
**Args:** `--seqType fq --single reads.fastq.gz --max_memory 32G --CPU 8 --output trinity_se/`
**Explanation:** --single for single-end reads; same Trinity pipeline with SE input

### Trinity assembly with strand-specific library
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --SS_lib_type RF --max_memory 50G --CPU 16 --output stranded_trinity/`
**Explanation:** --SS_lib_type RF for reverse-strand (dUTP) libraries; RF=reverse, FR=forward
