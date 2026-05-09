---
name: annembed
category: Bioinformatics / Embedding Analysis
description: A tool for embedding biological data points in high-dimensional spaces, enabling similarity search, clustering, and visualization. Supports various embedding formats and provides operations for computing distances, finding nearest neighbors, and reducing dimensionality.
tags:
- embedding
- nearest-neighbor
- bioinformatics
- similarity-search
- dimensionality-reduction
author: AI-generated
source_url: https://github.com/annembed/annembed
---

## Concepts

- **Input Formats**: Accepts TSV, CSV, or binary embedding files where rows represent data points and columns represent dimensions. The first column is typically an identifier, followed by numeric feature values. Header lines are optional.
- **Data Model**: Embeddings are treated as vectors in n-dimensional space. The tool maintains an in-memory index for fast similarity queries. For large datasets, supports sparse representations to reduce memory usage.
- **Key Operations**: Core functionalities include computing pairwise distances (Euclidean, cosine, Manhattan), performing approximate nearest neighbor (ANN) queries, batch similarity searches, and exporting results in standard formats.
- **Output Options**: Results can be streamed to stdout, written to files, or used as input for downstream tools. Supports BED, JSON, and custom text formats for interoperability with genome browsers and other bioinformatics pipelines.

## Pitfalls

- **Mismatched Dimension Counts**: Providing embeddings with different dimensionality than the index was built with causes silent failures or incorrect distance calculations. Always verify dimension consistency before querying.
- **Missing Values in Embeddings**: The tool does not automatically handle missing or NaN values and may produce NaN results or crash. Preprocess data to impute or remove incomplete vectors.
- **Incorrect File Delimiters**: Using the wrong delimiter (tab vs. comma) results in parsing errors or misaligned columns. Explicitly specify the delimiter with the appropriate flag; do not rely on auto-detection.
- **Floating-Point Precision Loss**: Repeated nearest-neighbor queries on very high-dimensional data can accumulate floating-point errors. Use double-precision input when accuracy is critical.

## Examples

### Compute pairwise Euclidean distances between two embedding sets
**Args:** `--mode distance --input-a embeddings_a.tsv --input-b embeddings_b.tsv --metric euclidean`
**Explanation:** This computes the Euclidean distance between each vector in `embeddings_a.tsv` and `embeddings_b.tsv`, useful for comparing gene expression profiles across datasets.

### Find the k-nearest neighbors for a single query vector
**Args:** `--mode knn --query query.tsv --index index.tsv --k 10 --metric cosine`
**Explanation:** Returns the 10 most similar vectors from the index using cosine similarity, ideal for finding similar cells or sequences in a reference embedding space.

### Build an ANN index for fast batch queries
**Args:** `--mode build --input embeddings.tsv --output index.ann --algorithm hnsw --ef-construction 200`
**Explanation:** Creates a Hierarchical Navigable Small World (HNSW) index from the input embeddings to accelerate subsequent nearest neighbor searches.

### Export nearest neighbor results in JSON format
**Args:** `--mode knn --query query.tsv --index index.tsv --output results.json --format json --k 5`
**Explanation:** Writes the top 5 nearest neighbors for each query to a JSON file, facilitating integration with downstream analysis pipelines.

### Reduce embedding dimensionality using PCA
**Args:** `--mode reduce --input embeddings.tsv --method pca --components 50 --output reduced.tsv`
**Explanation:** Applies Principal Component Analysis to reduce each embedding vector to 50 dimensions, useful for visualization or before applying additional clustering algorithms.

### Compute centroid distances for cluster validation
**Args:** --mode centroid-distance --clusters clusters.tsv --embeddings embeddings.tsv --metric manhattan
**Explanation:** Computes Manhattan distances between cluster centroids and the overall dataset centroid, providing a metric for assessing cluster cohesion and separation.