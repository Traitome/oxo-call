---
name: cgview
category: Genome Visualization
description: A tool for generating circular maps of bacterial chromosomes and plasmids, visualizing annotated genomic features such as genes, ORFs, and regulatory elements in SVG, PNG, or PDF formats.
tags: [genome, circular-map, visualization, bacterial-genomics, svg, png]
author: AI-generated
source_url: https://cgview.augustculture.com/
---

## Concepts

- **Input Data Models**: cgview accepts GenBank (.gbk), EMBL (.embl), FASTA, and XML formats as primary input for extracting genomic features including genes, CDS, rRNA, tRNA, and other annotations.
- **Output Formats**: The tool generates scalable vector graphics (SVG) for high-resolution publication-quality images, PNG for raster graphics, and PDF for vector output; the `-format` flag controls output type.
- **Configuration System**: Circular map appearance is controlled via a configuration file (created by companion tool `cgview-build` or manually) that specifies feature colors, label density, legend positioning, and ruler properties.
- **Feature Rendering**: Features are drawn as arrows on the circle's periphery (forward strand) and inwards (reverse strand), with arrow direction indicating transcriptional orientation; color coding follows the configuration file's feature type definitions.
- **Companion Tool**: `cgview-build` generates configuration files and processes GenBank/EMBL files into the format expected by cgview, handling feature parsing and filter application.

## Pitfalls

- **Missing Feature Annotations**: If input GenBank/EMBL files lack proper `/gene`, `/note`, or `/product` qualifiers, the resulting map may display blank or incomplete feature labels, reducing interpretability.
- **Mismatched Configuration Files**: Using a configuration file generated for a different organism or sequence length will produce visually distorted or misleading maps with improper scale bars and feature positioning.
- **Memory Exhaustion with Large Genomes**: Attempting to render very large sequences (>10 Mb) or entire chromosomes without adjusting the `-feature_size` threshold may produce cluttered images or cause memory errors.
- **Incorrect Feature Filtering**: Failing to properly filter features using the configuration file can result in hundreds of overlapping labels, making the circular map unreadable.
- **Output Format Case Sensitivity**: Using lowercase `-format png` instead of `-format PNG` (or the correct variant supported by your version) may cause the tool to default to SVG or fail silently.

## Examples

### Generate a basic circular map from a GenBank file
**Args:** -input sequence.gbk -output map.svg -format svg
**Explanation:** This creates a circular visualization of the genome stored in the GenBank file, outputting an SVG vector graphic suitable for publication.

### Create a PNG raster image at specific dimensions
**Args:** -input sequence.gbk -output map.png -format PNG -size 800
**Explanation:** This generates an 800x800 pixel PNG image of the circular map, useful for presentations or web display where vector graphics are not suitable.

### Specify a custom configuration file for feature styling
**Args:** -input sequence.gbk -conf custom_config.xml -output custom_map.svg
**Explanation:** This applies custom color schemes, label densities, and feature filtering rules defined in the configuration file rather than using defaults.

### Build a configuration file from a GenBank input
**Args:** sequence.gbk -o my_config.xml
**Explanation:** Using the companion tool `cgview-build` to generate a template configuration file that captures all features from the GenBank file for subsequent customization.

### Adjust feature size threshold to reduce clutter
**Args:** -input sequence.gbk -output clean_map.svg -conf config.xml -feature_size 500
**Explanation:** This renders the map while filtering out features smaller than 500 bp, reducing visual clutter on densely annotated genomes.