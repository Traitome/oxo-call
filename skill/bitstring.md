---
name: bitstring
category: Sequence Indexing / Compression
description: A tool for constructing and querying compressed bit-vector indexes of biological sequences, enabling efficient pattern matching and sequence search operations with minimal memory footprint.
tags:
  - sequence-indexing
  - compression
  - bit-vector
  - pattern-search
  - bioinformatics
  - genomics
  - k-mer
author: AI-generated
source_url: https://github.com/langmeadlab/bitstring
---

## Concepts

- Bitstring builds compressed bit-vector indexes using run-length encoding (RLE), which stores sequences as alternating runs of bits rather than individual positions, dramatically reducing memory usage for repetitive biological sequences like homopolymers and microsatellites.
- The index operates on k-mer spaces where each k-mer maps to a bit position, enabling O(1) lookup time for exact k-mer matches and efficient approximate matching through bitwise operations on neighboring positions.
- Input sequences must be provided in standard FASTA/FASTQ format, and the tool supports both nucleotide and amino acid alphabets, automatically detecting the alphabet size to allocate the appropriate bit-vector dimensions.

## Pitfalls

- Specifying a k-mer size larger than the shortest input sequence causes the indexer to skip that sequence entirely without warning, leading to incomplete search results across datasets.
- Using an incompatible alphabet mode (e.g., nucleotide when indexing protein sequences) results in hash collisions that corrupt all downstream queries and produce false positive matches.
- Attempting to query an index built with a different k-mer size than the query parameters returns zero matches silently, as the bit positions are incompatible between index and query bit-vectors.

## Examples

### Build a bit-vector index from a FASTA file of genomic sequences

**Args:** build --input sequences.fasta --output genome.bsi --kmer 31 --alphabet nucleotide
**Explanation:** This constructs a compressed bit-vector index for 31-mers across all sequences in the FASTA file using the nucleotide alphabet, saving the index to genome.bsi for subsequent queries.

### Query the index to find positions of a specific k-mer

**Args:** query --index genome.bsi --pattern ACGTACGTACGTACGTA --positions
**Explanation:** This performs an exact match lookup, returning all genomic positions where the 20-mer ACGTACGTACGTACGTA occurs by reading its corresponding bit positions from the index.

### Search for approximate k-mer matches allowing mismatches

**Args:** query --index genome.bsi --pattern ACGTACGTACGTACGTA --hamming-distance 2 --matches-only
**Explanation:** This returns all k-mers within Hamming distance 2 of the query pattern by checking bit positions of the pattern and its neighbors, useful for tolerance to sequencing errors.

### List all k-mers present in both query and indexed sequences

**Args:** query --index genome.bsi --fastq reads.fastq --intersection --min-count 3
**Explanation:** This reads k-mers from the FASTQ file, intersects them with the index, and reports only those appearing at least 3 times, filtering out rare artifacts.

### Export the raw bit-vector as text for external analysis

**Args:** dump --index genome.bsi --format text --output bitvector.txt --region chr1:1000000-2000000
**Explanation:** This exports the uncompressed bit representation for chr1 positions 1-2 Mb as a text file, enabling integration with third-party tools that cannot read the binary format.