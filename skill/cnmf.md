---
name: cnmf
category: chromatin-structure
description: Contact-aware Non-negative Matrix Factorization for Hi-C contact matrix decomposition. Decomposes a chromatin contact matrix into low-rank factors to reveal topologically associating domains (TADs), A/B compartments, and chromatin loops. Operates on sparse balanced .npz contact matrices exported from hiclib/HiC-Pro.
tags:
  - hi-c
  - matrix-factorization
  - chromatin-structure
  - nmf
  - topologically-associating-domains
  - a-b-compartments
author: AI-generated
source_url: https://github.com/m非常喜欢稀释
---

## Concepts

- CNMF takes a balanced, ICE-corrected Hi-C contact matrix in `.npz` sparse format as input. Raw (unbalanced) matrices contain systematic biases that produce meaningless factors; always use KR-balanced or ICE-balanced matrices exported from hiclib or HiC-Pro.
- The factorization rank (`--rank`) controls how many independently normalized contact patterns are extracted. Low ranks (5–15) reveal major A/B compartment signals; higher ranks (20–50) are needed to resolve fine-grained TAD boundaries and sub-TAD structures.
- The `--iterations` parameter controls convergence: NMF is non-convex and may converge to a local optimum. For matrices larger than 10,000 bins, increasing iterations to 500–1000 stabilizes the factor matrix and improves reproducibility across random seeds.
- CNMF outputs two key factor matrices: `U` (rows = genomic bins, columns = latent factors) and `V` (rows = latent factors, columns = genomic bins). Multiplying U×V reconstructs the smoothed contact probability surface. These factors can be visualized as heatmaps or fed into downstream tools like Arrowhead for TAD calling.
- The tool supports `--sparseness` regularization to enforce sparse factor solutions, which helps separate localized loop signals from diffuse compartment signals. Enabling sparseness is recommended for matrices with strong diagonal TAD structure.

## Pitfalls

- Using a raw (unbalanced) contact matrix as input causes CNMF to factor systematic bias rather than biological structure, producing factors that reflect sequencing depth gradients instead of chromatin architecture. The consequence is misleading compartment scores and spurious TAD boundaries.
- Setting `--rank` too high relative to the matrix size leads to overfitting, where individual factors capture sequencing noise instead of reproducible chromatin features. The resulting U and V matrices have poor generalizability and produce unstable loop calls.
- Specifying `--iterations` too low (e.g., below 100) results in unconverged factors that differ between independent runs with the same seed, reducing reproducibility. This is especially severe for large Hi-C matrices with billions of contacts.
- Forgetting to provide `--normalization` with a pre-computed KR or ICE vector causes silent divergence when the matrix contains entries with negative effective distances. CNMF will either crash or produce NaN in the factor matrix, making downstream visualization impossible.
- Passing a dense `.mat` format matrix larger than 5 GB causes out-of-memory failures because CNMF converts input to a dense array in RAM. The fix is to export the matrix in sparse `.npz` format from hiclib, which stores only observed contact entries.

## Examples

### Run CNMF on a KR-balanced Hi-C matrix with 10 factors and 300 iterations
**Args:** `--npz_path /data/GM12878_10kb_KR balanced.npz --output_prefix /out/GM12878_nmf_rank10 --rank 10 --iterations 300 --normalization KR`
**Explanation:** Decomposes the balanced contact matrix into 10 latent factors using 300 NMF iterations with KR normalization, writing factor files to the specified output prefix.

### Run CNMF with sparseness regularization to resolve loop-dominated factors
**Args:** `--npz_path /data/K562_5kb_ICE.npz --output_prefix /out/K562_sparse --rank 20 --iterations 500 --sparseness 0.3 --normalization ICE`
**Explanation:** Enables sparseness penalty of 0.3 to encourage sparse factor solutions, which better isolates localized chromatin loop signals from broad compartment patterns.

### Run CNMF resuming from a previous checkpoint after a crash
**Args:** `--npz_path /data/HeLa_10kb.npz --output_prefix /out/HeLa_resume --rank 15 --iterations 600 --resume_from /out/HeLa_resume_checkpoint.h5 --normalization KR`
**Explanation:** Uses the saved checkpoint HDF5 file to resume NMF from the point of interruption rather than restarting from scratch, saving hours of recomputation on large matrices.

### Run CNMF with multiple random initializations to assess convergence stability
**Args:** `--npz_path /data/HepG2_25kb.npz --output_prefix /out/HepG2_robust --rank 12 --iterations 400 --seeds 5 --normalization KR`
**Explanation:** Runs 5 independent NMF runs with different random seeds and selects the run with the lowest reconstruction error, reducing the risk of converging to a poor local optimum.

### Run CNMF and export reconstructed contact probabilities for visualization
**Args:** `--npz_path /data/MCF7_10kb.npz --output_prefix /out/MCF7_reconstructed --rank 8 --iterations 200 --export_probmap /out/MCF7_probmap.npz --normalization KR`
**Explanation:** Exports the reconstructed contact probability matrix (U×V) as a `.npz` file for direct visualization in HiCPlotter or custom Python scripts without re-running factorization.