---
name: bolt-lmm
category: genetics
description: A linear mixed model tool for genome-wide association testing that accounts for population structure, cryptically related individuals, and sample sharing. BOLT-LMM performs efficient mixed model regression to identify genetic associations while controlling for confounding.
tags:
  - genetics
  - GWAS
  - association-testing
  - linear-mixed-model
  - population-structure
  - LMM
author: AI-generated
source_url: https://github.com/ruderfer/BOLT-LMM
---

## Concepts

- **BED/BIM/FAM Format**: BOLT-LMM uses PLINK binary format (.bed, .bim, .fam) for genotype input. The .bed file contains compressed genotype data, the .bim stores variant information (chromosome, position, alleles), and the .fam holds family/phenotype data including sample IDs, sex, and disease status.
- **Phenotype File Requirements**: The phenotype file (--pheno) must be space or tab-delimited with at least two columns: Family ID (FID) and Individual ID (IID) matching the .fam file, followed by one or more phenotype columns. The header line is required.
- **Covariate and Principal Components**: Use --covar for covariates and --pcs for principal components to correct for population stratification. Including top PCs (e.g., --pcs 10) is recommended for GWAS to account for subtle population structure.
- **Mixed Model Algorithm**: BOLT-LMM uses a two-component variance component model that models the genetic relatedness matrix (GRM) to account for both close and distant relatedness, improving type I error control in the presence of cryptic relatedness.

## Pitfalls

- **Mismatch Between .fam and Phenotype File IDs**: If the FID/IID columns in the phenotype file do not exactly match those in the .fam file, BOLT-LMM will fail with an unclear error or silently skip samples. Consequences include reduced sample size and biased results.
- **Using Default Chromosome Coding**: BOLT-LMM assumes chromosome coding starting from 1-22 plus X/Y. Using non-standard chromosome names (e.g., "chr1" instead of "1" in the .bim) will cause all variants on that chromosome to be skipped without warning.
- **Insufficient RAM for Large GRM Calculations**: Computing the genetic relationship matrix for hundreds of thousands of variants requires substantial RAM. Insufficient memory causes the tool to crash or use disk swapping, dramatically increasing runtime.
- **Conflicting Sex Metadata**: Providing --phenoSex that contradicts the sex column in the .fam file leads to incorrect association tests for sex-specific analyses and may cause the model to not converge properly.

## Examples

### Run basic association test with quantitative trait

**Args:** --bfile data/eur_qc --pheno data/pheno.txt --pheno-name height --covar data/covars.txt --out results/bolt_height

**Explanation:** This runs a linear mixed model association test for the quantitative phenotype "height" using PLINK-formatted genotypes, including covariates to control for confounding variables.

### Run association test for binary case-control trait

**Args:** --bfile data/eur_qc --pheno data/disease_pheno.txt --pheno-name disease --covar data/covars.txt --covar-col-nums 3 --bolt-lmm-inf 1 --out results/bolt_disease

**Explanation:** This performs association testing for a binary disease phenotype using the BOLT-LMM infinitesimal model (--bolt-lmm-inf 1) which models the genetic architecture of the trait with a variance component.

### Include principal components for population correction

**Args:** --bfile data/eur_qc --pheno data/pheno.txt --pheno-name bmi --pcs data/pcs.txt --num-pcs 10 --out results/bolt_bmi_pcs

**Explanation:** This includes the top 10 principal components from the --pcs file to correct for population stratification, which is critical for avoiding false positives in GWAS with diverse sample collections.

### Set maximum MAC for minor allele count filtering

**Args:** --bfile data/eur_qc --pheno data/pheno.txt --pheno-name glucose --max-mac 20 --out results/bolt_glucose

**Explanation:** This filters out variants with a minor allele count less than 20, improving power by removing rare variants with unreliable effect estimates and reducing multiple testing burden.

### Run BOLT-LMM with REML variance component estimation

**Args:** --bfile data/eur_qc --pheno data/pheno.txt --pheno-name bp --reml --reml-step 2 --out results/bolt_bp_reml

**Explanation:** This uses REML (Restricted Maximum Likelihood) for variance component estimation with two REML steps, providing more accurate heritability estimation when the default method undershoots the true confounding variance.