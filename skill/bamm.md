---
name: bamm
category: Phylogenetics & Macroevolution
description: BAMM (Bayesian Analysis of Macroevolutionary Mixtures) models complex patterns of speciation, extinction, and trait evolution on phylogenetic trees using reversible jump MCMC. Analyzes diversification rate shifts and phenotypic optima across tree branches.
tags:
  - macroevolution
  - diversification
  - trait-evolution
  - phylogenetics
  - bayesian
  - rjMCMC
  - rate-shifts
author: AI-generated
source_url: https://github.com/macroevolution/bamm
---

## Concepts

- BAMM uses reversible jump Markov Chain Monte Carlo (rjMCMC) to explore a variable-dimensional parameter space, meaning the number and location of evolutionary rate shift events are inferred simultaneously rather than fixed a priori. The posterior probability of shift configurations is computed across all possible models.
- Input requires a phylogenetic tree in Newick format (no bootstrap values or internal node labels) and an optional trait data file. The trait file must have one named value per line matching the tree tip labels, and missing data must be encoded as `NA` or left blank. Trees with non-ultrametric branch lengths require the `--pcov` flag to adjust for clock-like properties.
- BAMM generates multiple output files: `eventData.txt` (shift locations and posterior weights), `rates.txt` (per-branch speciation/extinction rates), and `parameterFile.txt` (MCMC traces). The `bamm-results` object in R is used to load and analyze these outputs with the BAMMtools package, which also handles visualization of shift configurations on the tree.
- The sampling intensity is controlled by the number of MCMC generations (`--gen`), burn-in proportion (`--burnin`), and the proposal frequency of shift events. Convergence must be assessed by comparing multiple independent runs using the `BAMMtools::plotEffectiveSize` and Geweke diagnostic functions; inadequate mixing produces biased rate estimates.
- Diversification models supported include time-variable speciation and extinction rates, trait-dependent diversification (BBMV), and mass extinction events. Model selection is performed via Bayes factor comparison of the marginal likelihoods extracted from the rjMCMC chain, not by simple AIC on point estimates.

## Pitfalls

- Running BAMM without checking MCMC convergence is a frequent error. If independent runs produce conflicting shift locations or rate estimates, the results are unreliable for publication. Always run at least two independent chains from different seeds and compare them before interpreting the output.
- Providing a trait data file with mismatched tip labels causes silent failures or crashes. Every tip label in the tree must have a corresponding entry in the trait file. Trailing whitespace or special characters in labels frequently cause mismatches that are hard to diagnose without verbose logging enabled.
- Using an excessively short run (`--gen`) leads to unstable rate estimates, especially for large trees with many possible shift configurations. Small trees (fewer than 50 tips) may need at least 5 million generations, while large trees (over 500 tips) can require 50 million or more to achieve adequateESS values.
- Specifying the wrong branch length type (ultrametric vs. non-ultrametric) misleads the model. For non-ultrametric trees, omitting `--pcov` causes the speciation/extinction rate priors to be anchored incorrectly, distorting all inferred rates and shift posterior probabilities.
- Attempting to analyze trait evolution with BAMM without first verifying that the trait file has no systematic missing data biases produces misleading conclusions. Traits missing from entire clades skew the diversification model because the likelihood calculation ignores those branches entirely.

## Examples

### Run BAMM on an ultrametric tree with default settings for 10 million generations
**Args:** `--gen 10000000 --treeFile tree.nex --mcmcWrite 1000`
**Explanation:** This runs BAMM for 10 million MCMC generations on an ultrametric phylogeny in NEXUS format, writing the MCMC state to disk every 1000 generations for checkpointing and posterior diagnostics.

### Analyze trait-dependent diversification by providing a trait data file
**Args:** `--treeFile tree.nex --traitFile traits.txt --gen 20000000 --seed 42`
**Explanation:** The `--traitFile` argument enables the trait-dependent diversification model, where tip values influence rates of speciation and extinction along each branch, and the random seed ensures reproducibility across runs.

### Set burn-in to discard the first 25% of MCMC generations before computing the posterior
**Args:** `--treeFile tree.nex --gen 5000000 --burnin 0.25 --mcmcWrite 500`
**Explanation:** Specifying `--burnin 0.25` discards the initial 1.25 million generations, allowing the chain to reach stationarity before the remaining samples are used for estimating shift posterior probabilities.

### Run BAMM on a non-ultrametric tree with a clock variance parameter
**Args:** `--treeFile tree.nex --gen 10000000 --pcov 0.5 --poissonRatePrior 100`
**Explanation:** The `--pcov` flag activates the clock variance parameter for non-ultrametric branch lengths, and the Poisson rate prior controls the expected number of shift events, preventing overly diffuse rate estimates.

### Generate event data output for downstream analysis in R with BAMMtools
**Args:** `--treeFile tree.nex --gen 10000000 --seed 12345 --eventData eventData.txt`
**Explanation:** The `--eventData` flag writes the shift configuration posterior to a file that BAMMtools can load with `bammdata = readBammData()` in R, enabling visualization of the most probable diversification rate shift locations on the tree.

### Perform a model comparison run with a higher shift prior to test for fewer rate changes
**Args:** `--treeFile tree.nex --gen 15000000 --poissonRatePrior 50 --seed 999`
**Explanation:** Lowering the Poisson rate prior from the default (typically 1000) to 50 imposes stronger regularization toward fewer shift events, which is useful for testing whether a reduced shift model is sufficient given the data.

### Run BAMM in quiet mode by suppressing console output for long batch jobs
**Args:** `--treeFile tree.nex --gen 50000000 --mcmcWrite 5000 --silent`
**Explanation:** The `--silent` flag suppresses per-generation console output, reducing I/O overhead and improving performance during long batch submissions on compute clusters without sacrificing the output files.

### Analyze a large tree with increased hyperprior values to improve mixing
**Args:** `--treeFile large_tree.nex --gen 50000000 --lambdaInit 0.01 --muInit 0.01 --seed 777`
**Explanation:** For large trees, initializing the speciation (`--lambdaInit`) and extinction (`--muInit`) rates to lower values helps the rjMCMC sampler mix more effectively across rate configurations, reducing the risk of the chain getting stuck in localized optima.
---