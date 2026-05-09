---
name: bedtools
category: bioinformatics/genomics
description: A powerful suite of tools for genome arithmetic, enabling analysts to compare, merge, intersect, and complement genomic interval files in standard formats like BED, GFF, and VCF.
tags: [genome, intervals, BED, GFF, VCF, intersection, coverage, merge, complement]
author: AI-generated
source_url: https://bedtools.readthedocs.io/
---

## Concepts

- **Interval file formats:** bedtools operates on BED, GFF, VCF, and BAM files, all of which store genomic features as chromosomes, start positions, end positions, and optional fields like names and scores. The tool automatically detects formats based on file extensions or can be specified manually.
- **Order-dependent operations:** Many bedtools subcommands (intersect, closest, coverage) require at least one input file to be sorted by chromosome and start position. Unsorted files produce incorrect or empty results without error warnings.
- **Genome-aware tools:** Certain subcommands (genomeCoverageBed, complementBed, clusterBed) require a genome file listing chromosome names and their lengths. This file enables tools to distinguish between valid chromosomes and unknown/contigs.

## Pitfalls

- **Using unsorted input files:** When the -sorted flag is used with unsorted files, results may be silently incorrect or missing, because bedtools optimizes for sorted data. Always pre-sort BED files using `sort -k1,1 -k2,2n` before using sorted-aware operations.
- **Mismatched chromosome naming conventions:** One file using UCSC-style names (chr1, chr2) and another using RefSeq-style names (1, 2) will produce zero overlaps despite otherwise identical genomic regions. Normalize chromosome naming before running analyses.
- **Confusing strand orientation in -strand flag:** The -s flag requires explicit strand specification per operation, and using + on the wrong strand or omitting it when strand matters produces biologically incorrect comparisons.
- **Omitting required genome files:** Tools like genomeCoverageBed fail or produce incomplete results without a genome file when computing per-base coverage, because they cannot identify valid chromosome boundaries.

## Examples

### Find overlapping intervals between two BED files
**Args:** `intersect -a file1.bed -b file2.bed -wa -wb`
**Explanation:** This reports each overlap between file1 and file2, writing the original record from both files when they intersect.

### Merge overlapping or proximal intervals within a distance threshold
**Args:** `merge -i intervals.bed -d 100`
**Explanation:** This combines intervals in intervals.bed that are within 100 base pairs of each other into single consolidated intervals.

### Calculate per-base coverage depth from a BED file against a genome
**Args:** `genomeCoverageBed -i reads.bed -g genome.chrom.sizes -bg`
**Explanation:** This computes the number of reads covering each base position and outputs coverage in BEDGRAPH format for genome visualization.

### Get the complement of intervals relative to a reference genome
**Args:** `complementBed -i intervals.bed -g genome.chrom.sizes`
**Explanation:** This identifies all genomic positions in the reference that are not covered by any interval in intervals.bed.

### Retrieve the closest interval in file B for each interval in file A
**Args:** `closest -a features.bed -b motifs.bed -d`
**Explanation:** This finds the nearest motif to each feature and reports the distance in base pairs, using strand to constrain searches when relevant.

### Subtract one interval set from another to find unique regions
**Args:** `subtract -a peaks.bed -b blacklist.bed`
**Explanation:** This removes any regions in blacklist.bed from peaks.bed, outputting only the genomic regions unique to the peak file.

### Randomly sample a specified number of intervals from a file
**Args:** `random -n 1000 -i targets.bed`
**Explanation:** This randomly selects and outputs 1000 intervals from targets.bed, useful for creating control sets or downsampling large datasets.