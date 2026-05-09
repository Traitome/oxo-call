---
name: biopet-validateannotation
category: Annotation Validation
description: Validates genomic annotation files (GFF3, GTF formats) to ensure they conform to specification standards, checking for malformed features, invalid attributes, and structural issues.
tags: [annotation, validation, genomics, gff3, gtf, biopet, bioinformatics]
author: AI-generated
source_url: https://github.com/biopet/biopet
---

## Concepts

- **Input formats**: Supports GFF3 and GTF annotation formats; the tool auto-detects format based on file extension (.gff, .gff3, .gtf) but explicit specification is recommended for ambiguous files.
- **Validation checks**: Validates feature coordinates (start ≤ end), strand consistency (+, -, .), attribute syntax (ID/Parent relationships), and feature type validity (gene, mRNA, exon, CDS, etc.).
- **Exit codes**: Returns 0 for valid files, non-zero exit codes indicate validation failures; can be used in pipeline error handling.
- **Output modes**: Can output structured reports in text or JSON format for downstream processing or integration with workflow managers.

## Pitfalls

- **Omitting --force flag**: Running on large annotation files without --force will abort if any error is found, potentially halting automated pipelines that expect partial results.
- **Ignoring warning output**: Warnings indicate non-fatal issues (deprecated features, unusual attributes) that may cause downstream tools to fail or produce unexpected results.
- **Assuming auto-detection works**: When input files lack proper extensions or have mixed formats, auto-detection may incorrectly identify the format, leading to validation failures or false positives.
- **Not specifying output file for large validations**: Outputting to stdout for large annotation files creates excessive log noise and may be truncated in pipeline logs.

## Examples

### Validate a GFF3 annotation file with standard settings
**Args:** --input annotations.gff3
**Explanation:** Runs standard validation on a GFF3 file, checking feature structure and attribute syntax without suppressing warnings.

### Validate a GTF file and output JSON report
**Args:** --input transcripts.gtf --json --output validation_report.json
**Explanation:** Produces a machine-parseable JSON report of all validation errors and warnings for integration into automated pipelines.

### Force validation despite errors for debugging
**Args:** --input annotations.gff3 --force
**Explanation:** Continues validation even when errors are found, allowing complete error reporting rather than failing on the first issue.

### Suppress warnings and only show critical errors
**Args:** --input annotations.gff3 --warn-as-error false
**Explanation:** Filters output to only show critical validation failures, useful when warnings are expected and clutter output.

### Validate and generate a summary report
**Args:** --input annotations.gff3 --summary
**Explanation:** Outputs a concise summary of validation results including feature counts and error categories, useful for quick quality checks.