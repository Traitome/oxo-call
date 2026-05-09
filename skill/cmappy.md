---
name: cmappy
category: data-visualization
description: A Python library for converting, sampling, and manipulating matplotlib colormaps for scientific data visualization. Supports discrete colormap generation, format conversion, and color interpolation.
tags:
  - colormap
  - visualization
  - matplotlib
  - color-scale
  - heatmap
  - scientific-plotting
author: AI-Generated
source_url: https://github.com/cmappy/cmappy
---

## Concepts

- **Colormap Format Conversion**: cmappy converts colormaps between matplotlib native format, HEX strings, RGB tuples, and CSS color names. This enables interoperability with tools like R's viridis, Python's seaborn, and web-based visualization libraries.
- **Discrete Sampling**: The `sample` method interpolates a continuous colormap to produce a discrete palette with a specified number of color stops (N values). This is essential for creating categorical legends and finite-color-scale heatmaps in publications.
- **Colormap Composition**: cmappy supports reversing (`reversed`), alpha modulation (`set_under`, `set_over`), and blending multiple colormaps (`blend`). These operations allow creation of diverging and multi-panel color scales from single-hue palettes.
- **Normalization Objects**: Passing a `matplotlib.colors.Normalize` object to sampling methods ties color mapping directly to data value ranges (vmin, vmax, log scaling). This ensures consistent color scaling across multiple subplots and figures.

## Pitfalls

- **Mismatched Normalization**: Supplying a default linear Normalize to log-scaled data produces incorrect color distribution across the value range. Always pass `LogNorm()` when sampling colormaps for data spanning multiple orders of magnitude.
- **Integer Color Stop Misuse**: Sampling with `N` smaller than the number of unique data categories truncates distinctions in categorical heatmaps, causing label ambiguity in downstream interpretation.
- **Alpha Clipping Side Effects**: Using `set_under` or `set_over` with `clip=False` on data containing NaN values causes out-of-range entries to inherit the clipping color, contaminating null-value regions in the visualization.
- **Unsorted Color Stops**: Passing unevenly distributed value list to `to_mplColormap` produces non-uniform color transitions. Always provide linearly spaced control points or sort the input explicitly before conversion.

## Examples

### Convert a Matplotlib colormap to a list of HEX color strings
**Args:** `colormap Accent_loaded -N 256`
**Explanation:** Loads the Accent colormap, samples 256 evenly spaced color stops, and outputs each as a HEX string for use in external plotting tools.

### Generate a discrete 7-color palette matching journal figure requirements
**Args:** `sample viridis -N 7 -format list`
**Explanation:** Creates a 7-element discrete color list from the continuous viridis gradient, matching common journal specifications for categorical heatmaps.

### Blend two colormaps for a diverging expression scale
**Args:** `blend RdBu_r -- CmMapYlGn -N 256`
**Explanation:** Merges the red-blue and yellow-green colormaps to produce a 256-stop diverging scale with appropriate color divergence at zero expression.

### Reverse a colormap for inverted volcano plot coloring
**Args:** `colormap Blues_r load`
**Explanation:** Loads the reversed Blues colormap directly without explicit sampling, suitable for coloring low p-value regions darker in volcano plots.

### Export a colormap with explicit vmin/vmax normalization for single-cell RNA data
**Args:** `sample magma -N 128 --norm "0,5" --vmin 0.0 --vmax 5.0 --format dict`
**Explanation:** Samples the magma colormap with 128 stops and outputs a dictionary mapping normalized values (0-1) to RGB tuples, ensuring consistent coloring across multiple single-cell samples.

### Apply alpha clipping for undefined gene expression values
**Args:** `colormap Greens --set_under white --set_over black clip`
**Explanation:** Configures the Greens colormap to render under-range (NaN) and over-range values with white and black respectively, preserving null-data regions in sparse expression matrices.