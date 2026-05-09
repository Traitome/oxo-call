---
name: agg
category: genomics
description: UCSC tool for aggregating genomic scores over genomic intervals. Computes statistics (mean, min, max, sum, count) from score tracks (bigWig, wiggle, bedGraph) across regions defined in BED files. Supports group-based aggregation and differential analysis.
tags: [genomics, aggregation, bigWig, BED, UCSC, wiggle, bedGraph, intervals]
author: AI-generated
source_url: https://github.com/ucscGenomeTools/kent
---

## Concepts

- **Input formats**: `agg` accepts score tracks in bigWig, wiggle (ascii or binary), and bedGraph formats. Genomic intervals must be provided as BED files (3-12 columns). The score file is specified with `-iScoreIn=` and the interval file with `-iIntervalsIn=`.
- **Aggregation modes**: The tool computes per-base-pair statistics when scoring over intervals, with options to aggregate by group (using column 4 of BED as group name). Statistics include mean, min, max, sum, coverage (count of non-zero bases), and standard deviation.
- **Differential analysis**: By providing two score files (e.g., treatment and control), `agg` can compute differential signals between them across identical intervals, reporting log2 fold changes and statistical significance when a replicate-based mode is used.
- **Output structure**: Results are tab-separated with one row per input interval. Columns include the interval coordinates, group name, and computed statistics (mean1, mean2, fold change, p-value if applicable).

## Pitfalls

- **Mismatched genome assemblies**: Using a score file built for a different genome assembly (e.g., hg38 score file with hg19 interval coordinates) produces invalid or all-zero results. Always verify genome builds match between all input files.
- **Large interval files causing memory overflow**: Processing BED files with millions of intervals without the `-blockSize=` flag can exhaust memory. Use `-blockSize=500` or similar to process in chunks.
- **Missing strand information**: If intervals have strand info (+/-) but the score file is not strand-specific, aggregation may mix signals from both strands leading to incorrect mean values, especially for antisense transcripts.
- **Conflicting column interpretations**: Using BED6 for group aggregation without realizing column 4 is empty causes no group assignment, collapsing all results into a single aggregate rather than per-group results.
- **Floating point precision in fold changes**: Log2 fold changes from near-zero values produce extreme or undefined values (infinity), which can break downstream statistical analysis or visualization tools.

## Examples

### Aggregate bigWig scores over BED intervals
**Args:** `-iScoreIn=experiments.bigWig -iIntervalsIn=peaks.bed`
**Explanation:** Computes the mean score for each peak region in the BED file using signal values from the bigWig file, outputting one aggregate row per interval.

### Compute mean and coverage per group
**Args:** `-iScoreIn=K27ac.bigWig -iIntervalsIn=genes.bed -groupNameAsField`
**Explanation:** Uses the gene name in BED column 4 as group identifier, calculating both mean signal intensity and the number of base pairs with signal above zero for each gene.

### Compare two conditions with differential output
**Args:** `-iScoreIn1=treatment.bigWig -iScoreIn2=control.bigWig -iIntervalsIn=regions.bed`
**Explanation:** Computes differential signal between treatment and control conditions over the same regions, outputting mean values for both and log2 fold change.

### Process large files with memory chunking
**Args:** `-iScoreIn=chip-seq.bigWig -iIntervalsIn=huge_peaks.bed -blockSize=200`
**Explanation:** Processes the large peak file in chunks of 200 intervals at a time to prevent memory exhaustion while maintaining accurate aggregation.

### Output detailed per-base statistics
**Args:** `-iScoreIn=rnaseq.bigWig -iIntervalsIn=exons.bed - Statistics=mean,min,max,sum,coverage`
**Explanation:** Computes five statistics for each exon: average signal, minimum value, maximum value, total sum, and the count of bases with non-zero signal.

### Aggregate with strand-specific intervals
**Args:** `-iScoreIn=bigWig -iIntervalsIn=stranded_intervals.bed -strandSplit`
**Explanation:** When intervals have strand information (+/- in column 6), aggregates the score track separately for each strand, producing distinct rows for plus and minus directions.