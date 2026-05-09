---
name: bwa
category: sequence-alignment
description: BWA is a software package for mapping DNA sequences to a reference genome using FM-index based on Burrow-Wheeler Transform. It provides three algorithms: `mem` for short-to-medium reads, `aln` for short reads with iterative seeding, and `bwasw` for long reads with structural variations.
tags:
  - alignment
  - mapping
  - short-reads
  - reference-genome
  - sam
  - fastq
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- BWA builds a compressed FM-index over the reference genome, which enables fast substring search by exploiting the Burrow-Wheeler Transform. Index files (`.bwt`, `.pac`, `.ann`, `.amb`, `.alt`, `.sa`) are required before alignment and are created with `bwa index`. Indexing is memory-intensive and must complete before any alignment command.
- `bwa mem` is the recommended algorithm for all reads longer than 70 bp, including single-end and paired-end data. It performs local Smith-Waterman alignment to find maximal scoring matches, which naturally handles chimeric reads, clip-overlapping adapter sequences, and structural variants without additional flags.
- Input reads must be in FASTA or FASTQ format (plain or gzip-compressed via `.gz` suffix). `bwa mem` natively handles paired-end reads by supplying two FASTQ files separated by a space on the command line, producing properly flagged paired alignments in SAM output.
- The output is always SAM (Sequence Alignment/Map) format, written to stdout. Users must redirect or pipe the output to a file. The header includes `@SQ` lines with reference sequence names and lengths derived automatically from the indexed reference.

## Pitfalls

- Supplying only one FASTQ file to `bwa mem` when the data is paired-end causes the aligner to treat unpaired reads as single-end, discarding pair orientation and insert-size information. This corrupts downstream variant calling because read groups lose their mate information in the SAM FLAG field.
- Using the reference genome's `.fasta` file directly without first building an index with `bwa index` causes BWA to fail with a "FAIL" error for every read. The index is not optional; it must be built exactly once per reference.
- Specifying `-o` or `-i` options with `bwa aln` on paired-end reads without using `bwa sampe` for the final conversion produces inconsistent or missing alignments because `aln` generates per-read `.sai` files that must be merged in a paired-aware manner.
- Attempting to align reads longer than the reference genome causes excessive memory use and time because the FM-index traversal encounters many seed hits. Large reads (>10 kbp) should use `bwa bwasw` instead of `bwa mem`.
- Forgetting to redirect SAM output to a file (e.g., `> output.sam`) causes the output to be printed to the terminal, potentially producing very large terminal buffers and losing the alignment data if the session terminates.

## Examples

### Index a reference genome in FASTA format
**Args:** `index ref.fa`
**Explanation:** Building the FM-index is a prerequisite for all alignment subcommands; without it BWA cannot perform any mapping operation.

### Align single-end short reads to a reference
**Args:** `mem -t 8 ref.fa reads.fq.gz`
**Explanation:** `-t 8` allocates 8 CPU threads to accelerate alignment on multi-core systems, and gzip-compressed input is read directly without manual decompression.

### Align paired-end reads with multiple threads and clip-overlap
**Args:** `mem -t 16 -M -Y ref.fa R1.fq.gz R2.fq.gz`
**Explanation:** `-M` marks split alignments with a FLAG compatible with Picard/GATK tools, `-Y` enables soft-clipping of overlapping read pairs to avoid double-counting coverage at read ends.

### Align with a read group assigned and save output to SAM
**Args:** `mem -R "@RG\tID:sample1\tSM:sample1\tPL:ILLUMINA" ref.fa R1.fq.gz R2.fq.gz`
**Explanation:** The `-R` flag adds read-group metadata to every read in the SAM file, which is required for proper sample identification in downstream GATK or Mutect2 workflows.

### Align long nanopore reads using the BWA-SW algorithm
**Args:** `bwasw -t 12 ref.fa long_reads.fq.gz`
**Explanation:** `bwasw` is designed for reads >70 bp with structural variants and performs anchored Smith-Waterman chaining to handle large insertions and deletions.

### Convert SAM to sorted BAM using samtools
**Args:** `sort -O BAM`
**Explanation:** Piping SAM output directly into `samtools sort` with `-O BAM` produces a coordinate-sorted BAM file for efficient downstream variant calling or visualization.