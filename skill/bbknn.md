---
name: bbknn
category: single-cell-batch-correction
description: Batch-Balanced K-Nearest Neighbors (BBKNN) for correcting batch effects in single-cell RNA-seq data using scanpy-compatible neighbors graph replacement
tags: [batch-correction, single-cell, scanpy, neighbors-graph, hvg-selection]
author: AI-generated
source_url: https://github.com/Teichlab/bbknn
---

## Concepts

- **AnnData data model**: BBKNN operates on scanpy `AnnData` objects where the corrected neighbor graph is stored in `.obsp['connectivities']` and `.obsp['distances']`. The original `.X` matrix (or raw layer) is not modified; only the neighbor structure is replaced with batch-balanced edges that connect cells across batches while preserving within-batch local structure.

- **Batch axis and batch key**: BBKNN uses a categorical `batch_key` column in `.obs` to identify which cells belong to which batch. The `batch_axis` parameter (default 0) specifies which axis to treat as batches when using multiomics data (e.g., 0 for cells, 1 for features). Cells from different batches are forced to become neighbors while reducing cross-batch edges.

- **bbknn.bert Ritz function**: The primary entry point is `bbknn.bert Ritz(adata, batch_key='batch', ...)`, which wraps the `BBKNNGraph` class and updates the AnnData in place. It accepts parameters like `n_neighbors` (k per batch, default 15), `neighbors_within_batch` for fine-tuning connectivity, and can operate on PCA embeddings (`use_rep`), raw counts (`assay_name`), or any slot in `.varm`.

- **Input format flexibility**: BBKNN accepts AnnData (`.h5ad`), MuData (`.h5mu`), and loom files. When using PCA (`use_rep='X_pca'`), BBKNN computes neighbors in the reduced dimensional space rather than the raw count space, which is the recommended workflow after highly variable gene selection and PCA.

- **CLI interface**: The `bbknn` command-line tool processes h5ad files directly with flags for `--input`, `--output`, `--batch_key`, and `--n_neighbors`, making it suitable for pipeline integration without Python scripting.

## Pitfalls

- **Not selecting HVGs before correction**: Running BBKNN on all genes or unfiltered data introduces noise, captures batch-specific patterns, and leads to poor integration. Always run `sc.pp.highly_variable_genes()` and subset before PCA and BBKNN. Consequence: over-clustering artifacts and inflated batch effect removal.

- **Incorrect `batch_key` column**: Specifying a batch column that does not exist in `.obs` raises a `KeyError`. Using a continuous numeric column instead of categorical causes unexpected binning behavior. Consequence: silent mis-assignment of batch labels or runtime failure.

- **Over-correction from excessive `neighbors_within_batch`**: Setting `neighbors_within_batch` to 0 forces all neighbors to be cross-batch, destroying biological signal within each batch. Consequence: loss of cell-type-specific structure, artificial mixing, and downstream clustering artifacts.

- **Inconsistent library size normalization**: If batches have different sequencing depths or normalization strategies, BBKNN computes distances on incompatible scales. Consequence: batch-specific clusters persist despite neighbor graph replacement.

- **Using raw counts without log-transformation**: Passing `adata.raw` directly without `sc.pp.log1p()` can violate the assumption of approximate Gaussian structure for kNN distances. Consequence: reduced integration accuracy and poor UMAP/t-SNE visualization.

## Examples

### Basic batch correction using scanpy integration
**Args:** `--input processed.h5ad --output corrected.h5ad --batch_key sample --n_neighbors 5`
**Explanation:** This runs BBKNN after PCA embedding, replacing the default nearest-neighbor graph with batch-balanced neighbors where each cell connects to 5 neighbors per batch, producing an integrated object suitable for downstream clustering.

### Multi-batch integration with three samples
**Args:** `--input multi_batch.h5ad --output integrated.h5ad --batch_key library_id --n_neighbors 10 --neighbors_within_batch 3`
**Explanation:** With three batches, this preserves 3 within-batch connections per cell while adding 10 cross-batch connections, balancing biological signal preservation with effective batch correction across a complex multi-sample dataset.

### Using the CLI with loom input
**Args:** `--input cells.loom --output corrected.h5ad --batch_key batch --n_neighbors 8`
**Explanation:** The CLI accepts loom files directly and outputs an h5ad AnnData object, enabling pipeline workflows that use loom-format single-cell data without requiring Python scripting.

### PCA-based correction with custom perplexity tuning
**Args:** `--input pca_integrated.h5ad --output bbknn_corrected.h5ad --batch_key donor --n_neighbors 15 --neighbors_within_batch 5`
**Explanation:** This uses the pre-computed PCA representation from `adata.obsm['X_pca']` for neighbor computation rather than raw counts, with higher k and more within-batch edges to preserve fine-grained cell-state structure while correcting donor-level batch effects.

### Checking batch effect before and after
**Args:** `--input prebatch.h5ad --output postbatch.h5ad --batch_key experiment --n_neighbors 5`
**Explanation:** This applies BBKNN to an object where batch effects remain after initial processing, replacing the neighbor graph for downstream Leiden clustering to verify that batch-associated variation is reduced in the corrected output.