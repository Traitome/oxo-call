---
name: bedtools
category: genomic-intervals
description: A powerful toolset for genome arithmetic on BED, BAM, VCF, and GFF files
tags: [bed, genomic-intervals, intersection, coverage, annotation, ngs, jaccard, getfasta, bamtobed]
author: oxo-call built-in
source_url: "https://bedtools.readthedocs.io/"
---

## Concepts

- bedtools operates on BED, BAM, VCF, GFF, and GTF files for genomic interval arithmetic.
- The 'bedtools intersect' command is core: -a and -b specify the two interval files; -wa/-wb report the original intervals.
- Coordinates: BED files are 0-based half-open [start, end), but BAM/VCF/GTF are 1-based — bedtools handles this automatically.
- bedtools requires sorted input for tools like bedtools merge, closest, and coverage — sort with 'sort -k1,1 -k2,2n' first.
- bedtools genomecov is a subcommand for coverage: 'bedtools genomecov -ibam input.bam -bg > coverage.bedgraph' — the output subcommand name 'genomecov' must always appear as the first argument.
- The -g genome file (chromosome sizes) is required for tools like slop, makewindows, flank, complement, shuffle, and genomecov with BED input (not needed when using -ibam with BAM input).
- bedtools map applies aggregate functions (sum, mean, min, max, count, collapse, distinct, median, mode) to columns from B intervals overlapping each A interval — like SQL GROUP BY for intervals.
- bedtools merge -c/-o can aggregate columns while merging (e.g., -c 4 -o distinct to list gene names from merged intervals).
- bedtools intersect -f specifies minimum overlap fraction; -F specifies minimum overlap as fraction of B; -r requires reciprocal overlap (both -f and -F thresholds met).
- bedtools getfasta -s reverse-complements sequences on the minus strand; -split extracts and concatenates exon blocks from BED12.
- bedtools bamtobed converts BAM to BED6; -bed12 outputs blocked BED; -split splits on CIGAR N operations.
- bedtools groupby summarizes a column based on grouping columns, similar to SQL GROUP BY — works on any tab-delimited file, not just BED.

## Pitfalls

- bedtools ARGS must start with a subcommand (intersect, subtract, merge, closest, window, coverage, map, genomecov, cluster, complement, shift, slop, flank, sort, random, shuffle, sample, spacing, annotate, multiinter, unionbedg, pairtobed, pairtopair, bamtobed, bedtobam, bamtofastq, bedpetobam, bed12tobed6, getfasta, maskfasta, nuc, multicov, tag, jaccard, reldist, fisher, overlap, makewindows, groupby, expand, split, summary) — never with flags like -a, -b, or -ibam. The subcommand ALWAYS comes first. Use NEW-style names (intersect, sort, merge), NOT old-style names (intersectBed, sortBed, mergeBed).
- bedtools intersect without -wa or -wb only outputs the intersecting region, not the full intervals.
- bedtools merge requires sorted input — use 'sort -k1,1 -k2,2n input.bed' first.
- bedtools genomecov outputs a frequency table by default; use -bg for bedGraph or -bga to include zero-coverage regions.
- bedtools intersect with BAM input: use -abam and output will be BAM unless you add -bed.
- bedtools does not automatically sort input — many operations silently produce wrong results on unsorted data.
- bedtools intersect -c counts overlaps per A interval; -C counts per A per B file separately.
- bedtools intersect -v inverts the match (reports A entries with NO overlap in B) — this is the complement of the default behavior.
- bedtools getfasta requires a FAI-indexed FASTA (run samtools faidx first if needed).
- bedtools slop and flank require a -g genome file; coordinate boundaries are automatically clamped (start≥0, end≤chrom_length).

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
**Args:** `getfasta -fi reference.fa -bed intervals.bed -fo output.fa -s`
**Explanation:** -fi is the reference FASTA; -bed is the intervals; -fo is the output; -s reverse-complements minus-strand features

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

### apply aggregate function to overlapping intervals
**Args:** `map -a genes.bed -b scores.bed -c 5 -o mean`
**Explanation:** maps column 5 from B intervals overlapping each A interval; -o specifies operation (sum, mean, min, max, count, collapse, distinct, median, mode, stdev)

### merge intervals while preserving gene names
**Args:** `merge -i exons.bed -c 4 -o distinct`
**Explanation:** -c 4 selects the name column; -o distinct lists unique gene names from merged intervals; merge must have sorted input

### find A intervals with NO overlap in B (complement of intersect)
**Args:** `intersect -a genes.bed -b blacklist.bed -v`
**Explanation:** -v inverts the match: reports only A entries with no overlap in B; equivalent to subtract but reports full A intervals

### add flanking bases to intervals
**Args:** `slop -i peaks.bed -g genome.txt -b 1000 > peaks_expanded.bed`
**Explanation:** -b adds 1000 bp to both sides; use -l/-r for asymmetric; -pct treats values as fraction of feature length; requires -g genome file

### convert BAM to BED format
**Args:** `bamtobed -i aligned.bam > aligned.bed`
**Explanation:** converts BAM to BED6; -bed12 for blocked BED; -split to split on CIGAR N operations; -splitD for N and D operations

### compute Jaccard similarity between two interval sets
**Args:** `jaccard -a peaks1.bed -b peaks2.bed`
**Explanation:** outputs intersection, union, Jaccard statistic (intersection/union), and number of intersections; useful for comparing peak sets

### profile nucleotide content of intervals
**Args:** `nuc -fi reference.fa -bed regions.bed`
**Explanation:** reports GC content, AT content, and nucleotide frequencies per interval; -s for strand-aware; -pattern to count specific motifs

### left-join two interval files (report A even without B overlap)
**Args:** `intersect -a genes.bed -b snps.bed -loj`
**Explanation:** -loj performs left outer join: reports every A interval, with NULL B features when no overlap; -wao also reports overlap bases
