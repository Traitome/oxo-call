---
name: colord
category: bioinformatics/genomics-visualization
description: A tool from the UCSC Genome Browser toolkit that assigns consistent colors to genomic items for visualization in browser tracks. Provides color lookup, conversion, and randomization based on item names.
tags:
- ucsc
- color
- visualization
- genome-browser
- track
- rgb
- hex
author: AI-generated
source_url: https://github.com/ucscGenomeBrowser/kent
---

## Concepts

- **Color Assignment**: `colord` generates deterministic colors for input item names by hashing them to values in a predefined color palette, ensuring the same item always receives the same color across multiple runs.
- **Input/Output Formats**: The tool accepts item names via stdin or files, and outputs color assignments in multiple formats including RGB decimal (e.g., "255,0,0"), hex (e.g., "#FF0000"), or binary suitable for UCSC track definitions.
- **Color Palette Management**: It uses built-in color palettes optimized for genomic visualization with distinct, visually-separable colors that work on both light and dark backgrounds.
- **Database Integration**: Can reference external color database files to map specific items or categories to custom color schemes, supporting hierarchical or functional groupings.

## Pitfalls

- **Inconsistent Color Assignments**: Running colord without a fixed seed or palette file produces different colors each time, breaking reproducibility in downstream analyses that expect consistent labeling.
- **Mismatched Output Format**: Specifying an output format incompatible with the downstream visualization tool (e.g., using hex for binary-dependent tools) causes rendering failures or garbled colors.
- **Missing Item Names**: Providing empty input or files containing only whitespace results in no color output, but the tool exits quietly without a clear error message.
- **Duplicate Color Conflicts**: When two distinct items hash to the same color due to limited palette size, visual differentiation in browser tracks becomes impossible, leading to data misinterpretation.

## Examples

### Basic color generation from item names
**Args:** `-name=BRCA1 -name=TP53`
**Explanation:** Generates deterministic colors for specified gene names using the default palette, useful for quick color testing.

### Output in RGB format
**Args:** `--rgb`
**Explanation:** Returns colors as comma-separated RGB values (e.g., "0,100,200"), compatible with BED Graph and bedGraph format tracks.

### Using a custom palette file
**Args:** `-palette=myColors.txt`
**Explanation:** Loads colors from a user-defined palette file, enabling consistent branding or annotation-specific color schemes.

### Batch processing from input file
**Args