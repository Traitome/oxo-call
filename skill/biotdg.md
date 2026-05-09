---
name: biotdg
category: data-processing
description: A bioinformatics tool for transforming, generating, and manipulating biological sequence data and metadata. Supports batch processing of multiple input files with configurable output formats.
tags:
  - bioinformatics
  - data-transformation
  - sequence-processing
  - batch-processing
  - format-conversion
author: AI-generated
source_url: https://github.com/biotdg/biotdg
---

## Concepts

- biotdg processes biological sequence data (FASTA, FASTQ, GenBank) and converts between formats with optional filtering based on sequence length, quality scores, or annotation features.
- The tool maintains a streaming architecture that handles large files without loading entire datasets into memory, making it suitable for chromosome-scale inputs.
- Output can be directed to stdout for pipeline integration, or written to files with automatic naming based on input basenames and configurable suffixes.
- Batch processing is controlled via file glob patterns or explicit input files, with parallel execution supported through a --jobs flag that spawns worker processes.
- Metadata handling includes copying annotations, sequence identifiers, and quality encodings between input and output formats based on format compatibility.

## Pitfalls

- Specifying conflicting output format flags (e.g., both --fasta and --fastq) causes the tool to abort with an error instead of auto-selecting; use only one format flag per invocation.
- Using --filter with quality-based thresholds on FASTA input silently ignores the quality filter since FASTA lacks quality scores, leading to unexpected retention of all sequences.
- The --jobs flag spawns multiple processes that compete for stderr output; error messages may appear interleaved or incomplete when processing fails on worker processes.
- Output directory paths with trailing slashes are not normalized; on case-insensitive filesystems this can create duplicate directory entries or permission errors depending on filesystem implementation.
- The tool does not validate that output formats are compatible with input data; requesting GenBank output from an input lacking annotated features produces empty output files with no warning.

## Examples

### Convert a single FASTQ file to FASTA format

**Args:** input.fastq --output-format fasta --output output.fasta
**Explanation:** Converts FASTQ sequences and quality scores to FASTA format, discarding quality information since FASTA does not store it.

### Filter sequences by minimum length from multiple files

**Args:** --input "*.fastq" --min-length 50 --output-dir filtered/
**Explanation:** Processes all FASTQ files matching the glob pattern, retaining only sequences with 50 or more bases and writing each to the filtered directory.

### Generate a summary statistics report

**Args:** sample1.fastq sample2.fastq sample3.fastq --stats --stats-file summary.tsv
**Explanation:** Computes per-file statistics including total bases, read count, average length, and quality distributions, appending results to summary.tsv.

### Parallel processing with 4 worker jobs

**Args:** --input "data/*.fastq" --jobs 4 --output-dir processed/ --output-format fastq --compress
**Explanation:** Spawns 4 parallel workers to process FASTQ files from the data directory, writing results compressed with gzip and preserving original quality data.

### Extract specific sequence identifiers to a new file

**Args:** --input sequences.gb --extract-ids ids.txt --output-format fasta --output extracted.fasta
**Explanation:** Reads sequence identifiers from ids.txt (one per line), extracts matching sequences from the GenBank file, and writes them in FASTA format.