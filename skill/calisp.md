---
name: calisp
category: Variant Calling
description: A bioinformatics tool for calling genetic variants (SNPs and indels) from aligned sequencing reads. Calisp analyzes BAM/CRAM files against a reference genome and outputs variant calls in VCF format, with support for read-backed genotype calling and variant filtering.
tags: [variant-calling, snp, indel, vcf, genomics, genetics, sequence-analysis, bioinformatics]
author: AI-generated
source_url: https://github.com/calisp/calisp
---

## Concepts

- **Input Format**: Calisp accepts aligned reads in BAM or CRAM format indexed with the reference genome. A FASTA reference genome file must also be provided. Input files should be sorted and indexed using standard tools like SAMtools.
- **Output Format**: Primary output is a VCF (Variant Call Format) file containing variant calls with genotype information, quality scores (GQ), read depth (DP), and allele frequencies. Optional output includes a BED file of variant regions.
- **Read-Backed Genotype Calling**: Calisp performs probabilistic genotype calling using read-level evidence, considering read coverage, base quality scores, and mapping qualities to assign genotypes (homozygous reference, heterozygous, homozygous alternate).
- **Variant Filtering**: The tool supports hard filtering via quality thresholds (minQUAL, minDP) and soft filtering using tag annotations in the VCF. FILTER field annotations indicate low-quality calls.

## Pitfalls

- **Missing Index Files**: Running calisp without pre-indexed BAM/CRAM files will cause failures. Always ensure corresponding .bai or .crai index files exist in the same directory before running analyses.
- **Incompatible Reference Genome**: Using a reference genome version that differs from the one used for alignment will produce incorrect variant calls or no calls. Verify the exact reference build (e.g., hg38, GRCh38) matches between alignment and variant calling steps.
- **Low-Quality Variants Passed by Default**: The default minimum quality threshold (minQUAL=30) may be too lenient for high-confidence applications, leading to false positive variants. Adjust thresholds based on coverage and application requirements.
- **Memory Exhaustion with Large Datasets**: Processing whole-genome BAM files without specifying chunked processing can exceed available RAM, especially for high-coverage datasets. Use region-based (-r) or interval-based processing to manage memory.

## Examples

### Call variants from a single BAM file against a reference genome
**Args:** -r hs37d5.fa -i sample.bam -o variants.vcf
**Explanation:** This runs variant calling on a single sample using the provided reference genome and outputs all detected SNPs and indels to a VCF file.

### Call variants with increased minimum quality threshold
**Args:** -r hs37d5.fa -i sample.bam -o strict_variants.vcf --min-qual 50
**Explanation:** This applies a stricter quality filter (QUAL >= 50), reducing false positives at the cost of possibly missing true low-confidence variants.

### Call variants only for a specific genomic region
**Args:** -r hs37d5.fa -i sample.bam -o region_variants.vcf -r chr1:1000000-2000000
**Explanation:** This restricts variant calling to chromosome 1 positions 1,000,000-2,000,000, useful for targeted analysis or testing subsets of data.

### Call variants with multiallelic site handling disabled
**Args:** -r hs37d5.fa -i sample.bam -o biallelic.vcf --no-multiallelic
**Explanation:** This processes only biallelic sites, simplifying output and reducing computational time when multiallelic calls are not needed.

### Call variants with reduced read coverage for genotype assignment
**Args:** -r hs37d5.fa -i sample.bam -o lowcov.vcf --min-dp 5
**Explanation:** This reduces the minimum read depth required for genotype calls to 5x, suitable for low-coverage sequencing where higher depth is not available.

### Generate an annotated variants file including FILTER field
**Args:** -r hs37d5.fa -i sample.bam -o annotated.vcf --filter-expression "DP