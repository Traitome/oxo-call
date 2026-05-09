---
name: control-freec
category: variant-calling
description: Detect copy number variations (CNVs) and loss of heterozygosity (LOH) from aligned sequencing reads using read depth analysis with GC-content normalization and breakpoint detection.
tags:
  - copy-number-variation
  - cnv-detection
  - read-depth-analysis
  - gc-correction
  - break-point-detection
  - loss-of-heterozygosity
  - samtools
author: AI-generated
source_url: https://github.com/BoevaLab/FREEC
---

## Concepts

- **Read depth normalization**: control-freec normalizes read depth by comparing the test sample against a matched control, then applies GC-content correction using polynomial regression (configurable via `degree`) to account for systematic biases in sequencing coverage.
- **Multi-modal output**: The tool produces both numeric ratio files (text) and binary visualization outputs (BAM/CRAM format), enabling direct visualization of copy number alterations alongside the original alignments.
- **Ploidy and breakpoint modeling**: By specifying ploidy (default 2) and break point threshold, the tool models both uniform copy number states and discrete break points where copy number changes abruptly, making it suitable for detecting focal amplifications and deletions.
- **BAF (B-allele frequency) integration**: When provided with heterozygous SNP positions (via `SNPposFile`), control-freec calculates B-allele frequencies to distinguish between copy-neutral LOH, heterozygous deletions, and balanced rearrangements.
- **Window-based vs breakpoint modes**: The tool operates in two modes—window-based (`window` flag) for uniform segmentation, or break point detection mode for precise localization of copy number transition points.

## Pitfalls

- **Missing GC-content correction**: Running control-freec without `--confineGC` or setting an inappropriately high `degree` (e.g., degree=0) produces false positive CNVs in GC-rich or GC-poor regions, as the tool cannot account for systematic coverage biases inherent to NGS protocols.
- **Unmatched control/tumor pairing**: Using an unrelated or poorly matched control sample introduces systematic artifacts; the tool requires biologically relevant controls (e.g., matched normal for tumor samples, or different library preparations for germline CNV detection).
- **Insufficient sequencing depth**: Samples with coverage below 10x (or 15x for targeted sequencing) produce noisy ratio estimates, causing the segmentation algorithm to miss real CNVs or call spurious ones—always verify depth before analysis.
- **Incorrect chromosome notation**: Specifying chromosomes in non-standard format (e.g., "chr1" when the BAM uses "1") causes the tool to skip those chromosomes silently, producing incomplete or empty output files without warning messages.
- **Failure to index BAM/CRAM files**: control-freec requires position-sorted and indexed BAM/CRAM files; using unsorted or unindexed files causes failures or crashes during the read counting phase.

## Examples

### Detect CNVs in a tumor sample using a matched normal control
**Args:** `--confineGC --degree 3 --control normal.bam --sample tumor.bam --genome /path/hg19.fa --output outputDir/`
**Explanation:** The `--confineGC` and `--degree 3` flags enable GC-content correction using a third-degree polynomial to normalize read depth, while `--control` and `--sample` specify the paired normal and tumor BAM files for comparison.

### Detect CNVs with targeted breakpoint resolution
**Args:** `--breakPointThreshold 0.25 --window 50000 --ploidy 2 --sample target.bam --genome /path/hg19.fa --output outputDir/`
**Explanation:** The `--breakPointThreshold 0.25` sets sensitivity for detecting change points where copy number shifts, and `--window 50000` uses 50kb windows for segmentation, producing precise localization of CNV boundaries.

### Analyze specific chromosomes for faster runtime
**Args:** --chr 1 --chr 2 --chr 3 --sample sample.bam --genome /path/hg19.fa --output outputDir/ --sample sample.bam --genome /path/hg19.fa --output outputDir/ --sample sample.bam --genome /path/hg19.fa --output outputDir/
**Explanation:** Chromosome-restricted analysis reduces computation time and output file size when investigating specific chromosomes, useful for validating candidates or performing iterative refinement.

### Detect CNVs with B-allele frequency for LOH identification
**Args:** --SNPposFile snp_positions.txt --sample tumor.bam --genome /path/hg19.fa --output outputDir/ --sample tumor.bam --genome /path/hg19.fa --output outputDir/ --SNPposFile snp_positions.txt --sample tumor.bam --genome /path/hg19.fa --output outputDir/
**Explanation:** The `--SNPposFile` provides heterozygous SNP positions, enabling B-allele frequency calculation to distinguish copy-neutral LOH from heterozygous deletions or balanced events, critical for tumor analysis.

### Generate visualization-friendly output in BAM format
**Args:** --makeBam --sample sample.bam --genome /path/hg19.fa --output outputDir/ --sample sample.bam --genome /path/hg19.fa --output outputDir/
**Explanation:** The `--makeBam` flag writes copy number ratios directly into the BAM file annotations, allowing visualization of CNVs alongside read alignments in GenomeView or IGV without additional conversion steps.

### Run with parallel threads for large whole-genome datasets
**Args:** --maxThreads 8 --sample sample.bam --genome /path/hg19.fa --output outputDir/
**Explanation:** The `--maxThreads 8` enables multi-threaded read counting and segmentation, significantly reducing runtime for high-coverage whole-genome samples (30x+) where I/O and computation are bottlenecks.

### Generate ratio output in bedGraph format for custom analysis
**Args:** --outputBedGraph --sample sample.bam --genome /path/hg19.fa --output outputDir/
**Explanation:** The `--outputBedGraph` flag writes copy number ratio files in bedGraph format,便于 downstream integration with other tools like UCSC Genome Browser or custom segmentation algorithms.

### Use control-freec-build to generate reference index for hg19
**Args:** -g hg19 -o hg19.grp.fa
**Explanation:** The companion tool `control-freec-build` prepares the reference genome index required by control-freec, where `-g` specifies the genome name and `-o` the output fasta file prefix for efficient read mapping during CNV analysis.