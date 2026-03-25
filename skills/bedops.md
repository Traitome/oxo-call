---
name: bedops
category: genomic-arithmetic
description: Fast and highly scalable set operations and arithmetic on BED, GFF, VCF, and SAM genomic intervals
tags: [bed, intervals, genomic-arithmetic, intersection, union, complement, sort]
author: oxo-call built-in
source_url: "https://bedops.readthedocs.io/"
---

## Concepts

- BEDOPS requires input files to be sorted with sort-bed; unsorted input causes immediate failure with a clear error message.
- bedops performs set operations: --intersect, --union, --difference, --complement, --symmetric-difference on sorted BED files.
- bedmap maps signals from one BED file onto intervals of another, computing statistics (--sum, --mean, --max, --count) per element.
- starch is BEDOPS's compressed binary format; unstarch decompresses it; starchcat concatenates starch archives.
- BEDOPS is strand-unaware by default; it works on coordinate intervals only, making it very fast but requiring strand filtering upstream.
- bedextract rapidly retrieves all intervals overlapping a given region from a sorted BED or starch file without loading the whole file.

## Pitfalls

- All inputs to bedops and bedmap must be sorted with sort-bed; BED files sorted by other tools may not be compatible.
- bedops --complement requires chromosome sizes to compute regions not covered; without --chrom-sizes the complement will be wrong.
- bedmap --echo --mean returns one output line per reference interval, not per overlap — understand the mapping model before use.
- BEDOPS uses half-open intervals [start, end) like BED; confirm input tools also use 0-based half-open coordinates.
- Very large union operations can produce files with billions of lines; use starch to compress output immediately after computation.
- bedmap --fraction-ref and --fraction-map control overlap thresholds; confusing them leads to incorrect overlap filtering.

## Examples

### sort a BED file for use with BEDOPS tools
**Args:** `sort-bed input.bed > input.sorted.bed`
**Explanation:** sort-bed is required before any bedops/bedmap operation; output is written to stdout, redirect to file

### intersect two sorted BED files (intervals present in both)
**Args:** `--intersect a.sorted.bed b.sorted.bed > intersection.bed`
**Explanation:** --intersect returns intervals that overlap between both files; equivalent to BEDtools -u -a -b

### find intervals in file A that do not overlap file B
**Args:** `--difference a.sorted.bed b.sorted.bed > a_not_b.bed`
**Explanation:** --difference returns elements of the first file that have no overlap with any element in subsequent files

### compute coverage (sum of signal) from bigwig/bedgraph mapped to gene windows
**Args:** `bedmap --echo --sum --delim '\t' genes.sorted.bed signal.sorted.bedgraph > genes_with_coverage.bed`
**Explanation:** --echo prints the reference interval; --sum computes total signal overlapping each gene; --delim sets delimiter

### compress a sorted BED file to starch format
**Args:** `starch input.sorted.bed > input.starch`
**Explanation:** starch compresses BED to a compact binary format; subsequent bedops/bedmap operations accept starch files directly

### extract all intervals overlapping a specific region
**Args:** `bedextract chr1:100000-200000 input.sorted.bed`
**Explanation:** fast random-access retrieval from sorted BED; much faster than grep for targeted region queries

### merge overlapping intervals and compute union across three BED files
**Args:** `--merge a.sorted.bed b.sorted.bed c.sorted.bed > merged_union.bed`
**Explanation:** --merge unions all intervals and collapses overlapping ones into a single interval
