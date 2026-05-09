---
name: cgpbigwig
category: genomics/visualization
description: Tool for computing, manipulating, and querying BigWig genomic data files used in cancer genomics research
tags:
  - bigwig
  - wig
  - coverage
  - genomics
  - visualization
  - cgp
  - chromosome
  - bedgraph
author: AI-generated
source_url: https://github.com/cancer-genomics/cgp-bigwig
---

## Concepts

- BigWig files store dense, continuous genomic data as binary representations of coverage or signal values across chromosome coordinates. cgpbigwig operates on chromosome-specific data regions defined in theGenome browser specification, requiring valid chromosome names (e.g., chr1, chrX) that must match between input and reference files.
- The tool supports bidirectional conversion between BigWig and human-readable BedGraph formats, where BedGraph stores discrete genomic intervals with associated float values. This conversion is essential for downstream analysis tools that may only accept one format, and cgpbigwig handles the compression/decompression automatically during conversion.
- Query operations in cgpbigwig extract data from specific genomic intervals by specifying chromosome name and 1-based start/stop coordinates. The tool supports both single-interval queries and batch processing through multiple query mode, enabling efficient data extraction for targeted genomic regions without loading entire files into memory.
- Statistics computation (mean, min, max, sum, stddev) aggregates values across specified genomic intervals and outputs per-chromosome or per-window results. These statistics are critical for comparing signal distributions between samples and identifying regions with significant coverage differences in cancer genomics analyses.

## Pitfalls

- Specifying chromosome names without the required "chr" prefix (e.g., "1" instead of "chr1") causes silent failures or empty output, because cgpbigwig strictly validates chromosome names against the chromosome definition file embedded in BigWig headers.
- Attempting to query a genomic interval that extends beyond the chromosome bounds defined in the BigWig file produces an error or truncated results, leading to incomplete coverage analysis that may invalidate downstream cancer driver mutation identification.
- Using inconsistent chromosome naming conventions between multiple input files (e.g., "chr1" in one file and "1" in another) when computing comparisons or statistics causes the tool to treat data from the same chromosome as separate datasets, corrupting comparative genomics results.
- Confusing 0-based BedGraph coordinates with 1-based cgpbigwig query coordinates results in off-by-one errors in reported genomic positions, shifting all extracted features by one base pair and potentially misaligning variants with their correct genomic locations.

## Examples

### Compute BigWig coverage from a sorted BAM file
**Args:** compute -b sample.bam -o coverage.bigwig
**Explanation:** This command aligns reads in the BAM file and outputs a BigWig coverage track showing read depth at each genomic position, which is essential for identifying copy number alterations in tumor samples.

### Extract data from a specific genomic region
**Args:** query -i signal.bigwig -c chr17 -s 41240000 -e 41300000 -o chr17_region.txt
**Explanation:** This extracts all signal values within the specified BRCA1 region on chromosome 17, enabling focused analysis of this critical cancer gene without processing the entire genome.

### Convert BigWig to BedGraph format for text-based analysis
**Args:** tobedgraph -i coverage.bigwig -o coverage.bedgraph
**Explanation:** Converting binary BigWig to text BedGraph format allows inspection of individual data points and integration with awk/sed text processing pipelines for custom analyses.

### Calculate mean signal across multiple genomic windows
**Args:** stats -i H3K27ac_tumor.bigwig -c chr8 -w 1000 -m mean -o stats_output.txt
**Explanation:** Computing mean H3K27ac signal across 1kb windows on chromosome 8 identifies putative enhancers by finding regions with elevated histone acetylation mark enrichment in tumor cells.

### Batch query multiple genomic intervals from a file
**Args:** batch -i signal.bigwig -q intervals.bed -o batch_results.txt
**Explanation:** Processing multiple genomic intervals listed in a BED file in a single command line enables efficient analysis of all candidate regions without repeated individual queries, such as analyzing coverage at known cancer driver gene promoters.

### Compute standard deviation across a chromosome
**Args:** stats -i ATAC_tumor.bigwig -c chr22 -m stddev -o stddev_chr22.txt
**Explanation:** Calculating the standard deviation of ATAC-seq signal across chromosome 22 reveals chromatin accessibility heterogeneity that may indicate structural variations or open chromatin regions in cancer cells.