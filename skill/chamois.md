---
name: chamois
category: Variant Calling / Genomics
description: chamois is a bioinformatics tool designed for variant detection and genomic analysis. It identifies genetic variants from aligned sequencing data, supporting SNP and indel detection with configurable sensitivity and precision settings.
tags:
- variant calling
- SNP detection
- indel detection
- genomics
- bioinformatics
- germline variants
- somatic variants
author: AI-generated
source_url: https://github.com/example/chamois
---

## Concepts

- **Input Data Model**: chamois takes BAM/CRAM alignment files as primary input, along with a reference genome in FASTA format. The tool expects reads aligned with coordinate-sorted alignments and a corresponding index file.
- **Output Formats**: The tool produces VCF (Variant Call Format) files by default for variant calls, with optional JSON and BED outputs for specific annotations. VCF files include variant quality scores (QUAL), allele depths (AD), and genotype likelihoods (GL).
- **Variant Calling Modes**: chamois supports both germline and somatic variant detection modes, selectable via `--mode germline|somatic`. Germline mode uses haplotype-aware algorithms, while somatic mode includes paired tumor-normal analysis.
- **Filtering Thresholds**: Built-in quality filters include minimum read depth (`--minDP`), minimum allele frequency (`--minAF`), and variant quality score thresholds (`--minQUAL`). These can be combined with site-level and sample-level filters.

## Pitfalls

- **Missing Index Files**: Running chamois without pre-indexed BAM files causes immediate failure. Always generate BAI/CSI indexes using `samtools index` before variant calling to prevent file not found errors during chromosome iteration.
- **Mismatched Reference Genomes**: Using a BAM file aligned to one reference version with a different reference FASTA leads to incorrect coordinate mapping and false variant calls. Verify reference genome consistency between alignment and calling steps.
- **Insufficient Read Depth**: Calling variants with read depth below 10x (`--minDP` default) produces unreliable genotypes, especially for heterozygous calls. Low-depth regions result in high false-positive rates and missing true positives.
- **Mixed PLoidy Configuration**: Failing to set `--ploidy` for haploid regions (like chrX/Y in males) causes incorrect genotype likelihoods. Always specify ploidy for non-diploid analyses to avoid homozygous/heterozygous miscalls.

## Examples

### Call variants from a BAM file with default settings
**Args:** --input aligned.bam --reference ref.fa --output variants.vcf
**Explanation:** This runs variant calling with all default parameters (germline mode, minDP 10, minQUAL 30) on a single BAM file, outputting a VCF file with standard variant records.

### Perform somatic variant detection with tumor-normal pairs
**Args:** --mode somatic --tumor tumor.bam --normal normal.bam --reference ref.fa --output somatic.vcf
**Explanation:** This compares tumor and normal BAM files to identify somatic mutations unique to the tumor sample, using paired analysis to reduce false positives from sequencing artifacts.

### Adjust minimum allele frequency for rare variant detection
**Args:** --input reads.bam --reference ref.fa --output rare_variants.vcf --minAF 0.05 --minQUAL 50
**Explanation:** Setting minAF to 5% allows detection of low-frequency variants, while increasing minQUAL to 50 ensures only high-confidence variant calls are reported.

### Generate annotated output with JSON and BED formats
**Args:** --input sample.bam --reference ref.fa --output annotated --format vcf json bed
**Explanation:** This outputs variant calls in multiple formats: VCF for compatibility, JSON for programmatic parsing, and BED for genome browser visualization.

### Specify ploidy for haploid chromosome analysis
**Args:** --mode germline --input male_sample.bam --reference ref.fa --output male_calls.vcf --ploidy 1 --chromosome chrX
**Explanation:** Setting ploidy to 1 correctly handles haploid regions like chromosome X in male samples, preventing heterozygous miscalls in non-diploid contexts.