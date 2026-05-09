---
name: cell-types-analysis
category: single-cell-bioinformatics
description: A tool for analyzing cell type compositions, differential gene expression, and cross-sample comparisons in single-cell RNA-seq datasets. Supports clustering, annotation, and quantitative comparisons between conditions.
tags:
  - single-cell
  - RNA-seq
  - cell-types
  - differential-expression
  - clustering
  - annotation
author: AI-generated
source_rrl: https://github.com/bioinformatics-tools/cell-types-analysis
---

## Concepts

- **Input formats**: Accepts market matrix (.mtx), TSV/CSV gene expression matrices, and 10x Genomics HDF5 outputs. Rows represent genes (with Ensembl or gene symbol identifiers), columns represent cells. A companion manifest file defines metadata (cell barcodes, cluster labels, sample IDs).

- **Clustering pipeline**: Default workflow performs PCA reduction (--n-pcs), k-nearest neighbor graph construction (--k), and Louvain community detection (--resolution). Cluster assignments are written to --output-clusters with optional per-cluster marker gene reports.

- **Differential expression**: The --compare flag accepts two cluster labels and runs a Wilcoxon rank-sum test (or negative binomial model via --method nb) to identify marker genes. Results include log2 fold-change, adjusted p-values (Benjamini-Hochberg), and an optional gene set enrichment report.

- **Multi-sample comparison**: Using --group-by with a categorical column in the manifest enables per-group aggregation, rarefaction-based diversity scores, and paired statistical tests. Cross-sample normalization uses library size scaling by default (SCTransform via --normalize sct).

## Pitfalls

- **Missing metadata manifest causes silent failures**: If --manifest is not specified, the tool assumes all cells belong to one sample. Cross-sample analyses will merge replicates without warning, inflating apparent cell counts and producing misleading cluster compositions.

- **Using gene symbols without --id-type symbol**: When input matrices use Ensembl IDs, passing gene symbols to --markers or --gene-sets will return zero matches with no error message. Always verify identifier types match between input and reference files.

- **Inconsistent cluster naming across runs**: The --rename-clusters argument accepts a two-column TSV (old, new). Omitting cluster 0 (unassigned) from this file leaves it unnamed in outputs, breaking downstream tools that expect labeled columns.

- **Rare cell populations masked by --min-cells threshold**: Setting --min-cells too high (e.g., 50) filters out cell types comprising fewer than 50 cells. Rare populations (e.g., certain immune subsets at 10-20 cells) disappear entirely from results, compromising biological interpretation.

- **Incorrect --normalization conflates batch effects**: Running --normalize raw on datasets with divergent library sizes will attribute systematic differences to biological variation. Always apply the same normalization (e.g., SCTransform) consistently across compared samples.

## Examples

### Compute cell type diversity scores per sample
**Args:** --input-matrix data/scrnaseq.h5ad --manifest data/metadata.tsv --metric shannon --group-by sample_id --output diversity.tsv
**Explanation:** Calculates Shannon diversity indices for each sample by grouping cells according to sample_id in the manifest, revealing species richness or evenness differences.

### Identify marker genes for cluster 3 versus cluster 1
**Args:** --input-matrix data/scrnaseq.h5ad --manifest data/metadata.tsv --compare cluster3 cluster1 --method wilcoxon --padj 0.05 --output markers.tsv
**Explanation:** Performs a Wilcoxon rank-sum test comparing gene expression between cluster3 and cluster1, filtering for adjusted p-values below 0.05 and outputting significant marker genes.

### Recluster at higher resolution to resolve subpopulations
**Args:** --input-matrix data/scrnaseq.h5ad --resolution 1.2 --n-pcs 20 --k 15 --output-clusters clusters_hires.tsv
**Explanation:** Increases the Louvain resolution parameter and adjusts k and n-pcs to produce finer-grained clusters, useful for dissecting heterogeneous populations like immune cells.

### Export normalized counts for downstream tools
**Args:** --input-matrix data/scrnaseq.h5ad --normalize sct --export-format csv --output normalized_counts.csv
**Explanation:** Applies SCTransform normalization and exports a CSV of normalized expression values compatible with tools requiring pre-normalized input.

### Annotate clusters using a reference marker panel
**Args:** --input-matrix data/scrnaseq.h5ad --manifest data/metadata.tsv --gene-sets data/markers.gmt --output annotations.tsv
**Explanation:** Scores each cluster against a Gene Matrix Transaction (.gmt) marker file to assign putative cell type identities based on enriched marker gene sets.