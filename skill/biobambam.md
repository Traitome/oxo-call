---
name: biobambam
category: sequence_alignment
description: A bioinformatics toolsuite for BAM/SAM file processing, including PCR duplicate marking, sorting, filtering, and format conversion. Provides high-performance tools for NGS data manipulation.
tags:
  - bam
  - sam
  - duplicate-marking
  - sorting
  - ngs
  - sequencing
  - bioinformatics
author: AI-generated
source_url: https://github.com/gt1/biobambam2
---

## Concepts

- **BAM Input/Output**: biobambam tools operate on BAM (Binary Alignment Map) files, which are compressed binary versions of SAM files. Tools accept BAM input via stdin or files, and output BAM to stdout by default for piping. Use `SOCKET_WRITING=1` for streaming output to avoid full file buffering.
- **Duplicate Marking**: The `bamsormadup` tool marks PCR duplicates by comparing read positions and sequences. Reads with identical 5' positions on the same strand are candidates for duplicate marking, with later duplicates marked while keeping the first occurrence.
- **Collision Tolerance**: Duplicate detection uses a hash-based approach with configurable collision tolerance (`COLLTYPE` and `COLLOBS` parameters). Higher collision tolerance increases sensitivity but may incorrectly mark non-duplicates, especially in low-complexity regions.
- **Sorting and Collating**: Tools support coordinate-based sorting output via `SO coordinate` and read-name sorting via `SO queryname`. The collation tool (`bamcollate`) rearranges reads to optimize downstream processing like duplicate detection.

## Pitfalls

- **Memory Usage**: Large BAM files require significant memory when loading all reads for duplicate marking. Use `blocksize` parameter to control input block size, and set `verbose=1` to monitor processing. For machines with limited RAM, process files in chunks or use chromosomal subsets.
- **Output Format Confusion**: By default, biobambam tools output BAM format to stdout. Adding `-o output.bam` changes output to a file but may cause issues in pipelines expecting stdin for the next tool. Explicitly specify format with `-f bam` or `-f sam` when needed.
- **Duplicate Marking Sensitivity**: Default duplicate detection may miss optical duplicates in high-throughput sequencing data. Adjust `opticaldist` parameter for expected distance between clusters on the flow cell, as low values cause false duplicate marking.
- **Index Files**: Output BAM files after sorting require index regeneration with tools like `samtools index`. biobambam does not automatically create index files, so downstream tools requiring indexed BAM will fail.

## Examples

### Mark PCR duplicates in a BAM file and sort by coordinate

**Args:** `inputfile=sample.bam SO coordinate marks=1 verbose=1 T=/tmp/sorttmp`

**Explanation:** This runs `bamsormadup` to mark duplicate reads while sorting output by genomic coordinates, using verbose logging and a temporary directory for intermediate files.

### Convert a BAM file to FASTQ format

**Args:** `inputfile=sample.bam outputfile=sample_fastq.fq.gz`

**Explanation:** This uses the `bamtofastq` tool to extract all reads from the BAM file and output them as FASTQ (optionally gzipped), preserving read names and quality scores.

### Sort a BAM file by read name

**Args:** `inputfile=sample.bam SO queryname outputfile=sorted_queryname.bam`

**Explanation:** This sorts the input BAM by read name (QNAME) rather than genomic position, useful for preparing files for tools expecting queryname-sorted input.

### Filter reads by mapping quality

**Args:** `inputfile=sample.bam mapq=30 outputfile=filtered.bam`

**Explanation:** This filters reads using a minimum mapping quality threshold, outputting only reads with MAPQ >= 30 to remove low-quality alignments.

### Mark duplicates with custom optical distance

**Args:** `inputfile=sample.bam marks=1 opticaldist=2500`

**Explanation:** This sets the optical duplicate distance to 2500 (pixels on flow cell), adjusting for the specific sequencing platform and cluster density.

### Collate reads by reference position for efficient processing

**Args:** `inputfile=sample.bam col Collate=1`

**Explanation:** This collates the BAM file to group reads by reference position, optimizing the file layout for faster duplicate detection and other position-dependent operations.

### Output SAM format instead of BAM

**Args:** `inputfile=sample.bam SAM=1`

**Explanation:** This outputs SAM format (human-readable) instead of BAM to stdout, useful for debugging or piping to tools requiring text format input.