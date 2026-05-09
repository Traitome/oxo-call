---
name: bioframe
category: Genomics/Interval Operations
description: A Python library for manipulating genomic intervals as tabular data, providing pandas-style operations for genomic coordinate manipulation including overlap detection, interval merging, and set operations on genomic features.
tags: [genomics, intervals, pandas, bioinformatics, coordinates, bedtools-alternative, genome-analysis]
author: AI-generated
source_url: https://github.com/cggbio/bioframe
---

## Concepts

- **DataFrame-centric model**: bioframe operates on pandas DataFrames containing genomic intervals with required columns `chrom`, `start`, `end`, and optional `strand`. Functions take DataFrames as primary arguments and return modified DataFrames, enabling chained operations and integration with pandas workflows.

- **Interval input flexibility**: Functions accept intervals as Python tuples `('chr', start, end)`, pandas DataFrames with specified `chrom_col`, `start_col`, `end_col` parameters, or paths to BED files that are automatically loaded. This allows seamless conversion between formats without explicit parsing.

- **Set-theoretic operations**: Core functions implement genomic interval algebra—`overlap()` finds intersecting intervals between two sets, `subtract()` removes overlapping portions, `intersect()` returns shared regions, and `union()` combines non-overlapping intervals—mirroring traditional bedtools operations but in a DataFrame paradigm.

- **Null handling and empty results**: Functions return empty DataFrames with correct schemas when no overlaps or operations occur, rather than raising errors. This enables conditional workflows without try-except blocks for edge cases.

## Pitfalls

- **Column name mismatches**: Passing a DataFrame without explicitly specifying `chrom_col`, `start_col`, `end_col` when column names differ from defaults ('chrom', 'start', 'end') causes silent failures or incorrect results. Always verify column names or map them explicitly.

- **1-based vs 0-based coordinate confusion**: bioframe uses half-open intervals [start, end) matching BEDTools and UCSC conventions—1-based inclusive coordinates from BED files need conversion. Importing a BED file directly may cause off-by-one errors if coordinate system isn't validated.

- **Unintended interval expansion**: The `expand()` function modifies intervals in place by adding/subtracting amounts from both ends by default, which can cause negative coordinates or overflow beyond chromosome boundaries. Use `pad` parameter carefully and validate bounds afterward.

- **Mixing stranded and unstranded data**: Operations like `overlap()` have `keep_partial` and `strand_behavior` parameters affecting how stranded intervals interact. Default behavior may include antisense overlaps that users expect to exclude, producing unexpected result sizes.

- **Memory with large datasets**: Operations like `pairwise_overlap()` compute all pairwise comparisons, creating O(n²) memory requirements for large interval sets. For genome-scale data, chunk processing or using summary statistics functions is necessary.

## Examples

### Two DataFrames overlap detection
**Args:** `[df1, df2, chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Identifies overlapping intervals between two genomic interval DataFrames, returning a DataFrame with overlapping regions and counts of intersections.

### Merge overlapping intervals in a DataFrame
**Args:** `[df, chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Combines all overlapping and adjacent intervals within a single DataFrame into merged non-overlapping segments, reducing redundancy in interval lists.

### Subtract one interval set from another
**Args:** `[df_here, df_other, chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Removes portions of intervals in the first DataFrame that overlap with intervals in the second DataFrame, returning residual non-overlapping segments.

### Expand intervals by a fixed pad value
**Args:** `[df, chrom_col='chrom', start_col='start', end_col='end', pad=1000]`
**Explanation:** Extends each interval by 1000 base pairs in both directions, useful for creating flanking regions around genomic features for downstream analysis.

### Cluster intervals into groups
**Args:** `[df, chrom_col='chrom', start_col='start', end_col='end', max_dist=0]`
**Explanation:** Groups intervals within a DataFrame that are within specified distance thresholds, assigning cluster identifiers for downstream correlated analysis.

### Load genomic intervals from a BED file
**Args:** `['path/to/file.bed', chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Reads a BED file directly into a pandas DataFrame using bioframe's built-in parser, automatically handling standard BED format columns.

### Detect pairwise overlaps between two sets
**Args:** `[df1, df2, chrom_col='chrom', start_col='start', end_col='end', how='inner']`
**Explanation:** Computes all pairwise overlaps between two interval sets with configurable join behavior (inner, outer, left, right), similar to SQL joins but for genomic coordinates.

### Complement intervals relative to a chromosome
**Args:** `[df, chroms_df, chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Returns intervals representing gaps in coverage relative to a reference set of chromosome spans, identifying unmapped genomic regions.

### Count overlaps per interval
**Args:** `[df1, df2, chrom_col='chrom', start_col='start', end_col='end']`
**Explanation:** Adds a column to the first DataFrame counting how many intervals from the second DataFrame overlap each interval, enabling quantitative overlap analysis.

### Find the closest interval from another set
**Args:** `[df_query, df_ref, chrom_col='chrom', start_col='start', end_col='end', distance=5000]`
**Explanation:** For each interval in the query DataFrame, finds the nearest interval in the reference set within a maximum distance threshold.