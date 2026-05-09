---
name: ananse
category: Epigenomics / Chromatin Analysis
description: A bioinformatics tool for detecting allele-specific transcription factor binding events from ChIP-seq data using phased genetic variants. Ananse identifies binding sites that show bias toward the maternal or paternal allele by integrating aligned sequencing reads with SNP information.
tags: chip-seq, allele-specific-binding, epigenomics, transcription-factor-binding, chromatin, asb, snp
author: AI-generated
source_url: https://github.com/BoerAF/ananse
---

## Concepts

- Ananse operates on aligned ChIP-seq BAM files in conjunction with a VCF file containing phased heterozygous SNP genotype information. The tool requires that the reads are mapped to a phased reference genome so that allele-specific reads can be distinguished based on their overlapping SNPs.

- The primary output consists of BED files containing genomic regions with allele-specific binding (ASB) signals. Each region includes count information for both alleles, along with statistical significance values indicating whether the observed bias is likely biological rather than stochastic.

- Ananse implements a companion binary called `ananse-build` for preparing reference genomes. This tool constructs the required index files that enable read-level allele assignment during the main analysis, and must be run before performing binding detection.

- The tool supports multiple output formats including BED, narrowPeak, and summary reports. Sensitivity can be tuned through minimum read count thresholds and p-value cutoffs to balance discovery power against false positive rates.

- Ananse is designed to work with diploid organisms where phased genotype data is available (typically human or mouse datasets). It requires heterozygous SNPs within binding regions to distinguish allele origin, making dense variant calling regions particularly informative.

## Pitfalls

- Running ananse without first generating the reference index using `ananse-build` will cause the analysis to fail with ambiguous errors. Always ensure the companion build step completes successfully before proceeding to binding detection.

- Using unphased or partially phased genotype data leads to incorrect allele assignment, resulting in false positive allele-specific calls. Verify that your VCF file contains properly phased haplotypes (look for correct haplotype phasing in the VCF).

- Specifying an incorrect species genome assembly (e.g., using hg19 references with hg38-aligned reads) produces nonsensical results because SNP coordinates will not match the alignment. Ensure consistent genome builds across all input files.

- Ananse requires sufficient read depth at heterozygous sites to call allele-specific binding; low-coverage ChIP-seq experiments (below 10x) may yield noisy or undetectable ASB signals due to insufficient statistical power.

- Failing to filter duplicate reads in the input BAM file inflates read counts for certain genomic regions, potentially creating artificial allele imbalance where none exists. Preprocess your BAM files with standard deduplication tools before running ananse.

## Examples

### Running allele-specific binding detection on a ChIP-seq sample
**Args:** -b sample.bam -v variants.vcf -o asb_output.bed --genome hg38
**Explanation:** This command detects allele-specific binding events by analyzing the aligned reads in the BAM file against known heterozygous variants, outputting significant ASB regions to a BED file using the hg38 genome assembly for coordinate mapping.

### Calling narrowPeak format with high sensitivity
**Args:** -b chipseq.bam -v phased.vcf -o peaks.narrowPeak --format narrowPeak --pvalue 0.05 --minreads 3
**Explanation:** Produces peaks in narrowPeak format with relaxed sensitivity, capturing binding sites with at least 3 supporting reads per allele and a p-value threshold of 0.05 to maximize detection of potential allele-specific events.

### Limiting analysis to specific chromosomes
**Args:** -b treatment.bam -v genotype.vcf -o chr_analysis.bed --chromosomes chr1,chr2,chr3 --genome mm10
**Explanation:** Restricts the allele-specific binding analysis to chromosomes 1, 2, and 3 only, which is useful for testing pipelines or focusing on specific genomic regions of interest while reducing computational time.

### Generating a summary report with all candidate sites
**Args:** -b rep1.bam -v snps.vcf -o full_report.txt --format summary --all-sites
**Explanation:** Outputs a comprehensive summary report including all candidate binding sites regardless of statistical significance, useful for post-hoc filtering or for examining the full landscape of potential allele-specific binding before applying strict thresholds.

### Combining replicate analysis with higher confidence thresholds
**Args:** -b rep1.bam -b rep2.bam -v combined.vcf -o high_conf.bed --pvalue 0.01 --minreads 5 --genome hg38
**Explanation:** Analyzes multiple replicates together while applying stringent significance thresholds to only report high-confidence allele-specific binding sites supported by at least 5 reads per allele across both replicates, reducing false discoveries in high-quality datasets.

### Building reference genome index for allele assignment
**Args:** build -g hg38 -o hg38_asb_index/ --fasta reference.fa
**Explanation:** Uses the companion build tool to construct the allele assignment index from the reference genome, creating necessary lookup structures that enable the main ananse tool to determine which reads originated from each parental haplotype.