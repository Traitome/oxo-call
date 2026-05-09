---
name: bioc
category: biological-data-processing
description: A command-line tool for biological sample metadata processing, format conversion, and quality control operations in bioinformatics workflows.
tags:
  - bioinformatics
  - biological-data
  - metadata
  - sample-processing
  - format-conversion
  - quality-control
author: AI-generated
source_url: https://github.com/bioc-tool/bioc
---

## Concepts

- **Sample ID Mapping**: bioc uses a sample manifest file (TSV or CSV) to map input biological sample identifiers to standardized metadata fields. Each sample must have a unique identifier in the first column, with additional columns for sample attributes like tissue type, treatment condition, or batch number.

- **Input Format Flexibility**: The tool accepts multiple input formats including TSV, CSV, and JSON for sample metadata, and supports streaming from standard input (stdin) for pipeline integration. The format is auto-detected based on file extension, but can be explicitly specified with the `--format` flag.

- **Output Modes**: bioc generates three primary output types: a validated metadata table (default), a summary report with statistics (`--summary`), or an annotated dataset with added quality flags (`--annotate`). The output can be written to a file or streamed to stdout for downstream processing.

- **Validation Rules**: Built-in validation checks for sample consistency including duplicate detection, missing required fields, and value type enforcement (numeric fields must contain numeric values). Custom validation rules can be added via a rule definition file.

## Pitfalls

- **Duplicate Sample IDs**: Failing to ensure unique sample identifiers in the input manifest causes bioc to exit with an error, and the entire operation is aborted. Always pre-deduplicate sample IDs before running bioc if duplicates are expected or intentional, handle them with a separate flag like `--allow-duplicates`.

- **Missing Required Fields**: If a sample row lacks a required metadata field (default required fields: sample_id, tissue, condition), the tool skips that sample and logs a warning, potentially leading to silent data loss in downstream analysis. Review the warning messages carefully.

- **Inconsistent Column Headers**: Column names with leading/trailing whitespace or inconsistent casing cause field mismatches. The tool performs strict header matching by default, so ensure headers are clean and consistent, or use `--ignore-case` for case-insensitive matching.

- **Large File Memory Usage**: Loading very large manifest files (>100K samples) into memory can cause the process to be killed by the operating system. For large datasets, split the input into chunks using `--chunk-size` or process samples incrementally with `--streaming`.

## Examples

### Validate sample metadata from a TSV file
**Args:** `validate input_samples.tsv`
**Explanation:** The validate subcommand checks each row against required fields and data type constraints, reporting any violations without modifying data.

### Convert sample manifest from CSV to JSON format
**Args:** `convert --input-format csv --output-format json raw_samples.csv batch_metadata.json`
**Explanation:** Converts a comma-separated sample manifest to JSON, preserving all metadata fields while restructuring for downstream tools that consume JSON.

### Generate a summary report of sample batches
**Args:** `summary processed_samples.tsv --output summary_report.txt`
**Explanation:** Produces a text report containing batch distribution counts, sample type frequencies, and quality metrics for the processed dataset.

### Add quality flags to samples based on metadata
**Args:** `annotate samples.tsv --flag-field quality_status --rules custom_rules.yaml`
**Explanation:** Reads validation rules from the YAML file and adds a new quality_status column to each sample row based on rule matching results.

### Stream sample data through a Unix pipeline
**Args:** `filter --conditions treatment --format tsv |`
**Explanation:** Demonstrates stdout streaming capability - outputs filtered samples as TSV to be piped into another bioinformatics tool like downstream analysis scripts.

### Process multiple sample files in batch mode
**Args:** `merge --output combined_samples.tsv sample_set_1.tsv sample_set_2.tsv sample_set_3.tsv`
**Args:** `merge sample_set_1.tsv sample_set_2.tsv sample_set_3.tsv --output combined_samples.tsv`
**Explanation:** Combines multiple sample manifest files into a single output table, handling duplicate sample IDs according to the merge strategy (default: keep first).

### Extract unique tissue types from samples
**Args:** `distinct tissue --input all_samples.tsv`
**Explanation:** Extracts and lists all unique values from the tissue column, useful for quickly understanding the diversity of sample types in a large dataset without manual inspection.