---
name: berokka
category: Genomics / BED File Manipulation
description: A bioinformatics tool for sorting, merging, and manipulating BED (Browser Extensible Display) format genomic interval files. Supports various operations including coordinate sorting, overlap merging, and header management.
tags: [genomics, bioinformatics, BED, genomics, intervals, genome-annotations, file-manipulation, sorting, merging]
author: AI-generated
source_url: https://github.com/arangrhie/berokka
---

## Concepts

- **BED File Format**: berokka operates on standard BED format files containing genomic coordinates (chromosome, start, end, and optional score/name/strand columns). The tool expects 0-based half-open intervals typical of BED format, not 1-based coordinates used in other formats.
- **Sorting Behavior**: By default, berokka sorts BED entries first by chromosome (lexicographically), then by start coordinate (ascending), then by end coordinate (ascending). This produces coordinate-sorted output compatible with binary indexers and interval tree algorithms.
- **Merge Operations**: When merging overlapping or adjacent genomic intervals, berokka combines the regions and can perform various operations on the merged data, including taking the minimum start, maximum end, summing scores, or keeping the first/last entry.
- **Companion Binary**: The berokka package typically includes `berokka-build` for creating indexed versions of BED files for faster random access queries.

## Pitfalls

- **Coordinate System Confusion**: Using berokka output with tools expecting 1-based coordinates (like VCF or GFF) will cause off-by-one errors in genomic annotations. Always verify the coordinate system expected by downstream analysis tools.
- **Incorrect Column Handling**: BED files with variable numbers of columns may produce unexpected merge results if the tool assumes a fixed column count. Ensure your BED file has consistent columns before processing.
- **Memory Constraints with Large Files**: Processing genome-wide BED files (containing millions of intervals) without sufficient memory allocation can cause crashes or incomplete output. For large files, consider splitting by chromosome.
- **Overlapping Edge Cases**: Adjacent intervals (where one ends exactly where another begins) may or may not be merged depending on flags used, leading to inconsistent results if not explicitly specified.

## Examples

### Sort a BED file by genomic coordinates
**Args:** `-i unsorted.bed > sorted.bed`
**Explanation:** Sorts the input BED file by chromosome name, then start position, then end position, outputting to stdout for piping to other tools.

### Merge overlapping intervals in a BED file
**Args:** `-i overlapping.bed --merge > merged.bed`
**Explanation:** Combines all overlapping and adjacent genomic intervals into single non-overlapping regions, simplifying the annotation set.

### Keep strand information during merge operations
**Args:** `-i strand-aware.bed --merge --s`
**Explanation:** Preserves strand orientation when merging, ensuring that intervals on opposite strands are not incorrectly combined.

### Filter and output intervals by score threshold
**Args:** `-i scored.bed --score-filter 10 > high-score.bed`
**Explanation:** Retains only genomic intervals with a score field value greater than or equal to the specified threshold.

### Use the companion build tool to index a BED file
**Args:** `berokka-build -i annotation.bed`
**Explanation:** Creates an indexed version of the BED file using the companion binary for rapid interval queries in downstream analyses.