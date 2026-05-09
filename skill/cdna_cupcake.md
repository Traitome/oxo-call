---
name: cdna_cupcake
category: Bioinformatics - Circular RNA Analysis
description: A bioinformatics toolkit for analyzing and processing circular RNA (circRNA) and cDNA sequences from long-read sequencing data (PacBio, Oxford Nanopore). Provides filtering, clustering, collapsing, and reporting functionalities for isoform-level analysis.
tags:
  - circRNA
  - circular RNA
  - long-read sequencing
  - isoform
  - TAMA
  - PacBio
  - Nanopore
  - transcriptomics
author: AI-generated
source_url: https://github.com/TAMA-Server/cdna_cupcake
---

## Concepts

- **Input Format**: cdna_cupcake accepts BED6+ format files (minimum 6 columns with start/stop coordinates) for isoform definitions, often generated from long-read alignment pipelines. The tool also works with SAM/BAM files for sequence-aware filtering.
- **Isoform Clustering**: The tool groups overlapping isoforms based on genomic coordinates and optional sequence identity, collapsing redundant transcripts that represent the same circular RNA molecule using configurable merge criteria.
- **Filtering Parameters**: Key filters include minimum read count (`-c`), minimum isoform length (`-l`), and maximum allowed gap (`-g`). Filtered results can be output as BED files or summary reports for downstream analysis.
- **Output Modes**: cdna_cupcake supports multiple output formats including filtered BED files for genomic visualization, collapsed FASTA for sequence export, and cluster reports showing group membership and representative isoforms.
- **Companion Binaries**: The toolkit includes `cdna_cupcake-build` for pre-computing indexes to accelerate filtering on large datasets, and `cdna_cupcake-merge` for combining results from multiple samples.

## Pitfalls

- **Using Non-Circular Coordinates**: cdna_cupcake expects genomic coordinates that represent circular RNA structure (back-spliced junctions). Using standard linear RNA coordinates will produce incorrect clusters as the tool assumes circular connectivity between genomic ends.
- **Ignoring Strand Information**: Failing to specify the correct strand (`+` or `-`) leads to incorrect isoform grouping, especially when analyzing antisense circRNA or overlapping genes on opposite strands.
- **Setting Threshold Too Low**: Using overly permissive minimum coverage thresholds (`-c 1`) results in spurious isoforms being retained, inflating downstream analysis with low-confidence false positives.
- **Incompatible Input BED Versions**: Using BED files without the required metadata columns (gene_id, transcript_id, strand) will cause parsing failures. Ensure input files conform to BED6+ or proper TAMA format specifications.
- **Memory on Large Datasets**: Processing whole-genome isoform files without first building indexes can cause memory exhaustion. For datasets with >100,000 isoforms, pre-build indexes using `cdna_cupcake-build` before filtering.

## Examples

### Filter isoforms by minimum read count

**Args:** `-i input_isforms.bed -o filtered_output.bed -c 5`
**Explanation:** This removes all isoforms supported by fewer than 5 reads, keeping only high-confidence circular RNA predictions in the output BED file.

### Collapse redundant isoforms into representative clusters

**Args:** `-i iso_cluster.bed -o collapsed.bed --cluster`
**Explanation:** Groups overlapping isoforms into single representative entries, outputting a BED file with unified cluster memberships for downstream annotation.

### Filter by minimum isoform length

**Args:** `-i input.bed -o min_length.bed -l 200`
**Explanation:** Retains only isoforms of at least 200 nucleotides in length, filtering out short artifacts or partial circRNA predictions.

### Build index for large dataset processing

**Args:** `cdna_cupcake-build -i large_isoforms.bed -o isoform_index.idx`
**Explanation:** Pre-computes spatial indexing structures on the input BED file to enable faster subsequent filtering operations.

### Export filtered isoforms to FASTA format

**Args:** `-i filtered_isoforms.bed -o isoforms.fasta --fasta`
**Explanation:** Converts the filtered BED entries to FASTA format for sequence export, enabling downstream sequence analysis or database searches.

### Apply maximum gap tolerance for isoform merging

**Args:** `-i input.bed -o merged.bed -g 50 --merge`
**Explanation:** Merges isoforms separated by gaps up to 50 nucleotides, accommodating minor coordinate variations in circular RNA detection.

### Generate cluster summary report

**Args:** `-i clustered.bed -o summary.txt --report`
**Explanation:** Outputs a text report showing cluster composition, member counts, and representative isoform information for quality assessment.