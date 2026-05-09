---
name: bedtk
category: Bioinformatics - Interval Operations
description: A high-performance BEDTools-compatible toolkit written in Rust for genomic interval operations. Provides faster implementations of intersect, union, merge, subtract, closest, coverage, and window operations on BED format files.
tags: [genomics, intervals, BED, overlap, merge, coverage, bedtools-compatible, rust]
author: AI-generated
source_url: https://github.com/arneb/bedtk
---

## Concepts

- **BED format conventions**: bedtk follows standard BED format where coordinates are 0-based and half-open (start is inclusive, end is exclusive) by default, matching the BEDTools convention. Ensure input files have correct chrom, start, end columns at minimum.
- **Stranded operations**: Use the `-s` flag to require same-strand overlap or `-S` to require opposite-strand overlap when computing intersections or finding closest intervals. This is critical for RNA-seq or directional chip-seq analysis where strand specificity matters.
- **Overlap fraction requirements**: The `-f` flag specifies the minimum fraction of overlap required for a feature to be considered a match (e.g., `-f 0.5` requires 50% overlap). The `-r` flag can be combined to require reciprocal overlap where both features must overlap each other by the specified fraction.
- **Output modes**: bedtk supports multiple output modes including `-wa` (write all A features even without overlaps), `-wb` (write both A and B columns in overlap), and `-wo` (write only the overlapping base pairs) for detailed analysis of overlap extent.

## Pitfalls

- **Coordinate system confusion**: Mixing 0-based half-open coordinates with 1-based inclusive coordinates is a common error. bedtk uses BEDTools convention (0-based), while standard UCSC genome browser uses 1-based. Always verify coordinate systems match between files before running operations.
- **Forgetting to sort inputs**: bedtk requires input BED files to be sorted by chromosome and start position. Unsorted files produce incorrect or no results. Pre-sort files using `bedtk sort` or `sort -k1,1 -k2,2n` before operations.
- **Missing required positional arguments**: The `-a` and `-b` flags for specifying input files are positional in some bedtk subcommands, meaning the order matters. Passing files in the wrong order will swap which file is treated as the query versus database.
- **Insufficient overlap settings**: Running intersect without specifying `-f` can report any single-base overlap, which may inflate false positives in analysis requiring meaningful genomic overlap. Always consider appropriate overlap thresholds for your biological question.

## Examples

### Find intersections between two BED files
**Args:** `intersect -a peaks.bed -b enhancers.bed -wa -wb`
**Explanation:** Reports all peak records that overlap any enhancer, including both the full peak and enhancer records in the output for downstream analysis.

### Merge overlapping intervals in a single file
**Args:** `merge -i cpg_islands.bed`
**Explanation:** Combines all overlapping and adjacent intervals in the CpG island file into single consolidated regions, reducing redundancy.

### Calculate coverage of reads over gene exons
**Args:** `coverage -a genes.bed -b reads.bed`
**Explanation:** Computes for each gene the number of reads and fractional base coverage, outputting the standard BEDTools coverage format with coverage statistics.

### Find closest genes to peak locations
**Args:** `closest -a peaks.bed -b genes.bed -D b`
**Explanation:** Identifies the nearest gene to each peak, ordering results by distance and reporting the distance value in base pairs in the output.

### Get base-pair level overlap details
**Args:** `intersect -a chip_peaks.bed -b dnase_peaks.bed -wo`
**Explanation:** Reports only the exact number of overlapping base pairs between each peak pair, useful for quantifying shared regulatory regions.

### Require reciprocal 50% overlap between features
**Args:** `intersect -a tf_peaks.bed -b histone_marks.bed -f 0.5 -r`
**Explanation:** Reports only mutual overlaps where both the TF peak and histone mark overlap each other by at least 50% of their combined length.

### Subtract introns from a ChIP-seq dataset
**Args:** `subtract -a chip_peaks.bed -b introns.bed`
**Explanation:** Removes any portion of ChIP peaks that overlap intron regions, leaving only intergenic or exonic binding events for analysis.

### Compute coverage with per-base detail output
**Args:** `coverage -a exons.bed -b reads.bed -d`
**Explanation:** Outputs coverage per exon in per-base detail format rather than summary statistics, enabling base-level heatmap generation.