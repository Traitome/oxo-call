---
name: canvas
category: visualization
description: A genome browser and visualization tool for displaying genomic data, annotations, and variants in a canvas-based graphical interface. Supports multiple data formats including BAM, BED, VCF, and FASTA for integrated genomic analysis.
tags: [genomics, visualization, browser, annotations, variants, canvas]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/canvas
---

## Concepts

- **Canvas data model:** Operates on genomic coordinates (chromosome:start-end) and supports multiple annotation tracks. Each track contains features with defined genomic intervals, scores, and optional metadata fields.
- **Supported input formats:** Reads BAM/SAM for read alignments, BED/GTF for genomic features, VCF for variants, and FASTA/FASTQ for sequences. All inputs must be indexed with corresponding index files (.bai, .tbi, .fai) for random access.
- **Track types:** Supports line, bar, scatter, heatmap, and bridge tracks for displaying different data modalities. Tracks can be stacked vertically with configurable height and color schemes.
- **Rendering behavior:** Uses a canvas-based HTML5 rendering engine for interactive visualization. Regions are rendered on-demand based on viewport coordinates, with smooth pan and zoom interactions.
- **Coordinate system:** Uses 0-based half-open intervals for BED-like formats and 1-based closed intervals for VCF. Automatically converts between coordinate systems when exporting or comparing across formats.

## Pitfalls

- **Missing index files:** Running canvas without pre-indexed BAM/VCF files causes random access failures. Always generate index files using tools like `samtools index` for BAM and `tabix` for VCF before visualization.
- **Coordinate mismatches:** Confusing 0-based vs 1-based coordinate systems leads to off-by-one errors in feature placement. Verify the coordinate system of your input format before interpretation.
- **Large region queries:** Attempting to render entire chromosomes without zooming crashes the rendering engine. Start with smaller window sizes (e.g., 10kb-100kb) and zoom in progressively.
- **Incompatible track heights:** Setting excessive track heights with many features causes memory overflow. Limit visible features per track and enable pagination for dense annotations.
- **Font rendering issues:** Custom fonts not embedded in the canvas bundle display as default fallback. Ensure all custom annotations use standard web-safe fonts for consistent rendering.

## Examples

### Display a genomic region from a BAM file
**Args:** `--bam sample.bam --region chr1:100000-200000 --track type=line`
**Explanation:** Loads alignments from the specified BAM file and displays read depth as a line track for the given genomic interval.

### Overlay multiple VCF variant tracks
**Args:** `--vcf known_variants.vcf --vcf novel_calls.vcf --track type=bar --colors "#FF0000,#0000FF"`
**Explanation:** Displays two VCF files as stacked bar tracks with distinct colors for comparing known and novel variant calls.

### Export visualization as PNG image
**Args:** `--region chr3:500000-600000 --output final_plot.png --width 1200 --height 800`
**Explanation:** Renders the specified genomic region to a high-resolution PNG file for publication figures.

### Configure track labels and groupings
**Args:** `--bed annotations.bed --track type=bridge --label --group "Gene_Annotations"`
**Explanation:** Displays BED features as bridge tracks with visible labels organized under a named grouping in the sidebar.

### Interactive genome browsing session
**Args:** `--bam align.bam --vcf variants.vcf --bed genes.bed --interactive`
**Explanation:** Opens an interactive canvas session with multiple data tracks enabled for real-time pan and zoom exploration.