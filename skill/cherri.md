---
name: cherri
category: Bioinformatics / Sequence Analysis
description: A versatile bioinformatics tool for processing and analyzing genomic sequencing data, designed for read quality control, filtering, and format conversion tasks commonly performed in next-generation sequencing (NGS) pipelines.
tags: [ngs, sequencing, reads, quality-control, fastq, bioinformatics, genomics]
author: AI-generated
source_url: https://github.com/cherri-tool/cherri
---

## Concepts

- **Input Format**: cherri accepts FASTQ files (single or paired-end) as primary input, supporting both gzipped (.gz) and uncompressed formats. It can process multiple files simultaneously when provided as arguments.
- **Output Modes**: The tool offers three primary output modes: `--filter` for read filtering based on quality thresholds, `--convert` for format transformations (e.g., FASTQ to FASTA), and `--stats` for generating quality reports.
- **Quality Thresholds**: cherri uses Phred quality scores (Q-score) for filtering decisions, where a minimum quality score (specified via `--min-qual`) excludes reads containing bases below the specified threshold.
- **Paired-End Handling**: When processing paired-end data, use `--paired` to maintain read pairing integrity; the tool automatically syncs forward and reverse reads during filtering operations.
- **Streaming Support**: cherri supports streaming input/output via stdin/stdout, enabling integration into Unix pipelines without intermediate file storage.

## Pitfalls

- **Mismatched Pair Files**: Running cherri on paired-end data without the `--paired` flag filters forward and reverse reads independently, potentially breaking pair integrity and causing downstream alignment failures.
- **Incorrect Quality Score Encoding**: Specifying `--ascii` with the wrong offset (either 33 or 64) corrupts quality scores; most modern FASTQ files use Phred+33 encoding.
- **Empty Output Files**: Setting `--min-qual` too high (e.g., 40 or above) may filter out all reads entirely, producing empty output files that cause silent failures in subsequent pipeline steps.
- **Missing Input Files**: Not specifying input files or using incorrect file paths produces no error message in default mode; always verify file existence with `--check` before running large batches.
- **Compression Mismatch**:Outputting to a file with a `.fastq.gz` extension while the input is uncompressed without specifying `--compress` creates misleading file extensions that confuse downstream tools.

## Examples

### Filter low-quality reads from a single FASTQ file
**Args:** `--input reads.fastq --output filtered.fastq --min-qual 20 --filter`
**Explanation:** This filters out reads containing any base with a Phred quality score below 20, preserving high-confidence bases for downstream analysis.

### Convert FASTQ to FASTA format
**Args:** `--input reads.fastq --output reads.fasta --convert fasta`
**Explanation:** This transforms the input FASTQ file to FASTA format, stripping quality scores and creating a sequence-only file suitable for tools that accept FASTA input.

### Process paired-end reads while maintaining pair synchronization
**Args:** `--input R1.fastq R2.fastq --paired --output-dir ./filtered --min-qual 15 --filter`
**Explanation:** This processes both read files together, removing read pairs where either read fails the quality threshold, preserving proper pairing in the output directory.

### Generate quality statistics without filtering
**Args:** `--input reads.fastq --stats --output quality_report.txt`
**Explanation:** This performs quality analysis on the input file and writes summary statistics (base composition, score distribution, read lengths) without modifying the input data.

### Stream-filter data from stdin to stdout in a pipeline
**Args:** `--min-qual 25 --filter --ascii 33`
**Explanation:** This reads FASTQ data from standard input, applies stringent quality filtering (Phred ≥ 25), and outputs filtered reads to standard output for pipeline integration.