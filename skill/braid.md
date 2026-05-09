---
name: braid
category: phylogenetics / viral transmission analysis
description: A statistical inference tool for reconstructing viral transmission histories from genetic sequence data. Braid constructs time-resolved phylogenetic trees and estimates transmission parameters using Bayesian inference approaches. It accepts aligned FASTA sequences and outputs annotated trees in Newick format along with posterior estimates for transmission chain statistics.
tags:
  - phylogenetics
  - viral evolution
  - transmission chains
  - Bayesian inference
  - outbreak analysis
  - molecular epidemiology
author: AI-generated
source_url: https://github.com/etalang/braid
---

## Concepts

- **Input alignment format**: Braid requires a multiple sequence alignment in FASTA format. All sequences must be aligned with consistent coordinate systems. Sequences should include metadata in the sequence header using a specific parser tag format (e.g., ` SammpleID|date|location`) for subsequent phylodynamic modelling. Poorly aligned or unevenly trimmed alignments will cause inference failures or biased parameter estimates.

- **Time-resolved tree construction**: The tool applies a Gaussian process prior over continuous trait evolution to timestamp internal nodes. The clock rate is estimated from the data and can be constrained with a `--clock-rate` flag. Unreasonable clock rate priors (e.g., too low for fast-evolving viruses) lead to divergence time estimates that predate the sampling period, corrupting downstream transmission interval calculations.

- **MCMC posterior sampling**: Braid uses Markov Chain Monte Carlo (MCMC) to sample from the joint posterior of phylogenetic topologies and transmission parameters. Effective sample size (ESS) diagnostics are printed to stderr during runs. Chain convergence is assessed by checking that ESS values exceed 200 for all parameters. Short chain runs (fewer iterations) produce unreliable posterior summaries with inflated credible interval widths.

- **Output formats**: The tool writes sampled trees to a `.trees` file in Nexus format with embedded metadata, and parameter traces to a `.log` file compatible with `Tracer`. It can also export a single maximum a posteriori (MAP) tree via `--map-tree` for downstream applications. Output files are overwritten on subsequent runs with the same base name unless `--output-prefix` is changed.

- **Sample time requirements**: Each tip must have a decimal date value extracted from the sequence header for clock rate estimation. Dates that are missing, malformed, or in the future cause the sampler to halt with a parse error. Dates must be provided in ISO 8601 format (YYYY-MM-DD) or as decimal years (e.g., 2023.5) for successful parsing.

## Pitfalls

- **Using unaligned sequences**: Passing raw FASTA files without multiple sequence alignment causes Braid to produce incorrect branch length estimates and may crash the likelihood calculator. Always preprocess input with a dedicated aligner (e.g., MAFFT or Clustal Omega) before running Braid, and visually inspect the alignment for anomalous gap patterns.

- **Conflicting date metadata**: Mixing date formats in sequence headers (e.g., some using ISO 8601 and others using decimal years) results in silent parsing failures where only a subset of tips are dated. This leads to an incomplete phylogenetic tree where undated tips collapse onto a single polytomy, producing biologically meaningless transmission estimates.

- **Insufficient MCMC iterations**: Running Braid with fewer than 10,000 iterations often produces chains that have not mixed well, yielding unreliable posterior medians and credible intervals. This is particularly problematic for datasets with many taxa (>100 sequences), where Gelman-Rubin diagnostics may flag non-convergence even after 50,000+ iterations.

- **Ignoring burn-in**: The default MCMC burn-in discards 10% of samples, but datasets with complex likelihood surfaces may require 25–50% burn-in. Using the default without examining trace plots can result in posterior means that are biased toward the starting tree, especially for poorly fitting substitution models.

- **Mismatched reference tree priors**: If a reference phylogeny is provided via `--starting-tree`, the branch length units must match those expected by the clock model. Providing a substitution-length tree when a time-length tree is expected causes the Gaussian process to initialize on nonsensical branch lengths, leading to indefinite sampling failures.

- **Output file overwriting**: Braid silently overwrites any existing `.trees` and `.log` files with the same output prefix. In collaborative workflows, this can cause permanent data loss if a second user runs an analysis before archiving previous results. Always rename output prefixes for each analysis run.

## Examples

### Reconstruct a transmission history from an aligned FASTA file
**Args:** `align.fasta --output-prefix run01 --iterations 200000`
**Explanation:** This runs a full Bayesian MCMC analysis on an aligned FASTA file with 200,000 iterations, writing output files prefixed with `run01`. The high iteration count ensures adequate ESS values for reliable posterior estimates even for moderately sized datasets.

### Constrain the clock rate to a known value and run with a fixed starting tree
**Args:** `align.fasta --clock-rate 0.001 --starting-tree starting.nexus --output-prefix run02`
**Explanation:** Specifying `--clock-rate` fixes the molecular clock to 0.001 substitutions per site per year, bypassing estimation. The `--starting-tree` flag provides a pre-computed neighbor-joining tree to initialize the MCMC, reducing burn-in requirements and improving chain mixing for difficult alignments.

### Run a short exploratory analysis to check data compatibility
**Args:** `align.fasta --output-prefix test --iterations 5000 --thin 10`
**Explanation:** Running 5,000 iterations with thinning every 10 samples is suitable for a quick sanity check. This generates trace files quickly to verify that dates parse correctly, the alignment is valid, and the sampler initializes without errors before committing to a full production run.

### Export a maximum a posteriori tree for downstream visualization
**Args:** `align.fasta --map-tree map_tree.nexus --output-prefix final --iterations 500000`
**Explanation:** Specifying `--map-tree` instructs Braid to write the single highest-posterior-probability tree to the specified file. The high iteration count (500,000) ensures convergence, and the resulting Newick-compatible Nexus tree can be visualized in FigTree or used for transmission network plotting.

### Increase burn-in fraction to 30% for a complex likelihood surface
**Args:** `align.fasta --burn-in 0.3 --output-prefix robust --iterations 100000`
**Explanation:** Setting `--burn-in 0.3` discards the first 30% of MCMC samples before computing posterior summaries, which reduces bias toward the starting tree when the likelihood surface is multimodal. This is appropriate when previous runs showed poor Gelman-Rubin convergence or bimodal parameter traces.

### Generate a convergence diagnostic report after a run
**Args:** `align.fasta --output-prefix diagnostic --iterations 100000 --report-convergence`
**Explanation:** The `--report-convergence` flag produces an additional text file with ESS values, Gelman-Rubin R-hat statistics, and parameter trace summary statistics. This allows direct evaluation of whether 100,000 iterations were sufficient for all parameters or whether additional iterations are needed.

### Run with increased MCMC proposal adaptivity for large datasets
**Args:** `align.fasta --output-prefix large_data --iterations 1000000 --tune-interval 500`
**Explanation:** For datasets exceeding 200 sequences, the MCMC sampler may require adaptive tuning. The `--tune-interval 500` flag increases how frequently proposal distributions are adjusted during burn-in, which improves acceptance rates for topology proposals on large trees at the cost of longer runtime.