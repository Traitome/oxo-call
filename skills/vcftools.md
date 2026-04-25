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
**Explanation:** vcftools command; --vcf variants.vcf input VCF; --maf 0.05 keeps variants with ≥5% MAF; --max-missing 0.9 requires 90% genotype completeness; --recode outputs filtered VCF; --recode-INFO-all preserves INFO fields; --out filtered_variants output prefix

### calculate per-site nucleotide diversity and Tajima's D statistics
**Args:** `--vcf variants.vcf --site-pi --TajimaD 10000 --out popgen_stats`
**Explanation:** vcftools command; --vcf variants.vcf input VCF; --site-pi per-site nucleotide diversity; --TajimaD 10000 Tajima's D in 10kb windows; --out popgen_stats output prefix

### filter VCF to biallelic SNPs with minimum depth
**Args:** `--vcf variants.vcf --remove-indels --min-alleles 2 --max-alleles 2 --minDP 10 --recode --recode-INFO-all --out snps_only`
**Explanation:** vcftools command; --vcf variants.vcf input VCF; --remove-indels keeps only SNPs; --min-alleles 2 --max-alleles 2 biallelic only; --minDP 10 minimum depth filter; --recode --recode-INFO-all output filtered VCF; --out snps_only output prefix

### compute pairwise FST between two populations
**Args:** `--vcf variants.vcf --weir-fst-pop pop1_samples.txt --weir-fst-pop pop2_samples.txt --fst-window-size 50000 --out fst_results`
**Explanation:** vcftools command; --vcf variants.vcf input VCF; --weir-fst-pop pop1_samples.txt population 1 sample list; --weir-fst-pop pop2_samples.txt population 2 sample list; --fst-window-size 50000 windowed FST; --out fst_results output prefix

### convert VCF to PLINK format for downstream analysis
**Args:** `--vcf variants.vcf --plink --out plink_dataset`
**Explanation:** vcftools command; --vcf variants.vcf input VCF; --plink creates .ped and .map files; --out plink_dataset output prefix; use --plink-tped for transposed format

### filter variants by genomic region using BED file
**Args:** `--gzvcf variants.vcf.gz --bed exome_regions.bed --recode --recode-INFO-all --out exome_variants`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --bed exome_regions.bed keeps variants within BED regions; --recode --recode-INFO-all outputs filtered VCF; --out exome_variants output prefix; useful for exome analysis

### calculate sample heterozygosity for quality control
**Args:** `--gzvcf variants.vcf.gz --het --out heterozygosity`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --het computes observed heterozygosity per sample; --out heterozygosity output prefix; outliers may indicate contamination

### calculate IBD relatedness between samples
**Args:** `--gzvcf variants.vcf.gz --relatedness2 --out relatedness`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --relatedness2 calculates identity-by-descent sharing; --out relatedness output prefix; values >0.125 indicate relatives

### extract specific SNPs by position list
**Args:** `--gzvcf variants.vcf.gz --positions snp_list.txt --recode --recode-INFO-all --out subset_snps`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --positions snp_list.txt file with chr\tpos format; --recode --recode-INFO-all outputs filtered VCF; --out subset_snps output prefix; extracts specific variants

### filter to specific chromosome range
**Args:** `--gzvcf variants.vcf.gz --chr 1 --from-bp 1000000 --to-bp 2000000 --recode --recode-INFO-all --out chr1_region`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --chr 1 selects chromosome; --from-bp 1000000 --to-bp 2000000 define region; --recode --recode-INFO-all outputs filtered VCF; --out chr1_region output prefix; useful for fine-mapping

### remove indels and keep only SNPs
**Args:** `--gzvcf variants.vcf.gz --remove-indels --recode --recode-INFO-all --out snps_only`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --remove-indels filters out insertions/deletions; --recode --recode-INFO-all outputs filtered VCF; --out snps_only output prefix; keeps only SNVs

### calculate allele frequency by population
**Args:** `--gzvcf variants.vcf.gz --keep pop1_samples.txt --freq --out pop1_freq`
**Explanation:** vcftools command; --gzvcf variants.vcf.gz gzipped input VCF; --keep pop1_samples.txt restricts to samples in file; --freq outputs allele frequencies; --out pop1_freq output prefix; repeat for each population
