---
name: authentict
category: Sequence Authentication / Verification
description: A bioinformatics tool for verifying the authenticity and origin of biological sequences by comparing against reference databases and detecting contamination or mislabeling.
tags: [sequence-verification, authentication, quality-control, contamination-detection, bioinformatics-validation]
author: AI-generated
source_url: https://github.com/authentict/authentict
---

## Concepts

- authentict operates on FASTA, FASTQ, or tabular sequence files and outputs authentication reports indicating the likelihood that sequences match claimed sources, with confidence scores and database match statistics.
- The tool supports multiple reference databases (NCBI GenBank, SILVA, GTDB) and allows custom database input via the `--reference-db` flag, enabling domain-specific authentication for microbial, eukaryotic, or viral sequences.
- Authentication modes include `strict` (exact match required), `fuzzy` (allow minor mismatches), and `hybrid` (combine multiple databases with weighted scoring), selected via the `--mode` flag.
- Output formats include JSON (machine-readable), CSV (spreadsheet-compatible), and HTML (interactive report), controlled by the `--output-format` flag; JSON includes per-sequence confidence scores from 0.0 to 1.0.
- The tool performs k-mer based pre-screening for speed followed by local alignment refinement, and the `--kmer-size` flag (default 21) controls the k-mer length used in the initial screening phase.

## Pitfalls

- Using the wrong authentication mode for your data type causes false negatives: `strict` mode in highly polymorphic datasets (e.g., viral quasispecies) rejects legitimate variants, reducing effective sensitivity below 40%.
- Specifying an incorrect or outdated reference database produces unreliable authentication scores; relying on a pre-2015 GenBank dump for microbial sequences results in missing modern clade assignments.
- Setting `--kmer-size` too large (above 31) for short read data (under 75 bp) eliminates valid k-mers entirely, causing the tool to report all queries as unauthenticated with 0.0 confidence scores.
- Forgetting to compress output files when piping to downstream tools (e.g., `authentict query.fq --mode fuzzy | gzip > auth_report.json.gz`) results in downstream parsing errors due to line-wrapping corruption.
- Omitting the `--min-coverage` flag when analyzing metagenomic datasets allows partial matches from contamination to pass threshold filters, producing false positive authentications at rates up to 15% in low-diversity samples.

## Examples

### Authenticate a single sequence file against GenBank with default settings
**Args:** `query.fasta --mode fuzzy --output-format json`
**Explanation:** Runs fuzzy authentication against the default GenBank reference database and outputs machine-readable JSON results with per-sequence confidence scores.

### Batch authentication of multiple FASTQ files with custom database
**Args:** `samples/*.fastq --reference-db custom_silva.fasta --mode hybrid --output-format csv --threads 8`
**Explanation:** Processes all FASTQ files in parallel using 8 threads, compares against a custom SILVA database in hybrid mode, and exports results to a CSV spreadsheet.

### Generate an interactive HTML authentication report
**Args:** `metagenome.fa --mode strict --output-format html --min-coverage 0.8 --kmer-size 25`
**Explanation:** Performs strict authentication requiring 80% coverage, adjusts k-mer size to 25 for improved short read handling, and produces an HTML report for manual review.

### Export results to compressed JSON for archival storage
**Args:** `clinical_isolate.fq --mode fuzzy --reference-db gtdb --output-format json --min-score 0.85 | gzip > isolate_auth.json.gz`
**Explanation:** Authenticates clinical FASTQ data against GTDB with minimum score threshold of 0.85 and pipes compressed output directly to archive storage.

### Check sequence integrity without alignment refinement step
**Args:** `check.fasta --mode fuzzy --no-align --kmer-size 31 --output-format txt`
**Explanation:** Skips the slower local alignment refinement step, relying solely on k-mer screening for faster processing when approximate results are acceptable.