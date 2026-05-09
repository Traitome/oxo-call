---
name: cellrank
category: single-cell_analysis
description: CellRank is a toolkit for fate mapping and directional transition probability estimation in single-cell genomics, combining RNA velocity with Markov chain models to compute cellular transition probabilities and identify terminal states.
tags:
- rna-velocity
- trajectory-analysis
- fate-mapping
- markov-chain
- single-cell
- absorption-probabilities
- scvelo
- scanpy
author: AI-generated
source_url: https://cellrank.org
---

## Concepts

- CellRank operates on AnnData objects from scanpy, using the `.layers['velocity']` for splicing-based velocity estimates and `.obsp['connectivities']` for neighborhood graph weights; it does NOT require raw counts but needs pre-computed velocity vectors.
- The core concept is the **kernel** — a transition matrix computed from RNA velocity directions (velocity_kernel), graph structure (connectivity_kernel), or combined (velocity-weighted kernel); kernels are normalized to create valid Markov chains where rows sum to 1.
- **Absorption probabilities** represent the long-term fate likelihood: for each cell, the probability of reaching each terminal state (absorbing class) — computed by solving a linear system (I - Q)^-1 * R where Q is the transient-to-transient submatrix.
- CellRank distinguishes **terminal states** (possible endpoints like differentiated cells) from **initial states** (progenitor populations); these can be inferred automatically via eigenvector analysis or provided as annotations in `.obs`.
- Output is written back to the AnnData object: transition matrices in `.obsp['T']`, absorption probabilities in `.obs['absorption_probs']`, and fate probabilities in `.obs` with suffixes like `.{fate_name}_fate_prob`.

## Pitfalls

- Using velocity vectors computed with a different gene-loom or loom-file than the one used for expression clustering — this creates dimension mismatch errors because CellRank expects velocity vectors to align with the main gene-by-cell expression matrix.
- Forgetting to set the `backward` direction when analyzing trajectories toward earlier developmental states; the default computes forward-in-time transitions, leading to biologically incorrect fate maps for progressive differentiation.
- Passing a kernel computed with default normalization without checking convergence — sparse or poorly connected neighborhoods produce near-zero rows in the transition matrix, causing numerical instability in absorption probability computation.
- Using `write_connections=False` when you later need to plot lineage drivers — connection weights are not stored, and re-computing the kernel loses the exact transition weights needed for visualization, making it impossible to debug failed trajectories.
- Not specifying `n_jobs` on systems with many cores when computing permutation tests for gene-wise aggregation — the default single-threaded run can take hours on large datasets, wasting computational resources.

## Examples

### Compute transition matrix from RNA velocity and write to AnnData
**Args:** `tmc knet=velocity -d VelocytoDataset.h5ad --output-adata out.h5ad`
**Explanation:** Computes a velocity-based transition kernel using RNA velocity vectors stored in the VelocytoDataset and writes the resulting transition matrix back to the output AnnData file.

### Identify terminal states using Markov chain eigen decomposition
**Args:** `tmest in=out.h5ad n_starts=3 n_terminals=5 --cluster-attr clustering`
**Explanation:** Identifies 5 likely terminal states and 3 potential starting populations by analyzing the eigendecomposition of the transition matrix stored in the input AnnData.

### Compute absorption probabilities toward identified terminal states
**Args:** `tmmaps in=out.h5ad --backward --write-absorption --n-cycles 3`
**Explanation:** Solves the absorption probability matrix for reaching terminal states, using backward mode to trace developmental trajectories toward earlier states, writing results to `.obs`.

### Aggregate gene-wise transition probabilities for lineage drivers
**Args:** `tmrank-genes in=out.h5ad --basis umap --n-genes=50 --top-n=200`
**Explanation:** Computes gene-wise aggregation scores to identify likely lineage drivers using the UMAP basis for visualization space, reporting the top 50 genes with scores in `.obs['lineage_drivers']`.

### Plot directional entropy showing transition plasticity
**Args:** `tmplot in=out.h5ad --basis umap --directionality --size 1e3`
**Explanation:** Visualizes per-cell directionality entropy on the UMAP embedding, where high entropy indicates ambiguous trajectory direction, using a point size of 1000 for visibility.