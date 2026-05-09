---
name: biodigest
category: Sequence Similarity Search / Indexing
description: A bioinformatics tool for creating indexed collections of biological sequences and performing fast similarity searches. biodigest builds compressed indexes from FASTA/FASTQ sequence databases and supports fast k-mer based alignment-free search operations.
tags:
  - sequence-indexing
  - similarity-search
  - k-mer
  - bioinformatics
  - genomics
  - alignment-free
author: AI-generated
source_url: https://github.com/biodigest-tool/biodigest
---

## Concepts

- **Index Building**: biodigest uses a k-mer based indexing strategy where sequences are decomposed into overlapping k-mers and stored in a hash table for O(1) lookup during searches. The index size depends on the k-mer length and sequence complexity.
- **Input Formats**: biodigest accepts standard bioinformatics formats including FASTA (`.fa`, `.fasta`), FASTQ (`.fq`, `.fastq`), and multi-FASTA files. Plain text and gzipped (`.gz`) inputs are both supported.
- **Search Modes**: The tool supports exact k-mer matching (default) and approximate matching with configurable mismatches allowed per k-mer position. Results include alignment coordinates, sequence identities, and bit scores.
- **Output Formats**: Search results can be output in JSON, TSV, or a custom biodigest format (`.bdr`) for downstream processing. The `.bdr` format preserves all metadata and allows incremental index updates.

## Pitfalls

- **K-mer Length Mismatch**: Using a k-mer length that is too short leads to excessive false positives due to random k-mer collisions, while k-mers that are too long reduce sensitivity and may miss valid matches. The recommended range is 10-31 for nucleotide sequences.
- **Memory Scaling with Large Databases**: Building indexes for very large sequence collections (millions of sequences) requires substantial RAM. The index typically needs 2-4x the raw database size in memory during construction.
- **Input Sequence Quality**: Low-quality sequences with ambiguous bases (N characters) or excessive sequencing errors produce spurious k-mers that degrade search accuracy. Pre-filtering with quality thresholds is recommended.
- **Index Corruption from Interrupted Operations**: If the indexing process is interrupted (e.g., system crash, kill signal), the resulting index may be corrupted and unusable. Always ensure stable storage and complete operations.

## Examples

### Build an index from a FASTA database
**Args:** build --input sequences.fasta --output mydb --kmer 21 --threads 4
**Explanation:** Creates a k-mer index with k=21 from sequences.fasta, using 4 parallel threads for faster indexing. The output index files are stored in the directory mydb.

### Search a query sequence against an existing index
**Args:** search --index mydb --query query.fa --output results.tsv --format tsv
**Explanation:** Searches the query sequence(s) in query.fa against the pre-built index in mydb and outputs results in tab-separated format for easy parsing in other tools.

### Perform a search allowing mismatches
**Args:** search --index mydb --query query.fa --mismatches 2 --score-threshold 50
**Explanation:** Searches allowing up to 2 mismatches per k-mer and only reports matches with scores of 50 or higher to filter low-quality results.

### List available indexes in a directory
**Args:** list --dir /path/to/indexes
**Explanation:** Displays all biodigest indexes stored in the specified directory, showing index metadata including k-mer size, sequence count, and creation date.

### Update an existing index with new sequences
**Args:** update --index mydb --input new_sequences.fa
**Explanation:** Adds new sequences from new_sequences.fa to an existing index without rebuilding from scratch. The index is incremented with the new k-mers.

### Export search results in JSON format
**Args:** search --index mydb --query query.fa --output results.json --format json
**Explanation:** Outputs search results in JSON format, which is suitable for integration with web services or Python/R pipelines that require structured data.

### Build an index with custom k-mer length for proteins
**Args:** build --input proteins.faa --output protein_db --kmer 7 --type protein
**Explanation:** Creates a protein-specific index using k=7 (shorter k-mers are appropriate for protein sequences due to the 20-amino-acid alphabet). The --type flag optimizes the indexing for amino acid sequences.