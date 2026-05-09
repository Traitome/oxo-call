---
name: cellrank-krylov
category: Single-cell RNA-seq Analysis / Velocity Computation
description: Computes velocity-based transition kernels and transition matrices using Krylov subspace methods for single-cell RNA velocity analysis in CellRank. Handles sparse matrix computations necessary forMarkov chain construction and cell fate mapping.
tags:
  - single-cell
  - rna-velocity
  - cellrank
  - krylov-subspace
  - scseq
  - transition-matrix
  - markov-chain
author: AI-generated
source_url: https://cellrank.readthedocs.io/
---

## Concepts

- **Input Format**: Accepts sparse matrix market (.mtx) or h5ad files containing raw RNA counts and spliced/unspliced velocities; produces transition kernel matrices for CellRank's Markov chain construction.
- **Krylov Subspace Solver**: Uses GMRes or CG iterations to solve linear systems (Ax = b) where the matrix A represents the velocity-based transition probability structure, converging in typically 50-500 iterations depending on sparsity.
- **Output Data Model**: Outputs a dense or sparse transition matrix (NxN for N cells) with row-stochastic properties (rows sum to 1.0); these matrices serve as input to CellRank's eigenvalue decomposition for computing absorption probabilities and fate maps.
- **Memory Efficiency**: Handles matrices up to 50,000 cells by 50,000 cells in sparse format, requiring approximately 2-4 GB RAM for typical single-cell datasets; dense storage scales quadratically (N²).
- **Companion Workflow**: Typically run after CellRank's RNA velocity estimation (via scVelo or Velocyto.py) and before `cellrank-absorption` for computing fate probabilities.

## Pitfalls

- **Using Dense Input for Large Datasets**: Providing dense matrices for >10,000 cells causes memory overflow (100M+ elements at 8 bytes each = 800 MB minimum); always convert to sparse format first or subsample cells.
- **Incorrect Row-Stochastic Normalization**: Failing to normalize output rows to sum to 1.0 produces invalid probability distributions, leading to nonsensical absorption times and incorrect fate predictions downstream.
- **Mismatched Velocity Orientation**: Supplying velocity vectors computed with the wrong gene alignment (e.g., spliced-oriented vs. unspliced-oriented) produces transpose-inverted kernels, completely reversing the computed fate directions.
- **Insufficient Krylov Iterations**: Setting convergence tolerance too loose (e.g., >1e-4) yields inaccurate kernels with residual errors propagating through the entire CellRank pipeline, distorting terminal state identification.
- **Ignoring Convergence Warnings**: Proceeding with unconverged Krylov solves (output shows >1000 iterations or NaN entries) produces unreliable transition matrices that compromise all downstream absorption probability computations.

## Examples

### Compute transition kernel from spliced/unspliced counts
**Args:** --input-velocity /data/velocity_input.h5ad --output-kernel /outputs/kernel.mtx --method gmres --tol 1e-6
**Explanation:** Reads velocity-estimated counts from h5ad and computes transition kernel using GMRes with tight tolerance for accurate Markov chain construction.

### Build transition matrix with conjugate gradient solver
**Args:** --input-sparse /data/sparse_counts.mtx --output-transition /outputs/trans.mat --solver cg --max-iter 500 --verbose
**Explanation:** Uses conjugate gradient solver (optimal for symmetric positive-definite matrices) with 500 maximum iterations and verbose output for diagnosing convergence issues.

### Compute absorption-compatible kernel with L2 normalization
**Args:** --velocity-vectors /data/velocities.csv --normalize l2 --output /outputs/abs_kernel.csv --solver gmres
**Explanation:** Applies L2 normalization to velocity vectors before computing kernel, ensuring row-stochastic property essential for absorption probability calculations.

### Process large dataset with checkpointing
**Args:** --input /data/large_dataset.h5ad --output /outputs/large_kernel.h5 --checkpoint 100 --restart 50
**Explanation:** Enables checkpointing every 100 Krylov iterations with 50 restart vectors, enabling processing of 30,000+ cell datasets that would otherwise exceed memory limits.

### Generate sparse kernel with tolerance optimization
**Args:** --input-mtx /data/velocity_sparse.mtx --output-sparse /outputs/sparse_kernel.mtx --tol 1e-5 --adaptive-tolerance
**Explanation:** Uses adaptive tolerance detection to automatically adjust convergence criteria based on residual history, reducing compute time for well-conditioned datasets.