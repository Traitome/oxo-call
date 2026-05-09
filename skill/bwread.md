---
name: bwread
category: sequence_retrieval
description: Extract sequences from a BWA index using the FM-index. This tool maps reads back to their original positions in the reference genome by reading from a pre-built BWA index.
tags:
  - sequence_retrieval
  - FM-index
  - BWA
  - read-mapping
  - index-query
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- **BWA Index Requirement**: bwread requires a pre-built BWA index (created with `bwa index`) as its first argument. The index typically consists of three or more files with `.amb`, `.ann`, `.bwt`, `.pac`, and `.sa` extensions.
- **Input Format Support**: bwread accepts FASTQ or FASTA format input for the reads to be queried against the index. The tool reads these sequences and reports their occurrence positions in the reference.
- **Output to stdout**: By default, bwread writes SAM format output to standard output, which can be redirected to a file or piped into other tools like `samtools` for further processing.
- **Reverse Complement Matching**: bwread can match reads in both forward and reverse complement orientation when the appropriate option is enabled, enabling detection of reads mapping to the negative strand.

## Pitfalls

- **Missing Index Files**: Providing an index base name that points to incomplete or corrupted index files will cause bwread to fail with ambiguous error messages, making debugging difficult.
- **Conflicting Input Format Options**: Using incompatible options for input format (e.g., specifying both FASTQ and FASTA modes simultaneously) may result in silent failures or incorrectly parsed reads.
- **Memory Usage on Large Genomes**: For very large reference genomes (e.g., mammalian chromosomes), bwread can consume significant memory during index loading, potentially causing performance degradation on systems with limited RAM.
- **SAM Header Omission**: Unlike `bwa mem`, bwread does not produce SAM headers by default, which can cause issues when downstream tools expect header lines for proper BAM file processing.

## Examples

### Extract sequences from a BWA index using a FASTQ input file