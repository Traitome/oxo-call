---
name: biopet-bamstats
category: variant-calling-and-alignment-stats
description: Generates comprehensive statistics and metrics from BAM/SAM alignment files, including coverage depth, insert size distributions, flagstat summaries, and per-base quality scores.
tags: [bam, sam, alignment-stats, coverage, quality-metrics, ngs]
author: AI-generated
source_url: https://biopet.readthedocs.io/en/stable/software/bamstats
---

## Concepts

- **Input Format**: Accepts both BAM (binary) and SAM (text) alignment files. BAM files must be indexed (.bai) for efficient random access; the tool will attempt to use the index if present in the same directory.
- **Output Formats**: Produces multiple stat files including a JSON report with aggregate metrics, a text-based summary similar to samtools flagstat, and optional per-contig statistics. The JSON output is particularly useful for downstream pipelines.
- **Key Metrics Computed**: Calculates total mapped/unmapped reads, properly paired reads, duplicate reads, insert size mean and standard deviation, coverage breadth and depth, and base-level quality score distributions (per position).
- **Region-Specific Analysis**: Can restrict analysis to specific genomic regions using BED files, enabling focused statistics on targeted panels or genes of interest rather than the entire alignment.

## Pitfalls

- **Using Unindexed BAM Files**: Running biopet-bamstats on unindexed BAM files causes performance degradation for large files and prevents random access for region-specific queries; always index BAM files with samtools index before analysis.
- **Ignoring Read Groups**: When BAM files contain multiple read groups (common in multiplexed runs), failing to specify --readgroup or --assume-sorted can produce misleading aggregate statistics that conflate distinct samples.
- **Insufficient Memory for Deep Coverage**: High-coverage whole genome or exome datasets can exhaust default memory allocation, resulting in job failure; use --memory option to increase Java heap size proportionally to expected coverage.
- **Mismatched Reference Genomes**: Analysis proceeds without error even when the BAM header @RG and reference genome mismatch, producing nonsensical coverage and insert size statistics; verify reference consistency beforehand.

## Examples

### Generate basic BAM statistics for a whole genome alignment

**Args:** --input alignments.bam --output stats_output
**Explanation:** Processes the entire BAM file and writes statistics to the specified output directory without restricting to any genomic regions.

### Compute statistics only for a targeted capture region

**Args:** --input alignments.bam --output stats_output --bed targets.bed
**Explanation:** Restricts all metrics calculation to genomic coordinates defined in the BED file, useful for panel or exome datasets where full genome stats are misleading.

### Increase memory allocation for high-coverage whole genome (80x)

**Args:** --input highcov.bam --output stats_output --memory 8G
**Explanation:** Allocates 8GB of heap memory to handle the large number of aligned bases without out-of-memory errors, adjusting based on actual coverage depth.

### Output statistics in JSON format for pipeline integration

**Args:** --input alignments.bam --output stats_output --json
**Explanation:** Writes all computed metrics to a structured JSON file, enabling programmatic parsing by downstream pipeline components or visualization tools.

### Analyze only a specific read group from multiplexed samples

**Args:** --input multiplexed.bam --output stats_output --readgroup RG1
**Explanation:** Filters alignments to only include reads tagged with read group RG1, ensuring statistics reflect only that sample's metrics without aggregation across barcodes.