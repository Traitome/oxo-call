---
name: chromoplot
category: genomic-visualization
description: A command-line tool for generating linear and circular chromosome plots from genomic coordinate data, commonly used for visualizing genomic segments, copy number alterations, synteny blocks, and gene annotations across one or more reference genomes.
tags:
  - visualization
  - chromosomes
  - genomics
  - coverage-plots
  - synteny
  - copy-number
  - BED
  - CSV
author: AI-generated
source_url: https://github.com/lastern/chromoplot
---

## Concepts

- **Chromosomal coordinate system**: chromoplot maps data points onto a linear or circular chromosome ideogram using genomic start and end positions. Coordinates must be 0-based or 1-based consistently; mismatched indexing silently shifts plotted features by one base pair.
- **Multi-track layering**: data tracks (coverage WIG files, gene annotations BED, copy number CSV) are stacked vertically on separate lanes above or below the central axis. Each track has independent y-axis scaling, and track order is determined by the CLI argument sequence rather than data content.
- **Supported input formats**: chromoplot accepts tab-delimited BED files for feature annotations, comma-separated CSV for copy number or expression data, and UCSC WIG/BigWig for continuous signal coverage. Header rows are required for CSV but forbidden in BED; a missing or mislabeled header silently produces empty output.
- **Output formats and resolution**: renders to PNG (default, 300 DPI), SVG (vector, scalable), and PDF (vector for publication). PNG resolution is controlled by `--width` and `--height` in pixels; increasing these values linearly increases output file size without improving biological accuracy.
- **Circular plots (circos-style)**: the `--circular` flag inverts the layout to a circular ideogram and routes tracks as concentric rings. When mixing linear and circular modes in a pipeline, chromosome name fields must exactly match — `chr1` and `chr1` are distinct identifiers.

## Pitfalls

- **Off-by-one coordinate errors**: BED files use 0-based half-open intervals `[start, end)`, while most genomic databases (NCBI, Ensembl) use 1-based closed intervals `[start, end]`. Passing database coordinates directly into a BED-formatted input without conversion shifts every plotted feature by one base pair, causing misalignment that is visually difficult to detect at low zoom.
- **Missing chromosome labels**: If chromosome names in the input data do not exactly match the `--genome` reference build names (e.g., `chr1` vs `1` vs `chr01`), chromoplot silently skips the entire track. This produces a plot without报错, leading users to suspect a rendering bug rather than a data mismatch.
- **Memory exhaustion with large WIG files**: WIG/BigWig files spanning whole-genome coverage at base-pair resolution can exceed available RAM when loaded into chromoplot's default track buffer. The `--bin-size` parameter must be used to downsample input signal; failing to do so causes a crash with a non-descriptive `Killed` message.
- **Overlapping track regions with opaque fills**: When two tracks share genomic coordinates and both use default fill colors, the second track's opaque fill obscures the first, silently hiding data. Use `--alpha` on either track or assign non-overlapping genomic ranges explicitly.
- **Incorrect delimiter interpretation in CSV**: chromoplot auto-detects delimiters by default, but semicolon-delimited CSV files may be misparsed as single-column data when `--format csv` is not explicitly specified, producing garbled or empty plots.

## Examples

### Generate a basic linear chromosome plot for human chromosome 1 from a BED annotation file
**Args:** `--genome hg38 --chromosome chr1 --input genes.bed --output linear_chr1.png`
**Explanation:** The `--genome hg38` flag loads the correct chromosome length and naming conventions for build GRCh38, `--chromosome chr1` restricts rendering to chromosome 1, and `--input genes.bed` provides the feature annotation track.

### Create a circular multi-track plot comparing copy number and gene coverage across all autosomes
**Args:** `--circular --genome hg38 --input cnv.csv --track cov.wig --output circos_autosomes.png --chromosomes chr1 chr2 chr3 chr4 chr5 chr6 chr7 chr8 chr9 chr10 chr11 chr12 chr13 chr14 chr15 chr16 chr17 chr18 chr19 chr20 chr21 chr22`
**Explanation:** The `--circular` flag switches the layout to concentric rings, the explicit `--chromosomes` list includes all autosomes while omitting sex chromosomes, and two separate input tracks are layered as rings in the order specified.

### Render a high-DPI publication-ready PNG plot with downsampled signal for a whole-genome WIG file
**Args:** `--genome hg38 --input coverage.wig --output highres_genome.png --width 6000 --height 2000 --bin-size 10000`
**Explanation:** The `--bin-size 10000` flag aggregates signal into 10 kb windows to reduce memory usage, while `--width 6000 --height 2000` sets a resolution sufficient for print-quality output without exceeding RAM limits.

### Overlay two BED tracks with transparency to visualize gene exons and regulatory elements simultaneously
**Args:** `--genome hg38 --chromosome chr12 --input exons.bed --track promoters.bed --alpha 0.4 --output overlay_chr12.png`
**Explanation:** The `--alpha 0.4` flag applies 60% transparency to the second track so that overlapping regions with the first track remain visible, and explicit `--chromosome chr12` restricts rendering to a single chromosome for clear inspection.

### Export a vector SVG plot for downstream graphic editing from a CSV copy number file with a header row
**Args:** `--genome hg19 --input cnv_data.csv --format csv --output cnv_vector.svg --chromosomes chrX`
**Explanation:** The `--format csv` explicitly sets the parser to handle comma-separated input, `--genome hg19` uses the correct build naming convention, and `--chromosomes chrX` isolates the X chromosome for a focused copy number alteration view.