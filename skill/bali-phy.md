---
name: bali-phy
category: Phylogenetics and Sequence Alignment
description: Bayesian coalescent and phylogenetic sequence analysis tool that co-estimates multiple sequence alignments and phylogenies using Markov chain Monte Carlo (MCMC) sampling with statistical uncertainty quantification.
tags: [bayesian, phylogeny, sequence-alignment, MCMC, coalescent, statistical-alignment]
author: AI-generated
source_url: https://www.bali-phy.org/
---

## Concepts

- Bali-phy performs simultaneous Bayesian inference of multiple sequence alignments and phylogenetic trees, treating both as random variables rather than fixed point estimates. This approach provides full posterior distributions over alignments and trees, enabling proper quantification of uncertainty in both topology and indel evolution.
- The tool supports nucleotide, amino acid, and codon substitution models with flexible rate heterogeneity (discrete gamma, invariant sites, mixture models). Substitution model selection is critical; using the wrong model (e.g., nucleotide for highly-diverged protein sequences) produces systematically biased branch lengths and topologies.
- Input sequences must be pre-aligned only when the `--input-model` parameter specifies a fixed-alignment model. In the default statistical-alignment mode, Bali-phy treats gaps as uncertainty and integrates over all possible insert-delete histories. Providing an incorrectly pre-aligned FASTA file when using statistical alignment mode causes the model to learn contradictory gap histories.
- Output consists of posterior samples written to `.trees` files (annotated Nexus trees with alignment posterior probabilities at each column) and `.log` files containing MCMC diagnostics (likelihood, convergence statistics). The `--samples` flag controls thinning interval; insufficient samples result in poor posterior summaries and unreliable ESS (effective sample size) values.
- Bali-phy can leverage the BEAGLE library for GPU-accelerated likelihood calculations via the `BEAGLE_LIBRARY` environment variable. Without BEAGLE on large datasets (>200 sequences), runtime becomes prohibitively slow, and chains may stall without producing adequate posterior samples within reasonable wall-time limits.

## Pitfalls

- Forgetting to set the `--burnin` option causes the first portion of the posterior sample to include unrepresentative, non-equilibrium states from the early MCMC exploration. Consensus trees and parameter estimates will be systematically biased toward the starting tree and may fail to represent the true posterior distribution.
- Running multiple independent chains without the `-- chains` flag or `--fork` option and then failing to diagnose convergence with `bali-phy-coal` or `effective-size` results in an invalid analysis. Chains that have not mixed (low ESS, Gelman-Rubin R-hat >> 1.0) produce confidence intervals that are too narrow and may exclude the true parameter value.
- Specifying `--model DNA` for amino acid datasets (or vice versa) produces nonsensical alignments and trees with incorrect substitution rates and biased branch lengths. The model mismatch is silently accepted but yields biologically meaningless output; always verify the data type matches the substitution model before running the analysis.
- Not specifying `--seed` causes Bali-phy to derive random seeds from the system clock, making reproducibility across runs impossible. This is problematic for methods papers, grant proposals, or when comparing model variants where exact reproducibility is required.
- Attempting to resume a crashed or unfinished chain without the `--continue` flag starts a completely new analysis, discarding all completed posterior samples and wasting the original computation. Always check for a `.log` file in the output directory before re-running; use `--continue` to append samples from a fresh run to existing chains.

## Examples

### Aligning and estimating a phylogeny from a FASTA nucleotide file with default statistical alignment
**Args:** `sequences.fasta --model JukesCantor --name run1`
**Explanation:** Runs statistical alignment with a Jukes-Cantor substitution model, co-estimating the multiple sequence alignment and phylogenetic tree from raw unaligned FASTA input.

### Analyzing pre-aligned sequences with a fixed alignment model and amino acid substitution
**Args:** `aligned_proteins.fasta --model LG --input-model FixedAlignment --seed 42 --name protein_run`
**Explanation:** Uses a fixed input alignment with the LG amino acid substitution model, fixing the alignment rather than treating gaps as uncertain. The `--seed 42` ensures reproducible results.

### Running a quick exploratory analysis with short chain length and reduced sampling
**Args:** `data.fasta --model HKY --chainlen 100000 --samples 500 --burnin 10000`
**Explanation:** Executes a short MCMC run (100k iterations) with 500 samples after a 10k-iteration burnin for rapid exploratory analysis, suitable for debugging or parameter tuning before a production run.

### Resuming a previously interrupted run to add more posterior samples
**Args:** `--continue --chainlen 1000000 --samples 2000`
**Explanation:** Continues an existing analysis by appending new samples to the posterior, increasing total chain length to 1M iterations and producing 2000 thinned samples while preserving previously computed samples.

### Running multiple independent chains in parallel for convergence diagnosis
**Args:** `sequences.fasta --model GTR --chainlen 2000000 --fork 4 --samples 1000 --name multi_chain`
**Explanation:** Launches 4 independent MCMC chains from different random starts (controlled by `--fork 4`) to enable Gelman-Rubin convergence diagnosis and produce more reliable posterior summaries for the final consensus tree.

### Specifying codon substitution model for coding sequences with gamma-distributed rate heterogeneity
**Args:** `codon_sequences.fasta --model M0 --omega --ngammacat 4 --name codon_analysis`
**Explanation:** Runs a codon substitution model (M0) with estimated nonsynonymous/synonymous rate ratio (dN/dS) and 4 discrete rate categories for gamma-distributed rate heterogeneity, appropriate for coding DNA sequences under purifying or diversifying selection.