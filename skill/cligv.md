---
name: cligv
category: Bioinformatics
description: A command-line tool for genome visualization and interactive exploration of genomic data. cligv enables rapid viewing, filtering, and annotation of genomic intervals with support for multiple input formats including BED, VCF, and SAM.
tags: [genomics, visualization, interval-matching, bioinformatic-tools, command-line]
author: AI-generated
source_url: https://github.com/cligv/cligv
---

## Concepts

- **Input Formats**: cligv accepts standard genomic file formats including BED (0-based start), VCF, and SAM. The tool auto-detects format based on file extension (.bed, .vcf, .sam). GFF/GTF files require explicit specification via the `--format gff` flag.
- **Interval Operations**: cligv performs interval overlap detection using a sliding window approach. The `-w` flag controls window size in base pairs; default is 100bp. Overlapping features are merged automatically when using the `-m` merge flag.
- **Output Rendering**: Output is rendered in ASCII format by default. Use `--output graphical` for SVG generation. The `--width` flag controls output column width (default 120 characters). The `--compact` flag reduces spacing between features.
- **Annotation Layers**: Multiple annotation tracks can be loaded using separate `--track` flags. Track order determines vertical stacking in visualization. The `--color-by` flag enables color-coding by feature score or custom annotation field.

## Pitfalls

- **Zero-Based vs One-Based Coordinates**: BED files use zero-based start coordinates while SAM and VCF use one-based coordinates. Mixing formats without conversion produces off-by-one errors in overlap detection. Always verify coordinate system matches your reference genome build.
- **Missing Header Lines**: VCF files without required header lines (##fileformat, #CHROM) cause parsing failures. cligv exits with error code 2 when headers are malformed. Use `bgzip` to compress large VCF files before processing.
- **Memory Limits with Large Files**: Files exceeding 2GB cause memory allocation errors on 32-bit systems. Use the `--chunk-size` flag to process large files in segments (default 500MB). Exceeding available RAM triggers automatic chunking but slows processing.
- **Duplicate Interval Names**: Duplicate feature names in BED files cause silent overwriting during annotation. Enable `--warn-duplicates` to generate warnings. The last feature encountered takes precedence in output.

## Examples

### View a genomic region from a BED file
**Args:** `input.bed --region chr1:1000-2000`
**Explanation:** Displays features from input.bed overlapping chromosome 1 positions 1000-2000. The coordinate range follows the display region's inclusive boundaries.

### Find overlaps between two interval files
**Args:** `snps.bed genes.bed --intersect`
**Explanation:** Reports overlapping intervals between SNP positions and gene annotations. Output includes both SNP ID and gene name columns.

### Generate SVG visualization of multiple tracks
**Args:** `*.bed --output graphical --width 200 -m`
**Explanation:** Creates an SVG image with 200-character width, merging overlapping features across all loaded BED files.

### Filter variants by quality score threshold
**Args:** `variants.vcf --filter-expr "QUAL > 30" --format vcf`
**Explanation:** Removes variants with quality scores below 30 using VCF format parsing. Expression syntax follows standard arithmetic operators.

### Color-code features by expression level
**Args:** `expression.bed --color-by score --palette viridis`
**Explanation:** Applies viridis color gradient to features based on the score column, enabling rapid visual identification of high-expression regions.

### Process compressed input with automatic decompression
**Args:** `data.vcf.gz --format vcf --decompress`
**Explanation:** Handles gzip-compressed VCF files transparently without manual decompression. The --decompress flag is implicit for .gz extensions.

### Merge nearby features within 50bp distance
**Args:** `peaks.bed -m --distance 50`
**Explanation:** Merges features separated by less than 50bp into single consolidated regions, suitable for identifying broad binding site clusters.