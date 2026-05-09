---
name: colorbrewer
category: Visualization / Color Schemes
description: A tool for generating perceptually uniform color palettes optimized for maps and data visualizations. Provides sequential, diverging, and qualitative schemes with customizable number of classes.
tags: [color, palette, visualization, cartography, maps, cmap, plotting]
author: AI-generated
source_url: https://colorbrewer2.org
---

## Concepts

- **Scheme Types**: ColorBrewer provides three distinct palette categories — sequential (ordered gradients for magnitude), diverging (centered gradients for deviations), and qualitative (distinct colors for categorical data without inherent order).
- **Class Number Range**: Each palette supports 3–12 distinct color classes, where fewer classes produce more visually distinguishable colors while more classes create finer gradations but risk visual confusion.
- **Output Formats**: Palettes can be output as hex codes (e.g., "#e41a1c"), RGB tuples, or CSS-ready lists suitable for direct use in visualization libraries like matplotlib, R, or web frameworks.
- **Perceptual Uniformity**: Colors are selected to be distinguishable even by colorblind users and to print well in grayscale, making them suitable for scientific publications and accessibility-compliant figures.
- **Library Integration**: The Python `colorbrewer` module can be imported directly in scripts, accepting scheme name and number of classes as parameters to return color arrays for plotting functions.

## Pitfalls

- **Exceeding Maximum Classes**: Requesting more than 12 classes will fail because no palette has that many distinct colors — this wastes computation and requires reformulating the request.
- **Misapplying Qualitative Schemes**: Using qualitative palettes for ordered data leads to false visual hierarchy, as these schemes are designed only for categorical distinctions with no magnitude meaning.
- **Ignoring Colorblind Accessibility**: Certain palette-color combinations that work well on screen may become indistinguishable when printed or viewed by colorblind readers, leading to data misinterpretation.
- **Incompatible Output Format**: Exporting hex codes when the target visualization software expects RGB tuples will require manual conversion, adding unnecessary processing steps.
- **Scheme-Data Mismatch**: Applying a sequential palette to categorical data or vice versa produces misleading visualizations that do not accurately represent the underlying data structure.

## Examples

### Generate a 5-class sequential blue palette
**Args:** `seq Blues 5`
**Explanation:** Creates a five-step blue gradient suitable for visualizing data that increases in magnitude, such as population density or gene expression levels.

### Generate a 3-class diverging red-blue palette
**Args:** `div RdBu 3`
**Explanation:** Produces a three-class diverging scheme with red for negative, white for neutral, and blue for positive values, ideal for displaying log-fold changes in differential expression.

### Generate an 8-class qualitative palette for categorical groups
**Args:** `qual Set1 8`
**Explanation:** Provides eight highly distinct colors for categorical data where no ordering exists, such as sample conditions or cell types in a scRNA-seq visualization.

### Export a palette as RGB tuples for matplotlib
**Args:** `--format rgb seq YlOrRd 6`
**Explanation:** Returns six colors from the Yellow-Orange-Red sequential scheme as RGB tuples, directly usable in Python matplotlib plotting commands.

### List all available scheme names
**Args:** `--list`
**Explanation:** Displays all available ColorBrewer palette names and their supported class counts, useful for discovering valid inputs before generating a specific scheme.

### Request a colorblind-safe diverging palette
**Args:** `div Spectral 7`
**Explanation:** Generates the Spectral diverging scheme with seven classes, specifically designed to remain distinguishable for viewers with common forms of color vision deficiency.