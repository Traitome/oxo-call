---
name: nanoplot
category: qc
description: Visualization and statistics for long-read sequencing data from Oxford Nanopore and PacBio
tags: [nanopore, long-read, qc, visualization, pacbio, quality-control, statistics]
author: oxo-call built-in
source_url: "https://github.com/wdecoster/NanoPlot"
---

## Concepts

- NanoPlot generates quality statistics and plots for long-read sequencing data; input can be FASTQ, BAM, or sequencing summary.
- Use --fastq for FASTQ input; --bam for aligned BAM; --summary for ONT sequencing_summary.txt.
- Use -o for output directory; -p for file prefix; --threads for parallelism.
- Key output plots: read length histogram, quality histogram, N50 vs yield, per-read quality violin.
- Use --N50 to include N50 in the report title; --loglength for logarithmic length distribution.
- For filtered output statistics, use --minlength and --minqual to focus on high-quality reads.
- NanoStat (companion tool) provides text-based summary statistics from the same input types.

## Pitfalls

- NanoPlot is primarily a visualization tool — use NanoFilt or chopper for actual read filtering.
- For large datasets, NanoPlot can be slow — use --threads to speed up; or use NanoStat for stats only.
- The --summary input (sequencing_summary.txt) gives richer metadata than FASTQ alone.
- NanoPlot output files are named with the prefix from -p; ensure the output directory exists.
- Some plot types are only available for specific input types (e.g., summary statistics from --summary).

## Examples

### generate quality plots from Oxford Nanopore FASTQ reads
**Args:** `--fastq reads.fastq.gz -o nanoplot_output/ -p sample_qc --threads 8`
**Explanation:** --fastq input; -o output directory; -p file prefix; generates HTML report and plots

### generate quality statistics from ONT sequencing summary file
**Args:** `--summary sequencing_summary.txt -o nanoplot_summary/ -p run_qc --threads 4`
**Explanation:** --summary provides richest data including per-read timing and channel information

### plot quality statistics for aligned BAM file
**Args:** `--bam sorted.bam -o bam_qc/ -p aligned_sample --threads 8`
**Explanation:** --bam for aligned data; shows mapping quality and per-read alignment statistics

### plot quality statistics with read length and quality filters
**Args:** `--fastq reads.fastq.gz --minlength 1000 --minqual 9 -o filtered_qc/ -p hq_reads --threads 4`
**Explanation:** --minlength 1000 minimum read length; --minqual 9 minimum mean quality for displayed reads
