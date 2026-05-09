---
name: aligncov
category: Sequencing Analysis
description: Computes coverage statistics from sequence alignments in SAM/BAM/CRAM format, including per-base depth, mean coverage, and coverage uniformity metrics.
tags: [coverage, alignment, samtools, sequencing, quality-control]
author: AI-Generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- Input alignment files must be sorted by reference position before running aligncov; unsorted BAM files produce silently incorrect coverage results because reads are not associated with their correct genomic intervals.
- Coverage is computed by iterating through all reads that overlap each reference position, with each read contributing 1 to coverage depth at every base it spans, enabling accurate calculation of both read depth and base-level uniformity metrics.
- Output includes multiple statistics: mean coverage, minimum/maximum depth, standard deviation, and the fraction of bases exceeding a user-specified depth threshold (e.g., 10x, 20x).
- Reads are filtered before coverage calculation using mapping quality thresholds (-q flag) and SAM flag masks, meaning low-quality or ambiguous mappings can be excluded to focus on reliable alignments.
- aligncov handles soft-clipped bases at read ends by excluding clipped regions from coverage contribution, whereas hard-clipped bases are entirely removed from the CIGAR string before calculation.

## Pitfalls

- Using an unsorted BAM file as input causes aligncov to report coverage values for reads in file order rather than genomic position order, producing meaningless statistics for downstream analysis.
- Omitting the minimum mapping quality filter (-q 0) when alignments contain many multi-mapping reads inflates coverage estimates because each mapping location receives full read depth from all copies.
- Specifying an incorrect reference name (case-sensitive) results in zero coverage reported with no error message, because aligncov silently returns zero bases matching the specified chromosome.
- For paired-end data, not specifying proper filtering flags may count both reads of a pair independently, effectively doubling coverage depth compared to single-end sequencing at identical input depth.
- Using a very short window size for targeted regions can produce highly variable coverage estimates due to random sampling effects; smaller windows require higher depth for statistically reliable metrics.

## Examples

### Calculate mean coverage for an entire reference genome
**Args:** input.bam
**Explanation:** Running aligncov on a sorted BAM file without additional options outputs per-reference statistics including mean coverage, which is sufficient for whole-genome quality assessment.

### Filter reads by mapping quality before computing coverage
**Args:** input.bam -q 20
**Explanation:** Specifying mapping quality threshold of 20 excludes reads with more than 1 in 100 probability of being misaligned, improving coverage accuracy by removing unreliable mappings.

### Output coverage for a specific genomic interval
**Args:** input.bam chr1:1000000-2000000
**Explanation:** Providing chromosome and coordinates restricts analysis to that 1 Mb region, useful for evaluating coverage uniformity within a gene or targeted panel without processing the full genome.

### Calculate coverage uniformity metrics across multiple targets
**Args:** input.bam -b targets.bed
**Explanation:** Using a BED file with the -b flag computes separate coverage statistics for each interval listed, producing a per-target table ideal for targeted sequencing validation.

### Report fraction of bases above a depth threshold
**Args:** input.bam -d 30
**Explanation:** The -d flag outputs the percentage of genomic bases covered by at least 30 reads, a standard quality metric for clinical sequencing pipelines requiring minimum coverage uniformity.