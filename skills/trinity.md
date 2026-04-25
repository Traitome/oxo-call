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
- --samples_file for multiple samples with biological replicate info (tab-delimited).
- --min_kmer_cov sets minimum k-mer coverage for Inchworm (default 1, use 2 for deeply sequenced).
- --normalize_reads enables read normalization (default on); --no_normalize_reads to disable.
- --normalize_max_read_cov sets coverage threshold for normalization (default 200).
- --min_contig_length sets minimum contig length to report (default 200).

## Pitfalls
- Trinity is memory-intensive — always specify --max_memory to prevent OOM errors.
- Trinity is slow for large datasets — use Trinity on subsampled reads for initial validation.
- --CPU doesn't linearly improve speed — 16-32 CPUs is usually the practical limit.
- Trinity output transcripts may include redundant isoforms — cluster with CD-HIT-EST for non-redundant sets.
- For eukaryotic organisms with a reference, genome-guided assembly (--genome_guided_bam) is more accurate.
- Trinity requires all Inchworm, Chrysalis, Butterfly dependencies in PATH.
- --samples_file requires tab-delimited format with columns: condition, replicate, left_reads, right_reads.
- --min_kmer_cov 2 filters singleton k-mers; may miss low-expressed transcripts but reduces noise.
- Read normalization reduces runtime but may miss rare transcripts; adjust --normalize_max_read_cov.
- --min_contig_length default 200 may be too long for some applications; adjust as needed.

## Examples

### de novo transcriptome assembly from paired-end RNA-seq reads
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --max_memory 50G --CPU 16 --output trinity_output/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz paired-end R1; --right R2.fastq.gz paired-end R2; --max_memory 50G RAM limit; --CPU 16 threads; --output trinity_output/ output directory

### genome-guided Trinity assembly using STAR alignments
**Args:** `--genome_guided_bam star_aligned.bam --genome_guided_max_intron 10000 --max_memory 50G --CPU 16 --output genome_guided_trinity/`
**Explanation:** Trinity command; --genome_guided_bam star_aligned.bam STAR-aligned BAM input; --genome_guided_max_intron 10000 max intron size; --max_memory 50G RAM limit; --CPU 16 threads; --output genome_guided_trinity/ output directory; more accurate than de novo

### de novo assembly from single-end RNA-seq reads
**Args:** `--seqType fq --single reads.fastq.gz --max_memory 32G --CPU 8 --output trinity_se/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --single reads.fastq.gz single-end reads; --max_memory 32G RAM limit; --CPU 8 threads; --output trinity_se/ output directory

### Trinity assembly with strand-specific library
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --SS_lib_type RF --max_memory 50G --CPU 16 --output stranded_trinity/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz --right R2.fastq.gz paired-end reads; --SS_lib_type RF reverse-strand dUTP library; --max_memory 50G RAM limit; --CPU 16 threads; --output stranded_trinity/ output directory

### Trinity assembly from multiple samples with replicates
**Args:** `--seqType fq --samples_file samples.txt --max_memory 50G --CPU 16 --output multi_sample_trinity/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --samples_file samples.txt tab-delimited file with condition, replicate, left, right columns; --max_memory 50G RAM limit; --CPU 16 threads; --output multi_sample_trinity/ output directory

### Trinity with increased k-mer coverage threshold
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --min_kmer_cov 2 --max_memory 50G --CPU 16 --output high_cov_trinity/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz --right R2.fastq.gz paired-end reads; --min_kmer_cov 2 filters singleton k-mers; --max_memory 50G RAM limit; --CPU 16 threads; --output high_cov_trinity/ output directory; reduces noise

### Trinity without read normalization
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --no_normalize_reads --max_memory 100G --CPU 16 --output no_norm_trinity/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz --right R2.fastq.gz paired-end reads; --no_normalize_reads disables normalization; --max_memory 100G RAM limit (higher needed); --CPU 16 threads; --output no_norm_trinity/ output directory; retains all reads

### Trinity with custom normalization coverage
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --normalize_max_read_cov 50 --max_memory 50G --CPU 16 --output norm50_trinity/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz --right R2.fastq.gz paired-end reads; --normalize_max_read_cov 50 reduces coverage to 50x; --max_memory 50G RAM limit; --CPU 16 threads; --output norm50_trinity/ output directory; more aggressive normalization

### Trinity with shorter minimum contig length
**Args:** `--seqType fq --left R1.fastq.gz --right R2.fastq.gz --min_contig_length 100 --max_memory 50G --CPU 16 --output short_contigs/`
**Explanation:** Trinity command; --seqType fq FASTQ format; --left R1.fastq.gz --right R2.fastq.gz paired-end reads; --min_contig_length 100 minimum contig length; --max_memory 50G RAM limit; --CPU 16 threads; --output short_contigs/ output directory; default is 200

### genome-guided Trinity with custom max intron
**Args:** `--genome_guided_bam star_aligned.bam --genome_guided_max_intron 50000 --max_memory 50G --CPU 16 --output gg_trinity_large_intron/`
**Explanation:** Trinity command; --genome_guided_bam star_aligned.bam STAR-aligned BAM input; --genome_guided_max_intron 50000 larger intron size for organisms with large introns; --max_memory 50G RAM limit; --CPU 16 threads; --output gg_trinity_large_intron/ output directory
