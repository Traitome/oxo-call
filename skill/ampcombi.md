---
name: ampcombi
category: protein sequence analysis / antimicrobial peptide identification
description: A command-line tool for identifying, annotating, and analyzing antimicrobial peptides (AMPs) in protein sequence data. Ampcombi performs functional annotation, activity prediction, and feature extraction for AMP sequences sourced from FASTA input.
tags:
  - antimicrobial peptides
  - protein annotation
  - functional prediction
  - sequence analysis
  - microbiology
  - peptide mining
author: AI-generated
source_url: https://github.com/angelicaelena/ampcombi
---

## Concepts

- **Input Format**: Ampcombi accepts protein sequences in standard FASTA format. Sequences should represent full-length or partial peptide sequences. Multi-sequence FASTA files are processed in batch mode, with each sequence assigned a unique identifier internally.
- **AMP Identification Engine**: The tool uses curated databases of known AMP families and sequence similarity heuristics to classify input sequences. Hit quality is scored using e-value thresholds and bit-score cutoffs that can be configured via command-line flags.
- **Output Modes**: Ampcombi produces structured output in multiple formats including tabular (TSV), JSON, and HTML report formats. The JSON output preserves per-sequence scoring metadata, enabling downstream statistical analysis or integration with pipeline tools.
- **Companion Database Indexing**: The companion subcommand `ampcombi-build` constructs a local BLAST database from a custom AMP reference file. This index is used for accelerated sequence-similarity searches when the `--database` flag points to a user-defined resource.
- **Filtering and Thresholds**: Default classification thresholds may miss divergent AMP sequences. Adjusting `--identity-threshold`, `--coverage-min`, and `--evalue-max` flags allows users to trade off sensitivity versus specificity based on their biological question.

## Pitfalls

- **Incorrect FASTA Formatting**: Sequences with lowercase letters, whitespace interruptions, or missing header lines (starting with `>`) cause silent failures where the tool reports zero hits. This wastes compute time and masks the root cause unless `--verbose` logging is enabled.
- **Mismatched Database Version**: Using a stale or incompatible database built with an older `ampcombi-build` version produces unreliable alignments and artificially low scores, leading to false-negative classifications for legitimate AMPs.
- **Overly Stringent Thresholds**: Setting `--evalue-max` too low (e.g., `1e-20`) or `--identity-threshold` above 95% excludes borderline AMP candidates that may be biologically relevant, resulting in incomplete AMP catalogs in downstream analysis.
- **Memory Overflow with Large Inputs**: Processing multi-gigabyte FASTA files without the `--chunk-size` flag causes memory exhaustion on standard compute nodes. The tool may crash or become unresponsive, terminating analysis mid-run.
- **Ignoring Output Format Assumptions**: Downstream scripts expecting TSV format fail silently when the tool defaults to JSON due to missing `--output-format` specification, causing data parsing errors in automated workflows.

## Examples

### Identify AMPs in a single protein sequence FASTA file
**Args:** `analyze --input sequences.fasta --output amp_results.tsv --format tsv`
**Explanation:** This runs the standard identification pipeline on a FASTA file, writing classified AMPs with scores to a human-readable TSV file.

### Generate a JSON report for programmatic pipeline integration
**Args:** `analyze --input proteins.fasta --output amp_report.json --format json --evalue-max 1e-5`
**Explanation:** Produces machine-parseable JSON output with per-sequence scoring fields while setting a moderate e-value cutoff to balance sensitivity.

### Use a custom BLAST database for accelerated searching
**Args:** `analyze --input query_peptides.fasta --database custom_amps.db --output hits.tsv --format tsv`
**Explanation:** Points the analysis to a user-built reference database generated with `ampcombi-build`, enabling faster searches against specialized AMP collections.

### Run in verbose mode to diagnose zero-hit failures
**Args:** `analyze --input test.fasta --output out.json --verbose --log-level DEBUG`
**Explanation:** Enables detailed logging that reveals why sequences are failing validation, such as malformed headers or sequence masking issues.

### Process sequences in memory-efficient chunks for large files
**Args:** `analyze --input large_dataset.fasta --output results.tsv --chunk-size 500 --format tsv`
**Explanation:** Splits large input into batches of 500 sequences to avoid memory exhaustion while maintaining throughput on standard compute nodes.

### Build a custom AMP reference database from a FASTA file
**Args:** `build --reference amp_refs.fasta --database my_amps.db --format blast`
**Explanation:** Constructs a BLAST-formatted database index from a custom reference set of confirmed AMP sequences for use in subsequent `analyze` runs.

### Filter hits by minimum sequence coverage and identity
**Args:** `analyze --input peptides.fasta --output filtered.tsv --identity-threshold 80 --coverage-min 0.9 --format tsv`
**Explanation:** Applies strict filtering criteria requiring at least 80% sequence identity and 90% coverage to reduce false positives in conserved but non-AMPs homologs.