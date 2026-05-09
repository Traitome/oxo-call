---
name: atlas-metadata-validator
category: metadata-validation
description: Validates genomic assembly metadata files (TSV, JSON, YAML) against configurable schemas or built-in standards used in the ATLAS pipeline, reporting structural errors and field-level constraint violations.
tags: [metadata, validation, schema, genomics, atlas, json-schema, tsv, yaml]
author: AI-generated
source_url: https://github.com/galaxyproject/ATLAS
---

## Concepts

- **Schema-driven validation**: The validator reads a schema definition file (JSON Schema or YAML Schema) to check that every required field is present, values match expected types (string, integer, array), and enumerated fields contain only permitted values. Without a schema, the tool performs only basic structural checks (malformed JSON/YAML, unescaped characters, incorrect column counts in TSV).

- **Input formats and requirements**: Supported input formats are TSV (tab-separated with a header row), JSON (single object or array of objects), and YAML (single document). TSV files must contain exactly one header line; JSON files must be parseable by the standard JSON parser; YAML files must not contain tabs for indentation. The `--input-format` flag must match the actual file extension if the file has a non-standard extension.

- **Exit codes and output levels**: The tool exits with code 0 if all checks pass, code 1 if validation errors are found, and code 2 for fatal errors (unreadable file, missing schema reference). Verbose output (`-v`) includes the line number or JSON path for each error. Quiet output (`-q`) suppresses all non-error messages but still prints error summaries.

- **Report generation and error grouping**: By default, the tool prints errors to stdout grouped by severity: FATAL (file unreadable), ERROR (required field missing), WARN (optional field rejected). The `--report-file` option redirects the full report to a file while printing a one-line summary to stdout. JSON output format (`--format json`) produces a machine-readable report with fields `valid`, `errorCount`, `errors[]`, and `warnings[]`.

- **Built-in schema cache**: The tool ships with a cached set of standard schemas for common ATLAS metadata profiles (e.g., `atlas-sample`, `atlas-assembly`). These are referenced by name with `--schema` alone, without a file path. User-defined schemas take precedence over cached schemas when names overlap.

## Pitfalls

- **Mismatched input format flag**: Using `--input-format json` for a YAML file (or vice versa) causes the tool to fail with a FATAL error before any validation runs, wasting time on large files. Always verify that the flag matches the actual file format, not just the file extension.

- **Omitting required fields in TSV**: TSV validation treats any column in the schema marked as `required: true` as mandatory. If a TSV file omits the column entirely (not just the value), the tool reports one ERROR per missing column. Having the column present but empty is handled differently: the tool may emit a WARN or pass, depending on the schema's `allowEmpty` setting.

- **Invalid JSON/YAML syntax halting all validation**: A single syntax error in a JSON or YAML file (e.g., a trailing comma, mismatched quotes, or undefined anchors) aborts the entire run with exit code 2, producing no validation results for the rest of the file. Use a standalone parser or linter first for files larger than 1 MB.

- **Encoding issues with special characters**: TSV files saved in Windows-1252 or Latin-1 encoding may cause the validator to misinterpret tab characters embedded in field values, leading to spurious column-count errors. Always ensure TSV files are UTF-8 encoded.

- **Confusing WARN with ERROR**: A WARN indicates an optional constraint violation (e.g., a field value outside the recommended range) that does not fail validation. Scripts that check only the exit code may incorrectly treat a WARNING run as a failure. Always parse the report format explicitly when automating.

## Examples

### Validate a TSV metadata file against a named ATLAS schema
**Args:** `--schema atlas-sample --input-format tsv sample_metadata.tsv`
**Explanation:** This runs schema-constrained validation on a tab-separated file, checking that all required columns exist and values conform to type and enumeration constraints defined in the built-in `atlas-sample` profile.

### Validate a JSON file with verbose error output
**Args:** `-v --input-format json --schema custom_assembly_schema.json assembly_metadata.json`
**Explanation:** Verbose mode (`-v`) prints the JSON path for every error, such as `$.samples[2].collection_date`, making it easier to locate the offending record in large nested files.

### Generate a machine-readable JSON validation report
**Args:** `--format json --report-file validation_report.json --schema atlas-assembly input/metadata.yaml`
**Explanation:** The `--format json` flag produces structured output suitable for parsing by downstream scripts, while `--report-file` writes the full report to disk without cluttering stdout.

### Run quiet validation and check only the exit code
**Args:** `-q --input-format tsv --schema atlas-sample metadata.tsv`
**Explanation:** Quiet mode suppresses all output except the final one-line status ("PASS", "FAIL", or "FATAL"), allowing simple shell logic like `if atlas-metadata-validator ...; then echo "Clean"; fi` without parsing text.

### Validate a YAML file against a user-defined schema using a custom schema path
**Args:** `--input-format yaml --schema /path/to/local/schema.yaml sample_data.yaml`
**Explanation:** Providing an absolute or relative path to `--schema` loads the user-defined schema, which takes precedence over any built-in schema of the same name. This is useful for project-specific extensions or custom fields not covered by standard ATLAS profiles.