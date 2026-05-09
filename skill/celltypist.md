---
name: celltypist
category: single-cell bioinformatics
description: Automated cell type annotation tool for single-cell RNA-seq data using pre-trained models with probability scores
tags: scRNA-seq, cell-type-annotation, single-cell-analysis, machine-learning, transcriptomics
author: AI-generated
source_url: https://github.com/ventolab/CellTypist
---

## Concepts

- **Input Data Format**: Celltypist accepts `.h5ad` files (AnnData format) as primary input, which contain the count matrix, cell metadata, and gene annotations. Plain text formats like CSV or TSV with genes as rows and cells as columns are also supported but require explicit header specifications.

- **Pre-trained Models**: The tool ships with multiple community-curated models trained on reference datasets covering major tissue types (immune, brain, liver, etc.). Models are loaded by name (e.g., `Immuno_All_Blood`) and contain cell type labels used for prediction. Custom models can be built using the `celltypist-build` companion binary.

- **Probability Output**: Each cell receives a probability score for every cell type in the reference model. The `--min-prob` threshold filters predictions below a specified confidence, while cells below this cutoff are labeled as "Unknown" or use the `--majority-voting` flag for consensus predictions across nearby cells.

- **Output Writing**: Predictions are written to a new `.h5ad` file with added columns in obs, or exported as CSV. The `--output` flag specifies the result path, and verbose output prints a summary table showing cell type distribution statistics.

- **Ensemble Predictions**: Multiple models can be run sequentially using the `--model` flag repeatedly, allowing comparison across different reference atlases. The tool aggregates predictions but does not automatically merge conflicting labels.

---

## Pitfalls

- **Mismatched Model Vocabulary**: Using a model trained on a different species (mouse vs. human) or tissue type produces meaningless predictions that may appear confident but represent biological mismatch, wasting downstream analysis time.

- **Insufficient Pre-processing**: Raw count matrices without normalization (CP10k, TPM, etc.) fed to models expecting normalized input create biased predictions. Many pre-trained models expect log-transformed data; raw counts often yield low probability scores across all labels.

- **Threshold Caveats**: Setting `--min-prob` too high (e.g., 0.9) may classify most cells as "Unknown" because few cell types have perfect separation. Conversely, setting it too low (e.g., 0.1) accepts low-confidence assignments that propagate errors into clustering or marker detection.

- **H5AD Version Incompatibility**: Loading `.h5ad` files written with an older AnnData library version may fail silently or produce empty predictions. Always verify the file can be read with `anndata.read_h5ad()` before passing to celltypist.

- **Memory Limits on Large Datasets**: Processing datasets exceeding model training size (e.g., >100k cells) without subsampling causes memory exhaustion on standard workstations. The tool processes in batches; memory scales with batch size, not total dataset size.

---

## Examples

### Annotate cells using a pre-trained blood model

**Args:** `--model` Immuno_All_Blood `input.h5ad` `--output` predictions.h5ad

**Explanation:** Uses the built-in immune blood reference model to label each cell, writing prediction columns (predicted labels, probabilities) into the output h5ad file for downstream analysis.

### Generate predictions with probability threshold

**Args:** `--model` Immuno_All_Blood `input.h5ad` `--min-prob` 0.5 `--output` filtered.csv

**Explanation:** Filters out low-confidence predictions below 0.5 probability, exporting only cells with strong matches to the reference model vocabulary.

### Use majority voting for ambiguous cells

**Args:** `--model` Immuno_All_Blood `input.h5ad` `--majority-voting` `--output` mv_predictions.h5ad

**Explanation:** Applies neighborhood-based consensus to reclassify cells by checking neighboring cells' predictions, useful when individual probabilities are borderline but local context is coherent.

### Run prediction with verbose summary output

**Args:** `--model` Immuno_All_Blood `input.h5ad` `--output` out.h5ad `--verbose`

**Explanation:** Prints a table showing the count and percentage breakdown of predicted cell types, helping quickly assess annotation quality.

### Specify custom gene symbols column

**Args:** `--model` Immuno_All_Blood `input.csv` `--genes` GeneSymbols `--output` annotated.csv

**Explanation:** When input is CSV format, explicitly names the column containing gene identifiers so celltypist matches them correctly to the model vocabulary.

### Combine with celltypist-build for custom model

**Args:** `celltypist-build` reference.h5ad `--out` custom_model.pkl

**Explanation:** Builds a custom classifier from your reference dataset for use in subsequent predictions, useful when pre-trained models don't match your Tissue of interest.