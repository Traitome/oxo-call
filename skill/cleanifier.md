---
name: cleanifier
category: preprocessing
description: A command-line tool for cleaning FASTQ files by trimming adapters, filtering low-quality reads, removing contaminants, and applying length filters. Operates on unpaired or paired-end data with configurable quality thresholds and output formats.
tags:
  - ngs
  - preprocessing
  - quality-control
  - trimming
  - fastq
  - adapters
author: AI-generated
source_url: https://github.com/cleanifier/cleanifier
---

## Concepts

- **Input formats:** Cleanifier accepts uncompressed or gzipped FASTQ files (`.fastq`, `.fq`, `.fastq.gz`, `.fq.gz`) for single-end reads, and two paired-end files with matching read names and order.
- **Quality encoding:** Uses Phred+33 (Sanger) quality encoding by default; use `--quality-offset 64` for Phred+64 (Illumina 1.3-1.7) encoded files.
- **Trimming operations:** Removes adapter sequences at read ends using exact matching or configurable error tolerance, cuts terminals below quality thresholds, and strips Ns from ends.
- **Filtering rules:** Filters reads based on minimum length (`-m`), maximum length (`-M`), minimum mean quality (`-q`), and presence of ambiguous bases (`-n`).
- **Output modes:** Writes cleaned reads to stdout (for piping) or to specified output files; supports gzip compression when output filename ends in `.gz`.

## Pitfalls

- **Using incompatible quality offset:** Specifying the wrong quality encoding causes all bases to be evaluated incorrectly, resulting in either over-filtering (Phred+33 data with offset 64) or under-filtering (Illumina data with offset 33), corrupting downstream analysis.
- **Forgetting paired-end synchronization:** When processing paired-end files without `--pair`, read pairs become misaligned after filtering, causing downstream tools to fail or produce incorrect results.
- **Outputting to stdin instead of file:** Forgetting to specify `-o` or stdout redirection when processing large files fills up disk and crashes the pipeline.
- **Ignoring read name format requirements:** Paired-end input files must have identical read names (differing only by `/1` or `/2` suffixes), otherwise pairs are not matched correctly.
- **Setting thresholds too aggressively:** Using `-q 30` with `-m 75` on short reads (e.g., 50 bp) filters out most or all reads, leaving insufficient data for analysis.

## Examples

### Trim adapters from single-end FASTQ file using default settings
**Args:** `-a AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC input.fq.gz -o clean_input.fq.gz`
**Explanation:** Removes Illumina universal adapter sequences from the 3' end of reads using exact matching, writing cleaned output to the specified file.

### Filter paired-end reads by minimum quality and length
**Args:** `--pair -1 read1.fq.gz -2 read2.fq.gz -q 20 -m 50 -o clean_pair1.fq.gz -o2 clean_pair2.fq.gz`
**Explanation:** Filters both reads in each pair to require minimum Phred quality of 20 and minimum length of 50 bp, outputting only read pairs meeting both criteria.

### Cut low-quality bases from 3' ends before filtering
**Args:** `-i input.fq.gz -q 15 --cut-fail --gzip -o output.fq.gz`
**Explanation:** Cuts terminal bases from the 3' end until a base with quality ≥15 is encountered, then filters out reads that become too short after cutting.

### Remove reads containing ambiguous bases (Ns) from paired-end data
**Args:** `--pair -1 r1.fq -2 r2.fq -n 0 -o clean_r1.fq -o2 clean_r2.fq`
**Explanation:** Removes any read pairs where either read contains any ambiguous 'N' bases, ensuring all retained reads have complete base calls.

### Process very large files using multiple threads for faster output
**Args:** `-a AATGATACGGCGACCACCGAGATACACATGGGATCTTGGA input_R1.fastq.gz -o clean_R1.fq.gz --threads 8`
**Explanation:** Uses 8 parallel threads to clean a large FASTQ file with 8 additional bases of adapter sequence, significantly speeding up processing on multi-core systems.

### Strip adapters and filter to specific length range for miRNA-seq
**Args:** `-a TGGAATTCTCGGGTGCCAAGG --minimum-length 18 --maximum-length 30 input.fq.gz -o filtered.fq.gz`
**Explanation:** Removes small RNA adapter sequences from 3' ends and retains only reads between 18-30 bp, matching typical miRNA length requirements for downstream miRNA analysis.