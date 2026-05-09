---
name: bwa-meme
category: read_alignment
description: A Burrows-Wheeler Aligner (BWA) variant using the MEM algorithm for accurate alignment of short and long sequencing reads to a reference genome. Optimized for next-generation sequencing data with support for paired-end and single-end reads.
tags: [alignment, ngs, read-mapping, sequence-analysis, genomics, bwa, short-reads, long-reads]
author: AI-generated
source_url: https://github.com/arebaka/bwa-meme
---

## Concepts

- **Input formats:** Accepts FASTA (`.fa`, `.fasta`) and FASTQ (`.fq`, `.fastq`) files for both reference genomes and sequencing reads, supporting gzipped (`.gz`) and plain text formats.
- **Index requirement:** Requires a pre-built FM-index using the `bwa-meme index` subcommand before alignment; index files share the reference basename with extensions like `.bwt`, `.pac`, `.ann`, `.amb`, `.sa`.
- **Algorithm variant:** Implements the maximal exact match (MEM) algorithm, making it suitable for reads 70bp to 1Mbp in length with moderate to high sequence divergence from the reference.
- **Output format:** Produces SAM (Sequence Alignment/Map) format by default, describing read positions, mapping qualities (`MAPQ`), CIGAR strings, and optional tags for downstream analysis.
- **Paired-end support:** Aligns both reads in a pair and outputs proper mate information; use supplementary flags like `-m` to enable mate rescue and pairing heuristics.

## Pitfalls

- **Missing index:** Running `bwa-meme aln` or `bwa-meme mem` without first building an index results in immediate failure with a file-not-found error, wasting computation time on large inputs.
- **Mismatched read groups:** Providing FASTQ reads with quality scores to a FASTA reference index (or vice versa) in single-end mode does not produce errors but yields degraded alignment quality and inflated mismatch counts.
- **Oversized memory allocation:** Setting `-t` (threads) higher than available CPU cores without sufficient memory causes OOM (Out of Memory) kills, especially on systems with limited RAM during large batch alignments.
- **Output file overwrites:** Using the default output path without specifying a unique SAM file name causes overwrites when processing multiple samples sequentially; SAM files lack built-in timestamp protection.
- **Incorrect read orientation:** Failing to specify mate pair library type (`-b` for FR orientation) produces erroneous paired-end pairing information and丢弃s proper mapping for反向文库 libraries.

## Examples

### Build FM-index for a reference genome
**Args:** index reference.fasta
**Explanation:** Creates the FM-index files required for subsequent alignment operations, using the reference basename for all generated index components.

### Align single-end short reads to indexed reference
**Args:** aln -t 8 reference.fasta reads.fq > alignment.sam
**Explanation:** Aligns single-end reads with 8 threads, outputting SAM format to standard output for piping or redirection to a file.

### Align paired-end reads with standard parameters
**Args:** mem -t 12 reference.fasta read1.fq read2.fq > paired.sam
**Explanation:** Performs paired-end alignment of two FASTQ files using 12 threads, generating proper mate information and alignment coordinates in SAM format.

### Align with reduced gap opening penalty for long reads
**Args:** mem -W 16 -k 13 reference.fasta long_reads.fq > long_aligned.sam
**Explanation:** Adjusts minimum seed length and reduces gap penalty window for better alignment of long reads (Pacific Biosciences, Oxford Nanopore).

### Force smaller alignment bandwidth for highly conserved regions
**Args:** aln -w 15 -d 15 reference.fasta conserved_reads.fq > tight_aligned.sam
**Explanation:** Restricts the band width for dynamic programming to 15bp, improving local alignment precision in low-divergence genomic regions.

### Align with custom output map quality threshold
**Args:** mem -q 20 reference.fasta reads.fq > filtered_mapq.sam
**Explanation:** Filters alignments to only report those with mapping quality (MAPQ) of 20 or higher, reducing low-quality mappings in downstream variant calling.

### Run alignment with read group tags for deduplication
**Args:** mem -R '@RG\tID:sample1\tSM:projectA' reference.fasta reads.fq > annotated.sam
**Explanation:** Adds read group metadata to the SAM header, enabling downstream tools like GATK to properly identify and deduplicate reads by sample.

### Parallel alignment of multiple read files using GNU parallel
**Args:** -t 4 reference.fasta sample1.fq sample2.fq sample3.fq | xargs -I {} -P 4 sh -c 'bwa-meme mem reference.fasta {} > {.}.sam'
**Explanation:** Demonstrates invocation of the `mem` subcommand through parallel job scheduling, using the wrapper pattern to generate per-sample SAM outputs without including the tool name in Args.