---
name: clsify
category: Taxonomic Classification
description: A k-mer based sequence classification tool for assigning taxonomic labels to FASTA/Q input sequences by comparing against a pre-built reference database.
tags: [metagenomics, taxonomic-classification, sequence-analysis, k-mer, bioinformatics]
author: AI-generated
source_url: https://github.com/example/clsify
---

## Concepts

- **Database Requirement**: clsify requires a pre-built database created with the companion `clsify-build` command. The database consists of indexed k-mer manifests and must be specified via the `--db` flag during classification.
- **Input Formats**: Accepts FASTA (`.fa`, `.fasta`) and FASTQ (`.fq`, `.fastq`) files as input. Single-end and paired-end reads are supported; for paired-end data, provide both files consecutively as arguments.
- **K-mer Classification Algorithm**: clsify uses a sliding k-mer (default k=31) strategy to assign the lowest common ancestor (LCA) based on matching k-mers in the reference database, producing taxonomic paths (e.g., Domain > Phylum > Class > Order > Family > Genus > Species).
- **Output Modes**: Classification results can be output in plain text (default), JSON (`--outfmt json`), or Kraken-style report (`--outfmt report`). The report format includes percentage abundance per taxon.
- **Sensitivity vs Speed**: The `--min-kmer-freq` flag controls the minimum number of matching k-mers required before attempting classification; lower values increase sensitivity but slow runtime.

## Pitfalls

- **Running without a database**: Executing `clsify` without using the `--db` flag or pointing to a non-existent database path results in an immediate segmentation fault and program crash. Always verify the database path exists before running.
- **Mismatched k-mer size**: If the database was built with a non-default k-mer size (via `clsify-build -k`), classification will fail silently with zero assignments if the same k is not specified with `--k` during `clsify` execution.
- **Memory exhaustion with large databases**: Using the default `--threads` on systems with limited RAM can cause the OOM killer to terminate clsify. For large databases (>100GB), reduce thread count via `--threads 1` or `--threads 2`.
- **Input file permissions**: If the output file path is not writable (due to directory permissions or file lock), clsify terminates with a generic "Error writing output" message without specifying the permission issue.

## Examples

### Basic classification of a single FASTA file
**Args:** `--db /refs/microbial.db input.fa`
**Explanation:** Runs classification of all sequences in `input.fa` against the database located at `/refs/microbial.db` using default k=31 and outputting results to stdout.

### Paired-end FASTQ classification with JSON output
**Args:** `--db /refs/microbial.db --outfmt json --paired reads_R1.fq reads_R2.fq`
**Explanation:** Classifies paired-end reads from two FASTQ files and outputs results in JSON format, preserving pair association in the output.

### Specify non-default k-mer size for custom database
**Args:** `--db /refs/custom.db --k 25 input.fasta`
**Explanation:** Uses k=25 k-mers to match the custom database built with the same k-mer length, ensuring classification works correctly.

### Limit threads to prevent memory overflow
**Args:** `--db /refs/large.db --threads 1 input.fa`
**Explanation:** Restricts clsify to a single thread to reduce memory footprint when running on systems with limited RAM or when handling very large databases.

### Generate taxonomic abundance report
**Args:** `--db /refs/microbial.db --outfmt report --output abundance.txt input.fq`
**Explanation:** Produces a tabular report showing percentage abundance of each taxon, which is useful for metagenomic profiling analyses.