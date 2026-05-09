---
name: admixture
category: Population Genetics / Ancestry Estimation
description: ADMIXTURE is a fast, maximum-likelihood tool for estimating individual ancestry proportions and population allele frequencies from SNP genotype data in PLINK binary format. It is a faster command-line alternative to STRUCTURE.
tags: [admixture, ancestry, population genetics, SNP, PLINK, structure, Q matrix]
author: AI-generated
source_url: https://dalexander.github.io/admixture/download.html
---

## Concepts

- **Input format**: ADMIXTURE requires PLINK binary PED files consisting of three files with identical basename: `.bed` (genotype data), `.bim` (variant information), and `.fam` (family/individual information). Ensure all three files exist in the same directory before running.
- **Output files**: The tool produces a `.Q` file (N×K matrix of individual ancestry proportions, one row per individual) and a `.P` file (K×L matrix of population allele frequencies, one row per ancestral population, L = number of loci). These filenames are derived from the input file basename.
- **Cross-validation**: The `-C` flag enables leave-one-out cross-validation to estimate prediction error for a given K. Lower cross-validation error suggests better K choice. Use this to objectively select the number of ancestral populations.
- **K parameter**: The `-K` flag specifies the number of ancestral populations (K) to model. This is a required flag that must be set explicitly. K must be ≥1 and ≤ number of individuals. Run multiple analyses with different K values to find optimal K via cross-validation.
- **Multithreading**: The `-j` flag enables parallel computation using multiple CPU cores, dramatically reducing runtime for large datasets. Set to the number of available cores (e.g., `-j 8` for 8 cores).

## Pitfalls

- **Missing required files**: Running on a `.bed` file without its corresponding `.bim` and `.fam` files causes immediate failure. Always verify all three PLINK files exist and share the same basename prefix.
- **Incorrect K selection**: Setting K too high or too low without cross-validation validation produces biologically meaningless ancestry estimates. Always use `-C` to evaluate multiple K values before interpreting results.
- **Unsupervised mode limitations**: Unsupervised ancestry estimation (default mode) can produce non-reproducible results, especially for closely related populations or when populations lack strong differentiation. Use supervised mode (`-s` flag with known labels) when population membership is known.
- **Convergence negligence**: The algorithm may not converge within the default iteration limit for large or complex datasets. Check the log file for "Converged" status; if absent, re-run with increased `-m` iterations.
- **Conflicting flags**: Combining `-s` (supervised) with `-C` (cross-validation) is unsupported and causes the program to exit with an error. Cross-validation only works in unsupervised mode.

## Examples

### Run basic unsupervised ancestry estimation with K=3
**Args:** `--cv-error plink.bed -K 3`
**Explanation:** This runs a standard unsupervised ADMIXTURE analysis assuming 3 ancestral populations, computing the Q matrix of individual ancestry proportions and the P matrix of allele frequencies, and prints the cross-validation error to stderr.

### Select optimal K using 5-fold cross-validation
**Args:** plink.bed -K 4 -C
**Explanation:** This runs leave-one-out cross-validation for K=4, outputting the estimated prediction error which can be compared across different K values to select the most appropriate number of ancestral populations.

### Speed up analysis with 8 CPU threads
**Args:** plink.bed -K 5 -j 8
**Explanation:** This enables parallel computation across 8 CPU threads, significantly reducing runtime for large genomic datasets while estimating 5 ancestral populations.

### Increase iteration limit for complex datasets
**Args:** plink.bed -K 4 -m 1000
**Explanation:** This raises the maximum number of major iterations to 1000 (default is typically 200), which helps ensure convergence for datasets with complex ancestry patterns or many individuals.

### Run supervised analysis with known population labels
**Args:** plink.bed -K 3 -s
**Explanation:** This runs supervised mode where the algorithm uses the pre-defined population structure to estimate allele frequency priors, producing more stable ancestry estimates when some or all individual population memberships are known.