---
name: cogent
category: sequence-analysis
description: A fast sequence alignment and analysis tool for processing genomic reads against reference sequences, supporting batch processing and multiple output formats.
tags:
  - sequence-alignment
  - genomics
  - read-mapping
  - bioinformatics
  - fastq
author: AI-generated
source_-url: https://github.com/cogent-project/cogent
---

## Concepts

- **Input Formats**: cogent accepts unaligned reads in FASTQ or FASTA format, with support for gzipped (.gz) input files to reduce disk I/O overhead and storage requirements.
- **Index Structure**: The tool requires a pre-built reference index using the companion binary `cogent-build`, which constructs a lossless FM-index for memory-efficient storage and rapid suffix array traversal.
- **Output Modes**: Three primary output modes exist—SAM format for standard alignment output, BED format for genomic interval visualization, and JSON format for programmatic downstream processing and integration with pipelines.
- **Multithreading**: cogent utilizes shared-memory parallelism via the `-p` flag, spawning worker threads proportional to available CPU cores for linear speedup on multi-core systems.
- **Scoring System**: Alignment uses a Smith-Waterman local alignment model with configurable mismatch penalty (`-m`), gap open penalty (`-o`), and gap extend penalty (`-e`) parameters.

## Pitfalls

- **Missing Index**: Running cogent without first building an index with `cogent-build` causes immediate failure with an uninformative "reference not found" error that does not indicate the index is missing.
- **Incompatible Quality Scores**: FASTQ files using different quality encoding schemes (Phred+33 vs Phred+64) will produce silently incorrect alignments without warnings if the `-q` encoding flag is not specified.
- **Memory Exhaustion on Large Genomes**: Specifying a chromosome-scale reference (>3GB) without reducing the `-B` buffer parameter causes out-of-memory kills on systems with limited RAM, losing all progress.
- **Wrong Output Format for Downstream Tools**: Producing SAM output when downstream tools require BAM will fail silently if samtools is not used for conversion, because BAM conversion is not automatic.
- **Thread Overcommit**: Setting `-p` to a value exceeding physical CPU cores causes thread context-switching overhead that reduces performance below single-threaded speed, yielding slower results.

## Examples

### Build a reference genome index for human GRCh38
**Args:** `build -d GRCh38 /data/references/chr1.fa`
**Explanation:** The `build` companion binary constructs the FM-index from the FASTA reference file, enabling subsequent alignment operations against this genome.

### Align paired-end FASTQ reads to a reference
**Args:** `align -r ref.idx -1 reads_R1.fastq.gz -2 reads_R2.fastq.gz -o alignments.sam`
**Explanation:** The `-1` and `-2` flags specify paired-end read files, producing a SAM alignment file containing CIGAR strings and mapping qualities for each read.

### Map single-end reads with strict alignment criteria
**Args:** `align -r ref.idx -U reads.fastq -o strict_results.sam -m 6 -e 2 -o 5`
**Explanation:** The scoring parameters `-m` (mismatch), `-e` (gap extend), and `-o` (gap open) increase alignment stringency, keeping only high-quality mappings suitable for variant calling.

### Export alignments in BED format for UCSC Genome Browser
**Args:** `align -r ref.idx -1 sample_R1.fq -2 sample_R2.fq -O bed -b track1 > results.bed`
**Explanation:** The `-O bed` flag outputs genomic intervals in BED format directly to stdout, avoiding intermediate SAM-to-BED conversion in separate tools.

### Process reads with Illumina 1.8+ quality encoding
**Args:** `align -r ref.idx -U sample.fastq.gz -o output.sam -q illumina1.8`
**Explanation:** The `-q illumina1.8` flag correctly interprets ASCII-offset quality scores, preventing systematic misalignment due to quality score decoding errors.

### Parallel alignment on 16-core workstation
**Args:** `align -r ref.idx -1 R1.fq -2 R2.fq -o output.sam -p 16`
**Explanation:** The `-p 16` flag spawns 16 worker threads, achieving near-linear speedup for large read sets by distributing suffix array traversal across multiple CPU cores.