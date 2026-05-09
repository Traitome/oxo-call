---
name: anndata2ri
category: Data Format Conversion / Single-Cell Bioinformatics
description: Tool for converting single-cell data between Python AnnData/H5AD format and R Seurat RDS format, enabling interoperability between Python (scanpy) and R (Seurat) analysis pipelines.
tags:
  - single-cell
  - anndata
  - h5ad
  - seurat
  - r
  - python
  - data-conversion
  - bioinformatics
  - scanpy
  - reticulate
author: AI-generated
source_url: https://github.com/theislab/anndata2ri
---

## Concepts

- **AnnData Data Model**: The AnnData object stores single-cell data as a matrix with rows representing genes and columns representing cells, along with annotations including obs (cell metadata), var (gene metadata), and uns (unsupervised data like embeddings). The H5AD file format is the on-disk representation that anndata2ri reads and writes.

- **Bidirectional Conversion**: anndata2ri supports conversion from Python AnnData (H5AD) to R Seurat (RDS) format and vice versa. This enables workflows that start in one ecosystem (e.g., Python for preprocessing with scanpy) to continue in the other (e.g., R for differential expression with Seurat).

- **Reticulate Integration**: The tool uses the reticulate R package to bridge Python and R, requiring proper environment configuration. Both the Python session (with anndata and scanpy installed) and R session (with Seurat installed) must be correctly initialized for seamless data transfer.

- **Preservation of Data Layers**: During conversion, multiple data layers (raw counts, normalized, scaled) should be preserved. The tool handles the translation of these layers between ecosystems, though certain normalization methods may require re-computation in the target environment.

## Pitfalls

- **Gene Name Mismatches**: Gene names may appear differently between Python and R (e.g., "MT-CO1" vs "COX1" or ENSEMBL IDs vs gene symbols). Conversion may fail silently or produce mismatched gene alignments if gene naming conventions are not standardized before conversion.

- **Missing reticulate Initialization**: Failing to properly initialize reticulate with `reticulate::use_python()` or `reticulate::use_condaenv()` before conversion will cause runtime errors. The Python environment must be accessible from R.

- **Version Incompatibilities**: Mismatched package versions between Python (anndata/scanpy) and R (Seurat) can cause conversion failures. The H5AD format version must be compatible with the Seurat version attempting to read it.

- **Memory Constraints with Large Datasets**: Converting large H5AD files (hundreds of thousands of cells) may exceed available RAM, particularly when loading the entire object into memory before writing. Chunked processing or subsetting may be required.

- **Metadata Type Mismatches**: R factor columns and categorical data may not directly map to pandas categorical columns or Python enum types. Metadata types may be lost or incorrectly coerced during conversion.

## Examples

### Convert H5AD to Seurat RDS format
**Args:** `--input data.h5ad --output data.rds`
**Explanation:** Converts a Python-generated AnnData H5AD file to R Seatur RDS format, enabling continued analysis in R without re-importing raw counts.

### Convert RDS to H5AD format
**Args:** `--input data.rds --output data.h5ad --format h5ad`
**Explanation:** Converts an R Seurat object to Python AnnData H5AD format, allowing Python-based downstream analysis such as scanpy workflows.

### Specify conversion without loading to memory
**Args:** `--input data.h5ad --output data.rds --compress FALSE`
**Explanation:** Disables compression during RDS export to speed up conversion at the cost of larger file size, useful when memory is not a constraint.

### Convert with gene Symbol mapping
**Args:** `--input data.h5ad --output data.rds --gene-column symbol`
**Explanation:** Uses the gene symbols column from the AnnData var dataframe for gene names in the output Seurat object, ensuring readable gene identifiers.

### Batch convert multiple files
**Args:** `--input-dir ./h5ad_files --output-dir ./rds_files`
**Explanation:** Converts all H5AD files in a directory to RDS format, useful for pipeline automation processing multiple samples in batch.