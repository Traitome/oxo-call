---
name: bayescode
category: phylogenetics
description: A Bayesian MCMC tool for inferring phylogenetic trees and evolutionary parameters from molecular sequences using codon-based models.
tags: [phylogenetics, bayesian-inference, mcmc, codon-model, evolutionary-rate]
author: AI-generated
source_url: https://github.com/bayescode/bayescode
---

## Concepts

- **Bayesian MCMC Framework**: bayescode uses Markov Chain Monte Carlo sampling to explore the posterior distribution of phylogenetic trees and model parameters. The chain runs for a specified number of generations, with state logged at regular intervals.
- **Codon-Based Substitution Models**: The tool supports evolutionary models that operate at the codon level (e.g., MG94, Muse-Gaut), allowing detection of synonymous vs. nonsynonymous substitution rates (dN/dS). Input alignments must be in codon format (triplet nucleotides).
- **Tree Prior Distribution**: Users must specify a tree prior (e.g., Yule, Birth-Death, Coalescent). The prior interacts with the likelihood to produce the posterior; incorrect prior choice leads to biased tree estimates.
- **Output Files**: The primary output is a tree file in NEXUS or Newick format containing sampled trees. Parameter logs (rates, node heights) are written to a separate log file for post-processing with tools like LogCombiner or Tracer.

## Pitfalls

- **Alignment Not in Codon Format**: Providing nucleotide alignments that are not codon-aligned (multiples of 3) causes the likelihood engine to fail or produce nonsensical results. Always verify alignment length is divisible by 3 before running.
- **Excessive Chain Length**: Running too few generations leads to inadequate sampling of the posterior, resulting in low effective sample sizes (ESS) for parameters. Always check ESS > 200 in Tracer before trusting results.
- **Incompatible Priors**: Setting a coalescent prior with a tree topology that has very high branch support can cause logical inconsistencies, leading to chain stuck states or premature convergence warnings.
- **Uncalibrated Operators**: Adjusting proposal operator weights without understanding their purpose (e.g., tuning subtree slide parameters) often reducesmixing, prolonging convergence time unnecessarily.

## Examples

### Running a basic phylogenetic analysis with default settings
**Args:** -seq alignment.fasta -tree clock -out results.nex
**Explanation:** Specifies an input alignment file in FASTA format and uses a strict molecular clock tree prior, writing output to a NEXUS file for downstream tree viewing.

### Specifying a Yule process prior for species tree inference
**Args:** -seq alignment.fasta -prior yule -birth-rate 0.1 -out species_tree.nex
**Explanation:** Uses a Yule pure-birth process as the tree prior with a fixed birth rate of 0.1, appropriate for independent species lineages.

### Running with four heated chains for better mixing
**Args:** -seq alignment.fasta -nchains 4 -temp 0.1 0.25 0.5 1.0 -out mixed.nex
**Args:** -generations 5000000 -sample 1000
**Explanation:** Runs four parallel chains with temperatures decreasing toward 1.0 to enhance acceptance rates in cold chain, sampling every 1000 generations over 5 million total steps.

### Estimating dN/dS rates under the Muse-Gaut model
**Args:** -seq alignment.fasta -model MG94 -omega -log omegas.log -out mg94_trees.nex
**Explanation:** Uses the Muse-Gaut codon model to estimate nonsynonymous-to-synonymous rate ratio (omega), logging omega values to a separate file for post-analysis.

### Resuming a previously crashed run from checkpoint
**Args:** -seq alignment.fasta -resume checkpoint.state -continue -out resumed.nex
**Args:** -generations 2000000
**Explanation:** Loads the checkpoint state file and continues the run for an additional 2 million generations, preserving chain history and ensuring continuity.
---