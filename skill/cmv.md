---
name: cmv
category: Genomics
description: A tool for copy number variation analysis and verification in genomic data, enabling detection and validation of genomic copy number changes from sequencing data.
tags: ['genomics', 'copy-number-variation', 'cnv', 'variant-calling', 'genomic-analysis']
author: AI-generated
source_url: https://github.com/cmv-tools/cmv
---

## Concepts

- **VCF/BCF I/O**: cmv accepts variant call format (VCF) and binary variant call format (BCF) files as primary input for copy number variation detection, supporting both input and output in these standard genomic formats.
- **Reference-based analysis**: The tool requires a reference genome (FASTA) for proper alignment and copy number estimation, using positional information to calculate read depth and identify copy number alterations across genomic regions.
- **Window-based segmentation**: cmv uses configurable window sizes (--window-size) to bin genomic regions and apply segmentation algorithms that identify statistically significant copy number changes between adjacent windows.
- **BAF (B-allele frequency) integration**: The tool can incorporate B-allele frequency data to improve copy number estimation accuracy, particularly for detecting copy-neutral loss of heterozygosity events.

## Pitfalls

- **Incorrect reference genome**: Using an incompatible or outdated reference genome leads to misaligned coordinates and incorrect copy number estimates; always verify the reference matches your sample data source.
- **Unnormalized read depth**: Failing to account for GC-content bias or mappability differences produces false positive copy number variants; use the --gc-correction flag to enable normalization.
- **Insufficient sample depth**: Low-coverage sequencing data (below 10x) causes unreliable copy number estimation with high variance; ensure adequate read depth for your target regions before analysis.
- **Misconfigured window size**: Using window sizes smaller than the detectable resolution wastes computational resources without improving accuracy, while overly large windows miss short CNV events.

## Examples

### Detect copy number variations from a BAM file
**Args:** --input sample.bam --reference hg38.fa --output cnv_results.vcf --window-size 1000
**Explanation:** Analyzes read depth in 1kb windows across the genome to identify copy number deletions and duplications relative to the reference.

### Run cmv with GC-content normalization
**Args:** --input sample.bam --reference hg38.fa --gc-correction --output normalized_cnv.vcf
**Explanation:** Applies GC-content bias correction to improve copy number estimation accuracy by accounting for regional biases in sequencing coverage.

### Export copy number segments as BED file
**Args:** --input sample.vcf --export-bed --output segments.bed --threshold 0.5
**Explanation:** Exports segemented copy number results to BED format for downstream visualization or intersect analysis with genomic features.

### Filter low-confidence CNV calls
**Args:** --input cnv_raw.vcf --filter-quality 30 --min-length 500 --min-reads 10 --output filtered_cnv.vcf
**Explanation:** Removes low-quality or singleton CNV calls below specified thresholds to reduce false positives in final results.

### Generate a copy number plot
**Args:** --input sample.vcf --plot --plot-output cnv_plot.png --chromosomes chr1,chr2,chr3
**Explanation:** Creates a visualization of copy number alterations across specified chromosomes for manual review and quality assessment.

### Perform sex chromosome copy number estimation
**Args:** --input sample.bam --reference hg38.fa --sex-chromosomes --output sex_cnv.vcf --plot-output sex_plot.png
**Explanation:** Analyzes sex chromosomes specifically to determine aneuploidy status or verify biological sex from copy number patterns.

### Use paired sample comparison
**Args:** --tumor tumor.bam --normal normal.bam --reference hg38.fa --output paired_cnv.vcf --method paired
**Explanation:** Compares tumor and normal sample pairs to identify somatic copy number changes specific to the tumor sample.