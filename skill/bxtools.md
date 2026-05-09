---
name: bxtools
category: bioinformatics/sequence-analysis
description: A suite of tools for building and querying binary index formats (.bx) for fast k-mer counting, sequence retrieval, and genomic data manipulation. Includes companion binaries for index construction and various query operations.
tags: k-mer, binary-index, sequence-analysis, genomics, sequence-retrieval
author: AI-generated
source_url: https://github.com/bxtools/bxtools
---

## Concepts

- **Binary Index Format (.bx)**: bxtools uses a custom binary index format optimized for fast k-mer lookups and sequence retrieval. The index must be built using `bxtools-build` before any query operations can be performed.
- **Companion Binaries**: The bxtools suite consists of multiple executables: `bxtools-build` constructs indices from FASTA/FASTQ input, `bxtools-query` retrieves sequences by k-mer or position, and `bxtools-extract` extracts specific regions from indexed genomes.
- **Input Formats**: Accepts standard bioinformatics formats including FASTA (.fa/.fasta), FASTQ (.fq/.fastq), and plain text. Output can be written to stdout, files, or piped to other tools.
- **Memory-Mapped Indices**: The binary indices are memory-mapped for efficient random access without loading entire files into RAM, enabling rapid queries on large genomic datasets.

## Pitfalls

- **Forgetting to Build Index First**: Query operations will fail with "index not found" errors if the .bx index hasn't been created. Always run `bxtools-build` before attempting any query operations.
- **Mismatched K-mer Length**: Querying with a k-mer size different from what was used during index construction returns no results. The k-mer length is fixed at index creation time and cannot be changed during queries.
- **File Permission Issues**: Insufficient read permissions on input files or write permissions on output directories cause silent failures or permission denied errors. Verify file permissions before running.
- **Large Input Files Without Sufficient Disk Space**: Building indices requires temporary disk space (typically 2-3x the input file size). Running out of disk space corrupts the index and leaves partial files.

## Examples

### Build a binary index from a FASTA file
**Args:** build -i reference.fa -o reference.bx -k 25
**Explanation:** Creates a 25-kmer binary index from the reference FASTA file, enabling fast subsequent queries against the indexed sequence.

### Query sequences by exact k-mer match
**Args:** query -i reference.bx -k ATGCTAGCTAGCTAGCTAGCTA
**Explanation:** Retrieves all sequences in the index containing the exact 21-base k-mer, outputting positions and matching sequences.

### Extract a specific genomic region
**Args:** extract -i reference.bx -c chr1:1000000-2000000
**Explanation:** Extracts the sequence from chromosome 1 spanning positions 1,000,000 to 2,000,000 from the built index.

### Build index with multiple sequence inputs
**Args:** build -i genome1.fa genome2.fa -o combined.bx -k 31
**Explanation:** Combines multiple FASTA files into a single index, useful for querying across multiple genomic sequences in one operation.

### Query with output to file
**Args:** query -i reference.bx -k GATTACA -o matches.txt
**Explanation:** Queries the index for the k-mer and writes all matching results to the specified output file instead of stdout.

### Build index with custom buffer size
**Args:** build -i large_genome.fa -o large_index.bx -k 21 -b 4096
**Explanation:** Builds an index with a custom 4096-byte buffer size, which can improve performance when working with very large genomes.

### List available sequences in index
**Args:** info -i reference.bx
**Explanation:** Displays metadata about the index including sequence names, lengths, k-mer size, and total number of indexed sequences.