---
name: binge
category: Bioinformatics
description: A tool for identifying and extracting genomic features based on coordinate matching and overlap detection. Operates on BED, VCF, and other genomic coordinate files to find intervals matching specified criteria.
tags: [genomics, coordinate-matching, interval-overlap, bedtools-alternative, feature-extraction]
author: AI-generated
source_url: https://github.com/bingetools/binge
---

## Concepts

- **Genomic coordinate model**: binge operates on genomic intervals defined by chromosome, start position, and end position (0-based or 1-based depending on format). The tool compares these coordinates to identify matches, overlaps, or containment relationships between query and reference datasets.
- **Input formats**: binge accepts standard genomic file formats including BED (0-based half-open), VCF (1-based), and GFF/GTF. It can also work with SAM/BAM files for coordinate-based operations. Output is typically generated in BED format or as a filtered subset of the input.
- **Key behaviors**: The tool performs interval intersection, nearest-neighbor finding, and coverage calculation. By default, it reports overlapping intervals, but flags exist to customize matching stringency (e.g., requiring minimum overlap fraction or requiring complete containment).

## Pitfalls

- **Coordinate system mismatch**: Failing to account for 0-based vs 1-based coordinate systems will produce empty or incorrect results. BED files use 0-based start/1-based end coordinates, while VCF uses 1-based for all fields.
- **Sorting requirement**: binge requires input files to be sorted by chromosome and position. Unsorted input will silently produce incorrect overlap results or miss valid matches.
- **Strand specificity ignored**: By default, binge treats both positive and negative strands as matching. If strand-specific results are needed, omitting the strand filter leads to false positive overlaps on the opposite strand.

## Examples

### Find all intervals overlapping between two BED files
**Args:** -a query.bed -b reference.bed -wo
**Explanation:** Reports intervals from query.bed that overlap with reference.bed, outputting the number of overlapping base pairs (-wo flag).

### Extract intervals completely contained within a genomic region
**Args:** -a query.bed -b region.bed -f 1.0
**Explanation:** Returns only those query intervals fully contained within the reference regions, enforced by the -f 1.0 minimum overlap fraction requirement.

### Find nearest downstream intervals without overlap
**Args:** -a query.bed -b reference.bed -io -d
**Explanation:** Identifies the nearest non-overlapping intervals in reference.bed downstream of each query interval, sorted by distance (-d).

### Only match intervals on the same strand
**Args:** -a query.bed -b reference.bed -s
**Explanation:** Restricts overlap detection to intervals on the same genomic strand, preventing cross-strand false positives.

### Output only the query intervals with their best match
**Args:** -a query.bed -b reference.bed -wb -loj
**Explanation:** Reports each query interval with its best matching reference interval, using left outer join semantics to preserve intervals with no overlap.