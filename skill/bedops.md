---
name: bedops
category: Genomic Interval Set Operations
description: A fast, scalable toolkit for performing efficient set operations on genomic interval data including union, intersection, difference, and complement operations. Supports streaming I/O and multiple genomic formats.
tags:
  - genomics
  - intervals
  - set-operations
  - vcf
  - gff
  - bam
  - bed
  - starch
  - conversion
author: AI-generated
source_url: https://bedops.readthedocs.io/
---

## Concepts

- **Lexicographic Sorting Requirement**: All BEDOPS operations require input files to be sorted lexicographically by chromosome (or scaffold) name followed by start coordinate. Unsorted input causes silent empty results or incorrect overlaps without warnings.
- **Chromosome Naming Consistency**: Input files and reference data must use identical chromosome naming conventions (e.g., `chr1` vs `1`). A mismatch between files results in zero overlaps even when intervals cover the same genomic region.
- **Starch Compression Format**: BEDOPS uses a proprietary `starch` format (`.starch` extension) for compressed genomic data, providing faster I/O than bgzipped BED files. The `starch` metadata includes per-chromosome byte offsets enabling random access without full decompression.
- **Streaming Architecture**: Most BEDOPS tools read from stdin and write to stdout, enabling efficient UNIX-style piping. This design minimizes disk I/O and allows chaining multiple operations without intermediate files.
- **Reference Data Organization**: The `bedops-reference-data` command organizes user-provided reference genomes into a directory structure keyed by chromosome name, enabling efficient set operations against entire genomes without loading complete files into memory.

## Pitfalls

- **Unsorted Input Produces No Error**: When feeding unsorted BED data into `bedops`, no error or warning is raised; instead the tool produces empty output silently. Always verify sorting with `sort-bed` before set operations.
- **Chromosome Naming Mismatch Yields Zero Results**: A dataset using `chr1` joined with one using `1` will produce zero overlapping intervals with no indication of the naming inconsistency. Manually inspect chromosome columns or use `cut -f1` to verify naming conventions match.
- **Memory Exhaustion with Unsorted Large Files**: Attempting operations like `bedops --everything` on unsorted files exceeding available RAM causes the process to hang or crash because BEDOPS builds in-memory data structures during execution.
- **Converting Formats Without Verifying Chromosome Values**: Converting GFF to BED with `gff2bed` then using the output with VCF-converted BED data fails because chromosome column values may differ (e.g., `NC_000001.11` vs `chr1`) despite describing the same genomic region.
- **Assuming Bedtools-Compatible Output**: BEDOPS outputs sorted BED by default, but using `--chrom` flag outputs unsorted data organized by chromosome in separate blocks. This output format is incompatible with tools expecting standard sorted BED without preprocessing.

## Examples

### Finding interval overlaps between two BED files
**Args:** `--intersect