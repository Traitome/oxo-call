---
name: accusnv
category: Copy Number Variation Analysis
description: Allele-specific somatic copy number alteration (SCNA) detection tool that analyzes read depth and allele fractions from BAM sequencing files to identify genomic copy number changes in tumor samples.
tags:
  - copy-number-variation
  - allele-specific
  - somatic-mutations
  - tumor-analysis
  - read-depth
  - snp-analysis
  - genomics
  - cancer-genomics
author: AI-generated
source_url: https://github.com/ elemento/accusnv
---

## Concepts

- accusnv analyzes read coverage and allele fractions at SNP positions to detect copy number alterations, requiring indexed BAM files and a genomic SNP positions file (typically in BED or VCF format) as key inputs for accurate allele-specific read counting.
- The tool distinguishes between germline heterozygous SNPs and somatic copy number events by calculating the B-allele frequency (BAF) deviation from the expected 0.5 ratio at heterozygous sites, enabling detection of allele-specific copy number gains and losses.
- Output consists of tab-delimited segment files containing log2 copy number ratios, BAF values, and genomic coordinates, which can be visualized in genome browsers or downstream used for calling discrete copy number states (0, 1, 2, 3+ copies).
- The tool supports both tumor-only mode (using population allele frequencies) and tumor-matched mode (using matched normal sample) for improved specificity, with the choice significantly affecting sensitivity for low-frequency subclones.
- accusnv applies GC-bias correction and noise filtering to raw read depth signals before segmentation, which is critical for accurate copy number estimation in regions with repetitive sequence or extreme GC content.

## Pitfalls

- Running accusnv without first indexing the BAM file (using samtools index) causes the tool to fail with a cryptic error, as the tool relies on random access through the BAM index to efficiently query SNP loci.
- Specifying incorrect or outdated SNP database files leads to inflated or missing calls in clinically relevant regions, particularly for rare variants not present in older reference files, affecting downstream clinical interpretation.
- Using tumor-only mode on samples with high tumor purity heterogeneity results in underestimation of copy number alterations because allele fraction deviations become diluted below detection thresholds.
- Failing to set appropriate minimum read depth thresholds can cause false positive copy number calls in low-coverage regions, especially when sequencing depth is unevenly distributed across the genome.
- Ignoring the normal contamination parameter when analyzing impure tumor samples leads to systematic bias toward normal copy number states at contaminated loci, masking true somatic events.

## Examples

### Detect SCNAs from a tumor-only BAM using default SNP panel
**Args:** tumor_only --bam tumor_sample.bam --snp snp_database.bed --outdir results/
**Explanation:** This runs accusnv in tumor-only mode, using read depth and BAF deviations to call copy number alterations without a matched normal sample.

### Analyze tumor and matched normal BAM for precise SCNA detection
**Args:** tumor_normal --tumor tumor.bam --normal normal.bam --snp snp_database.bed --out results/
**Explanation:** This runs accusnv with tumor-normal matched mode, which provides higher specificity by directly comparing allele fractions between tumor and matched normal samples.

### Run with custom minimum depth and GC correction
**Args:** tumor_only --bam sample.bam --snp snp_panel.bed --min-depth 20 --gc-correct --outdir output/
**Explanation:** This sets a minimum read depth filter of 20 and enables explicit GC-bias correction to improve accuracy in GC-biased regions.

### Export results in annotated SEG format for visualization
**Args:** tumor_normal --tumor tumor.bam --normal normal.bam --snp snp.bed --seg-output --out results/
**Explanation:** This exports results in SEG format files compatible with genome browsers like IGV for downstream visualization and manual review of copy number segments.

### Specify normal contamination fraction for impure tumor samples
**Args:** tumor_only --bam impure_tumor.bam --snp snp.bed --normal-contamination 0.15 --outdir results/
**Explanation:** This accounts for 15% normal cell contamination in the BAF and copy number calculations, improving accuracy when tumor purity is known to be reduced.