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
**Explanation:** mosdepth command; --by 500 500bp windows; -t 8 threads; --prefix sample_coverage output prefix; sample_sorted.bam input BAM

### calculate coverage over target regions for WES
**Args:** `--by targets.bed -t 4 --prefix wes_coverage sample_sorted.bam`
**Explanation:** mosdepth command; --by targets.bed target regions; -t 4 threads; --prefix wes_coverage output prefix; sample_sorted.bam input BAM

### calculate per-base depth with MAPQ filter
**Args:** `-t 4 -Q 20 -F 1796 --prefix filtered_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -t 4 threads; -Q 20 minimum MAPQ; -F 1796 excludes duplicate+secondary+unmapped+supplementary reads; --prefix filtered_coverage output prefix; sample_sorted.bam input BAM

### get summary statistics only without per-base output
**Args:** `-n -t 8 --prefix summary_only sample_sorted.bam`
**Explanation:** mosdepth command; -n disables per-base output; -t 8 threads; --prefix summary_only output prefix; sample_sorted.bam input BAM

### calculate coverage with threshold analysis for QC metrics
**Args:** `--by exons.bed -T 1,10,20,30,50 --prefix thresholds_coverage sample_sorted.bam`
**Explanation:** mosdepth command; --by exons.bed region BED; -T 1,10,20,30,50 coverage thresholds; --prefix thresholds_coverage output prefix; sample_sorted.bam input BAM

### use fast mode for rapid WGS coverage calculation
**Args:** `-x -t 16 --prefix fast_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -x fast-mode skips cigar operations; -t 16 threads; --prefix fast_coverage output prefix; sample_sorted.bam input BAM

### calculate fragment coverage for ChIP-seq analysis
**Args:** `-a --by peaks.bed -t 8 --prefix fragment_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -a fragment-mode counts full fragment; --by peaks.bed regions; -t 8 threads; --prefix fragment_coverage output prefix; sample_sorted.bam input BAM

### calculate median coverage per region instead of mean
**Args:** `-m --by genes.bed -t 8 --prefix median_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -m use-median outputs median coverage; --by genes.bed regions; -t 8 threads; --prefix median_coverage output prefix; sample_sorted.bam input BAM

### analyze specific chromosome only
**Args:** `-c chr20 -t 8 --prefix chr20_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -c chr20 restricts to chromosome 20; -t 8 threads; --prefix chr20_coverage output prefix; sample_sorted.bam input BAM

### quantize coverage into bins for efficient storage
**Args:** `-q 0:1:10:50:100: --prefix quantized_coverage sample_sorted.bam`
**Explanation:** mosdepth command; -q 0:1:10:50:100: bins coverage into levels; --prefix quantized_coverage output prefix; sample_sorted.bam input BAM

### calculate coverage for CRAM file with reference
**Args:** `-t 8 -f reference.fa --prefix cram_coverage sample_sorted.cram`
**Explanation:** mosdepth command; -t 8 threads; -f reference.fa reference FASTA for CRAM; --prefix cram_coverage output prefix; sample_sorted.cram input CRAM
