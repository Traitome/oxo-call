---
name: cgat-report
category: reporting
description: Generates HTML reports from CGAT genomic analysis results, combining data from multiple CGAT tools into interactive visualizations and summary statistics.
tags: bioinformatics, genomics, reporting, visualization, HTML, QC
author: AI-generated
source_url: https://github.com/CGAT-Org/CGAT
---

## Concepts

- **Input Data Model**: cgat-report consumes output files from other CGAT tools (expression matrices, alignment stats, variant calls) in TSV/CSV format, along with a configuration YAML file defining report sections and data sources.
- **Report Structure**: The tool generates hierarchical HTML reports containing sections for sample metadata, QC metrics, expression summaries, and custom data tables, each rendered with interactive JavaScript charts.
- **Configuration-Driven**: Report content is controlled by a `.cgatreport.yml` configuration file specifying which data files to include, column mappings, visualization types (bar charts, heatmaps, scatter plots), and template selection.
- **Static Output**: The tool produces self-contained HTML files with embedded CSS and JavaScript, making reports portable and viewable in any web browser without a running server.

## Pitfalls

- **Missing Data Columns**: Referencing columns in the configuration that do not exist in input files causes silent failures where sections render as empty rather than throwing informative errors.
- **Inconsistent File Formats**: Mixing CSV and TSV input files without explicitly specifying the delimiter in the configuration leads to parsing errors or garbled data in final reports.
- **Memory Limits with Large Datasets**: Processing expression matrices with hundreds of thousands of rows without adjusting memory allocation results in crashed reports or truncated visualizations.
- **Configuration Syntax Errors**: Using incorrect YAML indentation or unsupported configuration keys silently ignores those settings, producing reports that differ unexpectedly from intended output.

## Examples

### Generate a basic report from expression data

**Args:** `--data expression_matrix.csv --config report.yml --output expression_report.html`

**Explanation:** Creates an HTML report using expression values from a CSV file, with section definitions and styling controlled by the YAML configuration file.

### Specify custom column mappings for visualization

**Args:** `--data rnaseq_results.tsv --map gene_id=Gene,fpkm=FPKM,pvalue=P_Value --output rnaseq_report.html`

**Explanation:** Maps input TSV columns to semantic names used in the report, allowing the tool to correctly identify which columns contain gene identifiers, expression values, and statistical significance.

### Include multiple data sources in one report

**Args:** `--data sample1.tsv --data sample2.tsv --config multi.yml --output combined_report.html`

**Explanation:** Aggregates multiple data files into a single comparative report, using the configuration to define how samples should be compared and which metrics to display.

### Use a specific report template

**Args:** `--template qc_dashboard --data qc_metrics.tsv --config qc_config.yml --output qc_report.html`

**Explanation:** Applies a pre-built QC dashboard template optimized for displaying quality control metrics, overriding default section layouts.

### Set verbose output for debugging

**Args:** `--data test.tsv --config test.yml --output debug.html --loglevel DEBUG`

**Explanation:** Enables debug-level logging to stderr, useful for troubleshooting configuration issues or missing data columns during report development.