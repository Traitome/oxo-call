---
name: shapeit4
category: population-genomics
description: Statistical method for haplotype phasing of whole-genome sequencing data
tags: [phasing, haplotype, population-genetics, gwas, imputation, vcf]
author: oxo-call built-in
source_url: "https://odelaneau.github.io/shapeit4/"
---

## Concepts
- SHAPEIT4 phases genotype data (unphased VCF) into haplotypes using statistical methods.
- Input: bgzipped and tabix-indexed VCF; optionally a genetic map and a scaffold panel.
- Use -I for input VCF; -O for output phased VCF; -R for region; -T for threads.
- A genetic map (-M) improves phasing accuracy — download from SHAPEIT4 documentation.
- Use --scaffold for a reference haplotype panel (improves accuracy in cosmopolitan populations).
- Chromosome-by-chromosome phasing is standard — run separately for each chromosome.
- SHAPEIT4 output is a phased VCF with phased genotypes (| separator).
- Use --sequencing for sequencing data; --genotyping for array genotype data.
- --pbwt-modulo controls PBWT index storage frequency (default 0.02 cM); lower values increase memory.
- --pbwt-depth sets number of conditioning haplotypes (default 4); increase for better accuracy.
- --effective-size sets population effective size (default 15000); adjust for your population.
- --window sets phasing window size in cM (default 2.5); increase for more conditioning haplotypes.
- --use-PS incorporates phase information from read-based phasing (PS field in VCF).

## Pitfalls
- Input VCF must be bgzipped and tabix-indexed — run bgzip + tabix before SHAPEIT4.
- SHAPEIT4 phases one chromosome at a time — use -R to specify chromosome region.
- Without a genetic map, phasing accuracy is reduced, especially for distant markers.
- SHAPEIT4 requires high-quality variant calls — filter low-confidence variants before phasing.
- Phasing is most accurate with large sample sizes (>100 samples).
- --pbwt-depth 4 (default) is usually sufficient; higher values increase runtime significantly.
- --pbwt-modulo 0.02 (default) balances memory and speed; decrease for higher resolution.
- --effective-size 15000 is for human populations; adjust for other species.
- --window 2.5 cM is default; increase for more conditioning but slower runtime.
- --use-PS requires PS field in VCF from prior read-based phasing (e.g., WhatsHap).

## Examples

### phase a chromosome using SHAPEIT4
**Args:** `--input variants.vcf.gz --map genetic_map_chr1.txt --region chr1 --output phased_chr1.vcf.gz --thread 8`
**Explanation:** --input unphased VCF; --map genetic map for chr1; --region restricts to chr1; --output phased VCF

### phase with a reference haplotype scaffold panel
**Args:** `--input variants.vcf.gz --scaffold reference_panel.vcf.gz --map genetic_map_chr22.txt --region chr22 --output phased_chr22.vcf.gz --thread 8`
**Explanation:** --scaffold reference panel improves phasing; often used with 1000 Genomes or HGSVC panel

### phase sequencing data with higher accuracy settings
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --sequencing --output phased.vcf.gz --thread 16 --mcmc-iterations 8b,1p,1b,1p,1b,1p,5m`
**Explanation:** --sequencing mode for WGS data; --mcmc-iterations for convergence tuning

### phase with increased PBWT depth for better accuracy
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --thread 8 --pbwt-depth 8`
**Explanation:** --pbwt-depth 8 increases conditioning haplotypes; better accuracy but slower

### phase with read-based phasing information
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --thread 8 --use-PS`
**Explanation:** --use-PS incorporates PS field from read-based phasing (e.g., WhatsHap output)

### phase with custom effective population size
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --thread 8 --effective-size 10000`
**Explanation:** --effective-size 10000 for smaller population; adjust based on your study population

### phase with larger window size
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --thread 8 --window 5`
**Explanation:** --window 5 increases phasing window to 5 cM; more conditioning haplotypes

### phase with reference panel instead of scaffold
**Args:** `--input variants.vcf.gz --reference reference_panel.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --thread 8`
**Explanation:** --reference uses haplotype reference panel; different from --scaffold usage

### phase a specific genomic region
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1:1000000-2000000 --output phased.vcf.gz --thread 8`
**Explanation:** --region chr1:1000000-2000000 phases only specified interval; useful for targeted analysis

### output binary format for multiple haplotype sampling
**Args:** `--input variants.vcf.gz --map genetic_map.txt --region chr1 --output phased.vcf.gz --bingraph phased.bin --thread 8`
**Explanation:** --bingraph outputs binary format; useful for sampling multiple likely haplotype configurations
