---
name: cami-amber
category: Metagenomics
description: A CAMI (Critical Assessment of Metagenome Interpretation) toolkit for processing, converting, and validating metagenomic data in the Amber format. Supports filtering, statistics generation, and format validation for microbial community datasets.
tags:
  - metagenomics
  - cami
  - amber-format
  - microbial-community
  - bioinformatics
  - format-conversion
author: AI-generated
source_url: https://github.com/CAMI/challenge
---

## Concepts

- **Amber Format Structure**: CAMI Amber format uses a tab-separated structure with header lines marked by `#` and contains sample metadata, taxonomic profiles (with relative abundances and confidence scores), and contig/scaffold information organized by BINID. Understanding the column order (KEY, TAXID, RELABUNDANCE, CONFIDENCE, etc.) is critical for correct parsing.
- **Input Format Flexibility**: cami-amber accepts multiple input formats including the native Amber format, TSV, and CSV files. It automatically detects delimiters but can be forced using `--format` (amber, tsv, csv). For large datasets, streaming input via stdin is supported for memory efficiency.
- **Output Modes**: The tool operates in multiple modes: stats (generating summary statistics), filter (removing low-confidence or low-abundance entries), convert (transforming between formats), and validate (checking format compliance). Each mode produces different output and has distinct required arguments.
- **Confidence Thresholds**: Many operations rely on confidence scores (0-1 range) stored in the Amber format. Filtering by `--min-confidence` removes entries below the threshold, which is essential for downstream analysis pipelines that require high-qualityTaxonomic assignments.

## Pitfalls

- **Missing Header Rows**: Omitting the required `#` header row in Amber format files causes silent failures where no error is reported but output is empty. The tool reads headers strictly and without them treats all lines as data, resulting in corrupted output that appears valid but contains misaligned columns.
- **Incorrect Column Order**: Supplying Amber format files with columns in the wrong order (e.g., placing TAXID before KEY) produces no warning but generates invalid output. Downstream tools consuming this output will fail with obscure errors, wasting Debugging time.
- **Floating-Point Precision in Abundances**: Using raw floating-point numbers without proper normalization can lead to relative abundances summing to values slightly off from 1.0 (e.g., 0.9999 or 1.0001). This triggers validation failures in strict mode; always use `--normalize` when converting between formats.
- **Memory Limits with Large Datasets**: Processing files larger than available RAM without the `--chunk-size` flag causes the tool to crash with memory errors. Always use streaming mode (`--stream`) for files exceeding 1GB or when working on memory-constrained systems.

## Examples

### Generate summary statistics from an Amber file
**Args:** `--mode stats --input sample.amber.tsv`
**Explanation:** Summarizes the taxonomic distribution, total bins, confidence distribution, and abundance ranges in the input file, useful for quick quality assessment before downstream analysis.

### Filter low-confidence taxonomic assignments
**Args:** `--mode filter --input raw.amber.tsv --output high_conf.amber.tsv --min-confidence 0.5`
**Explanation:** Removes all entries with confidence scores below 0.5, retaining only high-quality taxonomic calls for stricter metagenomic profiling pipelines.

### Convert TSV to native Amber format
**Args:** `--input taxonomic.tsv --output formatted.amber.tsv --format amber --normalize`
**Explanation:** Transforms a tab-separated taxonomic profile into proper Amber format with correct headers, normalized abundances summing to 1.0, and appropriate column ordering.

### Validate an Amber file for CAMI compliance
**Args:** `--mode validate --input submission.amber.tsv --strict`
**Explanation:** Checks the input file for format compliance, column order, header presence, and data type validity, reporting all violations for correcting submissions before the CAMI challenge.

### Extract top N abundant bins from a profile
**Args:** `--mode filter --input community.amber.tsv --output top_bins.amber.tsv --top-n 100 --sort-by abundance`
**Explanation:** Filters to retain only the 100 most abundant bins, sorted by relative abundance in descending order, useful for creating focused profiles or reducing dataset complexity.

### Stream process large files without loading into memory
**Args:** `--mode stats --input huge_dataset.amber.tsv --stream --chunk-size 50000`
**Explanation:** Processes the file in 50,000-line chunks to avoid memory exhaustion, suitable for metagenomic datasets with millions of contigs or taxonomic entries.