---
name: biscuit
category: bioinformatics/indexing
description: A tool for generating pileup index files (.bai) from SAM/BAM alignment files to enable fast random access for genomic visualization and analysis. Biscuit is commonly used in pangenome workflows and bisulfite sequencing pipelines to index alignment data for tools that require positional lookups.
tags:
- bam
- sam
- indexing
- pileup
- genomics
- visualization
- random-access
author: AI-generated
source_url: https://github.com/lh3/biscuit
---

## Concepts

- **Input Format**: Biscuit takes SAM or BAM files as input and generates a corresponding .bai (BAM index) file that stores the virtual file offsets for each genomic alignment, enabling O(1) random access to any genomic region without scanning the entire file.
- **Index Structure**: The generated .bai file contains a linear index with one entry per genomic chromosome/reference sequence, storing byte offsets to specific alignment blocks rather than per-read offsets, which is optimized for tools like IGV and samtools that perform region-based queries.
- **Output Naming**: By default, biscuit outputs the index file with a `.bai` extension appended to the input filename (e.g., input.bam → input.bam.bai), following the standard naming convention used by samtools and other BAM-aware tools.
- **Alignment Sorting Requirement**: Biscuit requires the input BAM file to be coordinate-sorted (not queryname-sorted) because the .bai index format uses genomic coordinates as the primary lookup key; queryname-sorted files will produce invalid or unusable indexes.

## Pitfalls

- **Using Queryname-Sorted BAM Files**: Running biscuit on queryname-sorted BAM files will generate an index that does not support coordinate-based random access, causing downstream tools to return incorrect or no results when querying by genomic region.
- **Mismatched Reference Names**: If the BAM header (@SQ lines) contains different chromosome names than what reference files expect, the index will not align properly with visualization tools, resulting in silent failures or empty display regions.
- **Unsorted Alignments Within Chromosomes**: Even with coordinate-sorting, if alignments within a chromosome are not properly sorted by position, the linear index will contain incorrect byte offsets, causing access errors or data corruption when querying.
- **File Permission Issues**: Attempting to write the .bai output to a read-only directory or over an existing file without write permissions will fail silently or produce permission denied errors without clear indication of the root cause.

## Examples

### Indexing a coordinate-sorted BAM file
**Args:** input_sorted.bam
**Explanation:** Generates input_sorted.bam.bai by reading the coordinate-sorted BAM file and building a linear index of byte offsets for each reference sequence.

### Specifying a custom output filename
**Args:** input.bam -o my_index.bai
**Args:** input.bam --output my_index.bai
**Explanation:** Writes the generated index to my_index.bai instead of the default input.bam.bai, useful when organizing indexes in separate directories.

### Verbose index generation output
**Args:** input.bam --verbose
**Explanation:** Enables verbose logging to stderr, printing progress messages about which reference sequences are being indexed and any warnings encountered during the process.

### Indexing with reference specification
**Args:** input.bam --reference ref.fasta
**Args:** input.bam -r ref.fasta
**Explanation:** Uses the provided reference fasta file to validate chromosome names in the BAM header and ensure index consistency with the reference used for alignment.

### Checking index validity without regenerating
**Args:** input.bam.bai --check
**Explanation:** Validates an existing .bai file's integrity and structure without regenerating it, useful for diagnosing whether index corruption is causing downstream tool failures.