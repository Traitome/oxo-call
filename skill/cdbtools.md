---
name: cdbtools
category: Bioinformatics - Database Management
description: A suite of utilities for managing and querying compact bioinformatics databases, particularly optimized for large-scale genomic and proteomic data retrieval with fast lookup operations.
tags: [bioinformatics, database, genomics, sequence-lookup, index, cdb]
author: AI-generated
source_url: https://github.com/bioinformatics/cdbtools
---

## Concepts

- The cdbtools suite uses a custom binary index format (.cdb) that provides O(1) average-case lookup time for sequence identifiers, making it suitable for real-time query applications in high-throughput pipelines.
- Database files are built from standard formats (FASTA, GenBank, BED) using the companion `cdbtools-build` binary, which creates sorted indices with optional compression and supports both DNA/RNA sequences and protein annotations.
- Query operations return results in tab-delimited format by default, enabling easy piping to downstream tools likeawk, sed, or custom parsers; the tool supports batch queries via stdin input in multi-line format.
- The index structure supports range queries and interval-based searches, allowing retrieval of all records overlapping a genomic region without decompressing the entire database.

## Pitfalls

- Attempting to query a database built with an incompatible schema version results in silent failures where queries return zero results rather than informative error messages; always verify schema version with `cdbtools-info` before querying.
- Using lossy compression (enabled via `-c fast` or `-c balanced`) can cause nucleotide ambiguities in sequence retrieval, leading to downstream alignment failures or incorrect variant calling when using these sequences as references.
- When building databases from multiple input files, failing to sort input by identifier beforehand produces corrupted indices where queries may miss entries or return duplicate, conflicting records.
- Specifying relative paths to database files when running queries from different working directories causes file-not-found errors; always use absolute paths or ensure the working directory is correctly set.

## Examples

### Query a single sequence by identifier
**Args:** `query --db /refs/genome.hg38.cdb --id BRCA1`
**Explanation:** Retrieves the full sequence record for the BRCA1 gene from the pre-built genome database, returning accession, description, and sequence data in tab format.

### Batch retrieve multiple sequences from stdin
**Args:** `query --db /refs/transcripts.c