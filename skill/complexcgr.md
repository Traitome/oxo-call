---
name: complexcgr
category: Sequence Visualization
description: Generate complex Chaos Game Representation (CGR) plots from biological sequences (DNA, RNA, or protein) with support for multiple visualization formats and pattern analysis.
tags: [bioinformatics, sequence-visualization, CGR, genomics, fractal-analysis]
author: AI-Generated
source_url: https://github.com/bioinformatics-tools/complexcgr
---

## Concepts

- **Sequence Encoding**: ComplexCGR maps each residue (A, C, G, T for DNA; amino acids for proteins) to unique positions on a 2D square by iteratively halving distances from vertices, creating distinctive fractal-like patterns that encode sequence compositional information visually.

- **Input Format Support**: The tool accepts raw sequences via stdin or file input in FASTA format (single or multiple sequences per file), plain text format (one sequence per line with header), and GenBank flatfile format with automatic sequence extraction from annotations.

- **Multi-Output Rendering**: Generated CGR plots can be exported as PNG (with configurable DPI: 72, 150, 300), SVG (vector graphics for publication), PDF (high-resolution printing), and ASCII text matrices for terminal-based inspection.

- **Pattern Significance**: The resulting CGR patterns reveal sequence biases, repeat structures, and compositional anomalies that appear as偏离 (deviations) from expected diagonal patterns or clustering near specific vertices representing over-represented residues.

## Pitfalls

- **Unsorted Input Sequences**: When processing multiple sequences from a FASTA file, the tool processes them in file order rather than sequence length or name order. Mixing short reads with full chromosomes in one run causes memory spikes; process full genomes separately from fragmented input.

- **Missing Sequence Validation**: The tool does not auto-detect molecule type (DNA vs. protein vs. RNA); passing DNA sequences with T nucleotides when the tool defaults to RNA mode produces invalid coordinate calculations and blank output without warnings.

- **Insufficient Memory for Large Sequences**: Sequences exceeding 10MB without the `--chunk-size` flag cause out-of-memory errors on systems with less than 8GB RAM. Long genomes (chromosomes >50Mbp) require explicit memory allocation via `--memory-limit` parameter.

- **Coordinate System Mismatch**: The `--bounds` parameter uses normalized coordinates (0.0-1.0), not raw sequence indices; specifying bounds >1.0 silently clips to 1.0, producing truncated visualizations that appear complete but miss peripheral data points.

## Examples

### Generate basic DNA CGR plot from FASTA file
**Args:** `input sequences.fasta --output sequence_cgr.png --format png`
**Explanation:** Reads DNA sequences from a FASTA file and renders a CGR plot as a PNG image at default 72 DPI resolution.

### Create publication-quality SVG with 300 DPI resolution
**Args:** `input chromosome1.fa --output cgr_publication.svg --format svg --dpi 300`
**Explanation:** Exports vector-quality SVG suitable for journal figures, rendered at 300 DPI for high-resolution printing.

### Process multiple sequences with automatic naming
**Args:** `input all_genes.fa --format png --dpi 150 --prefix gene_cgr_`
**Explanation:** Generates individual CGR plots for each sequence in the input file using original sequence headers as filenames, suitable for batch analysis.

### Analyze protein sequence with custom color scheme
**Args:** `input protein.fasta --output prot_cgr.png --moltype protein --palette hydrophobic`
**Explanation:** Interprets amino acid sequences with color-coding that highlights hydrophobic residue clustering patterns for structural analysis.

### Extract and visualize specific genomic region
**Args:** `input genome.gb --output region_cgr.png --start 1000 --end 5000 --sequence-id gene_abc`
**Explanation:** Extracts nucleotides from positions 1000-5000 of a named feature in a GenBank file and generates a focused CGR plot of that interval.