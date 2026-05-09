---
name: akgwas
category: Genomics
description: A command-line tool for performing genome-wide association studies (GWAS) with support for mixed linear models, covariate adjustment, and downstream functional analysis. Designed for population-scale genetic data.
tags: [gwss, genetics, snp, association, mixed-linear-model, bioinformatics]
author: AI-generated
source_url: https://github.com/example/akgwas
---

## Concepts

- **GWAS Data Model**: akgwas accepts PLINK PED/BED format files or VCF files containing genotype calls, along with a phenotype file in TSV/CSV format with sample IDs matching the genotype file. The tool expects additive genetic encoding (0/1/2) and handles missing data by imputation or exclusion.
- **Mixed Linear Models**: By default, akgwas uses a linear mixed model (LMM) to account for population structure and relatedness via a kinship matrix. The `--method` flag controls the statistical model: `linear` for simple linear regression, `logistic` for binary traits, or `mixed` for LMM with GRM.
- **Output Formats**: Results are written to stdout or a specified output file with columns CHROM, POS, REF, ALT, BETA, SE, PVALUE, and optionally ADJUSTED_PVALUE after multiple testing correction. The `--out-format` flag supports TSV, CSV, or BGEN-indexed binary output.

## Pitfalls

- **Mismatch between phenotype and genotype sample IDs**: If sample IDs in the phenotype file do not exactly match those in the genotype file (including case sensitivity), akgwas will silently drop unmatched samples, reducing statistical power and potentially biasing results.
- **Failing to specify covariates when needed**: Omitting important covariates (like population structure principal components, age, or sex) can lead to confounding false positives. The `--covar` flag is required when covariates are present in the phenotype file.
- **Using logistic method on continuous traits**: The `--method logistic` flag applies a binary outcome model and will produce invalid results or errors for continuous phenotypes. Conversely, `--method linear` on binary traits violates model assumptions and inflates type I error.

## Examples

### Run a basic GWAS with linear regression on a continuous trait
**Args:** --bfile mydata --pheno mydata.phen --trait height --out gwas_results.tsv
**Explanation:** Performs simple linear regression of each SNP on height using PLINK-formatted genotype files and outputs association statistics.

### Run GWAS with mixed linear model to account for population structure
**Args:** --bfile mydata --pheno mydata.phen --trait disease_status --method mixed --grm kinship_matrix.grm --out mixed_results.tsv
**Explanation:** Uses a linear mixed model with a pre-computed genetic relatedness matrix to correct for cryptic relatedness and stratification.

### Adjust for multiple testing using Bonferroni correction
**Args:** --bfile mydata --pheno mydata.phen --trait y --method linear --correction bonferroni --sig-thresh 0.05 --out corrected_results.tsv
**Explanation:** Applies Bonferroni correction to P-values and reports only variants meeting the significance threshold.

### Include covariates in the association model
**Args:** --bfile cohort --pheno cohort.phen --trait BMI --covar age,sex,pc1,pc2 --method linear --out covar_adjusted.tsv
**Explanation:** Includes age, sex, and the first two principal components as covariates to control for confounding.

### Export results in CSV format for downstream scripting
**Args:** --bfile mydata --pheno mydata.phen --trait response --method logistic --out-format csv --out logistic_results.csv
**Explanation:** Runs logistic regression for binary outcomes and outputs results in CSV format for easier parsing by scripts.

### Limit analysis to a specific genomic region
**Args:** --bfile mydata --pheno mydata.phen --trait y --chr 17 --from-bp 40000000 --to-bp 45000000 --out region_results.tsv
**Explanation:** Restricts association testing to chromosome 17 between positions 40MB and 45MB, reducing computational load for region-specific studies.