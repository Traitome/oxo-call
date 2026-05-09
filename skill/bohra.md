---
name: bohra
category: Sequencing/Read Processing
description: A command-line tool for filtering, trimming, and quality control of NGS read data. Supports FASTQ input/output, read length filtering, quality score thresholds, and adapter sequence removal.
tags: [ngs, reads, fastq, quality-control, filtering, trimming, bioinformatics]
author: AI-generated
source_url: https://github.com/bohra/bohra
---

## Concepts

- **Input formats**: bohra accepts raw FASTQ files (single-end or paired-end) via stdin or file arguments, supporting both gzip-compressed (.gz) and uncompressed formats.
- **Quality filtering**: Reads are evaluated based on a minimum quality score threshold (Phred scale) across a sliding window; reads falling below this threshold in any window position are discarded.
- **Output streams**: Filtered reads are written to stdout in FASTQ format, allowing piping to other tools; statistics are reported to stderr by default.
- **Adapter removal**: Built-in support for common Illumina adapter sequences (e.g., AGATCGGAAGAGC) with configurable overlap length and removal behavior.
- **Paired-end mode**: When processing paired-end data, bohra maintains read pairing — if either read in a pair fails filtering, both reads are discarded to keep synchronization.

## Pitfalls

- **Forgetting --paired option in paired-end mode**: Without the `--paired` flag, bohra processes each read file independently, potentially breaking read pairing and causing downstream alignment issues.
- **Setting quality threshold too high**: A threshold above Q30 may discard too many valid reads, especially for low-complexity regions or high-GC content sequences, leading to reduced coverage.
- **Not specifying output for paired-end data**: In paired-end mode, failing to specify both forward (`--out1`) and reverse (`--out2`) output paths causes reads to be sent to stdout, mixing pairs incorrectly.
- **Ignoring error messages about empty input**: bohra silently produces empty output files if input FASTQ files are malformed, contain no reads, or use unsupported encoding (e.g., SAM format).

## Examples

### Filter single-end reads by minimum quality score
**Args:** `--qual 20 input.fq.gz -o filtered.fq.gz`
**Explanation:** This removes all reads containing anyPhred-scaled quality score below 20, keeping higher-quality reads for downstream analysis.

### Trim adapters from paired-end reads
**Args:** `--paired --adapter AGATCGGAAGAGC -o out1.fq.gz --out2 out2.fq.gz R1.fq.gz R2.fq.gz`
**Explanation:** Detects and removes Illumina adapter sequences from both read files while maintaining proper read pairing between output files.

### Filter reads shorter than specified length
**Args:** `--minlen 50 input.fq.gz -o long_reads.fq.gz`
**Explanation:** Discards reads shorter than 50 bases, useful for removing degraded fragments before alignment.

### Use window-based quality filtering
**Args:** `--window-qual 15 --window-size 5 input.fq.gz -o quality_filtered.fq.gz`
**Explanation:** Evaluates quality in sliding 5-base windows; any window with average quality below 15 causes read rejection, smoothing out noise.

### Perform aggressive trimming with quality cut-off
**Args:** `--qual 25 --trim-qual --front 10 --tail 10 input.fq.gz -o trimmed.fq.gz`
**Explanation:** Trims 10 bases from both ends of each read where quality falls below 25, then applies overall quality filtering.

### Generate statistics without filtering (dry-run mode)
**Args:** `--stats-only input.fq.gz`
**Explanation:** Reports read count, average quality, length distribution, and adapter content without outputting filtered files.

### Process multiple files with same parameters
**Args:** `--qual 20 --minlen 30 --outdir ./filtered *.fq.gz`
**Explanation:** Applies identical quality and length filters to all FASTQ files in the current directory, writing outputs to the specified directory.