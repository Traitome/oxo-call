---
name: freebayes
category: variant-calling
description: Bayesian haplotype-based genetic variant detector for SNPs, indels, MNPs, and complex variants
tags: [variant-calling, snp, indel, bayesian, haplotype, vcf, ngs, germline]
author: oxo-call built-in
source_url: "https://github.com/freebayes/freebayes"
---

## Concepts

- FreeBayes performs haplotype-based variant calling; it considers reads in windows around candidate variants jointly.
- Use -f for reference FASTA; input BAM files are positional arguments (supports multiple samples).
- FreeBayes outputs VCF to stdout — always redirect to a .vcf file or pipe to bgzip.
- Use -p 2 for diploid (default); -p 1 for haploid; other ploidy values for polyploids.
- Use --min-alternate-count and --min-alternate-fraction to filter low-confidence variants.
- For population calling with multiple samples, list all BAMs on the command line — FreeBayes calls variants jointly.
- Use -r to restrict calling to a region: -r chr1:1000-2000; process chromosomes in parallel for speed.
- Parallelize FreeBayes with freebayes-parallel (uses GNU parallel) for large genomes.

## Pitfalls

- FreeBayes requires indexed BAM files (.bai) — run samtools index before FreeBayes.
- FreeBayes is slow on large genomes without parallelization — use freebayes-parallel or split by chromosome.
- The reference FASTA (-f) must be indexed with samtools faidx.
- Without --min-alternate-count filtering, FreeBayes may produce many low-quality variant calls.
- FreeBayes outputs multiallelic records that some downstream tools cannot handle — normalize with bcftools norm.
- For tumor-normal somatic calling, GATK Mutect2 or Strelka2 are better suited; FreeBayes is primarily germline.

## Examples

### call germline variants from a single sample BAM file
**Args:** `-f reference.fa -b sample.bam > variants.vcf`
**Explanation:** -f reference FASTA; -b input BAM; output VCF to stdout redirected to file

### call variants with minimum coverage and allele frequency filters
**Args:** `-f reference.fa --min-alternate-count 3 --min-alternate-fraction 0.2 -b sample.bam > filtered_variants.vcf`
**Explanation:** --min-alternate-count 3 requires ≥3 reads supporting alternate; --min-alternate-fraction 0.2 requires ≥20% frequency

### call variants jointly from multiple samples
**Args:** `-f reference.fa sample1.bam sample2.bam sample3.bam > cohort_variants.vcf`
**Explanation:** multiple BAMs as positional arguments for joint genotyping across samples

### call variants restricted to a specific genomic region
**Args:** `-f reference.fa -r chr1 -b sample.bam > chr1_variants.vcf`
**Explanation:** -r restricts calling to chr1; can specify chr1:1000-5000 for subregion

### call variants with population priors from a VCF
**Args:** `-f reference.fa --variant-input known_variants.vcf --only-use-input-alleles -b sample.bam > genotyped.vcf`
**Explanation:** --variant-input provides known sites; --only-use-input-alleles forces genotyping at known sites only
