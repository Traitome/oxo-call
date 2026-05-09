---
name: aniclustermap
category: Bioinformatics / Clustering Visualization
description: Generates cluster maps with optional animation for hierarchical clustering results, supporting multiple output formats and interactive visualizations
tags:
- clustering
- visualization
- heatmap
- dendrogram
- animation
- bioinformatics
author: AI-generated
source_url: https://github.com/aniclustermap/aniclustermap
---

## Concepts

- **Data Model**: Accepts distance matrices, phylogenetic trees (Newick format), or cluster assignment files as input. The tool builds a visual cluster map where sequences are arranged by hierarchical clustering order, with branch lengths and support values rendered proportionally.
- **Output Formats**: Generates static images (PNG, PDF, SVG) and interactive HTML with embedded JavaScript for pan/zoom. Animation mode produces MP4 or GIF by interpolating tree traversal steps between leaf nodes.
- **Key Behaviors**: The heatmap is constructed by reordering rows and columns according to the hierarchical clustering tree leaf order. Color intensities are derived from input expression matrices or numeric annotations. When `--animate` is specified, the tool renders successive snapshots as the clustering is revealed from leaves upward.
- **CLI Philosophy**: Flags are composable; multiple `--color-scheme` values apply to layered annotations. The default rendering uses Euclidean distance and Ward linkage unless `--distance` or `--linkage` overrides these.

## Pitfalls

- **Mismatched Input Dimensions**: Providing a distance matrix that does not match the number of labels in the cluster file causes silent column truncation or misaligned color cells, producing misleading visualizations rather than an error. Always verify row/column counts match between input files.
- **Overwriting Output Without Warning**: Specifying an output path that already exists results in silent overwrite without confirmation; always ensure the target directory is empty or specify `--force` to suppress warnings during batch processing.
- **Memory Exhaustion with Large Datasets**: Rendering clusters with >5,000 leaves in animation mode can exhaust available RAM because the tool buffers all frame states. Use `--subsample` or split the dataset into manageable chunks to prevent process termination.

## Examples

### Generate a static cluster heatmap from a distance matrix
**Args:** --input-data matrix.txt --cluster-tree tree.nw --output heatmap.png
**Explanation:** The tool reads the numeric matrix for cell intensities and reorders rows/columns by the Newick tree leaf order, rendering a PNG image.

### Create an animated clustering reveal in GIF format
**Args:** --input-data expression.csv --cluster-tree cluster.nw --animate --output-frames anim.gif --fps 2
**Explanation:** Animation builds frames sequentially, showing the clustering process from individual leaves merging upward, saved as a GIF at 2 frames per second.

### Customize color scheme for heatmap cells
**Args:** --input-data counts.txt --color-scheme RdYlBu --cluster-tree tree.nw --output colored.png
**Explanation:** Applies the Red-Yellow-Blue gradient to expression values, with low values in red and high values in blue for intuitive visual interpretation.

### Output interactive HTML with pan and zoom
**Args:** --input-data matrix.csv --cluster-tree tree.nw --output interactive.html --interactive
**Explanation:** Embeds JavaScript into the output HTML, enabling mouse-drag panning and scroll-wheel zoom for exploring large heatmaps.

### Add annotation tracks above and beside the heatmap
**Args:** --input-data values.txt --cluster-tree tree.nw --annot-top annot1.txt --annot-left annot2.txt --output annotated.png
**Explanation:** Renders additional color-coded bars above and beside the main heatmap, sourced from the annotation files, preserving the clustering leaf order.

### Adjust distance metric and linkage method
**Args:** --input-data dist.txt --cluster-tree tree.nw --distance pearson --linkage average --output recalc.png
**Explanation:** Overrides the default Euclidean/Ward settings, using Pearson correlation distance and average linkage for recomputing cluster hierarchy before rendering.