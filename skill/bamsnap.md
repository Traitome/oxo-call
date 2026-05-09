---
name: bamsnap
category: visualization
description: A tool for generating publication-quality visualizations from BAM/SAM alignment files, including coverage plots, read density displays, and genomic region snapshots.
tags: [bam, sam, visualization, coverage, alignments, genomics, ngs]
author: AI-generated
source_url: https://github.com/BradleyE/NGS-utilities
---

## Concepts

- Bamsnap operates on sorted and indexed BAM files paired with a reference genome to render genomic regions as pixel images, supporting zoom levels from base-pair resolution to whole-chromosome views.
- The tool accepts genomic interval inputs via chromosome:start-end notation or BED files, and overlays multiple signal tracks (coverage, alignments, gene annotations) in configurable colors and styles.
- Output formats default to PNG with optional SVG or PDF; images embed positional metadata (coordinates, read depth scale, reference bases) as overlays or legends for downstream figure assembly.
- Bamsnap infers sequencing technology (Illumina, PacBio, ONT) from BAM tags and adjusts read rendering (forward/reverse strand colors, soft-clip display) accordingly.
- Multi-sample mode accepts a manifest file listing multiple BAM paths and outputs either per-sample panels or a stacked lane view for comparative visualization.

## Pitfalls

- Providing an unsorted or unindexed BAM file causes bamsnap to fail silently or produce truncated images, because the underlying iterator assumes random-access by genomic coordinate.
- Omitting the required `-ref` parameter (reference genome FASTA) results in read sequences being displayed as gray blocks with no reference base calls, making alignments uninterpretable.
- Specifying genomic coordinates that exceed the chromosome length in the header (e.g., a BAM with chr1 but querying chr1_GL000229v1) produces empty output with no error message, wasting analysis time.
- Using incompatible color schemes across multiple tracks can render output visually confusing and unsuitable for publication; always verify contrast ratios in the generated image.
- Running bamsnap in parallel on the same output filename results in file corruption or partial writes, because the tool does not implement atomic write operations.

## Examples

### Display a genomic region with default coverage and alignment tracks
**Args:** -bam sample1.bam -ref hg38.fa -region chr7:117250-117500 -out sample1_region.png
**Explanation:** This renders the specified 250 bp window of chromosome 7 with default coverage histogram and read alignment glyphs overlaid on the reference sequence.

### Generate a multi-sample sashimi-style splice junction plot for a gene
**Args:** -bam tumor.bam normal.bam -ref hg38.fa -gene TP53 -mode sashimi -out tp53_sashimi.png
**Explanation:** This compares splice junction read density between tumor and normal samples across the TP53 gene locus, with junction arc thickness proportional to read count.

### Create a publication-ready zoomed view with reverse-strand coloring
**Args:** -bam reads.bam -ref hg38.fa -region chr12:102350-102650 -strand_colors -out zoomed_strand.png
**Explanation:** This produces a zoomed view where forward and reverse reads are colored distinctly (default blue/red), making strand-specific expression patterns visible.

### Overlay copy number calls from a BED file onto coverage
**Args:** -bam sample.bam -ref hg38.fa -region chr8:125000-145000 -bed cnv_calls.bed -out cnv_overlay.png
**Explanation:** This superimposes copy number segments from the BED file as colored bars above the coverage track, enabling direct visual correlation of CNV regions.

### Export a high-resolution PDF figure for journal submission
**Args:** -bam experiment.bam -ref hg38.fa -region chr3:198000-199500 -format pdf -out highres_figure.pdf
**Explanation:** This generates a vector PDF suitable for journal figures, preserving crisp text and line rendering at any zoom level without pixelation.

### Process multiple samples via manifest for comparative visualization
**Args:** -manifest samples.txt -ref hg38.fa -region chr22:23500000-23600000 -out cohort_view.png
**Explanation:** This reads a two-column manifest file (sample ID and BAM path per line) and renders all samples in stacked lanes, facilitating cohort-level quality assessment.