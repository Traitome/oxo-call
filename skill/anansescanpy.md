---
name: anansescanpy
category: Bioinformatics / Single-Cell Genomics
description: A Python toolkit for analyzing single-cell RNA-seq data with support for custom gene set enrichment, trajectory inference, and multimodal integration. Operates on AnnData objects and produces standardized output matrices for downstream analysis.
tags:
- single-cell
- scrna-seq
- scanpy
- gene-expression
- trajectory-analysis
- cell-typing
- gene-set-enrichment
author: AI-generated
source_url: https://github.com/anansescanpy/anansescanpy
---

## Concepts

- **Input Format:** Accepts `.h5ad` (HDF5 AnnData) files as primary input, which contain the count matrix, observations (cells), and variables (genes) in a unified structure. The tool reads directly from the AnnData object in memory or from file paths.
- **Output Format:** Produces updated `.h5ad` files with annotated embeddings, cluster labels, and gene set scores appended to the object's uns (unsupervised) and obs (observations) layers. Results can also be exported to CSV for external visualization tools.
- **Key Behaviors:** Implements gene set scoring using weighted gene co-expression modules, supports pseudotime ordering via diffusion-based trajectory inference, and provides automated cell type annotation by matching query clusters to reference atlases through label transfer.

## Pitfalls

- **Using mismatched gene identifiers:** Passing a count matrix with Entrez gene IDs when the reference annotation expects Ensembl IDs causes all genes to fail matching, resulting in empty gene set scores and silent failure in downstream steps.
- **Forgetting to normalize before scoring:** Running gene set enrichment on raw counts instead of normalized expression produces biased scores that correlate with library size rather than biological signal, leading to incorrect cell type assignments.
- **Ignoring the batch effect flag in integration:** Attempting to integrate datasets from different sequencing technologies without specifying `--batch-key` creates batch-specific clusters that reflect technical variation rather than biological cell states.

## Examples

### Computing gene set enrichment scores for a pathway
**