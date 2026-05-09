---
name: bayescan
category: population-genetics-selection
description: Bayesian method to detect locus-specific selection signals from genetic population data using F_ST coefficients and a hierarchical model that separates neutral loci from those under selection.
tags: [selection, FST, bayesian, population-genetics, locus-specific, MCMC]
author: AI-generated
source_url: https://bitbucket.org/mgambiere/bayescan
---

## Concepts

- **FSTA Input Format**: bayescan reads a tab- or space-delimited FST file where each line represents a locus and columns contain F_ST values estimated across multiple populations (e.g., from Stacks `populations` or similar software). The file must have a header row identifying population names, and missing values will cause the program to abort.
- **Two-Component Hierarchical Model**: The core model distinguishes neutral loci (sharing a common F_ST drawn from a Dirichlet distribution) from selected loci (each with an independent F_ST). This structure enables Bayesian inference to compute posterior probabilities that any given locus is under selection rather than neutral.
- **MCMC Sampling and Convergence**: The algorithm uses Markov Chain Monte Carlo to sample from the posterior distribution of F_ST values and selection indicators. Long chains with adequate thinning reduce autocorrelation; chains must be run long enough to pass the Geweke convergence diagnostic test (verified by plotting parameter traces).
- **Output Files**: The program produces multiple output files: `fst.txt` (per-locus posterior F_ST means), `omega.txt` (log10 posterior odds of selection), and `sel_chisq.txt` (chi-square test probabilities for selection). The `run_parameters.txt` file records all settings used for reproducibility.
- **Pilot Runs for Proposal Optimization**: Before the main analysis, bayescan performs a pilot run to tune proposal distributions for the MCMC sampler. The `-npop` flag controls the number of population groups modeled, which directly affects computational complexity and memory usage.

## Pitfalls

- **Convergence Failure**: Running chains that are too short produces biased posterior estimates, and the Geweke diagnostic may report non-convergence. This leads to unreliable selection calls and potentially missing true selection signals or flagging neutral loci as selected.
- **Insufficient Burn-in**: If the initial burn-in period (controlled by `-nbp`) is too short, early MCMC samples from the transient phase contaminate the posterior estimates, skewing F_ST values and selection probabilities.
- **High Autocorrelation Without Thinning**: Skipping thinning (`-thin 0`) or using too-wide intervals causes high autocorrelation between consecutive samples, inflating effective sample size and yielding overconfident (but inaccurate) posterior probabilities.
- **Unfiltered Rare Alleles**: Loci with extremely low minor allele counts produce unstable F_ST estimates that spuriously appear under selection. Pre-filtering input data by minimum allele count or sample size before running bayescan is essential.
- **Misunderstanding the Prior Odds Ratio**: The `-pr_odds` parameter (default 10) sets the prior odds against selection per locus. A higher value (e.g., 100) makes the test more conservative, requiring stronger evidence to call selection; users who ignore this may obtain an overly liberal set of candidate loci.

## Examples

### Detect selection with default parameters on a standard FST file
**Args:** `-input my_data.fst.txt -output bayescan_default`
**Explanation:** This runs a standard analysis using the built-in MCMC settings (pilot run of 5000 iterations, main chain of 50000, burn-in of 5000, thinning of 10) and prior odds of 10, outputting results to files prefixed with `bayescan_default`.

### Run a longer chain for higher statistical power on a large genomic dataset
**Args:** `-input snps.fst -output deep_sampling -n 100000 -b 10000 -p 50 -thin 10`
**Explanation:** Increasing the main chain length to 100000 iterations, burn-in to 10000, and pilot run to 50 provides more samples from the posterior, yielding tighter credible intervals for F_ST estimates on datasets with many loci.

### Use a conservative prior to reduce false positives in a hypothesis-free screen
**Args:** `-input candidate_sweeps.fst -output conservative_run -pr_odds 100`
**Explanation:** Setting the prior odds to 100 (instead of the default 10) requires tenfold stronger Bayesian evidence to label a locus as selected, making the analysis appropriate for genome-wide scans where many neutral tests are performed.

### Separate pilot and main runs to troubleshoot proposal tuning
**Args:** `-input problem_locus.fst -output tuned -n 50000 -b 10000 -p 100 -thin 10 -only_write_init 1`
**Explanation:** Writing only the initialization parameters (with `-only_write_init 1`) allows users to reuse the tuned proposal distributions from the pilot phase in a subsequent run, which is useful when the default proposals lead to low acceptance rates.

### Seeded run for reproducible results across analyses
**Args:** `-input replicate_A.fst -output reproducible -seed 42 -n 50000 -b 5000 -p 20`
**Explanation:** Specifying a random seed of 42 ensures that repeating the exact same command produces identical MCMC trajectories, which is critical for reproducible research and for verifying chain convergence.