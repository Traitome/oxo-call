---
name: alignstats
category: Sequence Alignment Statistics
description: A utility for computing and reporting alignment quality metrics from SAM/BAM files, including mapping rates, coverage statistics, insert size distributions, and alignment quality scores.
tags:
  - alignment
  - sam/bam
  - quality-control
  - sequencing
  - coverage
author: AI-generated
source_url: https://github.com/ExpressionAnalysis/alignstats
---

## Concepts

- **Input Formats**: alignstats accepts sorted and indexed BAM files as primary input. It automatically detects alignment records, skipping header lines (@SQ, @PG) and processing only mapped and unmapped reads with valid QNAMEs.
- **Metric Categories**: The tool reports four core statistic groups — mapping rate (total aligned vs. unaligned reads), coverage depth (mean, median, mode per chromosome), insert size distribution (mean, median, standard deviation, FF/FR orientation metrics), and base-level quality scores (Phred-scaled per-position averages).
- **Output Modes**: Results are written to stdout in a structured key-value format by default. Using the `--json` flag emits machine-readable JSON; `--summary` produces a compact one-line summary; `--perChromosome` generates separate statistics per reference sequence defined in the BAM header.
- **Index Dependency**: When using `--perChromosome` or computing coverage metrics, alignstats requires a corresponding BAI index file (.bai) adjacent to the input BAM. Without it, coverage and per-chromosome statistics are skipped with a non-fatal warning.

## Pitfalls

- **Unsorted BAM Input**: Running alignstats on an unsorted BAM file produces unreliable paired-end metrics because read ordering affects insert size calculations and mate information lookup. The output will include a warning but continue, yielding potentially misleading statistics.
- **Missing BAI Index**: If the BAI index file is absent and coverage or per-chromosome metrics are requested, the tool silently skips these calculations without exiting with an error. Users may interpret the absence of coverage data as zero coverage rather than a missing index.
- **Mixed Library Orientation**: When analyzing a BAM containing both FF and FR paired-end libraries without specifying `--libraryOrientation`, insert size distributions are computed across all reads, producing inflated standard deviation values that do not reflect either library's true insert size distribution.
- **Non-Standard Reference Names**: References with non-alphanumeric characters (spaces, colons) in their names cause column-aligned output tables to misalign, making per-chromosome statistics difficult to read. This is cosmetic but can cause parsing errors in downstream automated parsers.

## Examples

### Compute default statistics from a single BAM file
**Args:** reads.sorted.bam
**Explanation:** This runs alignstats in default mode, outputting mapping rate, overall coverage summary, insert size distribution, and quality score metrics to stdout in a human-readable key-value format.

### Export results as JSON for programmatic downstream processing
**Args:** reads.sorted.bam --json output.json
**Explanation:** The `--json` flag instructs alignstats to emit all statistics in valid JSON format, enabling straightforward parsing by automated pipelines or dashboards.

### Report per-chromosome statistics for a whole-genome alignment
**Args:** wgs.bam --perChromosome --reference GRCh38.fa
**Explanation:** The `--perChromosome` flag generates separate mapping rate, coverage depth, and insert size statistics for each reference sequence in the BAM header, while `--reference` annotates chromosome names using the provided FASTA index for display.

### Analyze a BAM with FR-oriented paired-end library
**Args:** pe_library.bam --libraryOrientation FR --minInsert 50 --maxInsert 800
**Explanation:** Specifying `--libraryOrientation FR` ensures insert size distribution calculations respect FR strand conventions, and `--minInsert`/`--maxInsert` filter outlier fragments outside the expected 50–800 bp range.

### Generate a compact one-line summary for multi-sample comparison
**Args:** sample1.bam --summary
**Explanation:** The `--summary` flag emits a single-line tab-delimited output containing key metrics (total reads, mapped %, mean depth, median insert), making it suitable for aggregating statistics across many samples in a spreadsheet.