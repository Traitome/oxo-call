---
name: bamstats
category: alignment_quality_assessment
description: A bioinformatics tool that calculates comprehensive statistics from BAM/SAM alignment files, including read length distribution, mapping quality metrics, coverage summary, and alignment breakdown by flag categories.
tags: [bam, sam, alignment-stats, quality-control, sequencing]
author: AI-generated
source_url: https://github.com/igordot/bamstats
---

## Concepts

- **Input Format**: Accepts both BAM (binary) and SAM (text) alignment files; BAM input is automatically detected by the `.bam` extension or binary magic bytes; use stdin with piping for streaming analysis.
- **Output Modes**: Generates JSON output (`-j/--json`) for programmatic parsing, plain text summary (`-t/--text`) for human review, or both simultaneously by specifying multiple output paths; default output is text format.
- **Statistics Scope**: Calculates metrics including total/filtered read counts, mapping quality distribution, read length histogram,flag category breakdown (properly paired, duplicates, secondary alignments), and per-chromosome coverage summary.
- **Filtering Options**: Supports inline filtering via `-f/--filter` using SAMtools-style filter expressions (e.g., `mapq >= 30 AND !duplicate`) to include only specific reads in statistics; filtering is applied before metric calculation.
- **Flag Decomposition**: Automatically decodes SAM flags into readable categories (paired, properly paired, mapped, unmapped, reverse strand, mate reverse strand, first mate, second mate); reported in both raw counts and percentages.

## Pitfalls

- **Empty BAM Files**: Running on an empty BAM file or a file with only header lines produces zero-count statistics without warning; always verify input file contains aligned reads using `samtools view -c` before expensive full analysis.
- **Misaligned Flag Interpretation**: Flag categories are mutually exclusive in reporting but overlapping in raw flags; the tool correctly handles this by assigning each read to its highest-priority category, but users may misinterpret "unmapped" counts if reads have multiple flag issues.
- **Large File Memory Consumption**: Reading entire BAM into memory for sorting or complex filtering can exceed available RAM on whole-genome files; use streaming mode or pre-sort by coordinate with `samtools sort` to reduce memory footprint.
- **Index Dependency**: Index file (`.bai`) is required for chromosome-specific or region-restricted statistics; without it, the tool may fall back to scanning the entire file or fail with an index-related error on some implementations.
- **Duplicate Marking Assumption**: Not all BAM files have duplicate marks (`0x400` flag); statistics showing "0 duplicates" may indicate the file was never processed by a duplicate marker tool, not that no duplicates exist.

## Examples

### Generate basic statistics for a BAM file in text format
**Args:** -i alignments.bam -o stats.txt
**Explanation:** Reads the BAM file and outputs a human-readable text report containing mapping quality distribution, read length histogram, and flag breakdown to the specified output file.

### Export statistics as JSON for programmatic parsing
**Args:** -i alignments.bam -j -o stats.json
**Explanation:** Generates JSON-formatted output compatible with automated pipelines, allowing easy integration with downstream analysis scripts or visualization tools.

### Filter to high-quality mappings only (MAPQ >= 30)
**Args:** -i alignments.bam -f "mapq >= 30" -o highq_stats.txt
**Explanation:** Applies a filtering expression before calculating statistics, including only reads with mapping quality of 30 or higher in the final metrics; useful for assessing confident alignments.

### Output both text and JSON formats in a single run
**Args:** -i alignments.bam -o stats.txt -j -o stats.json
**Explanation:** Generates dual output formats simultaneously by specifying two output file paths, preventing the need to re-read the BAM file for different format requirements.

### Calculate statistics only for a specific chromosome
**Args:** -i alignments.bam -o chr1_stats.txt -c chr1
**Explanation:** Restricts analysis to a single chromosome ("chr1"), significantly reducing runtime and memory usage compared to whole-genome analysis when only specific chromosome metrics are needed.