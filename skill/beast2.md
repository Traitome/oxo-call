---
name: beast2
category: phylogenetics
description: Bayesian Evolutionary Analysis Sampling Trees 2 - a tool for analyzing molecular sequences using Bayesian coalescent theory to estimate species divergence times, population sizes, and phylogenetic relationships.
tags: phylogenetic, bayesian, coalescent, molecular evolution, species tree, MCMC
author: AI-generated
source_url: https://www.beast2.org/
---

## Concepts

- BEAST2 uses XML configuration files to define the analysis model, which includes tree models, site models, clock models, and priors—not direct CLI flags for most analysis parameters.
- Input alignment formats include NEXUS, FASTA, and CSV, which are converted to BEAST XML internally; the XML file specifies all evolutionary model settings.
- Output files include a .log file with parameter estimates, a .trees file with sampled tree topologies, and a .state file for resuming analyses; these are essential for diagnosing convergence.
- The -beagle flag enables GPU acceleration via the BEAGLE library, significantly speeding up likelihood calculations for large alignments.
- BEAST2 supports multiple packages (BFD, SpeciesID) that extend core functionality; package management is handled separately via beast2-package-manager.

## Pitfalls

- Using a chain length that is too short results in poor mixing and unreliable posterior estimates, leading to invalid conclusions about divergence times or population sizes.
- Specifying an incorrect clock model (e.g., using a strict clock when rate variation exists) produces biased divergence time estimates; model selection should be based on statistical testing.
- Failing to check MCMC convergence via tools like Tracer can lead to publication of unreliable results; always verify effective sample sizes (ESS) > 200.
- Overwriting output files without saving previous results occurs because -overwrite silently replaces existing log and tree files; use a new directory or disable overwriting.
- Running analyses without adequate burn-in wastes computational resources; the default burn-in may be insufficient for complex models.

## Examples

### Run a basic BEAST analysis with an XML input file