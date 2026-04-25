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
- Input types: --fastq, --bam, --ubam (unaligned BAM), --summary (ONT sequencing_summary.txt), --fasta, --cram, --feather.
- NanoStat outputs a human-readable text summary to stdout.
- Use -t N for multi-threading; --name for a sample label in the output.
- NanoStat is complementary to NanoPlot — NanoStat for statistics, NanoPlot for visualizations.
- For batched analysis of multiple samples, pipe NanoStat output to files and compare.
- --tsv outputs statistics in TSV format for easier parsing and downstream analysis.
- --barcoded splits summary statistics by barcode for multiplexed ONT runs.
- --readtype {1D,2D,1D2} filters ONT summary by read type (1D=single, 2D=2D chemistry, 1D2=1D²).
- --no_supplementary excludes supplementary alignments from BAM statistics.
- -o and -p specify output directory and prefix for saving stats to file instead of stdout.

## Pitfalls
- NanoStat is text output only — for plots, use NanoPlot.
- FASTQ input requires Phred quality encoding — check if quality scores are present in the FASTQ.
- NanoStat on BAM uses alignment quality, not raw basecall quality — results differ from FASTQ mode.
- --tsv format is easier for programmatic parsing than default text output.
- --barcoded requires barcoded summary files; will fail on non-barcoded data.
- Multiple input files are supported; statistics are calculated across all inputs combined.
- --no_supplementary only affects BAM/CRAM input; has no effect on FASTQ/summary input.

## Examples

### get statistics for ONT FASTQ reads
**Args:** `--fastq reads.fastq.gz -t 4 --name sample_name`
**Explanation:** NanoStat command; --fastq reads.fastq.gz input FASTQ; -t 4 threads; --name sample_name label for output

### get statistics from ONT sequencing summary
**Args:** `--summary sequencing_summary.txt -t 4 --name run_qc`
**Explanation:** NanoStat command; --summary sequencing_summary.txt input ONT summary; -t 4 threads; --name run_qc label for output

### get statistics from aligned BAM file
**Args:** `--bam aligned_sorted.bam -t 4 --name aligned_stats`
**Explanation:** NanoStat command; --bam aligned_sorted.bam input BAM; -t 4 threads; --name aligned_stats label for output

### output statistics in TSV format for parsing
**Args:** `--fastq reads.fastq.gz -t 4 --tsv --name sample_stats`
**Explanation:** NanoStat command; --fastq reads.fastq.gz input FASTQ; -t 4 threads; --tsv tab-separated output; --name sample_stats label

### get statistics for barcoded ONT run
**Args:** `--summary sequencing_summary.txt --barcoded -t 4 --name barcoded_stats`
**Explanation:** NanoStat command; --summary sequencing_summary.txt input ONT summary; --barcoded splits stats by barcode; -t 4 threads; --name barcoded_stats label

### filter ONT summary by read type (1D2)
**Args:** `--summary sequencing_summary.txt --readtype 1D2 -t 4 --name 1d2_stats`
**Explanation:** NanoStat command; --summary sequencing_summary.txt input ONT summary; --readtype 1D2 filters for 1D² reads; -t 4 threads; --name 1d2_stats label

### exclude supplementary alignments from BAM stats
**Args:** `--bam aligned.bam --no_supplementary -t 4 --name primary_only`
**Explanation:** NanoStat command; --bam aligned.bam input BAM; --no_supplementary excludes supplementary alignments; -t 4 threads; --name primary_only label

### save statistics to file with prefix
**Args:** `--fastq reads.fastq.gz -o stats_dir/ -p sample1 -t 4`
**Explanation:** NanoStat command; --fastq reads.fastq.gz input FASTQ; -o stats_dir/ output directory; -p sample1 file prefix; -t 4 threads

### get statistics from FASTA file (no quality)
**Args:** `--fasta reads.fasta.gz -t 4 --name fasta_stats`
**Explanation:** NanoStat command; --fasta reads.fasta.gz input FASTA; -t 4 threads; --name fasta_stats label for output

### process multiple summary files together
**Args:** `--summary run1_summary.txt run2_summary.txt run3_summary.txt -t 8 --name combined`
**Explanation:** NanoStat command; --summary run1_summary.txt run2_summary.txt run3_summary.txt input ONT summaries; -t 8 threads; --name combined label; statistics calculated across all
