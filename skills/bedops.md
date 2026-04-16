---
name: bedops
category: genomic-intervals
description: Fast and highly scalable set operations and arithmetic on BED, GFF, VCF, and SAM genomic intervals
tags: [bed, intervals, genomic-arithmetic, intersection, union, complement, sort, starch, bedmap, closest-features]
author: oxo-call built-in
source_url: "https://bedops.readthedocs.io/"
---

## Concepts

- BEDOPS requires input files to be sorted with sort-bed; unsorted input causes immediate failure with a clear error message.
- bedops performs set operations: --intersect (-i), --union (-u/--everything), --difference (-d), --complement (-c), --symmdiff (-s), --element-of (-e), --not-element-of (-n), --merge (-m), --partition (-p), --chop (-w) on sorted BED files.
- bedmap maps signals from one BED file onto intervals of another, computing statistics (--sum, --mean, --median, --max, --min, --count, --stdev, --cv, --wmean, --bases, --indicator) per element.
- starch is BEDOPS's compressed binary format; unstarch decompresses it; starchcat concatenates starch archives.
- BEDOPS is strand-unaware by default; it works on coordinate intervals only, making it very fast but requiring strand filtering upstream.
- bedextract rapidly retrieves all intervals overlapping a given region from a sorted BED or starch file without loading the whole file.
- closest-features finds the nearest element in a query file for each element in an input file, reporting left/right distances.
- --range L:R pads or shrinks coordinates before the operation; with -e/-n, the first (reference) file is NOT padded.
- --chop (-w) splits intervals into windows of a specified size; --stagger offsets starting positions; --partition divides into non-overlapping sub-intervals.
- Format converters: vcf2bed, gff2bed, gtf2bed, bam2bed, sam2bed, wig2bed, psl2bed convert common formats to sorted BED for BEDOPS processing.
- bedmap --echo outputs the reference interval; --echo-map outputs all overlapping map intervals; combine with --delim '\t' for tab-separated output.

## Pitfalls

- CRITICAL: bedops ARGS must start with an operation flag (--intersect, --union, --difference, --complement, --symmdiff, --element-of, --not-element-of, --merge, --partition, --chop, --everything) — never with input files first. The operation flag ALWAYS comes before input files. Other BEDOPS tools are separate binaries: sort-bed, bedmap, bedextract, closest-features, starch, unstarch, starchcat, vcf2bed, gff2bed, gtf2bed, etc.
- All inputs to bedops and bedmap must be sorted with sort-bed; BED files sorted by other tools (e.g., `sort -k1,1 -k2,2n`) may not be compatible with BEDOPS's lexicographic chromosome ordering.
- bedops --complement requires chromosome sizes to compute regions not covered; without --chrom-sizes the complement will be wrong.
- bedmap --echo --mean returns one output line per reference interval, not per overlap — understand the mapping model before use.
- BEDOPS uses half-open intervals [start, end) like BED; confirm input tools also use 0-based half-open coordinates.
- Very large union operations can produce files with billions of lines; use starch to compress output immediately after computation.
- bedmap --fraction-ref and --fraction-map control overlap thresholds; confusing them leads to incorrect overlap filtering.
- --element-of (-e) defaults to 100% overlap; use `-e 1` for 1bp overlap or `-e 50%` for 50% overlap — the default is very strict.
- bedops operations other than -e/-n/-u flatten output to 3-column BED (chr, start, end); use -e/-n/-u to preserve all columns.
- sort-bed uses lexicographic chromosome ordering (chr1, chr10, chr2) unlike `sort -k1,1 -k2,2n` which sorts chr1, chr2, chr10.

## Examples

### sort a BED file for use with BEDOPS tools
**Args:** `sort-bed input.bed > input.sorted.bed`
**Explanation:** sort-bed is required before any bedops/bedmap operation; output is written to stdout, redirect to file

### intersect two sorted BED files (intervals present in both)
**Args:** `--intersect a.sorted.bed b.sorted.bed > intersection.bed`
**Explanation:** --intersect returns intervals that overlap between both files; equivalent to bedtools intersect

### find intervals in file A that do not overlap file B
**Args:** `--difference a.sorted.bed b.sorted.bed > a_not_b.bed`
**Explanation:** --difference returns elements of the first file that have no overlap with any element in subsequent files

### compute coverage (sum of signal) from signal file mapped to gene windows
**Args:** `bedmap --echo --sum --delim '\t' genes.sorted.bed signal.sorted.bedgraph > genes_with_coverage.bed`
**Explanation:** --echo prints the reference interval; --sum computes total signal overlapping each gene; --delim sets delimiter

### compress a sorted BED file to starch format
**Args:** `starch input.sorted.bed > input.starch`
**Explanation:** starch compresses BED to a compact binary format; subsequent bedops/bedmap operations accept starch files directly

### extract all intervals overlapping a specific region
**Args:** `bedextract chr1 input.sorted.bed`
**Explanation:** fast random-access retrieval from sorted BED by chromosome; much faster than grep for targeted region queries

### merge overlapping intervals and compute union across three BED files
**Args:** `--merge a.sorted.bed b.sorted.bed c.sorted.bed > merged_union.bed`
**Explanation:** --merge unions all intervals and collapses overlapping ones into a single interval

### find intervals in A that overlap B by at least 1bp
**Args:** `--element-of 1 a.sorted.bed b.sorted.bed > overlapping.bed`
**Explanation:** -e 1 means at least 1bp overlap (default -e is 100%, which is very strict); use -e 50% for 50% overlap

### chop intervals into windows of 100bp
**Args:** `--chop 100 regions.sorted.bed > windows.bed`
**Explanation:** --chop splits each interval into fixed-size windows; use --stagger to offset starting positions

### count how many map elements overlap each reference interval
**Args:** `bedmap --echo --count --delim '\t' genes.sorted.bed reads.sorted.bed > gene_read_counts.bed`
**Explanation:** --count reports number of overlapping elements from map file per reference interval; combine with --echo for gene name

### find the closest feature in B for each interval in A
**Args:** `closest-features --closest --dist a.sorted.bed b.sorted.bed > closest.bed`
**Explanation:** --closest reports only the single nearest element; --dist adds signed distance column; use --no-overlaps to exclude overlapping

### convert VCF to sorted BED format
**Args:** `vcf2bed < variants.vcf > variants.sorted.bed`
**Explanation:** vcf2bed converts and pipes sorted BED; also available: gff2bed, gtf2bed, bam2bed, sam2bed, wig2bed

### compute complement (regions NOT covered by any interval)
**Args:** `--complement --chrom-sizes hg38.chromsizes intervals.sorted.bed > gaps.bed`
**Explanation:** --complement requires chromosome sizes via --chrom-sizes to define the boundary; outputs regions not covered by input
