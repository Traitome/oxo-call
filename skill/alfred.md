---
name: alfred
category: bioinformatics/genomics
description: An alignment-free tool for genomic coordinate conversion, read simulation, and BAM/CRAM/VCF analysis with subcommands for GPS coordinate estimation, statistics, and data manipulation.
tags:
  - alignment-free
  - genomics
  - BAM
  - CRAM
  - VCF
  - coordinate-conversion
  - read-simulation
  - statistics
author: AI-generated
source_url: https://github.com/tobiasrausch/alfred
---

## Concepts

- **Multi-subcommand architecture**: Alfred provides distinct subcommands (`alfred gps`, `alfred stats`, `alfred convert`, etc.) for specialized tasks. Each subcommand has its own set of flags and expected input formats. Always identify the correct subcommand for your task before invocation.

- **Genomic coordinate systems**: Alfred handles both linear genomic coordinates (chromosome:position) and geographic coordinate estimation from allele frequencies. The `gps` subcommand converts genomic positions into approximate geographic coordinates using population genetics models, requiring a variant call file (VCF) as input.

- **Compressed alignment formats**: Alfred reads BAM and CRAM files natively, supporting both sorted and unsorted inputs. CRAM files offer smaller storage footprints but require additional reference sequences. The tool automatically detects the format from file extensions (.bam, .cram).

- **Output format flexibility**: Alfred produces outputs in multiple formats including TSV, JSON, and binary formats depending on the subcommand. The `stats` subcommand generates tab-separated statistics reports, while the `gps` subcommand outputs geographic coordinates that can be mapped directly.

## Pitfalls

- **Using the wrong subcommand for your operation**: Alfred is not a monolithic tool; each subcommand performs a specific function. Attempting to calculate statistics using `alfred gps` or vice versa will result in errors. Always verify your subcommand matches your analytical goal.

- **Ignoring sorting requirements**: Many Alfred subcommands require coordinate-sorted BAM/CRAM files. Running `alfred stats` on an unsorted file may produce incorrect or incomplete statistics, silently skipping records that are not properly ordered.

- **Missing reference genome for CRAM files**: When processing CRAM files, Alfred requires the exact reference genome used during alignment. Failing to provide the correct reference with `-r` causes the tool to fail with cryptic decompression errors rather than a clear reference-missing message.

- **Insufficient memory for large cohorts**: The `gps` subcommand loads entire VCF datasets into memory for coordinate estimation. For cohort sizes exceeding available RAM, the tool will crash with out-of-memory errors without partial processing options.

- **Incorrect chromosome naming conventions**: Alfred expects chromosome names matching the reference genome index (e.g., chr1 vs 1). Mismatched naming between input files and reference causes silent failures where no records are processed.

## Examples

### Compute alignment statistics from a coordinate-sorted BAM file
**Args:** `stats -b sample.sorted.bam`
**Explanation:** The stats subcommand calculates read depth, mapping quality distributions, and coverage metrics from a sorted BAM file, outputting tab-separated statistics to stdout.

### Estimate GPS coordinates from genomic variants
**Args:** `gps -v variants.vcf.gz -r GRCh38.fa`
**Explanation:** The gps subcommand converts genomic variant positions into approximate geographic coordinates by analyzing allele frequency patterns against population reference panels.

### Extract specific genomic regions from a BAM file
**Args:** `convert -b input.bam -c chr1:1000000-2000000`
**Explanation:** The convert subcommand with the `-c` flag extracts reads overlapping the specified genomic interval, outputting a new BAM file containing only region-filtered alignments.

### Process paired-end reads and compute insert size metrics
**Args:** `stats -b paired.sorted.bam -m insertsize`
**Explanation:** Adding the insertsize flag to the stats subcommand calculates proper pair orientations and insert size distributions, critical for quality control in paired-end sequencing experiments.

### Batch process multiple BAM files with a manifest file
**Args:** `stats -m batch.tsv`
**Explanation:** The manifest flag `-m` accepts a tab-separated file listing multiple BAM paths, enabling batch processing of statistics across many samples in a single invocation.