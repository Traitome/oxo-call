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
- --thresholds specifies coverage thresholds (e.g., 1,10,20,30) to count bases meeting each threshold.
- --fast-mode (-x) skips internal cigar operations for faster processing; recommended for most use-cases.
- --fragment-mode (-a) counts full fragment coverage including insert size (proper pairs only).
- --use-median outputs median coverage per region instead of mean for robust statistics.
- -c restricts calculation to a specific chromosome for targeted analysis.

## Pitfalls
- mosdepth has NO subcommands. ARGS starts directly with a prefix and BAM file (e.g., --by 500 -t 8 prefix sample.bam). Do NOT put a subcommand like 'depth' or 'coverage' before the prefix.
- Input BAM must be sorted and indexed; CRAM requires the reference with --fasta.
- Without --by, mosdepth computes per-base coverage to a .bed.gz file (can be large for WGS).
- For target-capture WES, always use --by targets.bed to restrict to capture regions.
- mosdepth default output includes zero-coverage bases — use -x to exclude for faster analysis.
- MAPQ filter (-Q 20) excludes duplicate/secondary reads — use -F 1796 to skip duplicates/secondary/supplementary.
- --fast-mode is recommended for most use-cases but may slightly undercount coverage in complex regions.
- --fragment-mode only works with proper pairs; singleton reads are ignored.
- CRAM files require --fasta reference; failure to provide causes errors.
- Default flag filter 1796 excludes duplicates (1024) + secondary (256) + unmapped (4) + supplementary (512); adjust if needed.

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

### calculate coverage with threshold analysis for QC metrics
**Args:** `--by exons.bed -T 1,10,20,30,50 --prefix thresholds_coverage sample_sorted.bam`
**Explanation:** -T specifies coverage thresholds; outputs bases meeting each threshold per region for coverage QC

### use fast mode for rapid WGS coverage calculation
**Args:** `-x -t 16 --prefix fast_coverage sample_sorted.bam`
**Explanation:** -x fast-mode skips cigar operations; much faster for WGS; recommended for most use-cases

### calculate fragment coverage for ChIP-seq analysis
**Args:** `-a --by peaks.bed -t 8 --prefix fragment_coverage sample_sorted.bam`
**Explanation:** -a fragment-mode counts full fragment including insert; ideal for ChIP-seq peak coverage

### calculate median coverage per region instead of mean
**Args:** `-m --by genes.bed -t 8 --prefix median_coverage sample_sorted.bam`
**Explanation:** -m use-median outputs median coverage; more robust to outliers than mean for gene coverage

### analyze specific chromosome only
**Args:** `-c chr20 -t 8 --prefix chr20_coverage sample_sorted.bam`
**Explanation:** -c restricts to chromosome 20; useful for targeted analysis or testing

### quantize coverage into bins for efficient storage
**Args:** `-q 0:1:10:50:100: --prefix quantized_coverage sample_sorted.bam`
**Explanation:** -q bins coverage (0,1-9,10-49,50-99,100+); creates smaller .bed.gz for genome browsers

### calculate coverage for CRAM file with reference
**Args:** `-t 8 -f reference.fa --prefix cram_coverage sample_sorted.cram`
**Explanation:** -f provides reference FASTA for CRAM decoding; required for CRAM input
