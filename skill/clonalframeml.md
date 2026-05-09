---
name: clonalframeml
category: Population Genetics / Phylogenetics
description: A Bayesian MCMC method for inferring recombination rates, mutation rates, and demographic history from bacterial whole-genome sequences. ClonalFrameML implements a coalescent-based model that accounts for clonal interference and horizontal gene transfer.
tags: [bacterial-population-genetics, recombination, coalescent, MCMC, phylogenetic-inference, genomics]
author: AI-generated
source_url: https://github.com/xgena/ClonalFrameML
---

## Concepts

- **Input format**: ClonalFrameML accepts multiple sequence alignments in FASTA, Nexus, or PHYLIP formats. The alignment must contain at least 4 sequences with no missing data; all sequences must be of equal length and properly aligned.
- **Coalescent model**: The tool models bacterial evolution using the coalescent with recombination, estimating key parameters including recombination rate (ρ), mutation rate (θ), and the relative frequency of recombinations that are fixed in the population.
- **MCMC inference**: ClonalFrameML uses Markov Chain Monte Carlo sampling to explore parameter space, requiring sufficient burn-in and chain length to achieve convergence. The output includes a consensus tree and parameter estimates with credible intervals.
- **Output files**: The tool produces a tree file in Newick format (`.tre`), a parameter log file with MCMC samples, and summary statistics including estimated recombination and mutation rates per site.

## Pitfalls

- **Unconverged MCMC chains**: Running insufficient MCMC iterations leads to unreliable parameter estimates with BiSEs (Bias-Standard Errors) that do not reflect true uncertainty, producing misleading recombination rates.
- **Inappropriate sequence selection**: Including sequences with excessive missing data, very close relatives (near-zero branch lengths), or highly recombinant regions causes the coalescent model to fail, often producing negative parameter estimates.
- **Ignoring recombination hotspots**: Treating the entire alignment as uniformly recombinogenic loses power; when recombination is concentrated in specific regions, the overall rate estimate will be biased toward the local rate.
- **Parallel runs without convergence diagnostics**: Running multiple independent chains but not checking for convergence (e.g., via Gelman-Rubin diagnostic) risks accepting a chains that have not explored the same parameter space, invalidating results.
- **Insufficient burn-in**: Starting the analysis too early in the MCMC chain includes initial parameter values far from the stationary distribution, biasing all summary statistics toward the starting point.

## Examples

### Running a basic analysis on a FASTA alignment
**Args:** -i input.fasta -o output_prefix -x 100000
**Explanation:** Runs ClonalFrameML on the input alignment named `input.fasta`, performing 100,000 MCMC iterations (default burn-in of 10%) and saving results with prefix `output_prefix`.

### Increasing MCMC iterations for better convergence
**Args:** -i input.fasta -o output_prefix -x 500000 -b 50000
**Explanation:** Runs a longer MCMC chain with 500,000 total iterations and 50,000 burn-in iterations to ensure the chain reaches stationary distribution before collecting samples.

### Using multiple independent chains for convergence checking
**Args:** -i input.fasta -o output_prefix -x 200000 -c 4
**Explanation:** Runs 4 independent chains each with 200,000 iterations; enables comparison of parameter estimates across chains to assess convergence.

### Specifying a seed for reproducibility
**Args:** -i input.fasta -o output_prefix -x 100000 -s 42
**Explanation:** Sets the random seed to 42, ensuring identical results across multiple runs of the same analysis.

### Adjusting the mutation rate prior
**Args:** -i input.fasta -o output_prefix -x 100000 --theta_prior_mean 0.01
**Explanation:** Sets the prior mean for the mutation rate parameter (θ) to 0.01, incorporating prior knowledge about the expected diversity level in the population.

### Reducing output verbosity
**Args:** -i input.fasta -o output_prefix -x 100000 -q
**Explanation:** Runs in quiet mode, suppressing detailed progress output to the terminal while still producing all standard output files.

### Analyzing a Nexus format alignment
**Args:** -i input.nex -o output_prefix -x 100000 --nexus
**Explanation:** Reads the input alignment in Nexus format and processes it under the same coalescent model as FASTA inputs, outputting identical file formats.