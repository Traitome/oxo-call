---
name: admixture
category: population-genomics
description: Fast population structure analysis using maximum likelihood estimation of ancestry proportions
tags: [population-genetics, ancestry, structure, gwas, admixture, plink, k-value]
author: oxo-call built-in
source_url: "https://dalexander.github.io/admixture/"
---

## Concepts

- ADMIXTURE estimates ancestry proportions (Q-matrix) and allele frequencies (P-matrix) for a specified K (number of populations).
- ADMIXTURE uses PLINK binary format (.bed/.bim/.fam) as input; data should be LD-pruned before running.
- Specify K (number of ancestral populations) as a positional argument: admixture data.bed K.
- Run multiple K values and use cross-validation error (--cv) to select optimal K.
- Use --seed for reproducibility; run multiple replicates to check convergence.
- Output: <input>.K.Q (ancestry proportions per individual) and <input>.K.P (allele frequencies per population).
- Use --supervised for supervised mode with labeled reference populations.
- ADMIXTURE is equivalent to STRUCTURE but orders of magnitude faster.

## Pitfalls

- ADMIXTURE requires LD-pruned data — high LD inflates estimated K and distorts ancestry proportions.
- Run multiple replicates per K (with different seeds) — different runs may give different local optima.
- The Q-matrix columns are not labeled with population names — interpretation requires external knowledge.
- Rare variants (MAF < 0.01) should be filtered before running ADMIXTURE.
- Without --cv, cross-validation error is not computed — always use --cv=10 for model selection.
- ADMIXTURE does not handle related individuals well — remove close relatives before analysis.

## Examples

### run ADMIXTURE for K=5 ancestral populations
**Args:** `data.bed 5 --cv=10 -j8`
**Explanation:** positional K=5; --cv=10 cross-validation with 10 folds; -j8 threads; outputs data.5.Q and data.5.P

### run ADMIXTURE with reproducible seed
**Args:** `data.bed 3 --seed=42 --cv=10 -j8`
**Explanation:** --seed for reproducibility; run same command with different seeds to check convergence

### run supervised ADMIXTURE with known reference populations
**Args:** `data.bed 3 --supervised -j8`
**Explanation:** --supervised mode uses .pop file (same name as .fam) with population labels for reference individuals

### run ADMIXTURE across multiple K values (shell loop)
**Args:** `data.bed K --cv=10 -j8 > admixture_K.log`
**Explanation:** run for K=2,3,4,5,etc in a loop; compare cross-validation errors to select optimal K

### run ADMIXTURE with 100 bootstrap replicates for standard errors
**Args:** `data.bed 5 -B100 -j8`
**Explanation:** -B100 performs 100 bootstrap replicates to estimate standard errors on Q and P matrices; output includes .Q_se and .P_se files

### run projection analysis onto a fixed P-matrix
**Args:** `data.bed 5 -P -j8`
**Explanation:** -P freezes allele frequency estimates (P-matrix) and only estimates Q; requires a pre-computed .5.P file from a reference run

### run multiple replicates for K=4 with different seeds to check convergence
**Args:** `data.bed 4 --seed=1 --cv=10 -j8 > run1.log`
**Explanation:** repeat with --seed=2, --seed=3, etc.; compare log-likelihood values across replicates to identify the highest-likelihood solution

### compare cross-validation errors across K values
**Args:** `data.bed 6 --cv=10 -j8 | tee admixture_K6.log`
**Explanation:** tee captures log while streaming; extract CV error lines with: grep "CV error" admixture_K*.log to compare across K values

### filter for minor allele frequency before running ADMIXTURE
**Args:** `data.bed 5 --maf=0.05 --cv=10 -j8`
**Explanation:** --maf=0.05 filters SNPs with minor allele frequency below 5% at runtime; reduces noise from rare variants without pre-filtering with PLINK

### run ADMIXTURE with accelerated EM for faster convergence
**Args:** `data.bed 5 --em --cv=10 -j8`
**Explanation:** --em uses the EM algorithm (slower per iteration but more reliable convergence than the default block relaxation); useful when default runs fail to converge
