---
name: blksheep
category: genomics/indexing
description: A companion indexing tool for creating compressed genomic reference databases optimized for rapid read mapping. blksheep generates binary index files fromFASTA/FASTQ reference sequences that can be used by downstream alignment tools like blkmapper.
tags: [genomics, index, reference, fasta, compression, mapping-prep]
author: AI-generated
source_url: https://github.com/example/blksheep
---

## Concepts

- blksheep builds a suffix-array-based index from input reference sequences, enabling fast substring queries and read alignment. The index is stored in multiple files with the `.blksheep.*` extension that must be kept together.
- Input reference sequences must be provided in standard FASTA format (single-line or multi-line sequences). Multiple FASTA files can be specified as arguments, and all sequences are combined into a single unified index.
- The built index supports a configurable hash table size (--hash-size) that trades memory usage for query speed. Larger hash sizes enable faster lookups but consume more RAM during alignment.
- Index files are binary and machine-architecture dependent. Built indices cannot be transferred between systems with different-endianness without rebuilding.

## Pitfalls

- If reference sequence files contain duplicate sequence names, blksheep may silently overwrite earlier entries or produce undefined behavior. Always ensure unique sequence IDs in your input FASTA files.
- Providing an insufficient --hash-size can cause dramatic slowdowns during downstream alignment, sometimes 10-100x slower than with default settings. The recommended minimum is 2^15 for small genomes.
- Attempting to index an empty FASTA file or a file containing only sequence headers with no bases will fail with a cryptic error. Ensure your input files contain actual nucleotide sequences.
- Building indices for very large genomes (e.g.,人類chromosomes) with default settings may consume excessive disk space. Use the --packed option to reduce index file size at the cost of slightly slower query speed.

## Examples

### Index a bacterial genome reference

**Args:** --threads 4 ecoli_k12.fasta --prefix ecoli_k12

**Explanation:** Creates index files for E. coli K-12 reference using 4 parallel threads, outputs files prefixed with "ecoli_k12" in the current directory.

### Build index for a multi-chromosome genome with custom hash size

**Args:** --hash-size 2097152 --threads 8 chr1.fa chr2.fa chr3.fa --prefix mm10

**Explanation:** Uses a 2M-entry hash table for faster queries on mouse genome with 3 chromosomes, outputting to mm10.blksheep.* files.

### Create a packed index for storage-constrained environments

**Args:** --packed --threads 1 small_genome.fa --prefix output

**Explanation:** Builds a compressed index using single thread to minimize memory footprint for small reference sequences.

### Index multiple FASTQ files with verbose output

**Args:** --verbose --threads 16 ref1.fa ref2.fa ref3.fa ref4.fa --prefix multi_ref

**Explanation:** Processes 4 reference files in parallel with detailed logging, creates index prefixed "multi_ref".

### Rebuild existing index with force overwrite

**Args:** --force --threads 4 existing_ref.fa --prefix existing_ref

**Explanation:** Overwrites previously existing index files without prompting, using standard thread count.