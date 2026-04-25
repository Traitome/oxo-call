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
- --plots controls which plots to generate: dot (default), kde, hex, pauvre, or none.
- --maxlength filters reads longer than specified; useful for excluding ultra-long outliers.
- --color sets the color scheme for plots; default is '#4CB391' (teal).
- --format sets output image format: png (default), jpg, svg, or pdf.
- --tsv_stats outputs statistics in TSV format for downstream parsing.

## Pitfalls
- NanoPlot is primarily a visualization tool — use NanoFilt or chopper for actual read filtering.
- For large datasets, NanoPlot can be slow — use --threads to speed up; or use NanoStat for stats only.
- The --summary input (sequencing_summary.txt) gives richer metadata than FASTQ alone.
- NanoPlot output files are named with the prefix from -p; ensure the output directory exists.
- Some plot types are only available for specific input types (e.g., summary statistics from --summary).
- --plots kde and hex can be slow for very large datasets; use dot for faster processing.
- --loglength is recommended for datasets with wide length distributions; linear scale may hide details.
- --maxlength without --minlength may exclude valid long reads; use both for targeted length range.
- Output directory must exist before running; NanoPlot does not create it automatically.

## Examples

### generate quality plots from Oxford Nanopore FASTQ reads
**Args:** `--fastq reads.fastq.gz -o nanoplot_output/ -p sample_qc --threads 8`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; -o nanoplot_output/ output directory; -p sample_qc file prefix; --threads 8 parallel

### generate quality statistics from ONT sequencing summary file
**Args:** `--summary sequencing_summary.txt -o nanoplot_summary/ -p run_qc --threads 4`
**Explanation:** NanoPlot command; --summary sequencing_summary.txt input ONT summary; -o nanoplot_summary/ output directory; -p run_qc file prefix; --threads 4 parallel

### plot quality statistics for aligned BAM file
**Args:** `--bam sorted.bam -o bam_qc/ -p aligned_sample --threads 8`
**Explanation:** NanoPlot command; --bam sorted.bam input BAM; -o bam_qc/ output directory; -p aligned_sample file prefix; --threads 8 parallel

### plot quality statistics with read length and quality filters
**Args:** `--fastq reads.fastq.gz --minlength 1000 --minqual 9 -o filtered_qc/ -p hq_reads --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --minlength 1000 minimum read length; --minqual 9 minimum mean quality; -o filtered_qc/ output; -p hq_reads prefix; --threads 4 parallel

### generate hexbin plots for large datasets
**Args:** `--fastq reads.fastq.gz --plots hex -o hex_qc/ -p hex_plots --threads 8`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --plots hex hexagonal binning plots; -o hex_qc/ output directory; -p hex_plots prefix; --threads 8 parallel

### plot with logarithmic length scale
**Args:** `--fastq reads.fastq.gz --loglength -o log_qc/ -p log_scale --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --loglength log scale for length axis; -o log_qc/ output directory; -p log_scale prefix; --threads 4 parallel

### filter reads by maximum length to exclude outliers
**Args:** `--fastq reads.fastq.gz --minlength 500 --maxlength 50000 -o range_qc/ -p length_filtered --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --minlength 500 minimum length; --maxlength 50000 maximum length; -o range_qc/ output directory; -p length_filtered prefix; --threads 4 parallel

### output statistics in TSV format for parsing
**Args:** `--fastq reads.fastq.gz --tsv_stats -o tsv_qc/ -p stats --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --tsv_stats outputs statistics as TSV; -o tsv_qc/ output directory; -p stats prefix; --threads 4 parallel

### generate plots in PDF format for publications
**Args:** `--fastq reads.fastq.gz --format pdf -o pdf_qc/ -p publication --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --format pdf vector graphics output; -o pdf_qc/ output directory; -p publication prefix; --threads 4 parallel

### use custom color for plots
**Args:** `--fastq reads.fastq.gz --color '#FF6B6B' -o colored_qc/ -p red_plots --threads 4`
**Explanation:** NanoPlot command; --fastq reads.fastq.gz input FASTQ; --color '#FF6B6B' custom plot color; -o colored_qc/ output directory; -p red_plots prefix; --threads 4 parallel
