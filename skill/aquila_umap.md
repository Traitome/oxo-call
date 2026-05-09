---
name: aquila_umap
category: Dimensionality Reduction
description: A command-line tool for performing UMAP (Uniform Manifold Approximation and Projection) dimensionality reduction on bioinformatics matrices such as gene expression count tables, single-cell data, or any numeric feature matrix. Supports multiple metrics, custom neighbor parameters, and various output formats.
tags: [umap, dimensionality-reduction, single-cell, visualization,machine-learning, embeddings]
author: AI-generated
source_url: https://github.com/aquila-tools/aquila_umap
---

## Concepts

- **Input Format**: The tool accepts tab-delimited (.tsv) or comma-separated (.csv) numeric matrices where rows represent samples/cells and columns represent features/genes. The first row must contain column headers, and the first column (optional) can contain sample IDs.
- **Metric Selection**: The `--metric` argument controls the distance metric used in the UMAP algorithm. Common options include `euclidean` (default), `cosine` (recommended for gene expression data), `manhattan`, and `correlation`.
- **Embedding Output**: Results are written as a tab-delimited file with three columns: sample_id (from input or auto-generated), UMAP1, and UMAP2. If `--n-components 3` is specified, a third UMAP3 column is included.
- **Deterministic Runs**: Using `--seed` with a fixed integer ensures reproducible embeddings across runs. Without a seed, results may vary slightly between executions even with identical parameters.

## Pitfalls

- **Missing Header Row**: Providing input files without a header row causes the tool to treat the first data row as column names, leading to incorrect embeddings and downstream analysis errors. Always ensure your matrix has a header row.
- **Excessive Memory Usage with Large Matrices**: UMAP is memory-intensive; matrices with >50,000 rows may cause out-of-memory errors. Using `--n-workers 1` reduces parallel memory consumption but increases runtime.
- **Inconsistent Results Across Runs**: Omitting `--seed` produces non-reproducible embeddings, which complicates comparison between analysis runs and can affect downstream clustering or marker identification.
- **Mismatched Metric for Sparse Data**: Using `euclidean` metric on highly sparse count matrices (e.g., scRNA-seq) often yields suboptimal embeddings. The `cosine` metric is more appropriate for sparse genomic data.

## Examples

### Running UMAP on a gene expression matrix with default parameters
**Args:** `--input expression_matrix.tsv --output results/umap_embed`
**Explanation:** Performs UMAP dimensionality reduction using default settings (15 neighbors, min_dist 0.1, euclidean metric) on the gene expression matrix and writes output to results/umap_embed.tsv.

### Using cosine metric for sparse single-cell RNA-seq data
**Args:** `--input scRNA_counts.tsv --output scRNA/umap_cosine --metric cosine --n-neighbors 30`
**Explanation:** Applies cosine distance metric, which is more suitable for high-dimensional sparse count data, with 30 neighbors to capture broader local structure in single-cell embeddings.

### Generating 3DUMAP embeddings for 3D visualization
**Args:** `--input combined_matrix.tsv --output 3d_embeddings --n-components 3 --min-dist 0.05`
**Explanation:** Produces three-dimensional UMAP coordinates useful for interactive 3D visualization tools, with tighter minimum distance to preserve more local cluster structure.

### Controlling randomness with a fixed seed
**Args:** `--input time_series_data.tsv --output reproducible_umap --seed 42`
**Explanation:** Sets random seed to 42 for deterministic output, ensuring identical embeddings across multiple runs for reproducibility in pipelines.

### Optimizing for large datasets with reduced memory
**Args:** `--input large_dataset.tsv --output umap_large --n-neighbors 50 --n-workers 1 --chunk-size 1000`
**Explanation:** Uses single worker and chunked processing to reduce memory footprint for large matrices, trading increased runtime for memory efficiency.

### Producing multiple output formats for different downstream tools
**Args:** `--input my_matrix.tsv --output analysis/umap --formats tsv,csv,json`
**Explanation:** Generates UMAP embeddings in all three formats simultaneously, allowing seamless integration with various downstream visualization and analysis tools.