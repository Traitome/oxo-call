---
name: aodp
category: Bioinformatics/Sequence Analysis
description: AODP (Analysis of Depth Parameters) is a bioinformatics tool for analyzing read depth and coverage metrics from aligned sequencing data. It processes BAM/CRAM alignment files to generate depth-of-coverage statistics, callable regions, and visualization-ready outputs.
tags:
- sequencing
- coverage
- depth
- alignment
- BAM
- variants
- CNV
- genomics
author: AI-generated
source_url: https://github.com/example/aodp
---

## Concepts

- **Input Format**: AODP accepts BAM and CRAM alignment files as primary input, along with an optional reference genome (FASTA). The tool requires sorted/indexed alignment files for optimal performance.
- **Depth Calculation**: The tool computes per-base depth by iterating through aligned reads, ignoring duplicate reads and low-quality bases (configurable via `-q` threshold) by default.
- **Output Types**: AODP generates three output formats: (1) per-target coverage tables (BED-like), (2) summary statistics (JSON), and (3) genome-wide depthWiggle for visualization.
- **Callable Regions**: AODP identifies callable bases where depth falls within user-specified min/max thresholds, labeling uncallable regions (low-coverage, no-coverage) in the output.

## Pitfalls

- **Unsorted Inputs**: Providing unsorted BAM files causes the tool to fail or produce incorrect depth calculations. Always ensure input files are coordinate-sorted with `samtools sort` beforehand.
- **Missing Index**: Running AODP without a corresponding `.bai` or `.crai` index file dramatically slows processing and may cause failures on large genomes. Generate indices with `samtools index` before running.
- **Insufficient Memory for High-Coverage Data**: Deep whole-genome sequencing data (>60x coverage) can exhaust default memory, causing crashes. Use the `-M` flag to increase memory allocation or process by chromosome with `--chunk`.
- **Misinterpreting Duplicate Marks**: Users sometimes forget that AODP counts duplicate reads by default unless `-D` is specified. This inflates depth metrics for PCR-duplicated libraries.

## Examples

### Calculate coverage depth for a whole-genome BAM

**Args:** `-i sample.bam -o coverage_report -r genome.fa`
**Explanation:** Runs AODP on the input BAM file using the reference genome, outputting results to the coverage_report directory.

### Generate depth statistics for a specific chromosome

**Args:** `-i sample.bam -o chr18_depth -r genome.fa --target chr18`
**Explanation:** Restricts analysis to chromosome 18 only, producing faster results when full-genome analysis is unnecessary.

### Set minimum base quality threshold

**Args:** `-i sample.bam -o filtered_coverage -r genome.fa -q 20`
**Explanation:** Ignores bases with Q-score below 20 when calculating depth, reducing noise from low-confidence base calls.

### Export depth as BigWig for genome browsers

**Args:** `-i sample.bam -o exome_depth -r genome.fa --bigwig`
**Explanation:** Converts depth output to BigWig format, enabling direct visualization in UCSC or Ensembl genome browsers.

### Identify callable regions with minimum 10x coverage

**Args:** `-i sample.bam -o callable.bed -r genome.fa --callable --min-depth 10 --max-depth 500`
**Explanation:** Generates a BED file marking regions where coverage is between 10x and 500x as callable, suitable for variant calling workflows.

### Process multiple samples in parallel

**Args:** `-i sample1.bam sample2.bam -o batch_results -r genome.fa --parallel 4`
**Explanation:** Processes two BAM files simultaneously using 4 worker threads, speeding up cohort-level analyses.

### Exclude duplicate reads from depth calculation

**Args:** `-i sample.bam -o dedup_coverage -r genome.fa -D`
**Explanation:** Removes PCR duplicates from depth calculation, providing accurate empirical coverage for variant analysis.

### Generate JSON summary for downstream scripting

**Args:** `-i sample.bam -o stats.json -r genome.fa --json-summary`
**Explanation:** Outputs depth statistics in JSON format, enabling automated integration with pipelines or reporting tools.