---
name: caspeak
category: Genomics / Peak Analysis
description: A UCSC-style tool for analyzing and manipulating genomic peak data (such as ChIP-seq peaks). Supports peak filtering, annotation overlap, and track comparison operations on BED/narrowPeak formats.
tags:
  - genomics
  - peaks
  - chip-seq
  - bed
  - narrowpeak
  - annotation
  - ucsc
author: AI-generated
source_url: https://genome.ucsc.edu
---

## Concepts

- **Input Formats**: caspeak accepts standard BED files and narrowPeak/broadPeak formats commonly produced by peak callers like MACS. The first three columns (chrom, start, end) are required; additional columns provide peak scores and signal values.
- **Overlap Operations**: The tool performs interval-based overlap detection between peak files and annotation sets (e.g., gene databases, repeat masks), computing statistics such as overlap fraction and nearest feature distance.
- **Filtering and Scoring**: Peaks can be filtered by signal intensity (p-value, q-value, fold enrichment), peak width, or genomic context (promoter, intron, intergenic). Filtering is applied via numeric thresholds in column-specific flags.
- **Output Formats**: Results are written in BED format with optional annotation columns appended; stdout produces standard genome coordinates suitable for downstream genome browser visualization.

## Pitfalls

- **Column Index Errors**: Specifying the wrong column index for filtering (e.g., using `-c` for q-value when column 8 contains p-value) silently produces empty or incorrect output without warning — verify column assignments beforehand.
- **Mismatched Chromosome Names**: Using a peak file with `chr1` alongside an annotation file with `1` (numeric) results in zero overlaps — ensure both files use identical chromosome naming conventions.
- **Unstranded Input**: Overlapping peaks with strand-specific annotations (e.g., TSS from RefGene) without specifying strand causes false positive associations, as both + and - strand features are matched regardless of orientation.
- **Memory Limits on Large Files**: Processing genome-wide peak files with millions of lines without chunking (`--chunk-size`) can exhaust RAM on systems with limited memory — process in batches for files exceeding available memory.

## Examples

### Filter peaks by q-value threshold
**Args:** `-i peaks.narrowPeak -q 0.01`
**Explanation:** Retains only peaks with q-value (column 9 in narrowPeak) less than or equal to 0.01, removing non-significant calls from downstream analysis.

### Find overlaps with RefSeq genes
**Args:** `-a peaks.bed -b refseq.bed -overlap`
**Explanation:** Identifies peaks that physically overlap with RefSeq gene annotations, reporting both the number of overlapping peaks and the genomic regions affected.

### Determine nearest TSS distance for each peak
**Args:** `-a peaks.bed -b tss_positions.bed -d`
**Explanation:** Calculates the distance from each peak center to the nearest transcription start site, appending this metric as an additional column in the output.

### Restrict overlaps to same strand
**Args:** `-a chip_peaks.bed -b tss.bed -s`
**Explanation:** Overlaps peaks only when both features are on the same genomic strand, preventing artifactual associations between peaks and downstream-oriented TSSs.

### Export peaks in BED6 format only
**Args:** `-i peaks.narrowPeak -cols 6 -o filtered_peaks.bed`
**Explanation:** Converts narrowPeak to minimal BED6 format (chrom, start, end, name, score, strand), stripping extra columns for compatibility with tools expecting standard BED.

### Chunk-process a large genome-wide peak file
**Args:** `-i huge_peaks.bed --chunk-size 500000 -o chunk_out/`
**Explanation:** Processes the input in 500,000-line memory-efficient batches, writing each chunk to a separate output file in the specified directory to avoid memory exhaustion.