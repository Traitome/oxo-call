---
name: biowdl-input-converter
category: Data Format Conversion
description: Converts sample sheets and input manifests between CSV, TSV, JSON, and YAML formats for BioWDL bioinformatics pipelines. Handles sample metadata, read grouping, and library pooling information.
tags:
- biowdl
- input-converter
- sample-sheet
- format-conversion
- csv
- tsv
- json
- yaml
- bioinformatics
- pipelines
author: AI-generated
source_url: https://github.com/biowdl/biowdl-input-converter
---

## Concepts

- **Multi-format I/O**: The tool reads from and writes to CSV, TSV, JSON, and YAML formats. Input and output formats are inferred from file extensions or specified explicitly using `--input-format` and `--output-format` flags.
- **Sample sheet schema**: Input files must contain required columns like `sample_id`, `read1`, and `read2` (optional for paired-end). Additional columns such as `library_id`, `platform`, and `lane` are preserved through conversion.
- **Validation layer**: The tool validates sample IDs for uniqueness, checks that referenced read files exist, and ensures required fields are present before producing output. Invalid inputs cause the tool to exit with an error.
- **Streaming conversion**: For large sample sheets, the tool processes rows one at a time without loading the entire file into memory, enabling handling of files with tens of thousands of samples.

## Pitfalls

- **Missing required columns**: Omitting `sample_id` or `read1` in the input sheet causes immediate failure. The tool cannot infer sample identifiers from file paths alone.
- **Duplicate sample IDs**: Duplicate sample identifiers in the input cause validation to fail, preventing any output from being generated. Each sample must have a unique identifier.
- **Mismatched output format**: Writing JSON output to a file named `.csv` still produces JSON content. The tool respects the `--output-format` flag over the filename extension.
- **Relative paths for read files**: Using relative paths for `read1`/`read2` columns works only if the working directory matches the expected location. Absolute paths are recommended for reproducible pipelines.
- **Unsupported characters in sample names**: Special characters like spaces, commas, or colons in `sample_id` or `library_id` fields can break downstream pipeline parsing. Use alphanumeric characters and underscores only.

## Examples

### Convert a CSV sample sheet to JSON format
**Args:** samples.csv --output-format json
**Explanation:** Reads the CSV sample sheet and converts it to JSON format, preserving all sample metadata columns. Use this when a downstream pipeline requires JSON input.

### Convert TSV to YAML with explicit format flags
**Args:** --input-format tsv --output-format yaml samples.tsv output.yaml
**Explanation:** Explicitly specifies both input and output formats to ensure correct parsing and generation, bypassing automatic format inference from file extensions.

### Convert a sample sheet and validate without writing
**Args:** samples.csv --output-format json --validate-only
**Explanation:** Runs full validation on the input sample sheet without producing output files. Useful for checking data integrity before committing to conversion.

### Convert to JSON and pretty-print with indentation
**Args:** samples.csv --output-format json --pretty
**Explanation:** Produces JSON output with formatted indentation and line breaks, making the output easier to manually inspect or version-control.

### Convert a multi-library sample sheet to YAML
**Args:** samples.csv --output-format yaml --output samples_meta.yaml
**Explanation:** Converts a sample sheet containing multiple libraries per sample to YAML format, preserving library-level metadata in nested structures.