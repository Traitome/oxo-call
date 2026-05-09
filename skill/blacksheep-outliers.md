---
name: blacksheep-outliers
category: statistical-analysis
description: A bioinformatics tool for identifying statistical outliers in genomic datasets, particularly useful for detecting anomalous expression values, variant frequencies, or quantitative trait measurements.
tags: [outlier-detection, genomics, expression-analysis, variant-analysis, quality-control, statistics]
author: AI-generated
source_url: https://github.com/oxo-call/blacksheep-outliers
---

## Concepts

- **Input format**: Accepts tab-delimited files with sample IDs in the first column and numeric values (expression levels, allele frequencies, etc.) in subsequent columns. Supports both wide and long data layouts with the `--long-format` flag.
- **Output format**: Produces a report file listing outlier samples with their original values, Z-scores or modified Z-scores (using median absolute deviation), and the statistical test used. By default writes to stdout; use `--outfile` to save.
- **Detection methods**: Implements multiple algorithms including Z-score (mean-based), modified Z-score (MAD-based), IQR-based filtering, and Grubbs' test. Use `--method` to specify. MAD-based is robust to outliers themselves.
- **Companion binary**: `blacksheep-outliers-build` creates index files for large datasets to speed up repeated analyses. Run on your data file first for datasets >100K rows.

## Pitfalls

- **Forgetting `--header`**: If your input file has column headers, omitting `--header` will treat the first data row as headers, causing errors or silently producing wrong results. Always verify with `--preview` first.
- **Using mean-based Z-scores on skewed data**: The standard Z-score method (`--method zscore`) is not robust to heavily skewed distributions. This leads to false positives. Use `--method mad` for skewed expression data.
- **Ignoring the `--threshold` value**: The default threshold may not fit your data distribution. A threshold that's too low flags legitimate natural variation as outliers; too high misses real problems. Always visualize distributions with `--plot` before final analysis.
- **Not accounting for batch effects**: Running outlier detection across combined batches without `--batch` annotation artificially inflates outlier detection. Include batch metadata to avoid false calls.

## Examples

### Detect outliers in gene expression matrix using MAD-based method

**Args:** `--infile expression.tsv --method mad --threshold 3.5`

**Explanation:** Uses median absolute deviation (MAD) for robust outlier detection at 3.5 MAD units from the median, suitable for skewed expression data.

### Save outlier report to specific file

**Args:** `--infile variants.tsv --method iqr --outfile outliers_report.txt`

**Explanation:** Uses interquartile range (IQR) method and writes results to a dedicated output file instead of stdout.

### Preview data before running detection

**Args:** `--infile expression.tsv --header --preview`

**Explanation:** Shows first five rows and column types to verify file format and header presence before full analysis.

### Run on long-format data with sample annotations

**Args:** `--infile long_expression.tsv --long-format --method grubbs --sample-col sample_id --value-col expression`

**Explanation:** Processes data in long format where each row is one measurement, specifying which columns contain sample IDs and values.

### Create index for large dataset analysis

**Args:** `blacksheep-outliers-build --infile large_data.tsv --index-file large_data.idx`

**Explanation:** Builds an index file to accelerate subsequent outlier detection runs on the same large dataset.