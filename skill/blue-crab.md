---
name: blue-crab
category: sequence_analysis
description: A bioinformatics tool for local short-read error correction and quality filtering. Operates on FASTQ files to identify and correct sequencing errors, trim low-quality bases, and filter out reads that fail quality thresholds.
tags:
  - sequence-analysis
  - quality-control
  - error-correction
  - fastq
  - short-reads
author: AI-generated
source_url: https://github.com/example/blue-crab
---

## Concepts

- **Input format**: blue-crab processes paired or unpaired FASTQ files. Files must be standard FASTQ format with quality scores encoded in either Phred+33 or Phred+64 depending on the sequencer's encoding.
- **Output format**: The tool produces corrected and filtered FASTQ files, optionally outputting statistics in TSV format detailing number of reads corrected, bases modified, and reads filtered.
- **Quality encoding detection**: The tool automatically detects quality score encoding (Phred+33 or Phred+64) by analyzing the distribution of ASCII characters in the quality string. This detection is used to set internal quality thresholds.
- **Error-correction algorithm**: Uses a k-mer based approach to identify likely sequencing errors by comparing low-frequency k-mers to nearby high-frequency k-mers within a read. Only corrections with support from neighboring high-quality bases are applied.
- **Filtering logic**: Reads containing more than a configurable fraction of bases below the quality threshold are discarded. Filtering is applied before error correction to avoid wasting computation on low-quality reads.

## Pitfalls

- **Mismatched quality encoding**: If the quality encoding is incorrectly detected or manually specified wrong, all quality thresholds will be applied incorrectly, leading to over-filtering of good reads or under-filtering of poor reads. This corrupts downstream analyses.
- **Using default parameters on high-error datasets**: The default error-correction sensitivity is tuned for standard Illumina data (~0.1% error rate). For Pacific Biosciences or Oxford Nanopore data (which have ~10-15% error rates), using defaults will make no corrections or incorrect ones.
- **Confusing input and output filenames**: When specifying multiple input files or paired outputs, accidentally swapping input filenames leads to pairing reads from different samples, which corrupts downstream analysis.
- **Ignoring paired-end read orientation**: For paired-end data, specifying the incorrect library orientation (forward-reverse vs reverse-forward) causes the tool to treat valid pairs as broken, filtering them out instead of processing them.
- **Insufficient disk space for temporary files**: During processing, blue-crab may create temporary files for intermediate results. Running on a nearly-full disk causes failure partway through, losing all progress.

## Examples

### Correct single-end FASTQ using default quality thresholds
**Args:** -i input.fastq -o output.fastq
**Explanation:** Applies built-in default quality thresholds (Phred ≥ 20) and default error-correction sensitivity to correct a single FASTQ file, writing corrected reads to output.fastq.

### Correct paired-end FASTQ with explicit quality encoding
**Args:** -1 input_R1.fastq -2 input_R2.fastq -o corrected/ --phred+33
**Explanation:** Processes paired-end FASTQ files while explicitly specifying Phred+33 quality encoding to avoid detection errors. Outputs paired corrected files to the specified directory.

### Filter reads below quality threshold without error correction
**Args:** -i input.fastq -o filtered.fastq --no-correct --min-qual 25
**Explanation:** Applies only read filtering without error correction, discarding any reads with more than 10% of bases below Phred 25. Useful for generating clean training sets.

### Specify custom error-correction sensitivity for long-read data
**Args:** -i input.fastq -o output.fastq --sensitivity high --freq-cutoff 0.7
**Explanation:** Uses higher error-correction sensitivity appropriate for high-error-rate data, requiring 70% local coverage support for a correction rather than the default 90%.

### Output detailed statistics to file
**Args:** -i input.fastq -o output.fastq --stats statistics.tsv
**Explanation:** Runs error correction and quality filtering while writing detailed per-read and summary statistics to the specified TSV file for downstream analysis.

### Trim adapters before error correction
**Args:** -i input.fastq -o output.fastq --trim-adapters AGATCGGAAGAGC
**Explanation:** Trims the specified adapter sequence from reads before applying error correction, preventing adapter-derived k-mers from causing false corrections.