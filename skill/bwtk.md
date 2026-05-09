---
name: bwtk
category: genomics/sequence-analysis
description: A Burrows-Wheeler Transform toolkit for building FM-indexes and performing fast pattern matching on genomic sequences. Provides efficient indexed search capabilities for read alignment and sequence retrieval.
tags:
  - bwt
  - fm-index
  - alignment
  - indexing
  - sequence-search
  - genomics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bwtk
---

## Concepts

- **FM-Index Construction**: bwtk builds compressed FM-indexes from input sequences (FASTA/FASTQ), enabling suffix-array-like queries with reduced memory footprint. The index stores the BWT string along with auxiliary sampled coordinates for backward search navigation.
- **Exact Pattern Matching**: Using the backward search algorithm on the FM-index, bwtk can locate exact matches of query sequences in O(m + occ) time where m is pattern length and occ is the number of occurrences. This is the core operation for read alignment.
- **Output Formats**: Results are reported as genomic coordinates (contig:position) with optional alignment quality scores. Some subcommands emit SAM-compatible alignments; others emit plain text with match positions and counts.
- **Index Persistence**: Built indexes are stored as binary files with `.bwtidx` extension, which can be loaded for subsequent searches without rebuilding. Index files contain the BWT, sampled suffix array ranks, and sequence metadata.

## Pitfalls

- **Mismatched Reference Sequences**: Using a different reference genome version than what the index was built with produces incorrect coordinates. Always verify the reference source and build date match your sample metadata.
- **Query Sequences with N Characters**: The backward search algorithm does not handle ambiguous 'N' bases correctly—their occurrences will be missed or reported with incorrect positions. Mask Ns or use ambiguity-aware tools instead.
- **Index File Corruption**: Loading a partially written or corrupted index file produces silent errors or garbage results. Check index integrity with the verify subcommand before large-scale searches.
- **Memory Exhaustion on Large Genomes**: Building indexes for whole chromosome-scale sequences without sufficient RAM causes crashes. Use streaming builds or chunked indexing for reference-sized inputs.
- **Case Sensitivity**: Input sequences are case-sensitive by default; 'a' and 'A' are treated as different symbols. Ensure query and reference case conventions match.

## Examples

### Build an FM-index from a reference FASTA file
**Args:** build -o genome.bwtidx reference.fa
**Explanation:** Creates a binary FM-index file from the input reference sequences, enabling subsequent pattern searches. The output file stores the BWT and auxiliary structures needed for backward search.

### Find exact matches of a short read sequence
**Args:** search genome.bwtidx ACGTACGTAGCT
**Explanation:** Performs backward search on the built index and reports all genomic positions where the 12-base query occurs exactly. Returns positions on the forward strand by default.

### Find exact matches and output in BED format
**Args:** search -f bed genome.bwtidx ACGTACGTAGCT
**Explanation:** Reports matching positions in BED format (chromosome, start, end), convenient for direct intersection with other genomic annotations in downstream tools.

### Query multiple sequences from a FASTQ file
**Args:** search -i queries.fq genome.bwtidx
**Explanation:** Reads all sequences from the input FASTQ file and searches each against the index, reporting matches for all queries in batch mode. Handles line-based multi-record input efficiently.

### Report only the count of matches without positions
**Args:** search -c genome.bwtidx ACGTACGTAGCT
**Explanation:** Returns just the number of occurrences found, useful for quick abundance estimation before retrieving actual coordinates. Faster than full position reporting for large indexes.

### Build index with auxiliary sampling for faster retrieval
**Args:** build -s 32 -o chr22.bwtidx chr22.fa
**Explanation:** Creates an FM-index with sampled suffix array entries every 32 positions, trading modest file size increase for faster position resolution during search operations.

### Verify integrity of an existing index file
**Args:** verify genome.bwtidx
**Explanation:** Reads the index metadata and auxiliary structures to confirm the file is not corrupted, checking signatures and structure consistency before running searches.