---
name: biopet-scatterregions
category: Bioinformatics - Genomic Region Processing
description: A Biopet tool that generates scatter/gather regions by splitting reference genomes or genomic intervals into smaller, non-overlapping blocks suitable for parallel processing in bioinformatics pipelines. Outputs BED format region files.
tags:
  - biopet
  - regions
  - scatter-gather
  - parallel-processing
  - bed
  - genomics
  - pipeline
author: AI-generated
source_url: https://biopet.github.io/biopet
---

## Concepts

- **Input Flexibility**: Accepts either a reference genome in FASTA format or an existing BED file containing target genomic intervals. When given a FASTA, it automatically calculates chromosome lengths and generates regions across the entire genome.
- **Output BED Format**: Produces a standard BED file with chrom, start, and end columns, plus a fourth column indicating the region index. This format is compatible with tools like GATK, BQSR, and variant callers that support scatter-gather parallelism.
- **Region Splitting Strategies**: Supports three primary modes — equal-base splitting (dividing genome into regions of roughly equal total bases), equal-count splitting (dividing into a specified number of regions), and chromosome-based splitting (one region per chromosome).
- **Overlap Control**: Includes an optional overlap parameter to add flanking sequence to each region, preventing edge case issues where variants or reads near region boundaries are missed during parallel processing.

## Pitfalls

- **Overlapping Regions Cause Duplicate Processing**: Setting overlap too high without post-processing deduplication will cause reads or variants to be processed multiple times, inflating file sizes and potentially creating false duplicate variants in variant calling pipelines.
- **Ignoring Unplaced Contigs**: By default, the tool may skip unplaced scaffolds or alternative haplotypes in the reference, which can cause silent data loss when analyzing non-standard chromosomes or complex genomes with many contigs.
- **Mismatched Region Count and Cluster Resources**: Generating too few regions underutilizes cloud computing clusters, while too many regions create excessive overhead from file I/O and job scheduling, negating parallelization benefits.
- **Missing Chromosome Length Information**: When providing a custom BED input instead of a reference FASTA, omitting chromosome lengths causes region calculation errors in equal-base splitting mode.

## Examples

### Generate 10 equal-sized scatter regions from a reference genome
**Args:** `-R hs37d5.fa -N 10 -o scatter_regions.bed`
**Explanation:** Splits the hs37d5 human reference into approximately 10 equal-base regions, suitable for parallel variant calling across a compute cluster.

### Create scatter regions with 100bp overlap between adjacent regions
**Args:** `-R GRCh38.fa -N 20 -o scatter.bed --overlap 100`
**Explanation:** Adds 100bp of flanking sequence to each region boundary, ensuring reads spanning region edges are captured correctly in parallel alignment jobs.

### Generate one region per chromosome from a reference
**Args:** `-R hg19.fa -o per_chr.bed --perChromosome`
**Explanation:** Creates a separate BED region for each chromosome, useful when chromosome-level parallelism is preferred over finer-grained region splitting.

### Use an existing target intervals BED file to generate scatter regions
**Args:** `-T targets.bed -N 50 -o scatter_targets.bed`
**Explanation:** Takes the provided target intervals file and subdivides it into 50 smaller regions, maintaining the original genomic coordinates while distributing workload.

### Generate scatter regions with minimum region size constraints
**Args:** `-R CHM1.fa -N 100 -o scatter.bed --minSize 1000000`
**Explanation:** Ensures no generated region is smaller than 1Mb, preventing the creation of trivially small regions that would cause excessive job overhead in parallel pipelines.