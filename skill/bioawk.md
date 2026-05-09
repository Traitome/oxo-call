---
name: bioawk
category: text-processing
description: An AWK extension for bioinformatics file formats (BED, SAM, VCF, FASTA, FASTQ) that provides automatic parsing and built-in fields for common genomic data types.
tags:
- awk
- bioinformatics
- genomics
- text-processing
- bed
- sam
- vcf
- fasta
- fastq
author: AI-generated
source_url: https://github.com/lh3/bioawk
---

## Concepts

- **Auto-detection of formats:** bioawk automatically recognizes input formats based on file extension and magic bytes, enabling format-specific field access (e.g., `$start`, `$end` for BED; `$cigar` for SAM; `$qual` for FASTQ) without manual parsing.
- **Native bioinformatics record types:** The tool adds support for BED (chrom, start, end, name, score, strand), SAM/VCF (with standardized columns like FLAG, MAPQ, ALT), and FASTA/FASTQ (with sequence and quality strings), treating each as first-class AWK records.
- **Bidirectional output:** bioawk can generate output in the same format as input using `print` or `printf` with format-specific variables, allowing seamless filtering and transformation pipelines without explicit format rebuilding.
- **Companion binary awareness:** The related tool `bioawk` includes the same parser; `bioawk-build` is **not** a separate companion binary—use bioawk directly for both parsing and basic operations.

## Pitfalls

- **Using standard AWK instead of bioawk:** Attempting to parse SAM/VCF files with plain gawk fails because these formats lack uniform field counts (VCF has optional INFO fields, SAM has variable-length tags), leading to silently corrupted output.
- **Assuming 0-based vs 1-based coordinates:** BED files use 0-based start coordinates in bioawk, while SAM uses 1-based coordinates—mixing these without conversion causes off-by-one errors in downstream analyses like genome browser visualization.
- **Omitting header lines when processing:** FASTQ and VCF files often require preserved headers (`@` or `##`) for downstream tools; bioawk processes all lines by default, and failing to filter headers leads to errors in alignment or variant calling pipelines.
- **Incorrect field capitalization:** Variables like `$qual` (uppercase) are distinct from `$QUAL` (a user variable); using wrong capitalization returns empty values, causing data loss without error messages.

## Examples

### Extract protein-coding sequences from a FASTA file
**Args:** `-c fasta '{ if ($name ~ /protein/) print > "proteins.fa" }'`
**Explanation:** The `-c fasta` flag enables FASTA parsing so `$name` contains the sequence identifier, allowing selective output to a new file.

### Count aligned reads in a SAM file with MAPQ > 30
**Args:** `-c sam '($flag