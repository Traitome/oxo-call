---
name: nanostat
category: qc
description: Calculates statistics for long-read sequencing datasets from Oxford Nanopore and PacBio
tags: [nanopore, pacbio, long-read, qc, statistics, quality-control, ont]
author: oxo-call built-in
source_url: "https://github.com/wdecoster/nanostat"
---

## Concepts

- NanoStat provides summary statistics for long-read data: N50, mean/median quality, total bases, read counts.
- Input types: --fastq, --bam, --ubam (unaligned BAM), --summary (ONT sequencing_summary.txt).
- NanoStat outputs a human-readable text summary to stdout.
- Use -t N for multi-threading; --name for a sample label in the output.
- NanoStat is complementary to NanoPlot — NanoStat for statistics, NanoPlot for visualizations.
- For batched analysis of multiple samples, pipe NanoStat output to files and compare.

## Pitfalls

- NanoStat is text output only — for plots, use NanoPlot.
- FASTQ input requires Phred quality encoding — check if quality scores are present in the FASTQ.
- NanoStat on BAM uses alignment quality, not raw basecall quality — results differ from FASTQ mode.

## Examples

### get statistics for ONT FASTQ reads
**Args:** `--fastq reads.fastq.gz -t 4 --name sample_name`
**Explanation:** --fastq input; -t 4 threads; --name label for output report

### get statistics from ONT sequencing summary
**Args:** `--summary sequencing_summary.txt -t 4 --name run_qc`
**Explanation:** --summary provides timing and per-read statistics from ONT MinKNOW

### get statistics from aligned BAM file
**Args:** `--bam aligned_sorted.bam -t 4 --name aligned_stats`
**Explanation:** --bam for aligned long-read BAM; shows alignment-specific statistics
