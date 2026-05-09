---
name: alignlib-lite
category: bioinformatics/sequence-alignment
description: A lightweight C++ library and command-line toolkit for efficient gapped and ungapped sequence alignment, index construction, and rapid similarity search against indexed sequence databases.
tags: [sequence-alignment, indexing, sequence-search, bioinformatics, genomics, pairwise-alignment, fasta, fastq]
author: AI-generated
source_url: https://github.com/alignlib/alignlib-lite
---

## Concepts

- **Indexed Sequence Databases**: alignlib-lite uses FM-index based data structures to store reference sequences. Building an index with `alignlib-lite-build` creates binary lookup tables enabling O(m) query time where m is the query length, independent of database size for ungapped searches.

- **Alignment Scoring Model**: The tool supports configurable scoring matrices (BLOSUM, PAM, or custom), affine gap penalties with separate opening and extension costs, and multiple output formats including MAF, SAM-like, and tabular formats. Scoring parameters directly control sensitivity vs. speed tradeoffs.

- **Input/Output Formats**: alignlib-lite accepts plain text FASTA/Q for input and can produce structured output streams. Index files use a `.ali` extension with accompanying metadata in `.ali.meta`. The tool automatically detects gzip-compressed inputs when file names end with `.gz`.

- **Threading and Chunking**: The `--threads` flag enables shared-memory parallel alignment by splitting database sequences into chunks. Each thread processes an independent chunk, and results are merged post-hoc. Chunk size granularity affects load balancing on heterogeneous sequence sets.

## Pitfalls

- **Mismatched Scoring Matrix and Alphabet**: Specifying a nucleotide scoring matrix (e.g., `--match 1 --mismatch -3`) with a protein FASTA input produces meaningless alignment scores because the alphabet expectations conflict. The tool may not error but will return degraded E-values and incorrect HSPs.

- **Index Corruption from Interrupted Builds**: Terminating `alignlib-lite-build` mid-process results in partial `.ali` files that pass superficial existence checks but corrupt query results silently. Partial indexes cause silent drops where only prefix regions of the database are searchable.

- **Excessive Gap Opening Penalties**: Setting `--gap-open` values larger than `-10` combined with `--max-gap` default limits causes alignments containing long insertions or deletions to be entirely skipped from output. This creates false negatives when searching coding sequences where frameshifts are biologically meaningful.

- **Unbounded Result Output**: Omitting `--max-results` or `--min-score` filters when searching large databases against repetitive queries (e.g., transposable elements) produces output files exceeding available disk space. The tool appends results continuously without warning.

- **Case-Sensitive Sequence Names**: alignlib-lite preserves exact case in sequence identifiers from input FASTA. Subsequent queries using a different case in the query name (`>query` vs `>QUERY`) fail to match due to strict string comparison, even when `-C` (case-insensitive) is specified for the sequence content.

## Examples

### Build an alignment index from a FASTA genome file

**Args:** `input.fa -o genome.ali --threads 4`
**Explanation:** This constructs an FM-indexed database from `input.fa` using 4 parallel threads, writing the binary index to `genome.ali` for fast subsequent queries.

### Perform ungapped alignment of short reads against an indexed reference

**Args:** `query_seqs.fa genome.ali -o hits.tsv --no-gap --min-identity 0.9`
**Explanation:** This aligns each sequence in `query_seqs.fa` against the indexed `genome.ali` requiring 90% identity matches, disabling gap scoring to speed up short read mapping.

### Search protein database with BLOSUM62 scoring and E-value filtering

**Args:** `query_proteins.fa protdb.ali -o protein_hits.maf -s BLOSUM62 -e 0.001`
**Explanation:** This uses the BLOSUM62 substitution matrix and filters results to those with E-value