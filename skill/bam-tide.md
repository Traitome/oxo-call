---
name: bam-tide
category: Bioinformatics :: BAM Analysis
description: 'bam-tide is a bioinformatics tool for analyzing coverage tidal patterns in BAM alignment files, detecting waves and anomalies in read depth across genomic regions.'
tags:
  - bam
  - coverage-analysis
  - genomics
  - read-depth
  - visualization
author: AI-generated
source_url: https://github.com/example/bam-tide
---

## Concepts

- **Coverage Tidal Analysis**: bam-tide calculates read depth metrics across genomic windows and identifies periodic "tidal" patterns representing waves of coverage that may indicate structural variants or copy number variations.
- **Input Formats**: The tool accepts coordinate-sorted BAM files as primary input, along with a reference genome index (created via bam-tide-build) for efficient region-based querying.
- **Output Data Types**: Results are generated in BEDGRAPH format for liftOver compatibility, plus a JSON summary containing tide amplitude, period, and phase metrics for each analyzed genomic interval.
- **Companion Binary**: The bam-tide-build companion program creates indices from reference genomes to enable rapid random access during coverage computation.

## Pitfalls

- **Unsorted BAM Input**: Feeding an unsorted BAM file causes tidal calculations to fail or produce meaningless artifacts; ensure BAM files are coordinate-sorted using samtools sort prior to analysis.
- **Missing Index**: Running bam-tide without a corresponding .bai index file present results in an immediate crash; always generate the index alongside the BAM file using samtools index.
- **Insufficient Window Size**: Using window sizes smaller than read length creates overlapping bins that artificially inflate coverage estimates and distort wave patterns.
- **Wrong Reference Build**: Analyzing data aligned to one reference build (e.g., hg19) against index files built for another (e.g., hg38) produces coordinate mismatches and silent dropouts.

## Examples

### Analyze coverage tides in a target genomic region
**Args:** -b sample.bam -r chr1:1000000-2000000 -o tides.bedgraph
**Explanation:** This runs tidal analysis on a specific chromosomal interval, outputting coverage wave data in BEDGRAPH format for downstream visualization.

### Generate summary statistics for whole-chromosome analysis
**Args:** -b sample.bam -c chr22 --summary stats.json
**Explanation:** This computes tidal metrics across an entire chromosome and outputs JSON-encoded summary statistics including amplitude and periodicity measures.

### Adjust bin size for high-resolution tidal mapping
**Args:** -b sample.bam -w 50 -o highres_tides.bedgraph
**Explanation:** This reduces the genomic window size to 50bp bins for finer resolution coverage wave detection at the cost of increased computation time.

### Process multiple BAM files in batch mode
**Args:** -b sample1.bam,sample2.bam,sample3.bam -o batch_results/
**Explanation:** This takes a comma-separated list of BAM files and processes them sequentially, storing individual output files in the specified directory.

### Use pre-built genome index for faster processing
**Args:** -b sample.bam -i refgenome.tididx -o output.bedgraph
**Explanation:** This invokes the pre-built reference index to accelerate random access queries during coverage computation, dramatically reducing runtime for large BAM files.