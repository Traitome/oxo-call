---
name: baypass
category: Population Genetics / Selection Detection
description: A Bayesian approach to detect loci under selection from population genetic data by modeling allele frequency covariances across populations and computing Bayes factors for each SNP.
tags:
  - selection
  - population genetics
  - SNPs
  - bayesian
  - adaptation
  - genotype-environment association
author: AI-generated
source_url: https://www1.montpellier.inra.fr/CBGP/software/baypass/
---

## Concepts

- **Input Format Requires Integer Allele Counts**: BayPass expects a genotype table file where each row is a SNP and each column is a population, with values being integer allele counts (e.g., 20, 40). Do NOT provide allele frequencies—the tool performs its own frequency calculations internally from the count data.
- **Output Includes Multiple Files for Different Analyses**: The tool generates a `_summary.txt` file with posterior probabilities and Bayes factors for selection, a `_trace.txt` file containing MCMC chains for convergence diagnostics, and auxiliary files for covariate associations when environmental variables are included.
- **Covariate File Enables Genotype-Environment Association Testing**: When a covariate file is provided, BayPass tests for associations between allele frequencies and environmental variables (e.g., latitude, temperature), allowing detection of local adaptation beyond simple selection scans.
- **Three Distinct Statistical Models Available**: The core model (`-coefbaypass`) uses allele frequency covariances across populations; the auxiliary model (`-aux`) compares focal versus alternative populations; the standard mode (`-standard`) provides a simpler frequentist-like test without Bayesian averaging.
- **MCMC Diagnostics Are Essential for Reliability**: BayPass uses Monte Carlo Markov Chain sampling; users must examine trace files for convergence and increase the number of iterations (`-nval`) and burn-in (`-nburn`) when analyzing large genomic datasets to ensure reliable posterior estimates.

## Pitfalls

- **Providing Allele Frequencies Instead of Counts Causes Complete Failure**: BayPass interprets numeric inputs as raw counts for calculating allele frequencies internally. Providing frequencies will skew the internal calculations and produce meaningless Bayes factors.
- **Neglecting MCMC Convergence Leads to Unreliable Results**: Without checking trace files for stable convergence, posterior probability estimates may be biased, resulting in false positive or false negative selection signals.
- **Incompatible Population Labels Break the Analysis**: The individual-to-population mapping file requires exact matches between individuals in the genotype file and their assigned populations; mismatched labels cause the tool to skip or misprocess those loci.
- **Including Monomorphic SNPs Reduces Detection Power**: Loci with zero variation across all populations add no information to the covariance model but increase computational burden and may introduce numerical instabilities in the MCMC sampler.
- **Ignoring Population Structure Produces False Positives**: Without accounting for strong population structure (e.g., hierarchical isolation-by-distance), the covariance-based selection signals may simply reflect genetic drift rather than selection.

## Examples

### Basic selection scan with default parameters
**Args:** -gfile genotype_data.txt -outprefix selection_scan -nval 5000 -nburn 1000
**Explanation:** Runs a standard Bayesian selection scan using allele frequency covariances across populations with 5000 MCMC iterations and 1000 burn-in steps, outputting results to files prefixed with "selection_scan".

### Detect selection using focal versus alternative population comparison
**Args:** -gfile genotypes.txt -outprefix focal_test -aux -focal focal_pop -altern populationsA,B,C -nval 8000 -nburn 2000
**Explanation:** Uses the auxiliary model to compare allele frequency differences between a focal population and a set of alternative populations, specifically testing for selection unique to the focal population.

### Association test with environmental covariate
**Args:** -gfile snps.txt -outprefix env_assoc -covfile environmental_vars.txt -nval 10000 -nburn 3000
**Explanation:** Tests for genotype-environment associations by including environmental covariates (e.g., temperature, latitude), computing Bayes factors for each SNP's association with the provided variable gradients.

### Enable parallel computation for large datasets
**Args:** -gfile large_genotypes.txt -outprefix parallel_run -nval 20000 -nburn 5000 -thr 8
**Explanation:** Processes a large genotype matrix with 20,000 MCMC iterations and multi-threaded execution using 8 cores, significantly reducing wall-clock time for genome-wide analyses.

### Specify custom prior for allele frequency variance
**Args:** -gfile custom.txt -outprefix custom_prior -priortau 0.1 -nval 6000 -nburn 1500
**Explanation:** Overrides the default prior on the tau parameter (allele frequency variance) to 0.1, useful when prior knowledge about the strength of selection or population differentiation is available.

### Input population labels from separate mapping file
**Args:** -gfile snp_matrix.txt -poplabels pop_map.txt -outprefix mapped_run -nval 4000 -nburn 500
**Explanation:** Uses an external population mapping file to assign individuals to populations, required when the genotype file contains individual-level data rather than pre-aggregated population counts.