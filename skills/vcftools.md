---
name: vcftools
category: variant-calling
description: Tool for working with VCF and BCF files — filtering, statistics, format conversion, and population genetics
tags: [vcf, filtering, population-genetics, statistics, format-conversion, snp, ngs]
author: oxo-call built-in
source_url: "https://vcftools.github.io/"
---

## Concepts

- VCFtools processes VCF/BCF files; use --vcf for VCF input or --gzvcf for gzipped VCF; --bcf for BCF.
- VCFtools always requires an output prefix (--out) — output files are named <prefix>.ext based on operation.
- Key filtering options: --maf (minor allele frequency), --max-missing (genotype missingness), --minQ (QUAL filter), --min-meanDP.
- Use --recode --recode-INFO-all to output a filtered VCF; default output is statistics only.
- Hardy-Weinberg filtering: --hwe 0.001 removes sites deviating from HWE with p-value < 0.001.
- Use --keep/--remove with sample lists to include/exclude specific samples.
- VCFtools can compute FST between populations: --weir-fst-pop pop1.txt --weir-fst-pop pop2.txt.
- --bed and --exclude-bed filter variants by genomic regions; useful for targeting exome or excluding blacklist regions.
- --positions and --exclude-positions filter by specific SNP positions (one per line: chr\tpos).
- --chr filters to specific chromosomes; --from-bp/--to-bp define position ranges.
- --relatedness and --relatedness2 calculate IBD sharing for sample relatedness inference.
- --het computes heterozygosity per individual; useful for quality control.

## Pitfalls

- vcftools has NO subcommands. ARGS starts directly with flags (e.g., --vcf, --gzvcf, --maf, --recode, --out). Do NOT put a subcommand like 'filter' or 'stats' before flags.
- Without --recode, VCFtools outputs statistics but does NOT write a filtered VCF file.
- --recode --recode-INFO-all is needed to include INFO fields in the output VCF.
- --out specifies a prefix, not a filename — VCFtools appends the appropriate extension.
- VCFtools removes multi-allelic sites without warning when using --min-alleles 2 --max-alleles 2.
- VCFtools is slower than bcftools for large files — use bcftools view for simple filtering when possible.
- The --max-missing parameter is the proportion of genotypes that CAN be missing (0=no missing allowed, 1=all missing allowed).
- --gzvcf is required for gzipped VCF files; --vcf only works with uncompressed VCF.
- --bed expects 0-based BED format; ensure coordinate system matches your VCF (1-based).
- --positions file format is chr\tpos (tab-separated), not chr:pos format.

## Examples

### filter VCF by minor allele frequency and missingness
**Args:** `--vcf variants.vcf --maf 0.05 --max-missing 0.9 --recode --recode-INFO-all --out filtered_variants`
**Explanation:** --maf 0.05 keeps variants with ≥5% minor allele frequency; --max-missing 0.9 requires 90% genotype completeness

### calculate per-site nucleotide diversity and Tajima's D statistics
**Args:** `--vcf variants.vcf --site-pi --TajimaD 10000 --out popgen_stats`
**Explanation:** --site-pi per-site nucleotide diversity; --TajimaD 10000 calculates Tajima's D in 10kb windows

### filter VCF to biallelic SNPs with minimum depth
**Args:** `--vcf variants.vcf --remove-indels --min-alleles 2 --max-alleles 2 --minDP 10 --recode --recode-INFO-all --out snps_only`
**Explanation:** --remove-indels keeps only SNPs; --min/max-alleles 2 keeps biallelic only; --minDP 10 minimum depth filter

### compute pairwise FST between two populations
**Args:** `--vcf variants.vcf --weir-fst-pop pop1_samples.txt --weir-fst-pop pop2_samples.txt --fst-window-size 50000 --out fst_results`
**Explanation:** population files list one sample ID per line; --fst-window-size for windowed FST calculation

### convert VCF to PLINK format for downstream analysis
**Args:** `--vcf variants.vcf --plink --out plink_dataset`
**Explanation:** --plink creates .ped and .map files; use --plink-tped for transposed PLINK format

### filter variants by genomic region using BED file
**Args:** `--gzvcf variants.vcf.gz --bed exome_regions.bed --recode --recode-INFO-all --out exome_variants`
**Explanation:** --bed keeps only variants within BED regions; useful for exome analysis or targeted panels

### calculate sample heterozygosity for quality control
**Args:** `--gzvcf variants.vcf.gz --het --out heterozygosity`
**Explanation:** --het computes observed heterozygosity per sample; outliers may indicate contamination or relatedness issues

### calculate IBD relatedness between samples
**Args:** `--gzvcf variants.vcf.gz --relatedness2 --out relatedness`
**Explanation:** --relatedness2 calculates identity-by-descent sharing; values >0.125 indicate 3rd degree or closer relatives

### extract specific SNPs by position list
**Args:** `--gzvcf variants.vcf.gz --positions snp_list.txt --recode --recode-INFO-all --out subset_snps`
**Explanation:** --positions file has format chr\tpos per line; extracts specific variants of interest

### filter to specific chromosome range
**Args:** `--gzvcf variants.vcf.gz --chr 1 --from-bp 1000000 --to-bp 2000000 --recode --recode-INFO-all --out chr1_region`
**Explanation:** --chr selects chromosome; --from-bp/--to-bp define the region; useful for fine-mapping loci

### remove indels and keep only SNPs
**Args:** `--gzvcf variants.vcf.gz --remove-indels --recode --recode-INFO-all --out snps_only`
**Explanation:** --remove-indels filters out insertions/deletions; keeps only single nucleotide variants

### calculate allele frequency by population
**Args:** `--gzvcf variants.vcf.gz --keep pop1_samples.txt --freq --out pop1_freq`
**Explanation:** --keep restricts to samples in file; --freq outputs allele frequencies; repeat for each population
