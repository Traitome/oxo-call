---
name: bigtools
category: genomic-data-query
description: Query and analyze BigWig and BigBed genomic track files with region-based statistical operations.
tags:
  - bigwig
  - bigbed
  - genome-browser
  - bedtools-compatible
  - region-query
  - bioinformatics
  - genomics
  - data-analysis
author: AI-Generated
source_url: https://github.com/gffutils/bigtools
---

## Concepts

- BigWig files store continuous value data (coverage, scores, signal strength) in a genome-wide binary format with B-tree indexed zoom levels enabling O(log n) random access to any genomic region, regardless of file size.
- BigBed files store discrete genomic annotations (peaks, genes, CNVs) in a binary B+-tree indexed format where each record contains chromosome, start, end, and optional score/name fields in 8-byte little-endian words.
- BigTools operates on BED-style interval notation (zero-based start, exclusive end) that differs from the one-based, inclusive-end format used by UCSC genome browser BED files—always verify coordinate conventions before comparison.
- Region queries return a vector of value tuples per chromosome position; aggregate operations (mean, sum, std) compute statistics across all bases covered by input intervals, not just the interval endpoints.
- Zoom levels in BigWig files store pre-computed summary statistics at multiple resolutions (64bp, 128bp, 256bp, etc.) allowing fast approximate queries on large genomic spans without scanning all base-pair data.

## Pitfalls

- Using one-based inclusive coordinates (UCSC style) instead of zero-based exclusive coordinates will silently return wrong genomic regions or no data at all, especially when querying at chromosome boundaries.
- Querying single-base-pair intervals without specifying strand produces strand-aggregated values, which may differ from the forward-strand-only values stored in some BigWig files created by aligners like STAR.
- Specifying a chromosome name that doesn't exactly match the chromosome identifiers in the BigWig header (e.g., "chr1" vs "1") returns empty results with no error, because chromosomes are matched as exact strings.
- Requesting zoom level queries spanning multiple pre-computed zoom levels produces inconsistent summary statistics because each zoom level uses different window sizes for aggregation.
- Forgetting that BigBed score fields are floating-point values means integer comparisons will incorrectly filter records when using threshold-based selection flags.

## Examples

### Get mean signal values across a genomic region
```
query --chrom chr22 --start 19550000 --end 19600000 --operation mean
```
**Explanation:** The query subcommand with mean operation returns the arithmetic mean of all signal values within the specified zero-based coordinates.

### Generate histogram of coverage values
```
histogram --input peaks.bw --bins 50 --min-val 0 --max-val 100
```
**Explanation:** The histogram subcommand aggregates all base-pair values into 50 equal-width bins spanning the specified value range.

### Extract BigBed feature records overlapping a region
```
query --bed --chrom chr7 --start 127000000 --end 127600000 --input annotations.bb
```
**Explanation:** Querying a BigBed file with the bed flag returns overlapping feature records using standard BED format with chromosome, start, end, name, and score fields.

### Compute standard deviation across multiple regions from BED file
```
summary --std --input signal.bigwig --bed-regions感兴趣的regions.bed
```
**Explanation:** Summary with std operation computes population standard deviation of signal values for all positions covered by the input BED regions file.

### Get minimum and maximum values in a chromosomal band
```
query --chrom chr1 --start 120000000 --end 125000000 --operation min --operation max
```
**Explanation:** Specifying multiple operation flags returns both minimum and maximum values in a single query, avoiding redundant file scans.

### Calculate total area under curve for a peak region
```
summary --sum --input chip-seq.bigwig --region chr3:178500000-178950000
```
**Explanation:** Sum operation returns the total integrated signal (sum of all base-pair values) which approximates area under curve for quantitative binding data.

### Extract zoomed summary statistics for large genomic span
```
query --chrom chr19 --start 0 --end 59000000 --zoom 4 --operation mean
```
**Explanation:** Using zoom level 4 (512bp resolution) enables fast approximate statistics on a whole chromosome arm by reading pre-computed zoom index rather than individual base pairs.