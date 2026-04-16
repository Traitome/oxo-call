---
name: nanocomp
category: qc
description: Comparison and visualization of multiple Oxford Nanopore sequencing runs or samples
tags: [ont, nanopore, qc, comparison, visualization, reads, nanostat]
author: oxo-call built-in
source_url: "https://github.com/wdecoster/NanoComp"
---

## Concepts

- NanoComp accepts multiple BAM, FASTQ, or sequencing summary files and generates comparative plots and statistics across samples.
- Input data types are specified with --bam, --fastq, or --summary (sequencing_summary.txt from Guppy/MinKNOW).
- --names provides human-readable sample labels corresponding to input files in the same order.
- Output includes HTML report, PNG plots, and a TSV statistics table; --outdir and --prefix control where files are written.
- NanoComp compares read length distributions, quality score distributions, and yield across samples simultaneously.
- --plot violin (default) or --plot box or --plot ridge selects the plot style; ridge plots work well for many samples.
- --color sets the color scheme for plots; useful for publication-quality figures.
- --title adds a custom title to the output report.
- --dpi controls the resolution of output PNG images; default is 100, increase for higher quality.
- --hide_stats suppresses statistical test annotations on plots for cleaner visualization.

## Pitfalls

- Input files must all be the same type; mixing --bam and --fastq in one run is not supported — use a single input type consistently.
- --names count must match the number of input files exactly; mismatches cause an IndexError.
- Processing very large BAM files is slow; use --downsample to limit reads per sample for faster comparative plots.
- NanoComp reads quality from the FASTQ quality string or BAM mean_qscore tag; re-basecalled BAMs without quality tags report Q0.
- The HTML report requires a browser to view; on headless servers use --no_static to skip HTML or copy the output directory.
- Filtering by read length (--minlength, --maxlength) is applied before statistics; always report the filter thresholds used.
- --dpi higher values increase image quality but also file size; balance quality vs storage needs.
- --hide_stats removes p-value annotations but does not affect the underlying statistical comparisons.

## Examples

### compare quality and length of reads from multiple FASTQ files
**Args:** `NanoComp --fastq run1.fastq.gz run2.fastq.gz run3.fastq.gz --names Run1 Run2 Run3 --outdir nanocomp_out/ --threads 8`
**Explanation:** --fastq lists all FASTQ inputs; --names assigns labels; --outdir for output; --threads for parallel processing

### compare multiple BAM files from different samples
**Args:** `NanoComp --bam sample1.bam sample2.bam sample3.bam --names Sample1 Sample2 Sample3 --outdir bam_comparison/ --threads 8`
**Explanation:** BAM comparison extracts read length and quality from alignment records; sorted indexed BAMs are recommended

### compare runs using sequencing summary files with ridge plots
**Args:** `NanoComp --summary run1_summary.txt run2_summary.txt --names Run1 Run2 --plot ridge --outdir summary_comparison/ --threads 4`
**Explanation:** --summary uses Guppy/Dorado sequencing summary files; --plot ridge is good for visualizing overlapping distributions

### compare samples filtering out reads shorter than 1 kb
**Args:** `NanoComp --fastq *.fastq.gz --names $(ls *.fastq.gz | sed 's/.fastq.gz//') --minlength 1000 --outdir filtered_comparison/`
**Explanation:** --minlength 1000 excludes reads below 1 kb before statistics; useful for long-read focused analyses

### downsample to 50000 reads per sample for quick comparison
**Args:** `NanoComp --bam sample1.bam sample2.bam --names S1 S2 --downsample 50000 --outdir quick_compare/ --threads 4`
**Explanation:** --downsample 50000 randomly subsamples each input to 50k reads; much faster for exploratory QC

### generate comparison with custom output file prefix
**Args:** `NanoComp --fastq run1.fastq.gz run2.fastq.gz --names Run1 Run2 --outdir results/ --prefix batch01 --threads 8`
**Explanation:** --prefix prepends batch01 to all output file names; useful when running multiple NanoComp comparisons in the same directory

### generate plots with custom title and color scheme
**Args:** `NanoComp --fastq *.fastq.gz --names $(ls *.fastq.gz | sed 's/.fastq.gz//') --title "Batch Comparison" --color red --outdir titled_comparison/ --threads 8`
**Explanation:** --title adds custom report title; --color sets plot color; useful for publication-ready figures

### generate high-resolution PNG output
**Args:** `NanoComp --bam sample1.bam sample2.bam --names S1 S2 --dpi 300 --outdir high_res/ --threads 8`
**Explanation:** --dpi 300 generates high-resolution PNGs suitable for publications; increases file size

### compare without statistical annotations
**Args:** `NanoComp --fastq *.fastq.gz --names $(ls *.fastq.gz | sed 's/.fastq.gz//') --hide_stats --outdir clean_plots/ --threads 8`
**Explanation:** --hide_stats removes statistical test annotations; produces cleaner plots for presentations

### filter by both length and quality
**Args:** `NanoComp --fastq *.fastq.gz --names $(ls *.fastq.gz | sed 's/.fastq.gz//') --minlength 1000 --minqual 10 --outdir filtered_qc/ --threads 8`
**Explanation:** --minqual 10 filters reads with quality < 10; combines length and quality filtering for stringent QC
