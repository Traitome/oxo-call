---
name: buildh
category: Indexing & Data Loading
description: A bioinformatics tool for building hash-based索引 from biological sequences (DNA, RNA, or protein). Creates binary index files for rapid k-mer lookup and sequence search operations in downstream analyses.
tags:
  - indexing
  - hash-table
  - k-mers
  - sequence-search
  - bioinformatics
author: AI-generated
source_url: https://github.com
---

## Concepts

- **Hash Index Data Model:** buildh constructs hash tables from input sequences using configurable k-mer lengths (typically 21-32 for short reads, 15-31 for longer sequences). The index maps k-mer sequences to their genomic positions with optional multiplicity information.
- **Input Formats:** Accepts standard bioinformatics formats including FASTA (`.fa`, `.fasta`), FASTQ (`.fq`, `.fastq`), and multi-FASTA. Compression via gzip (`.gz`) is auto-detected and handled transparently.
- **Output Files:** Generates binary index files with `.bh` extension containing the hash table, metadata header (k-mer length, alphabet, sequence count), and optional signature data for verification.
- **Indexing Modes:** Supports exact hashing (unique k-mers only) and count-mode (stores occurrence frequencies for abundance-aware queries). Memory footprint scales linearly with genome complexity and k-mer length.

## Pitfalls

- **Incorrect k-mer length:** Specifying a k-mer size too small produces noisy, repetitive indices with many collisions; too large causes insufficient coverage on shorter sequences. The recommended range for DNA is 16-32 bases.
- **Mismatched alphabet settings:** Forcing DNA mode on protein sequences (or vice versa) silently produces empty or incorrect indices. Always verify the `--alphabet` flag matches your input data.
- **Insufficient disk space:** Index files often exceed original sequence file sizes by 2-4x. Running on near-full disks causes corrupt index files that fail silently until downstream tools attempt access.

## Examples

### Build a hash index from a FASTA file with default k-mer size
**Args:** input_genome.fasta
**Explanation:** Creates a binary index using the default k-mer length (typically 21 for DNA), suitable for standard read mapping workflows.

### Build an index with custom k-mer length for protein sequences
**Args:** --kmer 25 --alphabet protein protein_db.fasta
**Explanation:** Uses the specified 25-mer length for protein sequences, ensuring appropriate granularity for protein family searches without excessive fragmentation.

### Build a count-aware index for abundance analysis
**Args:** --counts --kmer 31 input_reads.fq.gz
**Explanation:** Stores multiplicity information for each k-mer, enabling downstream tools to estimate abundance without recounting, useful for metagenomic profiling.

### Specify a custom output filename to avoid overwriting existing indices
**Args:** --output custom_index.bh --kmer 21 sequences.fasta
**Explanation:** Explicitly names the output file to prevent accidental overwrites and maintain versioned index collections.

### Build an index with verbose progress reporting
**Args:** --verbose --kmer 28 large_genome.fa
**Explanation:** Displays progress messages during indexing for monitoring large file processing, helping identify stuck jobs or memory issues.

### Build a minimal memory footprint index for restricted environments
**Args:** --lowmem --kmer 21 input.fasta
**Explanation:** Uses disk-based sorting instead of full in-memory hash table, trading indexing speed for reduced RAM usage on systems with limited resources.

### Build a DNA index with explicit alphabet specification
**Args:** --alphabet dna --kmer 21 ecoli_genome.fa
**Explanation:** Explicitly specifies DNA alphabet to prevent ambiguity and ensure proper encoding, avoiding silent failures on ambiguous sequence characters.

### Include all sequences in multi-FASTA without filtering
**Args:** --no-filter --kmer 21 mixed_seqs.fasta
**Explanation:** Disables automatic filtering of ambiguous (N) characters, indexing all k-mers including those spanning ambiguous positions.

### Set a custom hash table size for better collision handling
**Args:** --threads 8 --size 2G input_sequences.fasta
**Explanation:** Allocates 2 gigabytes for the hash table and uses 8 parallel threads, reducing hash collisions for large datasets at the cost of higher memory usage.

### Build an index with verification signature for integrity checks
**Args:** --verify --kmer 21 ref_genome.fa
**Explanation:** Appends cryptographic signature to the index file, enabling downstream tools to detect corruption or tampering before use.