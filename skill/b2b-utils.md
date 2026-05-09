---
name: b2b-utils
category: Bioinformatics utilities
description: A collection of utility programs for processing and managing B2B (back-to-back) format data in bioinformatics pipelines. Includes tools for format conversion, validation, and data manipulation.
tags:
  - bioinformatics
  - utilities
  - data-processing
  - format-conversion
  - validation
author: AI-generated
source_url: https://github.com/b2b-utils/b2b-utils
---

## Concepts

- **B2B Format Specification**: B2b-utils works with paired-end read data in back-to-back format, where forward and reverse reads are stored in separate files with matching identifiers. The tool expects paired read files to share identical headers with `.fwd` and `.rev` suffixes or paired notation.
- **Companion Binaries**: The suite includes `b2b-validate` for syntax checking, `b2b-stats` for summary statistics, `b2b-merge` for combining paired files, and `b2b-split` for dividing large datasets. Each companion binary can be invoked independently.
- **Standard I/O Behavior**: By default, b2b-utils reads from stdin and writes to stdout, enabling Unix pipeline integration. Use `-i/--input` for file input and `-o/--output` for file output when pipeline mode is not desired.
- **Multi-format Support**: The tool supports FASTQ, FASTA, and custom B2B binary formats. Format auto-detection occurs based on file extension or magic bytes in binary mode.

## Pitfalls

- **Mismatched Pair Files**: Specifying a forward file without its corresponding reverse file, or vice versa, causes the tool to fail with a cryptic pairing error. Always provide both `--fwd` and `--rev` flags when processing paired-end data.
- **Encoding Version Mismatch**: Using flags from an older B2B specification version with newer data files produces silent data corruption. Always verify version compatibility with `--version` flag before processing.
- **Insufficient Memory for Large Datasets**: Attempting to load entire datasets into memory without using streaming mode (`-S/--streaming`) causes out-of-memory errors on systems with limited RAM. Use streaming for datasets exceeding available memory.

## Examples

### Validate B2B paired-end files
**Args:** `--validate --fwd reads_fwd.fastq --rev reads_rev.fastq`
**Explanation:** Checks both forward and reverse FASTQ files for proper pairing, read quality, and format compliance before downstream processing.

### Compute statistics on binary B2B data
**Args:** `--stats --input data.b2b`
**Explanation:** Generates read length distribution, quality score histograms, and pairing summary statistics for the specified binary format file.

### Convert FASTQ to B2B binary format
**Args:** `--convert fastq2b2b --input reads.fq --output reads.b2b`
**Explanation:** Transforms standard FASTQ format into compressed B2B binary format for faster downstream processing and reduced storage footprint.

### Stream process paired-end data through filter
**Args:** `--filter --min-quality 20 --streaming --fwd input_fwd.fq --rev input_rev.fq | gzip > filtered.fq.gz`
**Explanation:** Applies quality filtering in streaming mode to handle large datasets without loading entire files into memory, outputting gzipped filtered reads.

### Extract forward reads only from B2B file
**Args:** `--extract --strand forward --input paired.b2b --output fwd_only.b2b`
**Explanation:** Extracts only forward strand reads from a paired B2B file, creating a new file containing only the selected reads.