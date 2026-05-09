---
name: ac-diamond
category: sequence_analysis
description: Fast protein similarity search tool for large-scale alignment of protein sequences against protein databases, similar to BLAST but with orders-of-magnitude speed improvements for big data applications.
tags:
- protein-alignment
- sequence-search
- similarity
- fast-aligner
- bioinformatics
- genomics
- ncbi-blast-alternative
author: AI-generated
source_url: https://github.com/bbuchfink/diamond
---

## Concepts

- **Input Format** — Accepts FASTA or FASTQ protein sequences (`.fa`, `.faa`, `.fasta`) as query files. Queries can be stdin when using `-` as filename, enabling pipeline composition.
- **Database Creation** — Uses the companion `ac-diamond-build` (or equivalent makedb command) to create a binary database from a protein FASTA file. The database file is memory-mapped for efficient concurrent access.
- **Output Formats** — Supports multiple output modes including plain text (`--out`), tabular with customizable columns (`--outfmt 6`), XML similar to NCBI BLAST, and SAM format for genomic pipelines.
- **Sensitivity Modes** — Provides five sensitivity presets (--sensitive, --more-sensitive, --very-sensitive, --ultra-sensitive, --banded Smith-Waterman) trading speed for alignment quality; default is faster but may miss distant homologs.
- **E-value Threshold** — Filters results using e-value (-1e-10 default) and minimum identity (-p 0) percentage; lowering e-value reduces false positives but may exclude true positives with high divergence.

## Pitfalls

- **Missing Database Creation** — Running alignment without first creating a database with the build step produces empty results with no meaningful error message, wasting compute time on queries that never align. Always verify database exists with `ac-diamond-build` before running queries.
- **Incompatible Query/DB Types** — Using nucleotide query with protein database (or vice versa) without specifying `--blastx` mode produces nonsensical alignments or zero results; ensure query type matches database type and alignment mode.
- **Memory Overflow with Large Datasets** — Default index uses 1GB RAM per 1M sequences; on systems with limited RAM this causes OOM killer termination. Use `--interval` to increase index spacing and reduce memory footprint, or use `--block-size` to process in chunks.
- **File Permission Errors** — Output file specified with `--out` will fail silently if directory lacks write permission, creating zero-byte files that appear successful but contain no alignments. Verify write permissions before running.
- **Thread Contention** — Default uses all available cores, but running multiple instances simultaneously causes thread contention, reducing overall throughput. Set `--threads` explicitly to leave headroom for parallel workloads.

## Examples

### Search protein sequences against a DIAMOND database
**Args:** blastp --query proteins.fasta --db reference_db --out alignments.tsv --outfmt 6 --evalue 0.001
**Explanation:** Uses blastp mode to align query proteins against a pre-built protein database, outputting standard tabular format with entries having e-value less than 0.001.

### Create a DIAMOND database from a FASTA file
**Args:** makedb --in uniprot_proteomes.fasta --db uniprot_db
**Explanation:** Builds a binary search database from input FASTA file for use in subsequent ac-diamond searches; creates index files alongside the database file.

### Find remote homologs with maximum sensitivity
**Args:** blastp --query query.fasta --db pfam_db --out remote_hits.tsv --ultra-sensitive --evalue 1e-5
**Explanation:** Uses ultra-sensitive mode to detect distant evolutionary relationships, accepting slower runtime to find homologs below default sensitivity thresholds.

### Generate XML output for NCBI BLAST compatibility
**Args:** blastx --query nucleotides.fasta --db nr_db --out blast_results.xml --xml
**Explanation:** Aligns nucleotide query proteins against protein database in XML format for integration with tools expecting NCBI BLAST XML output.

### Limit results to high-identity matches only
**Args:** blastp --query query.fasta --db db --out high_identity.tsv --min-score 50 --id 90
**Explanation:** Filters alignments to those with at least 90% sequence identity and bit score ≥50, reducing downstream analysis to close homologs only.

### Run chunked alignment for memory-constrained systems
**Args:** blastp --query large.fasta --db db --out results.tsv --block-size 2 --no-logfile
**Explanation:** Processes large query file in 2GB memory blocks to avoid OOM errors, disabling log file creation for cleaner output directory.

### Extract only query-subject pairs in tabular format
**Args:** blastp --query query.fasta --db db --outfmt 6 qseqid sseqid pident evalue bitscore
**Explanation:** Outputs tabular format with specific columns (query ID, subject ID, percent identity, e-value, bit score) for downstream programmatic parsing.