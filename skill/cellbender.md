---
name: cellbender
category: bioinformatics/single-cell-rna-seq
description: A probabilistic tool for removing background RNA signals from single-cell RNA-sequencing count matrices, identifying and subtracting ambient RNA contributions from empty droplets.
tags: single-cell, scRNA-seq, ambient RNA, denoising, 10x-Genomics, matrix-cleaning
author: AI-generated
source_url: https://cellbender.readthedocs.io/
---

## Concepts

- CellBender models the count distribution in each droplet as a mixture of genuine cell transcripts and ambient RNA background, using a zero-inflated negative binomial model to separate true biological signal from contamination.
- Input files must be in `.h5` (10x Genomics HDF5 format) or `.mtx` (Matrix Market format) with corresponding barcodes and genes files; output is a corrected count matrix in the same format as input.
- The `--expected-cells` parameter is critical: it tells CellBender approximately how many real cells are in the dataset, which calibrates the model's expectation of the cell-to-background ratio.
- CellBender supports GPU acceleration via `--cuda` flag, which dramatically speeds up training on large datasets (100k+ cells); without it, processing can take hours on standard hardware.
- The tool produces three output files: a cleaned count matrix, a CSV of removed background counts per gene, and a PDF report visualizing correction quality.

## Pitfalls

- Setting `--expected-cells` too low causes CellBender to over-aggressively filter counts, potentially removing real low-abundance transcripts; setting it too high leaves residual background noise in the output.
- Running CellBender on already-filtered data (e.g., after removing empty droplets via Cell Ranger) removes too few background counts since the model assumes unfiltered input with empty droplets present.
- Insufficient memory for large datasets (using `--low-memory` on data >50k cells) leads to segmentation faults or incomplete output files.
- Not specifying `--fpr` (false positive rate) when the dataset has very low library size may retain contaminated reads; the default 0.01 works for typical 10x data but may need adjustment for very sparse datasets.
- Using outdated input formats (e.g., older Cell Ranger HDF5 versions) causes parsing errors; ensure input files are in current 10x format.

## Examples

### Remove background RNA from a 10x Genomics h5 file
**Args:** remove-background --input possum_raw.h5 --output cleaned_counts.h5 --expected-cells 8000
**Explanation:** This runs the core CellBender function on a raw h5 file, specifying approximately 8000 real cells to calibrate the background subtraction model.

### Run with GPU acceleration enabled
**Args:** remove-background --input possum_raw.h5 --output cleaned_counts.h5 --expected-cells 5000 --cuda
**Explanation:** Enables CUDA GPU acceleration for faster model training, essential when processing large datasets (>10k cells) to reduce runtime from hours to minutes.

### Adjust false positive rate for sparse datasets
**Args:** remove-background --input possum_raw.h5 --output cleaned_counts.h5 --expected-cells 3000 --fpr 0.05
**Explanation:** Increases the false positive rate to 5% (from default 0.01) for very sparse datasets where many genuine transcripts may appear as low-count background.

### Output results in Matrix Market format
**Args:** remove-background --input possum_raw.h5 --output-dir ./cellbender_output --expected-cells 6000 --output-format mtx
**Explanation:** Writes output files as Matrix Market format (.mtx, .barcodes, .genes) instead of h5, useful for downstream tools that don't support HDF5.

### Generate PDF report for quality assessment
**Args:** remove-background --input possum_raw.h5 --output cleaned_counts.h5 --expected-cells 7000 --generate-report
**Explanation:** Produces a PDF report showing gene-wise background estimates, count distributions, and correction quality metrics for validating the cleaning results.

### Use low memory mode for systems with limited RAM
**Args:** remove-background --input possum_raw.h5 --output cleaned_counts.h5 --expected-cells 4000 --low-memory
**Explanation:** Enables memory-optimized processing at the cost of longer runtime, necessary when running on systems