---
name: commet
category: Bioinformatics/Comparative Genomics
description: A bioinformatics tool for comparative analysis of genomic sequences, typically used for identifying conserved regions, computing sequence similarity metrics, and generating comparative reports across multiple genomes or sequence collections.
tags:
  - comparative genomics
  - sequence analysis
  - alignment
  - conservation
  - genomics
author: AI-generated
source_url: https://github.com/example/commet
---

## Concepts

- **Input Format**: Accepts FASTA, FASTQ, or multi-FASTA sequence files as primary input; supports compressed (.gz) and uncompressed formats for both single and batch processing.
- **Output Modes**: Produces comparative reports in tabular format (CSV/TSV), JSON for programmatic access, and graphical summaries; results include similarity scores, alignment coordinates, and conservation statistics.
- **Core Algorithms**: Uses k-mer based filtering for speed, followed by pairwise alignment (Smith-Waterman or Needleman-Wunsch) for detailed comparison; supports progressive alignment for multiple sequences.
- **Indexing Model**: Employs a suffix array or FM-index for fast substring queries when comparing against a reference database; indices can be built with the companion `commet-build` utility for repeated queries.

## Pitfalls

- **Mismatched Sequence Encoding**: Providing DNA sequences in RNA format (with 'U' instead of 'T') or mixing uppercase/lowercase inconsistently causes silent failures in similarity detection; always standardize to DNA 'T' or uppercase before running.
- **Memory Exhaustion with Large Inputs**: Attempting to compare whole-genome sequences without chunking or indexing creates excessive memory overhead that crashes the process; use `--chunk-size` to limit memory or pre-build an index for large references.
- **Incompatible Scoring Matrices**: Using default scoring parameters (e.g., match/mismatch scores) designed for protein sequences on DNA input produces biologically nonsensical similarity scores; specify `--matrix dna` or equivalent for DNA-specific scoring.
- **Output Overwrite Without Backup**: Redirecting results to an existing file without specifying append mode or backup causes data loss; use `--output` with a new filename or check file existence before running.

## Examples

### Compare two DNA sequences and output similarity score
**Args:** `-i reference.fasta -q query.fasta --score-only`
**Explanation:** This runs a basic sequence comparison returning only the similarity score, useful for quick filtering before detailed analysis.

### Generate alignments with configurable gap penalties
**Args:** `-i ref.fasta -q query.fasta --gap-open 10 --gap-extend 0.5 --output alignment.tsv`
**Explanation:** Setting custom gap penalties helps tailor alignment sensitivity for sequences with expected insertion/deletion events, such as variable repeat regions.

### Build an index for fast repeated queries against a reference database
**Args:** `build -i reference_db.fasta --index-type fm --index-dir ./idx/`
**Explanation:** Pre-building an index with the companion binary dramatically speeds up multiple query comparisons against the same large reference set.

### Process multiple queries in batch using parallel threads
**Args:** `-i ref.fasta -q query_batch.fasta --num-threads 4 --batch-mode`
**Explanation:** Enabling multi-threaded batch processing reduces wall-clock time significantly when comparing many sequences against a single reference, with `--batch-mode` optimizing memory for bulk operations.

### Export results in JSON format for programmatic parsing
**Args:** `-i ref.fasta -q query.fasta --format json --output results.json`
**Explanation:** JSON output is preferred when integrating commet results into downstream pipelines or building visualization dashboards that require structured data.

### Mask low-complexity regions before comparison
**Args:** `-i ref.fasta -q query.fasta --mask-low-complexity --output filtered_results.tsv`
**Explanation:** Masking low-complexity (e.g., repetitive) regions prevents false-positive similarities from simple repeats and focuses analysis on biologically meaningful conserved domains.