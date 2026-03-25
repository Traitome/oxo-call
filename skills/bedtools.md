---
name: bedtools
category: genomic-intervals
description: A powerful toolset for genome arithmetic on BED, BAM, VCF, and GFF files
tags: [bed, genomic-intervals, intersection, coverage, annotation, ngs]
author: oxo-call built-in
source_url: "https://bedtools.readthedocs.io/"
---

## Concepts

- bedtools operates on BED, BAM, VCF, GFF, and GTF files for genomic interval arithmetic.
- The 'bedtools intersect' command is core: -a and -b specify the two interval files; -wa/-wb report the original intervals.
- Coordinates: BED files are 0-based half-open [start, end), but BAM/VCF/GTF are 1-based — bedtools handles this automatically.
- bedtools requires sorted input for tools like bedtools merge, closest, and coverage — sort with 'sort -k1,1 -k2,2n' first.
- bedtools genomecov is a subcommand for coverage: 'bedtools genomecov -ibam input.bam -bg > coverage.bedgraph' — the output subcommand name 'genomecov' must always appear as the first argument.
- The -g genome file (chromosome sizes) is required for tools like slopBed, makewindows, and genomecov with BED input (not needed when using -ibam with BAM input).

## Pitfalls

- bedtools intersect without -wa or -wb only outputs the intersecting region, not the full intervals.
- bedtools merge requires sorted input — use 'sort -k1,1 -k2,2n input.bed' first.
- bedtools genomecov outputs a frequency table by default; use -bg for bedGraph or -bga to include zero-coverage regions.
- Always include the subcommand name (e.g., 'genomecov', 'intersect', 'subtract') as the first argument — never omit it.
- bedtools intersect with BAM input: use -abam and output will be BAM unless you add -bed.
- bedtools does not automatically sort input — many operations silently produce wrong results on unsorted data.

## Examples

### find intervals in file A that overlap with file B
**Args:** `intersect -a query.bed -b features.bed -wa`
**Explanation:** -a is query; -b is features to intersect with; -wa outputs original A intervals

### subtract regions in B from regions in A
**Args:** `subtract -a regions.bed -b blacklist.bed`
**Explanation:** removes any parts of A that overlap B; useful for removing blacklisted regions

### merge overlapping intervals in a BED file
**Args:** `merge -i input.bed`
**Explanation:** input must be sorted (sort -k1,1 -k2,2n); outputs merged non-overlapping intervals

### compute per-base coverage from a BAM file
**Args:** `genomecov -ibam sorted.bam -bg > coverage.bedgraph`
**Explanation:** genomecov is the subcommand; -ibam takes BAM input; -bg outputs bedGraph (chrom/start/end/depth); requires sorted BAM

### find closest non-overlapping feature in B for each interval in A
**Args:** `closest -a query.bed -b annotations.bed -d`
**Explanation:** -d reports distance to closest feature; output includes original A and B intervals plus distance

### count overlaps between A intervals and B features
**Args:** `intersect -a genes.bed -b reads.bam -c`
**Explanation:** -c counts the number of B features overlapping each A interval; useful for read counting

### get sequences for intervals in a BED file
**Args:** `getfasta -fi reference.fa -bed intervals.bed -fo output.fa`
**Explanation:** -fi is the reference FASTA; -bed is the intervals; -fo is the output FASTA

### compute coverage including zero-coverage positions
**Args:** `genomecov -ibam sorted.bam -bga > coverage_all.bedgraph`
**Explanation:** genomecov subcommand; -bga includes regions with zero coverage unlike -bg; useful for complete coverage maps

### intersect two BED files and report original B intervals that overlap A
**Args:** `intersect -a query.bed -b features.bed -wb`
**Explanation:** -wb outputs original B intervals (instead of -wa for A intervals); outputs the overlapping B entries

### make windows of fixed size across a genome
**Args:** `makewindows -g genome.txt -w 1000 > windows.bed`
**Explanation:** makewindows subcommand; -g is chromosome sizes file; -w is window size in bp; outputs BED with tiled windows

### compute coverage histogram for a BAM file
**Args:** `genomecov -ibam sorted.bam > coverage_histogram.txt`
**Explanation:** genomecov without -bg outputs a frequency table (depth, count, length, fraction) per chromosome

### window-based intersection: find A intervals within N bp of B
**Args:** `window -a genes.bed -b snps.bed -w 1000`
**Explanation:** window subcommand; -w expands the search window by 1000 bp around each A interval; finds nearby B features
