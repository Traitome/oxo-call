---
name: cistrome_beta
category: Transcription Factor Binding Site Query Tool
description: A command-line utility for querying and retrieving transcription factor (TF) and histone modification binding site data from the Cistrome database. Supports filtering by cell type, factor, genome assembly, and peak signal strength, with output in standard genomic interval formats such as BED and narrowPeak.
tags:
  - chip-seq
  - transcription-factor
  - binding-sites
  - epigenomics
  - genomics
  - peak-calling
author: AI-generated
source_url: https://cistrome.org/db/#/
---

## Concepts

- **Cistrome Database Structure**: The Cistrome database stores ChIP-seq peak data indexed by experiment, including metadata such as the target transcription factor (TF), cell type, tissue, and treatment condition. The tool queries this database via a remote API, meaning network connectivity is required and responses depend on the current database version.
- **Genome Assembly and Coordinate Systems**: All binding site coordinates are tied to a specific genome assembly (e.g., hg38, mm10). Coordinates retrieved with one assembly cannot be directly compared or merged with coordinates from another assembly without a liftover step; specifying the correct genome is essential for downstream analyses such as bedtools intersections or GREAT region annotation.
- **Output Formats**: The tool supports multiple standard genomic interval formats. The `narrowPeak` format (used by ENCODE) includes extra columns for signal value, p-value, q-value, and peak summit, while standard `BED` format provides chr, start, end, and an optional name column. The appropriate format choice depends on whether downstream tools (e.g., MACS2, deepTools) require enrichment scores or summit positions.
- **Filtering by Metadata and Signal Strength**: The tool allows filtering by TF name, cell type, experiment ID, and peak signal value thresholds. Filtering at the query level (using the `--cell-type` or `--signal-threshold` flags) is more efficient than post-filtering in-memory because it reduces the amount of data transferred from the API.

## Pitfalls

- **Incorrect or Ambiguous TF Name**: Transcription factor names must match exactly what is stored in the Cistrome database (e.g., "CTCF", not "ctcf", "CTCF ChIP-seq", or "CTCF_MYC"). A misspelled or case-mismatched TF name returns an empty result set with no error message, which can silently propagate into downstream analysis failures.
- **Forgetting to Specify Genome Assembly**: Without the `--genome` flag, the tool defaults to the most recent or most common assembly, which may not match the reference genome used in your downstream pipeline. Using hg19 coordinates with an hg38-aligned BAM file produces incorrect peak-to-gene assignments and false negatives in overlap analyses.
- **Requesting Excessive Data Without Pagination**: Large queries that retrieve thousands of binding site entries without specifying `--max-records` may exceed memory limits or encounter API timeout errors. On resource-constrained systems, this results in a crashed process with no partial output saved.
- **Assuming Uniform Peak Quality**: Not all experiments in Cistrome have uniform quality or sufficient depth. Peaks from low-quality experiments (e.g., low read depth, high background) may produce unreliable binding site calls. The `--min-signal` flag mitigates this but does not replace manual experiment quality inspection.
- **Ignoring Strand Orientation**: The `narrowPeak` format preserves strand orientation via the "天然" (natural) strand column, but standard BED format does not inherently encode strand information. Converting from narrowPeak to BED without explicitly handling the strand column causes loss of orientation information critical for certain analyses such as motif center analysis.

## Examples

### Query all CTCF binding sites for HepG2 cell line in BED format

**Args:** `query --factor CTCF --cell-type HepG2 --genome hg38 --format bed`
**Explanation:** Returns all CTCF peaks in the HepG2 cell line as a BED file with hg38 coordinates, suitable for direct lift-over or intersection with other hg38 datasets.

### Download narrowPeak data for EP300 with signal threshold filtering

**Args:** `query --factor EP300 --format narrowpeak --signal-threshold 10 --genome hg38`
**Explanation:** Downloads EP300 (p300 acetyltransferase) binding peaks with an enrichment signal value of at least 10, retaining p-value, q-value, and peak summit columns for downstream peak calling validation.

### Export binding sites for a specific experiment by experiment ID

**Args:** `query --experiment-id 47701 --format bed --genome hg38`
**Explanation:** Directly retrieves peak data from a single Cistrome experiment using its unique experiment ID, bypassing metadata search when the specific experiment is already known.

### Find all binding sites within a genomic region for SP1 transcription factor

**Args:** `query --factor SP1 --genome hg38 --region chr19:40000000-50000000 --format bed`
**Explanation:** Restricts the query output to peaks overlapping chr19:40-50Mb, reducing output size and enabling targeted analysis of a specific chromosomal locus without downloading the full dataset.

### Batch query multiple transcription factors for K562 cell line

**Args:** `query --factor-list factors.txt --cell-type K562 --genome hg38 --format narrowpeak --output-dir ./k562_peaks`
**Explanation:** Reads multiple TF names from a file and queries each sequentially, saving individual narrowPeak files to the specified output directory for integrative analysis across TFs in the same cell type.

### Retrieve histone modification H3K27ac data in bedGraph format

**Args:** `query --factor H3K27ac --format bedgraph --genome mm10 --cell-type Embryo --signal-threshold 5`
**Explanation:** Exports H3K27ac histone acetylation marks from embryonic samples in bedGraph format, a genome browser-compatible format with signal values for visualizing enhancer activity in single-cell or bulk ChIP-seq tracks.

### Check available metadata for a transcription factor before querying

**Args:** `info --factor ESR1 --show-experiments`
**Explanation:** Lists all experiments associated with ESR1 (estrogen receptor alpha), including cell types and publications, helping verify data availability before launching a potentially large or time-consuming query.