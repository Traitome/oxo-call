---
name: bam2plot
category: visualization
description: Generate graphical plots from BAM alignment files including coverage tracks, read depth histograms, and genomic window visualizations.
tags: bam, visualization, coverage, genomics, alignment, plots
author: AI-generated
source_url: https://github.com/bam2plot/bam2plot
---

## Concepts

- **Input Format**: bam2plot accepts coordinate-sorted and indexed BAM files (.bam with .bai index), or uncompressed SAM files. Files must contain aligned reads with valid CIGAR strings and reference coordinate information.
- **Output Formats**: Generated plots can be exported as PNG (default), PDF, SVG, or EPS. The tool supports multiple plot types including coverage depth, per-base read quality, and genomic browser-style tracks.
- **Region Specification**: Users can restrict analysis to specific genomic regions using 1-based coordinates (chr:start-end notation). Without region specification, plots are generated for the entire referenced sequence.
- **Data Resolution**: The tool supports downsampling and windowing to manage memory for large BAM files. The `--window-size` and `--step-size` flags control the binning for coverage calculations.

## Pitfalls

- **Using Unsorted or Unindexed BAM Files**: Providing BAM files that are not coordinate-sorted or lack corresponding .bai index files will cause the tool to fail with an alignment retrieval error. Always sort with samtools sort and index with samtools index before processing.
- **Requesting Regions Without Valid Reference Names**: Specifying a chromosome name that does not exist in the BAM header (e.g., "chr1" when the file uses "1") results in an empty output with no data. Use `samtools idxstats` to verify available reference names first.
- **Memory Exhaustion with Whole-Genome Plots**: Generating coverage plots for entire genomes without downsampling can consume excessive memory. Use `--window-size` to bin data or `--max-depth` to cap read pileup memory.
- **Ambiguous CIGAR Handling**: Alignments with ambiguous or complex CIGAR strings (e.g., containing N operations for introns) may be counted incorrectly as gaps in coverage. Review CIGAR string interpretation with the `--cigar-mode` flag.

## Examples

### Generate a basic coverage plot for a genomic region
**Args:** `input.bam --region chr1:1000000-2000000 --output coverage.png --plot-type coverage`
**Explanation:** This reads alignments from chr1 positions 1-2 Mb and writes a coverage depth track in PNG format. The tool automatically calculates per-base read depth and renders it as a line plot.

### Create a read depth histogram with logarithmic scaling
**Args:** `input.bam --region chr3:500000-6000000 --output depth_hist.png --plot-type histogram --log-scale`
**Explanation:** The `--log-scale` flag applies logarithmic transformation to the depth values before plotting, useful for visualizing large dynamic ranges where low-coverage regions would otherwise be invisible.

### Export a multi-track plot showing coverage and base quality
**Args:** `input.bam --region chr2:1-100000 --output combined.png --plot-type multi --tracks coverage,quality`
**Explanation:** This generates a composite figure with two vertically stacked tracks: coverage depth and per-base quality scores. The `--tracks` flag accepts comma-separated track types.

### Downsample large BAM files for quick preview visualization
**Args:** `input.bam --region chr1:1-10000000 --output preview.png --max-reads 100000 --random-seed 42`
**Explanation:** The `--max-reads` flag randomly samples alignments up to the specified count, enabling rapid generation of representative plots without processing the full file.

### Generate PDF output for publication-quality figures
**Args:** `input.bam --region chr4:5000000-8000000 --output figure1.pdf --plot-type coverage --resolution 300`
**Explanation:** Writing to PDF at 300 DPI produces vector-scalable output suitable for publication. The resolution flag only affects rasterized elements within the PDF.

### Use windowed binning to reduce memory usage
**Args:** `input.bam --region chr5:1-50000000 --output binned.png --plot-type coverage --window-size 1000 --step-size 500`
**Explanation:** The `--window-size` aggregates coverage into 1 kb bins, and `--step-size` creates 500 bp overlapping windows. This drastically reduces memory for whole-chromosome plots while preserving resolution.