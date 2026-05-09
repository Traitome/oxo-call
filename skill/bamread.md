---
name: bamread
category: Sequence Data Processing
description: A tool for extracting and filtering reads from BAM (Binary Alignment Map) files. Reads aligned sequences from BAM files with support for region-based extraction, read filtering by flags, and output in SAM, BAM, or JSON format.
tags:
  - bam
  - sam
  - reads
  - extraction
  - filtering
  - alignment
author: AI-generated
source_url: https://github.com/lh3/bamread
---

## Concepts

- **BAM Binary Format**: BAM is the binary, compressed form of SAM (Sequence Alignment Map). The tool automatically detects whether the input is BAM or SAM based on file extension and magic bytes; using `.bam` extension ensures binary decompression for efficient I/O.
- **Zero-based Coordinates**: Genomic coordinates in BAM files use zero-based start positions (the first base is position 0), which differs from SAM's optional one-based format—always specify regions using zero-based coordinates.
- **BAM Index Requirement**: For random-access region queries (e.g., `chr1:1000-2000`), a corresponding `.bai` index file must exist in the same directory with the same filename plus `.bai` extension; without an index, the tool must scan the entire file.
- **Flag Filtering**: SAM format uses bitwise flags to encode read properties (paired, mapped, reverse strand, etc.). The tool supports numeric flag masks with `--require-flags` (all flags must be present) and `--skip-flags` (flags that exclude reads).

## Pitfalls

- **Forgetting the BAM Index**: Attempting region-based extraction on an unindexed BAM file causes the tool to iterate through all reads linearly, wasting time on large files. Index with `samtools index` before using `bamread` for any coordinate-based queries.
- **Confusing Zero-based and One-based Coordinates**: Specifying `--region chr1:999-2000` extracts positions 999-2000 (zero-based), which corresponds to bases 1000-2001 in one-based human convention—off-by-one errors lead to missing the first base of the intended range.
- **Inverting Flag Logic**: Using `--skip-flags 4` excludes unmapped reads as intended, but forgetting that `--require-flags 2` also requires the read to be paired creates overly restrictive filters that may return zero reads when both flags are combined incorrectly.
- **Output Format Mismatch**: Writing BAM output (`-o out.bam`) requires the input to already be sorted by coordinate; outputting unsorted BAM will break downstream tools like GATK that expect sorted input.

## Examples

### Extract all reads from a specific chromosomal region

**Args:** `--region chr1:1000000-2000000 input.bam`

**Explanation:** Extracts all reads overlapping the specified genomic interval using zero-based coordinates, requiring the BAM file to be indexed for efficient random access.

### Filter for properly paired reads only

**Args:** `--require-flags 2 input.bam`

**Explanation:** Retains only reads where flag 2 (properly paired) is set, removing singleton reads, discordant pairs, and reads that failed pairing criteria.

### Convert BAM to SAM text format

**Args:** `--output-format sam -o reads.sam input.bam`

**Explanation:** Converts the binary BAM file to human-readable SAM format, useful for manual inspection or piping to text-based analysis tools.

### Exclude PCR duplicates from analysis

**Args:** `--skip-flags 1024 input.bam`

**Explanation:** Removes duplicate reads (flag 1024) that arise from PCR amplification artifacts, keeping only unique molecules for quantitative analysis.

### Extract reads with minimum mapping quality

**Args:** `--min-mapq 30 input.bam`

**Explanation:** Filters reads by mapping quality (MAPQ) score, retaining only reads withconfidence score ≥30 (roughly 99.7% accuracy) to reduce alignment uncertainty.

### Output reads as JSON for programmatic parsing

**Args:** `--output-format json -o reads.json input.bam`

**Explanation:** Exports alignment data in JSON format with structured fields for each read, enabling easy parsing by scripts and pipelines.

### Extract reads by read name for molecule verification

**Args:** `--read-name SRR1234567.1 input.bam`

**Explanation:** Selects a specific read by name (first in pair), useful for debugging, examining specific molecules, or extracting paired-end read groups.

### Limit output to first 1000 reads for quick inspection

**Args:** `--max-records 1000 input.bam`

**Explanation:** Restricts output to the first 1000 reads in file order, providing a quick preview of the file contents without processing the entire large BAM file.

### Extract reverse-strand reads for strand-specific analysis

**Args:** `--require-flags 16 input.bam`

**Explanation:** Selects only reads on the reverse strand (flag 16), essential for detecting antisense transcription or strand-specific RNA-seq protocols.

### Combine multiple filters for stringent selection

**Args:** `--require-flags 2 --skip-flags 1024 --min-mapq 30 input.bam`

**Explanation:** Applies multiple filters simultaneously—properly paired, not duplicated, and high-quality mapping—creating a stringent read set for variant calling or expression analysis.