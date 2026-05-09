---
name: celltypist-so
category: single-cell RNA-seq annotation
description: A command-line tool for supervised cell type annotation of single-cell or single-nucleus RNA-seq data using pre-trained or custom machine learning models, outputting cell type predictions and confidence scores per cell.
tags: [scRNA-seq, cell-type-annotation, supervised-learning, single-cell, gene-expression]
author: AI-generated
source_url: https://github.com.com/sonytype/celltypist-so
---

## Concepts

- **Input formats**: CellTypist accepts data in `.h5ad` (AnnData, the standard Python single-cell format), `.h5` (Loom/HDF5), `.csv`, or `.mtx` directory formats. AnnData objects must contain a raw or normalized gene-by-cell count matrix in `.X`. Passing a `.h5ad` file directly is the most common and recommended path.
- **Model types and training**: CellTypist trains an logistic regression classifier (ElasticNet-regularized) on labeled reference data. Pre-trained models (e.g., `Immune_All_High.pth`) can be used for prediction out-of-the-box, or `celltypist-so-build` can train a custom model from a labeled reference `.h5ad` file using the `--train` flag and a `--labels` column name pointing to the cell type label field in `adata.obs`.
- **Prediction and output**: The `celltypist-so predict` command writes two output files: a `.csv` file with cell barcodes and predicted labels, and a `.pkl` file containing the full probability matrix (cell × label). Use `--outdir` to specify the output directory and `--prefix` to name output files. The probability matrix is essential for downstream ambiguity resolution.
- **Confidence thresholds and micro-labeling**: Use `--confidence` (float, 0.0–1.0) to relabel low-confidence predictions as `"Unknown"`. The `--assign` flag additionally produces a hard assignment by choosing the highest-probability label per cell, while the probability table enables soft interpretation of borderline calls.
- **Batch correction and mode**: The `--mode` argument controls cross-validation style (`canonical` or `augment`) during training. The `--batch-correction` flag applies ComBat-seq-style harmonization across samples before training, which is critical when reference datasets span multiple technologies (10x Chromium vs. Drop-seq).

## Pitfalls

- **Passing a non-count matrix as input**: If the `.X` slot of the AnnData object contains normalized or log-transformed data instead of raw counts, the model will produce systematically biased predictions. Always verify with `adata.X[:10, :5]` that values are non-negative integers or floats before passing to `celltypist-so predict`.
- **Missing or misspelled label column**: Using `--labels` with a column name that does not exist in `adata.obs` causes a silent failure or an error about missing keys. Always verify exact column names with `list(adata.obs.columns)` before training, especially when the label column contains special characters or spaces.
- **Incompatible reference and query gene spaces**: Training on a reference with gene IDs from one annotation system (e.g., gene symbols) and predicting on a query annotated with another (e.g., Ensembl IDs) will produce `NaN` predictions for all cells. Ensure gene IDs are harmonized using `--gene-column` and `--id-type` or a custom mapping file.
- **Overwriting output files without warning**: `celltypist-so predict` silently overwrites files in `--outdir` if they share the same `--prefix`. Loss of previous probability matrices is irreversible; use distinct prefixes or a new output directory for each run.
- **Training with insufficient label diversity**: If any cell type label has fewer than ~20 cells in the reference training set, the logistic regression model will fail to converge or produce degenerate weights. Use `celltypist-so inspect` on the labeled reference to check per-label cell counts before training.

## Examples

### Train a custom model from a labeled reference H5AD file
**Args:** `--train reference.h5ad --labels celltype --model-output custom_model.pth --id-type symbol`
**Explanation:** This trains a logistic regression model using the `celltype` column in `adata.obs` as ground truth labels, exporting the serialized model to `custom_model.pth` for later prediction.

### Predict cell types using a pre-trained model on a query dataset
**Args:** `--predict query.h5ad --model pretrained_model.pth --outdir ./predictions --prefix run1 --confidence 0.7`
**Explanation:** This applies a pre-trained model to label each cell in `query.h5ad`, relabeling cells with prediction confidence below 0.7 as `"Unknown"` and writing results to `./predictions`.

### Train with batch correction across multiple samples
**Args:** `--train multi_sample_ref.h5ad --labels celltype --batch-correction --mode augment --model-output harmonized_model.pth`
**Explanation:** This harmonizes sample-level batch effects using ComBat-seq before training, which is essential when the reference contains cells from 10x Chromium, Drop-seq, and Smart-seq2 libraries.

### Inspect label distribution in a reference dataset before training
**Args:** `inspect reference.h5ad --labels celltype --min-count 20`
**Explanation:** This prints a per-label cell count summary, highlighting labels with fewer than 20 cells that may cause model convergence failures, allowing you to merge rare labels or collect more data.

### Export probability matrix for downstream ambiguity analysis
**Args:** `--predict query.h5ad --model custom_model.pth --outdir ./probs --prefix full_probs --probabilities full_probs.pkl`
**Explanation:** This writes the full cell-by-label probability matrix to a `.pkl` file, enabling soft-label interpretation, threshold tuning, and multi-label resolution in downstream scripts.