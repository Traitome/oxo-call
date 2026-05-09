---
name: amplicon_coverage_plot
category: Visualization
description: Plots coverage depth across amplicon regions from sequencing data, supporting per-amplicon visualization, aggregation, and summary statistics for amplicon-based sequencing assays.
tags: [coverage, amplicon, visualization, sequencing, depth, plot]
author: AI-generated
source_url: https://github.com/oxo-software/amplicon_coverage_plot
---

## Concepts

- **Input formats**: The tool accepts amplicon definitions via BED/GTF files specifying chromosomal intervals, and coverage data from indexed BAM/CRAM files or pre-computed coverage WIG files. Understanding the coordinate system (0-based for BED, 1-based for GATK-style intervals) is critical to avoid off-by-one errors in plotted regions.
- **Output formats and targets**: Plots can be rendered to PNG (raster), PDF (vector), or SVG. By default, the tool writes to stdout for piping into downstream tools, but explicit file paths via `--output` redirect rendering to disk. The `--width` and `--height` flags control figure dimensions in inches.
- **Aggregation modes**: The tool supports three coverage summary modes: raw per-base coverage (default), mean coverage across amplicon body (ignoring primer regions), and maximum coverage within the amplicon. Mode selection via `--mode` affects how depth values are computed and displayed, which is especially important for amplicon-based assays where primer effects can distort apparent depth.
- **Sample grouping and colorization**: When multiple samples are provided (multiple BAM files or a sample sheet CSV), the `--group-by` flag enables color-coding by sample name, batch, or custom label. The `--palette` flag accepts a comma-separated list of hex color codes applied in order, with automatic cycling when more groups than colors are provided.

## Pitfalls

- **Chromosome naming mismatch**: If chromosome names in the BED file (e.g., "chr1") do not match those in the BAM file header (e.g., "1" or "NC_000001.11"), the tool silently reports zero coverage for all amplicons. Always verify name consistency with `samtools idxstats` before running.
- **Missing index files**: The tool requires a `.bai` index alongside each BAM file. Attempting to plot without pre-built indices produces an error "BAM file has no index", not a silent failure. Always run `samtools index` before analysis, or use `--force` to suppress this requirement at the cost of slower random access.
- **Off-by-one errors with `--mode mean`**: The `--mode mean` calculation excludes the first and last `--padding` base pairs (default 5 bp) to avoid primer overlap. If your amplicon definitions already exclude primers, reducing `--padding` to 0 is necessary to avoid trimming legitimate coverage data.
- **Insufficient memory for large cohorts**: Each additional sample BAM file is held open simultaneously for multi-sample comparison. Providing more than 50 samples without `--streaming` may exhaust memory, resulting in a crash. Use `--streaming` to process samples in chunks or limit cohorts to smaller batches.

## Examples

### Plot coverage for a single amplicon from one BAM file
**Args:** `amplicons.bed sample.bam --output coverage.png --mode mean`
**Explanation:** This plots mean coverage across each interval in the BED file, excluding a 5-bp padding region at each end to avoid primer bias, and saves the resulting figure to a PNG file.

### Generate a multi-sample coverage comparison with custom colors
**Args:** `--bed amplicons.bed sample1.bam sample2.bam sample3.bam --group-by sample --palette #E41A1C,#377EB8,#4DAF4A --output multi_sample.pdf`
**Explanation:** This aggregates three BAM files and color-codes each sample's coverage trace in the specified palette, producing a PDF suitable for publication with distinct visual differentiation.

### Create a heatmap of coverage per amplicon across a cohort
**Args:** `--bed amplicons.bed cohort/*.bam --heatmap --output heatmap.png --zmin 10 --zmax 500`
**Explanation:** This generates a heatmap where rows are amplicons and columns are samples, with color intensity mapped to coverage depth clamped between 10x and 500x for consistent visualization across the cohort.

### Filter amplicons by minimum coverage and export to CSV
**Args:** `--bed amplicons.bed sample.bam --mode max --min-coverage 50 --csv-report filtered_report.csv`
**Explanation:** This computes maximum coverage within each amplicon, discards any amplicon with less than 50x depth, and exports the remaining intervals with depth values to a CSV file for downstream filtering pipelines.

### Plot coverage with shaded confidence intervals for replicates
**Args:** `--bed amplicons.bed rep1.bam rep2.bam rep3.bam --mode mean --error-band --output replicates.pdf`
**Explanation:** This plots mean coverage traces for three replicates and overlays a shaded band representing the standard deviation, producing a figure that visually communicates variability across biological replicates.