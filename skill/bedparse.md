---
name: bedparse
category: Genomic Interval Processing
description: A command-line tool for parsing, filtering, validating, and transforming genomic interval data in BED format. Supports all major BED format variants (BED3 through BED12+), enables field extraction, coordinate-based filtering, score thresholds, strand awareness, and statistical summaries.
tags: [bed, genomic-intervals, bioinformatics, genomics, coordinate-filtering, data-validation, wig, bigbed]
author: AI-generated
source_url: https://github.com/yerbard/bedparse
---

## Concepts

- **BED format variants are column-driven**: BED3 has 3 mandatory fields (chrom, chromStart, chromEnd); BED4 adds name and score; BED6 adds strand; BED12 adds thickStart, thickEnd, itemRgb, blockCount, blockSizes, blockStarts. bedparse auto-detects the format variant from the column count and will reject or misparse files with an unexpected column count. Always verify the BED variant your downstream tool expects.
- **Coordinates in BED files are 0-based half-open intervals**: chromStart is the zero-based start of the feature on the chromosome, and chromEnd is the zero-based end. This differs from 1-based closed intervals used in GFF/GTF or 1-based inclusive end positions in SAM/BAM. Mixing coordinate systems across tools silently produces off-by-one errors in results.
- **Filtering operations can be combined with AND/OR logic**: bedparse supports chained filters (e.g., `--filter-score-min`, `--filter-chrom`, `--filter-strand`) that apply together as AND conditions. Understanding whether filters compose as intersection or union is critical for correctly isolating genomic features, especially when targeting promoters or specific gene regions.
- **Score fields are tab-delimited and string-typed by default**: In BED format the score column is a string; values like "250" and "." are both valid. Some operations treat score numerically for thresholds while others treat it as a categorical label. bedparse output formatting preserves the original string representation unless explicit numeric conversion is requested.
- **Strand field uses dot (.) for unstranded features**: BED6+ format encodes strand as "+", "-", or "." for no strandedness. bedparse respects this convention in filtering and can return unstranded features even when strand-specific queries are issued, depending on filter configuration.

## Pitfalls

- **Assuming 1-based coordinates from UCSC-style BED files**: BED files are defined in the UCSC spec as 0-based half-open intervals. Using `bedparse extract` to pull out a region and then feeding those coordinates into a tool that expects 1-based coordinates (e.g., IGV, Ensembl) will silently shift every interval by one base pair, misaligning all downstream analyses.
- **Mismatching the BED variant when exporting or converting**: Attempting to output BED12 format from a tool that only produces BED6 fields will fill missing columns with zeros or dot characters, potentially breaking tools like `bedtools` or UCSC utilities that expect a specific column count. Always validate column consistency post-conversion.
- **Forgetting that chromosome names are case-sensitive**: Chromosome names like "chr1" vs "1" vs "CHR1" are distinct in bedparse filtering. If your reference genome build uses one naming convention (e.g., ENSEMBL-style without "chr" prefix) but your query BED file uses another, strand-aware filtering will silently return zero records.
- **Overlooking the strand field when performing overlap operations**: When running operations like `bedparse annotate` or `bedparse overlap`, unstranded features with strand "." in the BED file will be reported as overlapping regardless of strand query, unless strand matching is explicitly enforced via flags. This can cause false positive associations in functional genomics pipelines.
- **Using float-formatted score thresholds on string-typed score columns**: Score thresholds like `--filter-score-min 10.5` only function correctly when the score column contains parseable numeric strings. If the source file uses non-numeric score values (e.g., "NA", "q10"), numeric filters will fail silently or produce empty sets without a type error warning.

## Examples

### Filter a BED file to regions on chromosome 1 only
**Args:** `filter --chrom chr1`
**Explanation:** This restricts all subsequent operations to entries where the chromosome field equals "chr1", enabling single-chromosome analysis without modifying the source file.

### Extract a specific genomic window from a BED file
**Args:** `extract --chrom chr7 --start 117280000 --end 117320000`
**Explanation:** This retrieves all BED records that overlap the genomic interval 117,280,000–117,320,000 on chr7, useful for isolating a candidate locus for downstream validation.

### Filter entries by minimum score threshold
**Args:** `filter --score-min 100`
**Explanation:** This removes all BED entries whose score field is below 100, retaining only high-confidence or high-coverage regions such as ChIP-seq peaks or significant GWAS hits.

### Compute summary statistics for a BED file
**Args:** `stats file.bed`
**Explanation:** This generates a summary report including total number of intervals, mean/median/max width, chromosome distribution, and score statistics, providing a quality-control view before further analysis.

### Validate a BED file and report format issues
**Args:** `validate file.bed --format bed6`
**Explanation:** This checks each line for correct column count, valid coordinate ordering (chromStart