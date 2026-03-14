---
name: bcftools
category: variant-calling
description: Tools for variant calling and manipulating VCF/BCF files
tags: [vcf, bcf, variant-calling, snp, indel, genotyping, ngs]
author: oxo-call built-in
source_url: "https://samtools.github.io/bcftools/bcftools.html"
---

## Concepts

- BCF is binary VCF (smaller/faster); bcftools works with both. Use -O b for BCF output, -O z for gzipped VCF (recommended), -O v for plain VCF.
- bcftools view is the Swiss-army knife for VCF filtering: -s selects samples, -r for regions, -f for FILTER field, -i/-e for INFO/FORMAT expressions.
- bcftools call performs variant calling from bcftools mpileup output; use -m for multiallelic calling (modern), -v to output only variants.
- The standard variant calling pipeline: bcftools mpileup -f ref.fa bam | bcftools call -m -v -o variants.vcf
- bcftools norm deduplicates and normalizes indels (left-align, split multi-allelic); use -f ref.fa for proper normalization.
- VCF files must be bgzip-compressed and tabix-indexed for region queries: bgzip -c file.vcf > file.vcf.gz && tabix -p vcf file.vcf.gz

## Pitfalls

- bcftools view filters by FILTER field with -f; use -i 'QUAL>20' for INFO-based filtering (not -f QUAL).
- bcftools call requires sorted BAM input; use samtools sort first.
- Multi-sample VCF merge with bcftools merge requires all input VCFs to be bgzipped and tabix-indexed.
- bcftools stats outputs statistics, not a VCF — do not try to pipe it back into VCF tools.
- -O z outputs gzipped VCF but does NOT automatically tabix-index it — run tabix after.
- Region strings use the format 'chr:start-end' with 1-based coordinates (same as VCF).

## Examples

### call variants from a BAM file against a reference genome
**Args:** `mpileup -f reference.fa -O u input.bam | bcftools call -m -v -O z -o variants.vcf.gz`
**Explanation:** -f specifies reference; mpileup -O u pipes uncompressed BCF; call -m uses multiallelic model; -v outputs only variant sites

### filter VCF to keep only high-quality SNPs (QUAL > 30, depth > 10)
**Args:** `view -i 'QUAL>30 && INFO/DP>10 && TYPE="snp"' -O z -o filtered.vcf.gz input.vcf.gz`
**Explanation:** -i applies INFO field filter expression; -O z outputs bgzipped VCF

### merge multiple VCF files from different samples
**Args:** `merge -O z -o merged.vcf.gz sample1.vcf.gz sample2.vcf.gz sample3.vcf.gz`
**Explanation:** all inputs must be bgzip'd and tabix-indexed; outputs merged multi-sample VCF

### extract a specific sample from a multi-sample VCF
**Args:** `view -s SAMPLE_NAME -O z -o sample.vcf.gz multisample.vcf.gz`
**Explanation:** -s specifies sample name; use -s ^SAMPLE to exclude instead

### normalize indels and split multi-allelic variants
**Args:** `norm -m -any -f reference.fa -O z -o normalized.vcf.gz input.vcf.gz`
**Explanation:** -m -any splits all multi-allelic records; -f enables left-normalization of indels

### compute variant statistics for a VCF file
**Args:** `stats input.vcf.gz > stats.txt`
**Explanation:** outputs detailed statistics including ts/tv ratio, indel lengths, quality distributions

### select only SNPs from a VCF file
**Args:** `view -v snps -O z -o snps.vcf.gz input.vcf.gz`
**Explanation:** -v snps selects only SNP records; use -v indels for indels only

### annotate VCF with a reference VCF (add ID field from dbSNP)
**Args:** `annotate -a dbsnp.vcf.gz -c ID -O z -o annotated.vcf.gz input.vcf.gz`
**Explanation:** -a is the annotation source; -c specifies which columns to annotate
