---
name: freebayes
category: variant-calling
description: Bayesian haplotype-based genetic variant detector for SNPs, indels, MNPs, and complex variants
tags: [variant-calling, snp, indel, bayesian, haplotype, vcf, ngs, germline, gvcf, pooled]
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
- --gvcf outputs gVCF format with coverage information for all positions; --gvcf-chunk controls record granularity.
- --haplotype-length controls complex variant detection; default 3bp allows SNP clusters to be called as MNPs.
- --pooled-discrete and --pooled-continuous enable pooled sequencing analysis; use with appropriate --ploidy.
- --cnv-map specifies copy number variations per sample or region for accurate genotyping in variable ploidy regions.
- --use-best-n-alleles limits alleles evaluated; reduces memory for high ploidy or complex regions.
- --min-repeat-entropy detects interrupted repeats; increases sensitivity in repetitive regions.

## Pitfalls

- freebayes has NO subcommands. ARGS starts directly with flags (e.g., -f, -b, -r, -p) or input BAM files. Do NOT put a subcommand like 'call' or 'detect' before flags.
- FreeBayes requires indexed BAM files (.bai) — run samtools index before FreeBayes.
- FreeBayes is slow on large genomes without parallelization — use freebayes-parallel or split by chromosome.
- The reference FASTA (-f) must be indexed with samtools faidx.
- Without --min-alternate-count filtering, FreeBayes may produce many low-quality variant calls.
- FreeBayes outputs multiallelic records that some downstream tools cannot handle — normalize with bcftools norm.
- For tumor-normal somatic calling, GATK Mutect2 or Strelka2 are better suited; FreeBayes is primarily germline.
- --gvcf output is much larger than standard VCF; use --gvcf-chunk to control granularity.
- --pooled-discrete requires --ploidy set to total alleles in pool; incorrect ploidy causes wrong genotypes.
- High ploidy with many alleles can exhaust memory; use --use-best-n-alleles to limit evaluation.
- --haplotype-length default (3) may miss larger complex variants; increase for longer haplotype detection.

## Examples

### call germline variants from a single sample BAM file
**Args:** `-f reference.fa -b sample.bam > variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; -b sample.bam input BAM; output VCF to stdout redirected to variants.vcf file

### call variants with minimum coverage and allele frequency filters
**Args:** `-f reference.fa --min-alternate-count 3 --min-alternate-fraction 0.2 -b sample.bam > filtered_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --min-alternate-count 3 requires ≥3 reads supporting alternate; --min-alternate-fraction 0.2 requires ≥20% frequency; -b sample.bam input BAM; output to filtered_variants.vcf

### call variants jointly from multiple samples
**Args:** `-f reference.fa sample1.bam sample2.bam sample3.bam > cohort_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; sample1.bam sample2.bam sample3.bam multiple BAMs as positional arguments; output to cohort_variants.vcf; for joint genotyping across samples

### call variants restricted to a specific genomic region
**Args:** `-f reference.fa -r chr1 -b sample.bam > chr1_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; -r chr1 restricts calling to chr1; -b sample.bam input BAM; output to chr1_variants.vcf; can specify chr1:1000-5000 for subregion

### call variants with population priors from a VCF
**Args:** `-f reference.fa --variant-input known_variants.vcf --only-use-input-alleles -b sample.bam > genotyped.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --variant-input known_variants.vcf provides known sites; --only-use-input-alleles forces genotyping at known sites only; -b sample.bam input BAM; output to genotyped.vcf

### generate gVCF output for joint genotyping
**Args:** `-f reference.fa --gvcf --gvcf-chunk 10000 -b sample.bam > sample.g.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --gvcf outputs gVCF format; --gvcf-chunk 10000 emits record every 10kb; -b sample.bam input BAM; output to sample.g.vcf; suitable for GATK-style joint calling

### call variants from pooled sequencing data
**Args:** `-f reference.fa -p 20 --pooled-discrete --use-best-n-alleles 4 pool.bam > pooled_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; -p 20 sets ploidy to 20 alleles; --pooled-discrete models pooled samples; --use-best-n-alleles 4 limits memory; pool.bam input BAM; output to pooled_variants.vcf

### call variants with copy number variation map
**Args:** `-f reference.fa --cnv-map cnv.bed -p 2 sample.bam > cnv_aware_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --cnv-map cnv.bed provides per-region copy numbers; -p 2 diploid ploidy; sample.bam input BAM; output to cnv_aware_variants.vcf; essential for accurate genotyping in CNV regions like cancer samples

### detect complex variants with extended haplotype length
**Args:** `-f reference.fa --haplotype-length 10 --min-repeat-entropy 2 sample.bam > complex_variants.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --haplotype-length 10 allows 10bp contiguous matches; --min-repeat-entropy 2 improves repeat region sensitivity; sample.bam input BAM; output to complex_variants.vcf

### report monomorphic sites for complete genome coverage
**Args:** `-f reference.fa --report-monomorphic -r chr1 sample.bam > all_sites.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --report-monomorphic outputs all positions including non-variant; -r chr1 restricts to chr1; sample.bam input BAM; output to all_sites.vcf; useful for generating complete reference panels

### genotype only specific alleles from input VCF
**Args:** `-f reference.fa --haplotype-basis-alleles targets.vcf --report-all-haplotype-alleles sample.bam > targeted_genotypes.vcf`
**Explanation:** freebayes command; -f reference.fa reference FASTA; --haplotype-basis-alleles targets.vcf restricts to input alleles; --report-all-haplotype-alleles shows all alleles at haplotype sites; sample.bam input BAM; output to targeted_genotypes.vcf
