---
name: cgranges
category: genomic-interval-operations
description: A high-performance Rust library and CLI tool for genomic coordinate range operations including intersection, union, complement, and coverage calculations. Supports BED, GTF, and other genomic interval formats with efficient interval tree-based algorithms.
tags: [genomics, intervals, range-operations, bed, overlap, bioinformatics]
author: AI-generated
source_url: https://github.com/lh3/cgranges
---

## Concepts

- **Coordinate system**: cgranges uses 0-based half-open intervals by default (matching BED format), but can handle 1-based inclusive coordinates (matching GTF/GFF) when specified. Understanding this distinction is critical for correct output interpretation.
- **Interval tree efficiency**: The tool employs an interval tree data structure enabling O((n+m) log n) complexity for overlap queries between two interval sets, making it scalable for genome-scale analyses.
- **Input format flexibility**: Accepts standard BED files (0-based), GTF/GFF files (1-based), and custom interval notation. The tool automatically detects and adapts to different coordinate conventions based on input file headers.
- **Streaming mode**: Can process large files in streaming fashion without loading entire datasets into memory, enabling analysis of chromosome-scale files on modest hardware.

## Pitfalls

- **Coordinate system mismatch**: Mixing 0-based and 1-based interval inputs without explicit conversion produces incorrect overlap results. A 1-based interval [1,10] in GTF becomes [0,10) in BED coordinates—a common source of false positives.
- **Chromosome name inconsistency**: Genome build differences (e.g., chr1 vs 1, chrM vs MT) cause intervals to be silently ignored during operations. Always verify chromosome naming conventions match across all input files.
- **Wrong strand orientation**: By default, cgranges processes intervals on both strands; specifying strand-specific operations (with +/- flags) is required for sense-strand analysis, otherwise antisense genes are included in overlaps.
- **Empty output interpretation**: Producing no output can mean either genuine no-overlap or input format errors. Always validate input files parse correctly before interpreting empty results as biological conclusions.

## Examples

### Find overlapping intervals between two BED files
**Args:** intersect file1.bed file2.bed
**Explanation:** Returns all interval pairs that overlap between the two files, using default 0-based half-open coordinates.

### Calculate genome-wide coverage from a BED file
**Args:** coverage input.bed
**Explanation:** Computes the number of intervals covering each base pair in the genome, useful for identifying densely annotated regions.

### Perform strand-specific overlap detection
**Args:** intersect -s + input.bed genes.gtf
**Explanation:** Finds overlaps only where both intervals are on the positive strand, filtering out antisense and intergenic interactions.

### Union multiple interval files
**Args:** union file1.bed file2.bed file3.bed
**Explanation:** Merges all intervals from multiple files into a single non-overlapping set, consolidating discrete genomic features.

### Get complement intervals relative to a reference
**Args:** complement regions.bed genome.bed
**Explanation:** Returns intervals in genome.bed that do not overlap with any region in regions.bed, identifying gap or excluded regions.

### Filter intervals by minimum length threshold
**Args:** filter -l 100 input.bed
**Explanation:** Retains only intervals with length >= 100 base pairs, removing short spurious or artifact intervals from the dataset.