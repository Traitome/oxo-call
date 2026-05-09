---
name: bamaligncleaner
category: sequence-analysis/alignment-cleaning
description: A tool for cleaning and filtering BAM alignments, removing low-quality reads, fixing mates, and marking duplicates in aligned sequencing data.
tags:
  - bam
  - alignment
  - quality-control
  - samtools
  - read-filtering
  - duplicate-marking
author: AI-generated
source_url: https://github.com/etsui/bamaligncleaner
---

## Concepts

- **BAM Input/Output Format**: bamaligncleaner operates on sorted and indexed BAM files. Input files must be coordinate-sorted with associated .bai index files. The tool accepts both uncompressed BAM and gzipped BAM output, with CRAM format support where available.

- **Quality-Based Read Filtering**: The tool filters reads based on multiple quality metrics including MAPQ (mapping quality) scores, base quality values, alignment scoring, and soft-clipping rates. Reads falling below configurable thresholds are removed or redirected to a separate reject file.

- **Mate Pair Resolution**: When processing paired-end data, bamaligncleaner can fix improperly paired reads by checking insert size distributions, verifying read orientation (FRstrandness), and recalculating mate information. Mate修复 is essential for downstream tools like GATK that require properly paired reads.

- **Duplicate Marking and Removal**: The tool implements optical duplicate detection by examining read positions on the flow cell. Coordinates within a configurable distance threshold (default 100bp) with identical cluster IDs are marked as duplicates. Duplicate marking preserves one read while flagging others.

- **Reference-Based Read Classification**: For variant calling workflows, bamaligncleaner classifies reads by their alignment to specific genomic regions. This enables selective extraction of reads overlapping target regions (via BED files) or removal of off-target alignments before variant detection.

## Pitfalls

- **Unsorted Input BAM Files**: Running bamaligncleaner on unsorted or name-sorted BAM files causes mate pair resolution to fail catastrophically. Off-by-one megabyte+ output files may be generated with millions of orphaned reads. Always presort with `samtools sort -o sorted.bam input.bam` before processing.

- **MAPQ Threshold Mismatch**: Setting the MAPQ threshold too low (e.g., 10) retains ambiguously mapped reads that increase false positive variant calls. Setting it too high (e.g., 60) removes valid multi-mapping reads needed for structural variant detection. Default of 20 is inappropriate for RNA-seq data where MAPQ values are compressed.

- **Ignoring Read Group Information**: When read groups are absent or malformed in the BAM @RG tag, duplicate marking produces incorrect results because reads from the same sample are not grouped together. This leads to either over-marking or under-marking of duplicates, corrupting downstream allele frequency calculations.

- **Memory Exhaustion on Large Files**: Processing whole-genome BAM files larger than 50GB without streaming generates out-of-memory errors. The tool must be used in streaming mode (`-- streaming`) for files exceeding available RAM, though streaming mode disables some downstream operations like indexed lookup.

- **Off-Target Region Contamination**: When extracting target regions via BED file, reads partially overlapping target boundaries may be inconsistently handled depending on the `-O` overlap fraction parameter. Using default overlap settings can silently exclude or include reads that should be filtered, skewing coverage calculations.

## Examples

### Filter BAM by mapping quality score
**Args:** `input.bam --min-mapq 30 --out filtered.bam`
**Explanation:** This removes all reads with MAPQ scores below 30, which are ambiguously mapped or align to multiple genomic positions, producing a cleaner set of uniquely aligned reads for variant calling.

### Mark duplicates in tumor-sample BAM
**Args:** `--tumor tumor.bam --output-marked tumor.marked.bam --duplicate-metrics tumor.dup.metrics --max-distance 2500`
**Explanation:** This marks optical duplicates in the tumor sample using a 2500bp cluster distance threshold, outputting duplicate metrics for downstream Mutect2 analysis.

### Split tumor/normal paired analysis
**Args:** `--tumor tumor.bam --normal normal.bam --min-mapq 20 --out tumor.clean.bam --out-normal normal.clean.bam`
**Explanation:** This simultaneously processes both tumor and normal BAM files with identical filtering parameters, ensuring matched quality between samples for somatic variant detection workflows.

### Extract reads overlapping target regions
**Args:** `input.bam --region targets.bed --operation intersect --out target.reads.bam --overlap-fraction 0.5`
**Explanation:** This extracts all reads overlapping target regions by at least 50% of their length, which is required for targeted NGS panels where only on-target bases should be considered for variant calling.

### Fix mate information for paired-end data
**Args:** `input.bam --fix-mate --fix-orphan --out fixed.bam`
**Explanation:** This recalculates mate coordinates and insert sizes while flagging orphaned reads (reads with unmapped mates), which is required before running GATK BaseRecalibrator that expects properly paired alignments.

### Clean and sort BAM in pipeline
**Args:** `--input unsorted.bam --min-base-quality 20 --min-length 50 --sort-output --out clean.sorted.bam`
**Explanation:** This combines filtering by base quality and read length with coordinate sorting in a single pass, reducing disk I/O compared to running separate sort and filter operations on large BAM files.