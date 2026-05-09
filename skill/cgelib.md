---
name: cgelib
category: Genomics / Genome Library Management
description: A C library for computational genomics providing data structures and functions for genome coordinate operations, interval handling, and genomic feature management. Supports efficient storage and retrieval of genomic intervals, annotations, and multi-genome comparisons.
tags:
  - genomics
  - genome-coordinates
  - intervals
  - genomic-features
  - bioinformatics
  - c-library
author: AI-generated
source_url: https://github.com/hyeshik/cgelib
---

## Concepts

- **Genomic Interval Representation**: cgelib represents genomic features using chromosome, start position, end position, strand, and optional metadata fields. Coordinates are 0-based for the start and 1-based for the end (half-open intervals), following BED format conventions.

- **Indexing with cgelib-build**: The companion binary `cgelib-build` creates indexed genome libraries from FASTA format genome sequences, enabling rapid lookup of genomic regions. The library supports multiple genome assemblies in a single index.

- **Input/Output Formats**: cgelib operates on standard genomic formats including BED, GTF/GFF3 for annotations, and FASTA for genome sequences. It uses a custom binary `.cgi` format for indexed genome libraries that enables efficient random access.

- **Memory-Efficient Operations**: The library uses coordinate-sorted indexes and memory-mapped I/O for handling large genomes (human genome ~3 billion base pairs) with minimal memory footprint, making it suitable for whole-genome analyses.

## Pitfalls

- **Confusing 0-based vs 1-based Coordinates**: cgelib internally uses 0-based coordinates for start positions (half-open intervals), but many visualization tools expect 1-based coordinates. Mixing these conventions leads to off-by-one errors in feature positions.

- **Using Unsorted Input for Interval Operations**: When performing interval intersection or merge operations, input coordinates must be sorted by chromosome and position. Unsorted input produces incorrect results or causes runtime errors.

- **Mismatched Genome Builds**: Using annotation data with a different genome build than the indexed library leads to silent coordinate mismatches. Always verify chromosome names and lengths match between annotation and reference genome.

- **Neglecting Strand Orientation**: cgelib tracks strand information for features, but many operations default to treating both strands as equivalent. Failing to specify strand-specific behavior results in features being counted twice or improperly oriented.

## Examples

### Create an indexed genome library from a FASTA file