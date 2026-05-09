---
name: aci
category: Sequence Analysis
description: A bioinformatics tool for analyzing sequence data, computing coverage metrics, and generating alignment statistics. Supports SAM, BAM, and FASTQ input formats.
tags: [sequence-analysis, coverage, bioinformatics, sam, bam]
author: AI-generated
source_url: https://github.com/example/aci
---

## Concepts

- **Input formats**: aci accepts SAM, BAM, and FASTQ files as primary input. BAM files must be indexed for random access. The tool automatically detects format based on file extension (.sam, .bam, .fastq, .fq).
- **Coverage computation**: The tool computes per-base and region-based coverage depth. For genomic regions, use 1-based inclusive coordinates matching standard BED format.
- **Output modes**: Results can be output in text, JSON, or tab-delimited formats. The default output is human-readable text with optional machine-readable JSON for pipeline integration.
- **Multi-threading**: aci uses multiple threads for large file processing via the `-t` flag. Thread count defaults to 1; specifying 0 enables auto-detection based on available CPU cores.

## Pitfalls

- **Unindexed BAM files**: Running aci on unindexed BAM files causes severe performance degradation and may fail completely for genome-wide analyses. Always index BAM files with `samtools index` before processing.
- **Coordinate system mismatch**: Using 0-based half-open coordinates (common in BED files) instead of 1-based inclusive coordinates results in off-by-one errors in coverage reporting. Always verify coordinate system before specifying genomic regions.
- **Memory limits on large genomes**: Processing whole-genome BAM files without specifying region constraints can exhaust available memory. Use the `-r` flag to restrict analysis to specific regions for large files.
- **Invalid input encoding**: aci requires UTF-8 encoded input files. Files with alternate encodings (ISO-8859-1, Windows-1252) will produce mangled output or silent failures.

## Examples

### Compute coverage for a specific genomic region
**Args:** `-i input.bam -r chr1:1000-5000`
**Explanation:** This runs coverage analysis on chromosome 1 from position 1000 to 5000 using the specified BAM file. The coordinates are 1-based inclusive.

### Process multiple FASTQ files with JSON output
**Args:** `-i sample1.fastq sample2.fastq -o results.json -f json`
**Explanation:** This analyzes multiple FASTQ input files and outputs results in JSON format for downstream pipeline processing.

### Use 4 threads for parallel processing
**Args:** `-i large_sample.bam -t 4`
**Explanation:** This enables multi-threaded processing with 4 worker threads, significantly speeding up analysis of largealignments.

### Generate coverage BEDGRAPH file
**Args:** `-i input.bam -o coverage.bg -F bedgraph`
**Explanation:** This outputs coverage data in BEDGRAPH format, which displays per-base coverage values suitable for visualization in genome browsers.

### Specify minimum base quality threshold
**Args:** `-i input.bam -r chr3:500-2000 -q 20`
**Explanation:** This filters alignments to only include bases with quality scores of 20 or higher when computing coverage in the specified region.