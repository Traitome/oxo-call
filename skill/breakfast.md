---
name: breakfast
category: genomics interval operations
description: A bioinformatics tool for performing set operations on genomic intervals. Supports intersect, union, complement, and subtract operations on BED, BAM, VCF, and other genomic file formats.
tags: [genomics, intervals, bedtools-style, overlap-analysis, genome-arithmetic, bioinformatics]
author: AI-generated
source_url: https://github.com/example/breakfast
---

## Concepts

- **Interval file formats**: breakfast operates on BED, BAM, VCF, GFF, and `bedGraph` files. The tool treats these as genomic intervals with start/stop coordinates on a specific chromosome.
- **Strand awareness**: Most subcommands support strand-specific operations using `-s` (same strand) or `-S` (opposite strand) flags, enabling analysis of forward/reverse transcription patterns.
- ****Multi-file operations**: breakfast can compare an interval file against multiple database files simultaneously using the `-b` flag, enabling high-throughput overlap queries against genome annotations.
- **Sorted input requirement**: Most operations require input files to be sorted by chromosome and start coordinate (using `breakfast-sort` or `sort -k1,1 -k2,2n`), otherwise results will be incorrect or the tool will error.
- **Implicit overlap logic**: When computing intersections without explicit `-wa` (write all) or `-wb` (write base) flags, breakfast outputs only the overlapping portions of intervals, not the original input features.

## Pitfalls

- **Using unsorted input files**: Running breakfast on unsorted BED files produces incorrect interval outputs or silent failures where non-overlapping intervals are incorrectly reported as overlapping. Always pre-sort with `breakfast-sort -i input.bed`.
- **Forgetting the `-header` flag for BED files without headers**: Attempting operations on BED files that lack a chromosome column header results in parsing errors; include `-header` only when the file actually has column names.
- **Mixing chromosome naming conventions**: Combining files that use different chromosome naming (e.g., "chr1" vs "1" or "chrM" vs "MT") produces zero overlaps because strings don't match exactly; normalize chromosome names beforehand.
- **Using incorrect `-a` and `-b` file order**: The tool interprets the first file (`-a`) as the query and the second (`-b`) as the database; swapping them changes which intervals appear in output and breaks assumptions about overlap counting.
- **Assuming `-wo` outputs one line per overlap**: The `-wo` (write overlap) flag writes one line per overlapping feature pair, which can create massive output files when querying small intervals against many overlapping database features.

## Examples

### Find overlaps between a peaks file and gene annotations
**Args:** `-a peaks.bed -b genes.gff -wo`
**Explanation:** Reports every instance where a peak overlaps a gene feature, outputting both the full peak region and the overlapping gene portion in each output line.

### Extract reads completely contained within a target region
**Args:** `-a reads.bed -b target.bed -wo -u`
**Explanation:** Returns only those reads where the entire read interval lies within the target region, using `-u` to require complete containment rather than partial overlap.

### Identify unique peaks not overlapping any blacklist regions
**Args:** `-a peaks.bed -b blacklist.bed -v`
**Explanation:** Uses the `-v` flag to report peaks from file A that have no overlap whatsoever with any blacklist region, outputting only the unique peaks.

### Calculate base-pair coverage of genes by a signal track
**Args:** `-a genes.bed -b signal.bedgraph -wo -c`
**Explanation:** Groups overlapping signal values by gene and reports the total per-gene coverage after computing the overlap in base-pairs across both files.

### Perform strand-specific overlap analysis for RNA-seq data
**Args:** `-a mrnas.bed -b sirnas.bed -s -wo`
**Explanation:** Finds overlaps only where both intervals are on the same strand (`-s`), which is essential for analyzing antisense transcript interactions.

### Merge overlapping intervals in a single file
**Args:** `-a intervals.bed -bo`
**Explanation:** Merges any overlapping or directly adjacent intervals within the input file itself, producing a non-redundant set of genomic regions.

### Find the closest gene downstream of each peak
**Args:** `-a peaks.bed -b genes.bed -wo -d`
**Explanation:** Reports the closest gene downstream of each peak by computing the distance to the nearest gene after finding all overlapping genes with `-wo`.