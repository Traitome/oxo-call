---
name: bird_tool_utils_python
category: Utility / Data Processing
description: A Python-based bioinformatics utility for processing and manipulating next-generation sequencing (NGS) data files, including format conversion, validation, filtering, and statistics generation for common bioinformatics formats like FASTA, FASTQ, SAM, BAM, VCF, and BED.
tags: [bioinformatics, ngs, sequence-analysis, data-processing, python-utility, format-conversion, quality-control]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bird_tool_utils_python
---

## Concepts

- **Multi-format Input/Output Support**: bird_tool_utils_python operates on standard bioinformatics file formats (FASTA, FASTQ, SAM, BAM, VCF, BED) with automatic format detection based on file extension and magic bytes. Use flags like `-i/--input` and `-o/--output` to specify paths; the tool automatically infers format when not explicitly declared.
- **Streaming and Chunked Processing**: For large files (e.g., >10GB BAM/VCF), the tool uses streaming iteration to avoid loading entire files into memory. This is critical for whole-genome-scale datasets; use `--chunk-size` to control buffer sizes (default 1000 records) and `--no-index` to disable index-dependent operations that require random access.
- **Index-Dependent Operations**: Certain operations (random access by coordinate, subsetting by genomic region) require corresponding index files (.bai, .csi for BAM; .tbi for VCF). The tool automatically looks for index files in the same directory with standard naming conventions; missing indices force sequential scanning which is significantly slower for large files.

## Pitfalls

- **Mismatched Index Files**: Specifying a BAM file without a corresponding .bai index when using region-based queries causes the tool to perform a full sequential scan instead of indexed retrieval. This can slow down processing by 10-100x on whole-genome files, and may cause memory issues on systems with limited RAM.
- **Format Auto-detection Failures**: The tool's automatic format detection relies on file extensions and magic bytes. Non-standard extensions (e.g., `sequences.txt` instead of `.fastq`) require explicit `--format` specification; otherwise the tool defaults to FASTA, corrupting FASTQ data during parsing.
- **Insufficient Disk Space for Output**: When converting between formats (especially BAM to FASTQ, which expands data ~4x), output to the same filesystem without checking available space causes partial writes and corrupted output files, requiring complete re-processing.
- **Mixed Compression States**: The tool expects consistent compression within input files. Gzip-compressed and uncompressed FASTA records mixed in a single file will cause parsing errors; pre-process with `seqtk` or `bgzip` to normalize compression before using bird_tool_utils_python.

## Examples

### Convert FASTQ to FASTA format
**Args:** `convert -i reads.fastq -o reads.fasta --input-format fastq --output-format fasta`
**Explanation:** This converts a FASTQ file to FASTA by stripping quality scores, useful for tools that accept FASTA input only.

### Validate BAM file integrity
**Args:** `validate input.bam --strict`
**Explanation:** Performs thorough validation of BAM file structure, including checking SAM header validity, record-level CRC checks, and coordinate sorting order. The `--strict` flag enforces additional checks that may reject technically valid but suboptimal files.

### Extract reads mapping to a specific genomic region
**Args:** `filter -i alignment.bam -o region_reads.bam --chromosome chr1 --start 1000000 --end 2000000`
**Explanation:** Extracts all reads overlapping the specified genomic interval using indexed retrieval when available, producing a smaller BAM file for targeted analysis.

### Generate sequencing coverage statistics
**Args:** `stats alignment.bam --coverage --per-chromosome`
**Explanation:** Computes read depth and coverage statistics across the entire alignment, broken down by chromosome; outputs JSON-formatted summary for downstream integration.

### Count records in a compressed VCF file without decompressing
**Args:** `count -i variants.vcf.gz --filter "PASS == 1"`
**Explanation:** Uses bgzip block-level iteration to count VCF records matching the expression without full decompression, significantly faster for large files; supports basic filter expressions for common fields.

### Convert between genomic interval formats
**Args:** `convert -i peaks.bed -o peaks.tsv --input-format bed --output-format tsv`
**Explanation:** Converts BED format genomic intervals to tab-separated values, stripping BED-specific columns (name, score, strand) to a simplified coordinate table.

### Subsample reads at a specified fraction
**Args:** `sample -i reads.fastq.gz -o subsampled.fastq.gz --fraction 0.1 --seed 42`
**Explanation:** Randomly selects 10% of reads from the input FASTQ file using a reproducible seed for consistent sampling across re-runs; maintains original quality score encoding.