---
name: bugseq-porechop
category: Nanopore sequencing data processing
description: A bioinformatics tool for trimming Oxford Nanopore sequencing adapters and low-quality ends from FASTQ reads. Identifies and removes common nanopore sequencing adapters, barcodes, andkit-specific sequences from raw read files to improve downstream analysis accuracy.
tags: [nanopore, fastq, adapter-trimming, sequencing, basecalling, read-quality]
author: AI-generated
source_url: https://github.com/rrwick/Porechop
---

## Concepts

- **Input format**: Accepts single or multiple FASTQ files containing Oxford Nanopore sequencing reads. Can process both single-read and multi-FASTQ files in batch mode. Files can be uncompressed FASTQ or gzipped FASTQ (.fastq, .fq, .fastq.gz, .fq.gz).
- **Adapter detection**: Uses built-in database of common nanopore kit adapters (SQK-LSK109, SQK-RBK001, SQK-RAB-201, etc.) and searches for barcode sequences. Identifies adapters at both read ends and internal positions using alignment algorithms.
- **Output modes**: Produces trimmed reads in FASTQ format to stdout or specified output file. Supports summary reporting of trimming actions including number of reads trimmed, adapters found, and read length statistics before/after trimming.
- **Scoring system**: Employs alignment scoring to distinguish true adapters from sequencing artifacts. Uses minimum score thresholds to avoid false positive trimming. Higher scores indicate stronger adapter matches.

## Pitfalls

- **Over-trimming low-complexity sequences**: When using aggressive trimming settings, legitimate reads with homopolymer runs or simple repeats may be incorrectly identified as adapter sequences, resulting in loss of valid biological data.
- **Ignoring unknown kit adapters**: Default adapter database may not include custom or newly-released nanopore kits, leading to incomplete trimming when processing data from newer sequencing kits not yet in the builtin database.
- **Input file format mismatch**: Providing wrong file extensions or corrupted FASTQ files causes tool failure. Using .FASTA instead of .FASTQ loses quality information needed for proper trimming decisions.
- **Missing output directory permissions**: Attempting to write output files to directories without write permissions fails silently or produces permission errors, leaving no trimmed output for downstream workflows.

## Examples

### Trim adapters from a single FASTQ file
**Args:** `-i input_reads.fastq -o trimmed_reads.fastq`
**Explanation:** Reads the input FASTQ file, identifies and removes known nanopore sequencing adapters, and writes cleaned reads to the output file.

### View detected adapters without modifying files
**Args:** `-i input_reads.fastq --discard_middle`
**Explanation:** Scans the input reads and reports found adapter sequences without performing trimming, useful for auditing new datasets.

### Trim adapters and discard poorly trimmed reads
**Args:** `-i input.fastq --require_three_adapters -o output.fastq`
**Explanation:** Only keeps reads where at least three adapter sequences were found, ensuring aggressive trimming and reducing false positives.

### Process multiple FASTQ files in batch
**Args:** `sample1.fastq sample2.fastq sample3.fastq -o batch_output/`
**Explanation:** Takes multiple input files and writes trimmed outputs to a specified directory, maintaining original filename prefixes.

### Enable verbose debugging output
**Args:** `-i reads.fastq -o trimmed.fastq -vv`
**Explanation:** Runs with very verbose logging, showing detailed alignment scores and each adapter match for troubleshooting trimming issues.

### Set minimum adapter alignment score
**Args:** `-i input.fastq -o output.fastq --min_score 75`
**Explanation:** Raises the minimum score threshold for adapter detection to reduce false positives on low-quality reads with partial adapter matches.