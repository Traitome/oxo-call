---
name: apoc
category: sequence-analysis
description: A suffix array-based tool for finding approximate occurrences of patterns in biological sequences with configurable mismatch and indel tolerance. Supports FASTQ/FASTA input and outputs position-based matches with alignment coordinates.
tags:
  - pattern-matching
  - approximate-search
  - suffix-array
  - sequence-alignment
  - motif-discovery
author: AI-generated
source_url: https://github.com/bioinformatics-tools/apoc
---

## Concepts

- APOC builds a suffix array index over the reference sequence, enabling fast approximate pattern search by traversing the array withedit-distance bounded walks rather than full dynamic programming matrices.
- The input sequence must be in FASTA or FASTQ format, with quality scores preserved for downstream filtering; multi-sequence files are processed sequentially unless the `--batch` flag is specified.
- Matches are reported with 1-based genomic coordinates, CIGAR-style alignment strings, and the number of mismatches/indels, allowing direct integration with BEDTools-style interval workflows.

## Pitfalls

- Specifying an edit-distance threshold larger than `k=3` dramatically increases runtime and memory consumption because the suffix array traversal branches exponentially with k; for typical reads, `k≤2` is strongly recommended.
- Using APOC on unsorted multi-FASTA files without pre-sorting by sequence identifier produces non-deterministic coordinate outputs across runs, breaking reproducibility for downstream analysis.
- Forgetting to invoke `apoc-build` before querying causes the tool to attempt in-memory suffix array construction on every run, multiplying execution time by the number of queries.
- Mixing 0-based and 1-based coordinate conventions in downstream BED files leads to off-by-one errors in peak calling or motif enrichment analysis.

## Examples

### Building a suffix array index from a reference genome
**Args:** `apoc-build --sequence ecoli_genome.fasta --output ecoli_index`
**Explanation:** This constructs the persistent suffix array index required for all subsequent queries, reducing repeated query overhead significantly.

### Finding approximate matches of a 20bp motif allowing 1 mismatch
**Args:** `apoc-occurrences --pattern GACTGAATTGCGATCGATAA --genome ecoli_index --max-mismatches 1 --output matches.txt`
**Explanation:** The 1-based coordinates and mismatch count are written to the output file for downstream filtering in R or Python.

### Searching for variants with up to 2 indels in FASTQ reads
**Args:** `apoc-occurrences --query reads.fastq --reference ref_index --max-indels 2 --min-quality 20 --format sam`
**Explanation:** Setting a minimum base quality threshold eliminates low-confidence approximate matches caused by sequencing errors.

### Batch processing multiple pattern files against the same index
**Args:** `apoc-occurrences --pattern-dir motifs/ --reference ref_index --max-mismatches 1 --parallel 4`
**Explanation:** The parallel flag distributes pattern files across 4 cores, roughly quadrupling throughput for large motif sets.

### Extracting flanking sequences around approximate matches
**Args:** `apoc-extract --matches matches.bed --flank 50 --genome genome.fasta --output flanking_seqs.fasta`
**Explanation:** Extending 50bp on each side of each match enables motif enrichment analysis or primer design workflows.