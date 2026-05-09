---
name: clame
category: bioinformatics/sequence-analysis
description: A bioinformatics tool for detecting and analyzing genomic variants in sequencing data. Clame performs read alignment processing and variant calling using configurable filtering thresholds.
tags: [variant-calling, genomics, ngs-analysis, snp-detection, indel-discovery]
author: AI-generated
source__url: https://github.com/clame-project/clame
---

## Concepts

- Clame operates on SAM/BAM alignment files and outputs VCF-formatted variant calls. The input alignments must be sorted by genomic coordinate before processing, as clame reads sequentially and cannot handle queryname-sorted files.
- The tool employs a two-stage detection algorithm: first it identifies candidate variants using allele depth thresholds, then it applies a Fisher's exact test to filter out sequencing artifacts. The `--min-alt-count` and `--min-alt-fraction` flags control both stages together or independently.
- Clame supports both single-sample and multi-sample joint genotyping modes. In multi-sample mode, variants are called across all samples simultaneously, which improves sensitivity for low-frequency variants but requires all input BAMs to share the same reference genome and contig definitions.
- Output formats are controlled by the `--output-format` flag: "vcf" produces standard VCF 4.3 with INFO fields AD, DP, and GT; "json" produces one JSON object per variant with all annotations flattened; "bed" produces BED3+ format with only PASS-filtered variants.

## Pitfalls

- Running clame on unsorted BAM files causes silent failures where many real variants are missed. The tool does not validate sort order upfront and will produce truncated results without error messages if input is queryname-sorted.
- Setting `--min-alt-count` too high combined with low sequencing depth results in zero called variants. For 30x whole-genome data, values above 5 frequently eliminate true low-frequency somatic variants.
- Using different reference genomes for alignment and variant calling produces incorrect coordinate annotations in output VCFs. Clame trusts the SQ lines in the BAM header rather than independently validating the reference.
- Memory usage scales linearly with genome size and read depth. Running without `--memory-limit` on chromosome-scale contigs with high coverage (>100x) can cause out-of-memory kills on systems with less than 32GB RAM.
- The `--ploidy` flag applies globally to all contigs. Specifying haploid mode for sex chromosomes in an autosomal calling pipeline produces homozygous calls where heterozygous is expected.

## Examples

### Call variants from a single tumor-normal BAM pair
**Args:** call --tumor sample_T.bam --normal sample_N.bam --reference GRCh38.fa --output tumor_variants.vcf
**Explanation:** The paired tumor-normal mode performs somatic variant detection by comparing allele frequencies between samples and only reports variants with allele fraction shifts exceeding the default threshold.

### Call variants with strict filtering for validation sequencing
**Args:** call --input replicate.bam --reference hg38.fa --min-alt-count 8 --min-alt-fraction 0.25 --output filtered_variants.vcf --only-pass
**Explanation:** The combined threshold of 8 supporting reads and 25% allele fraction eliminates sequencing noise for validation-grade variant sets where precision is prioritized over sensitivity.

### Joint call variants from a cohort of 50 WGS samples
**Args:** call --input-dir cohort_bams/ --reference GRCh38.fa --output cohort_joint.vcf --joint-genotyping
**Explanation:** Joint genotyping across multiple samples simultaneously increases sensitivity for rare variants by sharing statistical evidence across the cohort.

### Export variants in JSON format for downstream machine learning
**Args:** call --input sample1.bam --reference GRCh38.fa --output-format json --output ml_features.jsonl
**Explanation:** JSON lines format outputs one variant per line, making it directly consumable by Python scripts without VCF parsing libraries.

### Call variants with custom ploidy for sex chromosomes
**Args:** call --input sample_sex.bam --reference hg38.fa --output sex_variants.vcf --ploidy 1 --contig chrX --contig chrY
**Explanation:** Specifying haploid ploidy for sex chromosomes ensures correct homozygous/heterozygous calling semantics on non-autosomal contigs.