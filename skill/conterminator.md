---
name: Conterminator
category: Sequence Processing
description: A versatile tool for trimming, masking, and manipulating sequence termini in FASTA and FASTQ files, supporting adapter trimming, quality-based trimming, and end-specific sequence operations.
tags:
  - sequence-processing
  - trimming
  - quality-control
  - fastq
  - fasta
  - adapters
  - bioinformatics
author: AI-Generated
source_-url: https://github.com/bigbio/conterminator
---

## Concepts

- **Input formats**: Conterminator reads raw or gzipped FASTA (`*.fasta`, `*.fa`) and FASTQ (`*.fastq`, `*.fq`) files, auto-detecting the format by file extension and the presence of quality strings. Output format is inferred from the input but can be overridden with `--out-format`.
- **Trimming modes**: Three primary trimming strategies exist — `adapter` mode aligns a provided adapter sequence to each read's termini and removes matching bases; `quality` mode trims bases from read ends whose quality scores fall below a per-base threshold; and `length` mode enforces a minimum or maximum total read length after any other operation.
- **Strand-awareness**: By default, Conterminator trims both the 5' and 3' termini equally. The `--ends` flag restricts trimming to only one side: `5` trims the left end, `3` trims the right end, and `both` (the default) trims symmetrically.
- **Output modes**: Reads can be written to a new file (`--output`), prepended/stripped with a prefix (`--prefix`, `--strip-prefix`), or filtered out entirely when they fall below a length or quality threshold after trimming. Discarded reads are redirected to `--discarded` if that flag is provided.
- **Paired-end mode**: When two input files are supplied with `--pe-mode`, Conterminator processes read pairs in sync, discarding or writing both reads if either read becomes too short, ensuring consistent pairing in the output.

## Pitfalls

- **Mismatched paired-end files**: Supplying files with different read counts in `--pe-mode` causes Conterminator to abort with a parsing error, but only after loading both files into memory. Always validate read counts with `wc -l` before processing to avoid wasted compute time.
- **Conflicting trim parameters**: Using `--trim-adapter` and `--trim-quality` in the same invocation applies both operations sequentially, which can shrink reads to near-zero length or eliminate them entirely, especially with short input sequences. Check post-trim lengths with `--report-lengths` before committing to full-scale processing.
- **Missing gzipped file detection**: On some filesystems, `.gz` extension case-sensitivity issues (e.g., `.GZ`) cause Conterminator to fall back to plain-text parsing, silently producing garbled output. Always verify file extensions are lowercase.
- **Quality score offset mismatch**: The default quality offset is 33 (Sanger/Illumina 1.8+). If your FASTQ uses Illumina 1.5+ with offset 64, you must set `--qual-offset 64`; otherwise `--trim-quality` produces incorrect trimmed sequences and may discard valid reads.

## Examples

### Trim 3' adapter sequence from single-end reads
**Args:** `input.fastq.gz --trim-adapter AGATCGGAAGAGC --output trimmed.fastq.gz`
**Explanation:** This trims the `AGATCGGAAGAGC` TruSeq adapter from the 3' end of all reads in the input FASTQ file and writes the cleaned reads to a new gzipped output file.

### Quality-based trimming at a phred threshold of 20 from both ends
**Args:** `input.fq --trim-quality 20 --ends both --output clean.fq`
**Explanation:** This removes any trailing (and leading) bases with quality scores below phred 20 from both the 5' and 3' termini of each read, preserving only high-confidence bases in the output.

### Trim left (5') end only and enforce minimum read length
**Args:** `input_R1.fq.gz --trim-adapter TGACTGGAGTTC --ends 5 --min-length 36 --output R1_trimmed.fq.gz`
**Explanation:** This removes the specified adapter sequence exclusively from the 5' end and discards any reads shorter than 36 bases after trimming, ensuring only full-length, informative reads remain.

### Paired-end processing with sync discard
**Args:** `R1.fq.gz R2.fq.gz --pe-mode sync --trim-quality 25 --min-length 50 --discarded dropped.fq.gz --output paired_clean/`
**Explanation:** This processes both FASTQ files in paired mode, applying quality trimming to each read independently while discarding both mates if either falls below 50 bases after trimming, writing all discards to a separate file.

### Mask both termini with an N-run and write length report
**Args:** `genomic_reads.fa --mask-both --mask-char N --mask-length 10 --report-lengths lengths.txt --output masked_reads.fa`
**Explanation:** This replaces 10 bases at each end of every sequence with `N` characters for downstream repeat masking pipelines and generates a per-read length summary file for quality auditing.