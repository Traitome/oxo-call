---
name: airr
category: bioinformatics/immunology
description: A suite of tools for Adaptive Immune Receptor Repertoire (AIRR) sequencing analysis, including filtering, merging, building databases, and statistical analysis of B-cell and T-cell receptor sequences.
tags:
  - immunology
  - repertoire-analysis
  - b-cell
  - t-cell
  - vdj
  - antibodies
  - immune-profiling
author: AI-generated
source_url: https://airr-community.github.io/airr-standards/tools/python_tools.html
---

## Concepts

The `airr` tool suite operates on tab-separated or CSV-formatted repertoire files following the AIRR Data Standard (MiAIRR). Each record represents a single adaptive immune receptor sequence with mandatory fields for sequence ID, V/D/J gene calls, junction序列, and supporting metadata.

The primary subcommands handle distinct stages of AIRR analysis: building databases for fast lookup (`airr-build`), filtering low-quality or incomplete sequences (`airr-filter`), collapsing duplicate sequences (`airr-collapse`), tracking clonal ancestry (`airr-ancestry`), and computing repertoire statistics (`airr-stats`).

Input files must follow strict field naming conventions (e.g., `sequence_id`, `v_call`, `j_call`, `junction_aa`, `junction_aaa`) as defined in the MiAIRR specification; using incorrect or non-standard column names will cause tool failure or silent errors in downstream analysis.

## Pitfalls

Using inconsistent field names between input files (e.g., `v_gene` instead of `v_call`) causes the tool to ignore those columns entirely, leading to empty output files without raising warnings—this is particularly dangerous when processing large datasets where manual verification is impractical.

Specifying duplicate or conflicting filter criteria (e.g., specifying both `--min-sequence-quality` and `--max-sequence-length` that exclude all records) produces an empty output file without user notification, which may be interpreted as "no data available" rather than a parameter error.

Failing to specify the correct output format when piping to downstream tools (defaulting to TSV when downstream Expects CSV) causes format conversion failures in pipeline integration, requiring re-processing of entire datasets.

## Examples

### Filter out low-quality sequences by quality score
**Args:** `--input test.tsv --output filtered.tsv --min-sequence-quality 30`
**Explanation:** Removes any sequence with a Phred quality score below 30, preserving only high-fidelity reads for downstream repertoire analysis.

### Collapse duplicate sequences while keeping the most abundant variant
**Args:** `--input repertoire.tsv --output collapsed.tsv --collapse-by junction_aa --method abundance`
**Explanation:** Reduces the dataset by merging sequences with identical junction amino acid sequences, retaining the variant with the highest read count to eliminate PCR bias.

### Build an indexed database for fast sequence lookup
**Args:** `--input input.tsv --output db --index sequence --db-type hash`
**Explanation:** Creates an indexed database file enabling O(1) lookup times for sequence retrieval in large-scale repertoire queries.

### Extract statistics about V/J gene usage
**Args:** `--input test.tsv --stats vj_usage --output vj_stats.tsv`
**Explanation:** Generates a summary table showing the frequency distribution of V and J gene usage across the entire repertoire, essential for understanding immune diversity.

### Merge two repertoire files while removing duplicates
**Args:** `--input file1.tsv --input file2.tsv --output merged.tsv --dedup`
**Explanation:** Combines two AIRR-format files and removes duplicate sequences based on the sequence_id field, creating a unified dataset for meta-analysis.