---
name: biopet-validatevcf
category: variant-calling
description: Validates VCF (Variant Call Format) files for format compliance, structural correctness, and adherence to the VCF specification. Checks header records, sample definitions, and variant entries for common errors.
tags: [vcf, validation, bioinformatics, variants, ngs]
author: AI-generated
source_url: https://github.com/biopet/biopet
---

## Concepts

- **VCF Structure Requirements**: VCF files must contain a header section (lines starting with `##`) defining metadata and column headers (`#CHROM` line), followed by data records with fixed-field columns (CHROM, POS, ID, REF, ALT, QUAL, FILTER, INFO, FORMAT, sample columns). The tool validates that all required fields are present and properly formatted.
- **Input Format Support**: biopet-validatevcf accepts both bgzip-compressed and plain text VCF files. The tool auto-detects the VCF specification version (v4.0, v4.1, v4.2, v4.3) from the header and validates accordingly, supporting different feature sets per version.
- **Validation Scope**: The tool performs structural validation including header syntax, sample name uniqueness, INFO and FORMAT field definitions, allele consistency (REF must match reference at reported position), coordinate validity (POS > 0), and proper genotype encoding.
- **Multi-Sample Handling**: When processing multi-sample VCFs, the tool verifies that the number of sample columns matches the sample names declared in the header and that all genotype entries conform to the declared FORMAT fields.

## Pitfalls

- **Misleading Header Definitions**: Omitting or misspelling INFO or FORMAT field definitions in the header can cause false error reports or allow invalid entries to pass validation. Always declare all custom fields used in your VCF.
- **Reference Mismatch Errors**: Providing a REF allele that does not match the reference sequence at the specified genomic position will be flagged as an error, but this check requires an indexed reference FASTA (if supported) — without it, the tool may miss these errors.
- **Inconsistent Sample Columns**: Adding or removing sample columns without updating the header's sample list causes validation to fail with misleading column count errors. Keep header and data in sync.
- **Compression Format Issues**: Using gzip (.gz) instead of bgzip (.bgz) for block-compressed VCFs may cause read errors. Ensure bgzip is used for indexed compressed files to enable proper random access validation.

## Examples

### Validate a basic VCF file for format compliance
**Args:**