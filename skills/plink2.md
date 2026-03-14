---
name: plink2
category: population-genomics
description: Whole-genome association analysis toolset for GWAS, population structure, and linkage analysis
tags: [gwas, population-genetics, association-study, snp, genotype, linkage, pca]
author: oxo-call built-in
source_url: "https://www.cog-genomics.org/plink/2.0/"
---

## Concepts

- PLINK2 processes genetic variant data in PLINK binary (.bed/.bim/.fam or PLINK2 .pgen/.pvar/.psam) or VCF format.
- PLINK binary format: .bed (genotype), .bim (variant info), .fam (sample info) — core trio for most analyses.
- Import VCF to PLINK2 format: plink2 --vcf variants.vcf --make-pgen --out dataset.
- Use --bfile for PLINK1 binary, --pfile for PLINK2 binary, --vcf for VCF input.
- Use --maf, --geno (missingness per SNP), --mind (missingness per individual), --hwe for quality filtering.
- LD pruning: --indep-pairwise 50 10 0.1 creates prune.in/prune.out files; --extract prune.in applies pruning.
- PCA: use --pca 10 to compute top 10 principal components; output .eigenvec and .eigenval.
- Association test: --glm for linear/logistic regression; --1 for case/control phenotype.

## Pitfalls

- PLINK1 (.bed/.bim/.fam) and PLINK2 (.pgen/.pvar/.psam) have different command flags (--bfile vs --pfile).
- Multiallelic variants in VCF must be split before PLINK conversion: bcftools norm -m -any variants.vcf first.
- --maf filter removes low-frequency variants — do NOT apply for rare variant analysis.
- Sex mismatch in .fam file causes errors in X-chromosome QC; fix sex with --update-sex.
- For GRM/relatedness analysis, LD pruning must be applied BEFORE computing GRM.
- PLINK2 uses sample IDs from VCF (IID column); ensure these match downstream phenotype files.

## Examples

### convert VCF to PLINK2 binary format
**Args:** `--vcf variants.vcf --make-pgen --out plink_dataset --set-missing-var-ids @:#[b37]\$r,\$a --max-alleles 2`
**Explanation:** --make-pgen creates PLINK2 .pgen/.pvar/.psam; --set-missing-var-ids sets variant IDs; --max-alleles 2 for biallelic

### perform quality control filtering on PLINK dataset
**Args:** `--pfile plink_dataset --maf 0.01 --geno 0.05 --mind 0.1 --hwe 1e-6 --make-pgen --out qc_filtered`
**Explanation:** --maf 0.01 minor allele frequency; --geno 0.05 SNP missingness; --mind 0.1 sample missingness; --hwe Hardy-Weinberg

### perform LD pruning and compute PCA
**Args:** `--pfile plink_dataset --indep-pairwise 50 10 0.1 --out ld_prune && plink2 --pfile plink_dataset --extract ld_prune.prune.in --pca 20 --out pca_results`
**Explanation:** two steps: first LD prune; then PCA with pruned variants; --pca 20 top 20 PCs

### run genome-wide association study (GWAS) for binary phenotype
**Args:** `--pfile plink_dataset --pheno phenotypes.txt --pheno-name case_control --covar covariates.txt --glm hide-covar --out gwas_results`
**Explanation:** --glm for logistic regression; --pheno phenotype file; --covar covariates (PCs, age, sex); --hide-covar suppresses covariate output

### compute kinship/relatedness matrix
**Args:** `--pfile plink_dataset --extract ld_prune.prune.in --make-king-table --out kinship_matrix`
**Explanation:** --make-king-table computes KING relatedness coefficients; use LD-pruned variants
