---
name: chromatiblock
category: ChIP-seq Analysis / Epigenomics
description: A bioinformatics tool for identifying chromatin blocked regions from sequencing data by detecting significantly depleted read density compared to background. Useful for finding transcription factor binding sites, nucleosome positions, and other protein-DNA interactions that block enzymatic access.
tags: [chip-seq, dnase-seq, atac-seq, peak-calling, chromatin, epigenetics, genomics, blocked-regions]
author: AI-generated
source_url: https://github.com/berkeleyfalsebed/chromatiblock
---

## Concepts

- **Input Format**: ChromatiBlock accepts sorted BAM files containing aligned sequencing reads from ChIP-seq, DNase-seq, or ATAC-seq experiments. The tool requires an index file (.bai) alongside each BAM file for efficient random access.
- **Statistical Model**: The tool uses a negative binomial distribution to model background read density, identifying regions where observed reads fall significantly below expected levels (p-value threshold adjustable via `--pval`).
- **Output Format**: Identified blocked regions are output in BED format with four columns: chromosome, start position, end position, and statistical score (negative log10 p-value). Higher scores indicate more significant blocking.
- **Companion Binary**: chromatiblock-build creates background models from control datasets (input DNA or untreated samples) to improve Blocked region detection accuracy.

## Pitfalls

- **Missing Control Sample**: Running without a control (input DNA or IgG) sample will use genomic average as background, which may miss subtle blocked regions in high-complexity genomes. This leads to false negatives in regions with naturally lower mappability.
- **Incompatible BAM Sorting**: ChromatiBlock requires position-sorted BAM files (not query-name sorted). Using incorrectly sorted BAM files causes silent failures or missing output regions.
- **Insufficient Read Depth**: Samples with fewer than 10 million mapped reads may produce unreliable blocked region calls due to inadequate statistical power, resulting in high false positive rates or empty outputs.
- **Mismatched Genome Build**: Using a background model built for one genome build (e.g., hg19) on data aligned to another (e.g., hg38) produces meaningless coordinates that don't match the actual data.

## Examples

### Identifying blocked regions from ChIP-seq data
**Args:** `--bam chip_sample.bam --out blocked_regions.bed --pval 0.01`
**Explanation:** Runs blocked region detection on the ChIP-seq sample with a p-value threshold of 0.01, output results to BED file.

### Using a control sample for improved detection
**Args:** --bam chip_sample.bam --control input_control.bam --out blocked_with_control.bed --pval 0.05`
**Explanation:** Compares ChIP sample against input DNA control to identify genuine blocked regions by subtracting background signal.

### Adjusting minimum region length
**Args:** `--bam chip_sample.bam --out long_regions.bed --min-length 200 --pval 0.001`
**Analysis:** Filters output to only include blocked regions at least 200bp in length with high statistical significance.

### Running on a specific chromosome
**Args:** `--bam chip_sample.bam --chr chr1 --out chr1_blocked.bed --pval 0.01`
**Explanation:** Restricts analysis to chromosome 1 only, useful for testing or region-specific investigation.

### Using chromatiblock-build to create a control model
**Args:** `--genome hg38 --out genome_background.model input_dna.bam`
**Explanation:** Builds a background model from input DNA data for the hg38 genome to use as reference in subsequent blocked region detection.