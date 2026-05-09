---
name: atlas-data-import
category: Data Integration
description: Import external experimental data files into Atlas databases for downstream analysis and integration with existing datasets.
tags: [data-import, csv, tsv, fasta, batch-processing, data-ingestion]
author: AI-generated
source_url: https://bitbucket.org/barylib/atlas/src/master/README.md
---

## Concepts

- **Data type inference**: atlas-data-import automatically detects column data types (numeric, text, categorical, genomic coordinates) based on content sampling, reducing manual schema definition overhead.

- **Input format support**: The tool accepts tab-delimited (TSV), comma-delimited (CSV), and FASTA sequence files as primary import formats. TSV is recommended for structured experimental data to preserve precision in numeric fields.

- **Batch mode operations**: Large datasets can be processed via batch import using glob patterns (e.g., `data_*.tsv`) or manifest files listing individual data files, enabling reproducible pipeline automation.

- **Data validation and error reporting**: Import validation occurs in two passes—syntax validation before database insertion and semantic validation afterward—generating detailed error logs with row and column coordinates for failed records.

- **Atlas database schema alignment**: Imported data is mapped to existing Atlas database schemas using column header matching with fuzzy name resolution, supporting both exact and approximate header-to-field alignments.

## Pitfalls

- **Mismatched column headers**: Import fails silently when column headers in the source file do not align with any Atlas schema field, resulting in data loss without warning unless `--verbose` logging is enabled.

- **Trailing whitespace in numeric fields**: Whitespace characters appended to numeric values cause type coercion failures during import. Using `--trim-fields` prevents these silent rejections in the validation phase.

- **Inconsistent row delimiters**: Files with mixed line endings (CRLF/LF) cause row boundary misidentification, producing truncated records and shifted column alignments in downstream data.

- **Oversized batch imports without transaction batching**: Importing large files without `--batch-size` segmentation exhausts available memory, leading to out-of-memory failures on constrained compute nodes.

- **Encoding mismatches for non-ASCII characters**: Source files encoded in Latin-1 or Windows-1252 fail UTF-8 validation, causing import abortion when metadata contains accented species names or chemical identifiers.

## Examples

### Import a TSV file with default settings
**Args:** `--input experiment_data.tsv --database experiments`
**Explanation:** This imports a TSV file into the experiments database using automatic type inference and default validation thresholds.

### Import CSV with numeric trimming enabled
**Args:** `--input sample_readings.csv --format csv --trim-fields --database readings`
**Explanation:** This handles CSV files with whitespace-padded numeric values by stripping whitespace before type validation.

### Batch import multiple files using glob pattern
**Args:** `--input "cohort_*/expression_data.tsv" --batch-manifest --database rna_seq`
**Explanation:** This recursively imports all expression_data.tsv files from cohort directories matching the glob pattern into the rna_seq database.

### Import with verbose logging and error output file
**Args:** `--input clinical_data.tsv --database clinical --verbose --log import_errors.log`
**Explanation:** This enables detailed logging including skipped rows and column alignment reports, writing errors to a separate file for troubleshooting.

### Import with custom batch size for memory-constrained environments
**Args:** `--input large_dataset.tsv --database genomics --batch-size 5000`
**Explanation:** This limits in-memory row buffering to 5000 records, preventing out-of-memory crashes on compute nodes with limited RAM.