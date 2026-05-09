---
name: bamkit
category: sequence_alignment/bam_manipulation
description: A versatile command-line toolkit for manipulating, analyzing, and converting BAM alignment files. Provides utilities for filtering, sorting, merging, splitting, and extracting statistics from SAM/BAM format alignment data.
tags: [bam, sam, alignment, manipulation, filtering, sorting, indexing, bioinformatics]
author: AI-generated
source_url: https://github.com/bamkit/bamkit
---

## Concepts

- **BAM/SAM Format Handling**: bamkit operates on binary BAM or text SAM alignment files. Use the `--input` flag to specify the input file and `--output` to write results; when omitted, output goes to stdout for piping.
- **Read Filtering**: Filter reads by flags (e.g., remove duplicates with `--flag 1024`), MAPQ threshold (`--min-mapq`), chromosome/region (`--chrom chr1:1000-5000`), or read name patterns (`--read-name-pattern`).
- **Sorting Modes**: Sort BAM output by genomic coordinate (`--sort coordinate`), read name (`--sort name`), or template/consensus order (`--sort template`). Coordinate sorting is required for valid BAM indexing.
- **Index Generation**: After sorting by coordinate, generate companion `.bai` indices using the companion binary `bamkit-index` to enable random access and efficient region queries.

## Pitfalls

- **Unsorted Input for Indexing**: Attempting to create an index on unsorted BAM data causes `bamkit-index` to fail with an error, wasting computation time. Always sort with `--sort coordinate` before indexing.
- **Losing Unpaired Reads**: Using `--proper-pair-only` discards single-end reads and read pairs failing mate criteria, skewing downstream metrics if unpaired alignments are required for analysis.
- **Memory Overflow with Large Files**: Processing huge BAM files without chunking flags (`--chunk-size`) can exhaust RAM on systems with limited memory, causing termination mid-operation.
- **Misinterpreting Flag Filters**: Negative flag filters like `--exclude-flag 4` remove unmapped reads silently; forgetting this can lead to incomplete datasets without warning if unmapped reads were intended.

## Examples

### Filter reads with minimum mapping quality
**Args:** `--input alignments.bam --min-mapq 30 --output highq_filtered.bam`
**Explanation:** Retains only reads with MAPQ ≥ 30, removing low-confidence alignments common in repetitive genomic regions.

### Remove duplicate reads
**Args:** `--input dedup_input.bam --flag 1024 --exclude --output dedup.bam`
**Explanation:** Excludes reads flagged as PCR duplicates (flag 1024), useful before variant calling to avoid artificial allele support.

### Sort BAM by coordinate
**Args:** `--input unsorted.bam --sort coordinate --output sorted_coord.bam`
**Explanation:** Sorts alignments by reference name and position order, required for valid index creation and many downstream tools.

### Extract reads in a genomic region
**Args:** `--input sample.bam --chrom chr2:500000-600000 --output region_subset.bam`
**Explanation:** Isolates all alignments overlapping the specified chr2 interval, enabling targeted analysis without processing whole file.

### Convert BAM to SAM format
**Args:** `--input data.bam --output-format sam --output data.sam`
**Explanation:** Converts binary BAM to human-readable SAM format, useful for debugging or manual inspection of alignments.

### Count alignments per chromosome
**Args:** `--input alignments.bam --count-per-chrom --output stats.txt`
**Explanation:** Generates a table of alignment counts per reference sequence, providing quick summary statistics for quality assessment.

### Merge multiple BAM files
**Args:** `--input file1.bam --input file2.bam --input file3.bam --output merged.bam`
**Explanation:** Combines multiple BAM files into a single output, preserving all alignments with updated file headers.

### Filter unmapped reads
**Args:** `--input mixed.bam --exclude-flag 4 --output mapped_only.bam`
**Explanation:** Removes all unmapped reads (flag 4) from the dataset, leaving only successfully aligned sequences for coverage analysis.

---