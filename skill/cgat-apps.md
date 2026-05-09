---
name: cgat-apps
category: bioinformatics/genomics
description: A modular Python toolkit for high-throughput genomic data analysis, including RNA-seq, ChIP-seq, and variant calling workflows.
tags: [rnaseq, chipseq, variant-calling, genomics, python, pipeline]
author: AI-Generated
source_url: https://github.com/ClinicalGenomicsUppsala/cgat
---

## Concepts

- **Modular architecture**: cgat-apps organizes functionality into distinct tools (e.g., `cgat-apps-expression`, `cgat-apps-variant`), each callable via subcommand dispatch through a unified CLI entry point.
- **Standard input/output streams**: Most tools read from STDIN or specified files (BED, GTF, VCF, CSV) and output to STDOUT or named files, enabling Unix-style piping between operations.
- **Configuration via parameter files**: Analysis parameters are typically defined in INI-style configuration files rather than hard-coded defaults, allowing reproducible workflow control.
- **Python library dependency**: Tools depend on CGAT-core library functions; the correct Python environment must be active, or tools may fail with import errors at runtime.

## Pitfalls

- **Missing Python environment activation**: Running `cgat-apps` without the correct conda/venv environment causes `ModuleNotFoundError` exceptions because required CGAT dependencies are not on the Python path.
- **Malformed configuration files**: Using tabs instead of spaces in INI config sections causes silent parsing failures, producing empty outputs or zero-count results with no error message.
- **Unsorted input BED/GTF files**: Some subcommands require sorted input (e.g., for overlap operations); unsorted inputs produce incorrect intersection counts without warning.
- **Incompatible chromosome naming conventions**: Mixing `chr1` and `1` notation between input files causes join failures where reads appear to map to no features.

## Examples

### Compute gene-level expression counts from BAM alignment
**Args:** `expression --bam-files sample1.bam --gtf-file annotations.gtf --formatcsv`
**Explanation:** Calculates expression counts per gene by overlapping reads in the BAM file with transcript features defined in the GTF annotation file, outputting results as CSV.

### Call genomic variants from tumor-normal BAM pair
**Args:** `call-variants --tumor-bam tumor.bam --normal-bam normal.bam --reference hg38.fa --output-vcf tumor.vcf`
**Explanation:** Identifies somatic mutations by comparing aligned reads from tumor and normal samples against the specified reference genome, writing variant calls to VCF format.

### Perform interval-based overlap analysis on BED file
**Args:** `intersect --bed-query intervals.bed --bed-database peaks.bed --wa -u`
**Explanation:** Finds interval entries from the database that overlap query intervals and reports each overlap instance uniquely with the query interval identifier preserved in the output.

### Aggregate counts across replicate samples
**Args:** `aggregate-counts --input counts1.csv --input counts2.csv --input counts3.csv --group replicate --output summary.csv`
**Explanation:** Combines multiple count files by grouping identifiers, summing values within each group to produce a consolidated summary table.

### Annotate variants with functional impact predictions
**Args:** `annotate-variants --vcf-input variants.vcf --database vep_cache --format vcf --output annotated_variants.vcf`
**Explanation:** Adds functional consequence annotations (missense, nonsense, splice-site) to each variant record using a cached variant effect predictor database, preserving original VCF structure in output.

### Filter and sort genomic intervals for downstream analysis
**Args:** `filter-bed --input raw_peaks.bed --sort --chrom --start --end --output filtered_sorted.bed`
**Explanation:** Removes low-confidence interval records and sorts the remaining entries by chromosome and coordinate, producing a properly ordered BED file suitable for overlap operations.