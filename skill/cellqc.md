---
name: cellqc
category: Bioinformatics / Quality Control
description: A command-line tool for automated quality control of single-cell RNA sequencing data. Computes per-cell and per-sample metrics, identifies low-quality cells, doublets, and empty droplets, and generates summary reports in HTML, JSON, and text formats.
tags:
  - single-cell
  - RNA-seq
  - quality-control
  - qc
  - ngs
  - droplets
author: AI-generated
source_url: https://github.com/single-cell-tools/cellqc
---

## Concepts

- **Input formats**: cellqc accepts three standard single-cell matrix formats — Market Exchange Format (MEX, directory with `matrix.mtx` + gene/cell TSV files), HDF5 (`.h5` or `.h5ad` AnnData format), and a plain three-column TSV (genes × cells × counts). The tool auto-detects the format from the file extension and structure, so specifying the correct input path is the only requirement.
- **Quality metrics computed**: The tool calculates per-cell UMI counts, detected gene counts, mitochondrial fraction (`mt-`), ribosomal fraction, and the percentage of ambient/housekeeping genes. These are used to flag cells as Low-Quality (LQC), Doublet, Empty Droplet, or PASS based on adaptive thresholds derived from the median absolute deviation (MAD) method.
- **Adaptive thresholding**: Instead of hard-coded cutoffs, cellqc derives per-metric thresholds from the distribution itself using MAD from the median (default: 3× MAD). This makes the tool robust across different tissues, sequencing depths, and library preparations. Users can override with explicit `--min-genes`, `--max-umi`, or `--max-mt` flags.
- **Output artifacts**: After execution, cellqc produces a summary HTML report (`cellqc_report.html`), a per-cell JSON file (`cellqc_cells.json`), a per-metric summary TSV (`cellqc_summary.tsv`), and optionally a filtered matrix in the same format as the input (`--out-filtered`). These can be streamed into downstream tools like `cellqc-filter` or `scanpy`.
- **Doublet and empty-droplet detection**: cellqc runs an internal empty-droplet test (comparing the knee plot inflection point to cell barcodes) and optionally executes `Scrublet` or `DoubletFinder` wrappers when `--doublet-detect` is enabled. This requires raw unfiltered counts as input alongside the filtered matrix.

## Pitfalls

- **Feeding an already-filtered matrix causes inflated LQC rates**: If you pass a matrix that was pre-filtered by another tool (e.g., Cell Ranger `--force-cells`), cellqc's adaptive MAD thresholds are computed on a biased distribution. Many legitimate cells may be reclassified as LQC or dropped, because the low-quality tail was already removed. Always use the raw, unfiltered count matrix as input and let cellqc handle the filtering.
- **Missing `--species` causes gene-level metrics to silently fail**: Mitochondrial and ribosomal gene lists are species-specific (e.g., human vs. mouse). Without `--species hs` or `--species mm`, cellqc falls back to generic regex matching, which will misclassify mitochondrial genes and produce incorrect `mt-` percentages. The flag is optional but highly recommended for human and mouse datasets; without it, the HTML report will display a warning but still generate output.
- **Running with `--threads 1` on large datasets (≥100k cells) causes very slow execution**: cellqc parallelizes per-cell metric computation across threads. Setting `--threads 1` forces single-threaded evaluation, making runtime scale linearly with cell count. For a 100k-cell dataset, this can increase runtime from minutes to hours. Use `--threads` matching the number of available cores, or omit the flag to let cellqc auto-detect.
- **Conflicting `--min-genes` and `--max-umi` cutoffs produce zero passing cells**: If the minimum gene cutoff is set above the maximum UMI cutoff, or vice versa, no cells will satisfy both conditions simultaneously, resulting in an empty filtered output file and no downstream analysis possible. Always verify that your chosen thresholds are biologically plausible for your dataset.
- **Omitting `--out-dir` writes output into the input directory**: cellqc defaults to writing all output files in the current working directory or the parent directory of the input file when `--out-dir` is not specified. This can overwrite existing files in the input directory and pollute the raw data folder with HTML and JSON artifacts.

## Examples

### Basic QC on a raw MEX matrix from Cell Ranger
**Args:** `raw_counts/ --species hs --out-dir qc_output/`
**Explanation:** Runs cellqc on a Cell Ranger MEX directory, auto-detecting the format, annotating human mitochondrial genes, and writing all reports to `qc_output/` instead of the current directory.

### QC with explicit minimum gene and UMI cutoffs
**Args:** `raw_counts/ --min-genes 200 --min-umi 500 --species mm --out-dir qc_mouse/`
**Explanation:** Overrides adaptive MAD thresholds with explicit hard cutoffs of 200 detected genes and 500 UMI counts for a mouse dataset, useful for comparing results across multiple runs with identical thresholds.

### HTML report only, skipping filtered matrix output
**Args:** `raw_counts.h5ad --species hs --format h5ad --no-filtered --out-dir qc_report/`
**Explanation:** Runs QC on an HDF5 AnnData file and produces the HTML report and JSON summaries without writing a filtered matrix, which saves disk space when only the QC verdict is needed.

### Multi-threaded QC for a large dataset with doublet detection
**Args:** `raw_counts/ --species hs --threads 16 --doublet-detect --out-dir qc_large/`
**Explanation:** Executes cellqc using 16 threads for parallel computation and runs the Scrublet-based doublet detection pass, which is recommended for datasets larger than 50,000 cells where doublet rates increase.

### Generate filtered matrix in the same MEX format as input
**Args:** `raw_counts/ --species hs --out-filtered --out-format mex --out-dir qc_filtered/`
**Explanation:** Produces a filtered count matrix in MEX format in the output directory, allowing seamless chaining into downstream pipelines that expect the original format.