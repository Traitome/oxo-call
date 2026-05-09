---
name: blastn2dotplots
category: sequence_analysis/visualization
description: Generate dot plot visualizations from BLASTN alignment results. Accepts BLAST XML, tabular, or CSV output and produces publication-quality dot plots showing sequence similarity relationships.
tags:
  - blast
  - dotplot
  - visualization
  - sequence-alignment
  - pairwise-comparison
author: AI-generated
source_url: https://github.com/ blastn2dotplots
---

## Concepts

- **Input Formats**: The tool accepts BLAST output in multiple formats: XML (`-outfmt 0`), tabular (`-outfmt 6`), or CSV (`-outfmt 7`). The XML format contains the richest metadata including alignment scores, e-values, and subject sequence positions essential for accurate dot plot rendering.

- **Dot Plot Data Model**: A dot plot visualizes all-against-all sequence comparisons where each point represents a matching base or high-scoring segment pair (HSP) between two sequences. The X-axis represents the query sequence and the Y-axis represents the subject/target sequence.

- **Output Image Formats**: Generated dot plots can be exported as PNG (default raster), PDF (vector for publication), or SVG (scalable vector for further editing). Resolution for raster outputs is controlled via the `--dpi` flag, with 300 DPI suitable for print and 72 DPI sufficient for screen viewing.

- **Score Filtering**: Use the `--min-score` and `--max-evalue` flags to filter alignments based on alignment bit-score or expect value. This reduces visual clutter by hiding weak or non-significant matches that would otherwise obscure meaningful relationships in the dot plot.

## Pitfalls

- **Missing Sequence Length Information**: If the input BLAST file does not contain subject sequence lengths (common in older XML outputs or incomplete tabular formats), the dot plot Y-axis scale will be inaccurate, leading to misleading visual representations of sequence coverage and alignment density.

- **Consequence**: The resulting dot plot will have compressed or stretched subject sequence representation, making it impossible to accurately assess alignment coverage or identify structural variations like rearrangements or inversions.

- **Incompatible Input Format**: Attempting to parse BLAST output generated with incompatible flags (e.g., `-outfmt 0` XML without sequence data, or `-outfmt 1` plain text) will cause parsing failures because the tool expects specific column structures or XML elements.

- **Consequence**: The tool will exit with a parsing error without generating any output, wasting computation time and requiring users to regenerate the BLAST output with appropriate flags before retrying.

- **Memory Exhaustion with Large Datasets**: For genome-scale BLAST results containing millions of HSPs, default buffer settings may cause memory exhaustion during dot plot rendering.

- **Consequence**: The process terminates abnormally with an "out of memory" error; use the `--batch-size` flag to process alignments in chunks or increase available system memory.

- **Overlapping Points at High Identity Regions**: When two sequences share extensive regions of high similarity (e.g., duplicated genes or very close homologs), individual alignment points overlap, creating a solid black block that obscures alignment details.

- **Consequence**: Users cannot distinguish between multiple independent HSPs in repetitive regions; apply the `--point-size` reduction or use `-- HSP-filter` to merge overlapping points analytically.

## Examples

### Generate a basic dot plot from BLAST XML output
**Args:** `--input query_vs_ref.xml --output dotplot_basic.png`
**Explanation:** Creates a simple PNG dot plot at default 72 DPI using all alignments in the XML file without filtering, suitable for quick visual inspection of overall similarity.

### Create a publication-ready PDF dot plot with filtering
**Args:** `--input alignments.xml --output publication_dotplot.pdf --format pdf --min-score 50 --max-evalue 0.001`
**Explanation:** Generates a vector PDF dot plot including only alignments with bit-score >= 50 and e