---
name: brooklyn_plot
category: Visualization
description: A bioinformatics tool for generating publication-quality plots from genomic data. Supports various output formats including PNG, PDF, and SVG. Can visualize coverage profiles, variant calls, genomic intervals, and alignment data.
tags: [visualization, genomics, plotting, coverage, variants, output-formats]
author: AI-generated
source_url: https://github.com/brooklyn-tools/brooklyn_plot
---

## Concepts

- **Input formats**: Accepts standard bioinformatics formats including BAM/CRAM for alignments, VCF for variants, BED for genomic intervals, and coverage WIG files. The tool automatically detects format based on file extension.
- **Output formats**: Generates raster (PNG, TIFF) or vector (PDF, SVG) output. Vector formats are recommended for publications as they scale without quality loss.
- **Data normalization**: Coverage data is normalized to reads per million (RPM) by default when using `--normalize` flag. This enables comparison across samples with different sequencing depths.
- **Region specification**: Use 1-based genomic coordinates with `--region chrom:start-end`. The tool uses inclusive end coordinates, matching standard BED format conventions.
- **Visualization modes**: Supports overlay (multiple tracks stacked), joint (shared y-axis), and mirror (inverted signal) visualization modes for comparing multiple samples.

## Pitfalls

- **Mismatched chromosome names**: If chromosome names in your input files don't match the reference (e.g., "chr1" vs "1"), the tool will produce empty plots without warnings. Always verify chromosome naming consistency before running.
- **Memory consumption with large files**: For genome-wide BAM files, memory usage scales with file size. Processing whole-genome data without downsampling may cause out-of-memory errors on systems with limited RAM.
- **Incorrect coordinate ordering**: Providing start coordinate greater than end (e.g., `--region chr1:1000-500`) silently defaults to full chromosome instead of erroring. Always verify coordinate ordering.
- **Missing index files**: BAM/CRAM files require corresponding .bai/.crai index files in the same directory. Without indices, the tool will fail with a file access error.
- **Non-numeric genomic coordinates**: Providing coordinates with thousand-separators (e.g., "1,000,000") causes parsing failures. Use plain integers without commas.

## Examples

### Generate a coverage plot from a BAM file
**Args:** `--input sample.bam --output coverage.png --region chr1:1000000-2000000 --type coverage`
**Explanation:** Creates a coverage profile visualization for a specific genomic region using default parameters and PNG output format.

### Create a PDF plot with normalized RPM values
**Args:** `--input alignment.bam --output normalized_plot.pdf --normalize RPM --type coverage`
**Explanation:** Normalizes coverage to reads per million to enable comparison across differently sequenced samples, outputting vector PDF.

### Visualize variant alleles as a track
**Args:** `--input variants.vcf --output variants.png --region chr2:50000000-51000000 --type variants --min-qual 30`
**Explanation:** Plots variant calls filtered by minimum quality score (Phred score 30) within the specified genomic window.

### Overlay multiple coverage tracks
**Args:** `--input sample1.bam sample2.bam --output overlay.png --region chr3:1-1000000 --type coverage --mode overlay`
**Explanation:** Stacks multiple sample coverage profiles on the same plot for direct visual comparison of read depth across samples.

### Generate mirror plot for structural variation visualization
**Args:** `--input tumor.bam --output mirror_plot.pdf --region chr17:30000000-35000000 --type coverage --mode mirror`
**Explanation:** Creates a mirror-mode plot where positive and negative signals are displayed on opposite sides of the axis.