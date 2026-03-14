---
name: mosdepth
category: utilities
description: Fast BAM/CRAM depth calculation per base, region, and window for coverage analysis
tags: [coverage, depth, bam, wgs, wes, qc, statistics, regions]
author: oxo-call built-in
source_url: "https://github.com/brentp/mosdepth"
---

## Concepts

- mosdepth computes depth of coverage for BAM/CRAM files; much faster than samtools depth.
- Use --by N for windowed coverage or --by regions.bed for per-region coverage.
- Output files: <prefix>.mosdepth.summary.txt (overall stats), <prefix>.per-base.bed.gz (per-base coverage), <prefix>.regions.bed.gz.
- Use -t N for multi-threading; -x to exclude zero-depth bases from output (reduces file size).
- Use -n to disable per-base output (faster for summary stats only).
- The --quantize flag bins coverage for memory-efficient storage of large coverage tracks.
- Use -Q N (minimum MAPQ) and -F N (exclude reads with flag N) for filtering.
- mosdepth summary includes mean depth, coverage fractions — useful for WGS QC reporting.

## Pitfalls

- Input BAM must be sorted and indexed; CRAM requires the reference with --fasta.
- Without --by, mosdepth computes per-base coverage to a .bed.gz file (can be large for WGS).
- For target-capture WES, always use --by targets.bed to restrict to capture regions.
- mosdepth default output includes zero-coverage bases — use -x to exclude for faster analysis.
- MAPQ filter (-Q 20) excludes duplicate/secondary reads — use -F 1796 to skip duplicates/secondary/supplementary.

## Examples

### calculate genome-wide depth of coverage in 500bp windows
**Args:** `--by 500 -t 8 --prefix sample_coverage sample_sorted.bam`
**Explanation:** --by 500 windows; -t 8 threads; --prefix output prefix; creates .summary.txt and .regions.bed.gz

### calculate coverage over target regions for WES
**Args:** `--by targets.bed -t 4 --prefix wes_coverage sample_sorted.bam`
**Explanation:** --by targets.bed restricts to capture regions; outputs per-region coverage

### calculate per-base depth with MAPQ filter
**Args:** `-t 4 -Q 20 -F 1796 --prefix filtered_coverage sample_sorted.bam`
**Explanation:** -Q 20 minimum MAPQ; -F 1796 excludes duplicate (1024) + secondary (256) + unmapped (4) + supplementary (2048) reads

### get summary statistics only without per-base output
**Args:** `-n -t 8 --prefix summary_only sample_sorted.bam`
**Explanation:** -n disables per-base .bed.gz output; much faster for summary statistics only
