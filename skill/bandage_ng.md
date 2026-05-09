---
name: bandage-ng
category: graph-visualization
description: Bandage NG visualizes De Bruijn and assembly graphs from GFA files as PNG or SVG images, annotating nodes with sequences, coverage, and depth information. Supports node filtering by length and k-mer count.
tags:
  - graph-visualization
  - assembly-graphs
  - GFA
  - de-bruijn-graphs
  - bioinformatics
  - next-generation-assembly
author: AI-generated
source_url: https://github.com/riwarz/BandageNG
---

## Concepts

- Bandage NG consumes **GFA (Graphical Fragment Assembly) format** files as primary input. Both GFA1 and GFA2 variants are supported, but GFA2 with the `LN` (line length / coverage) segment tags is required for coverage-based color rendering.
- Output is produced via the **imager pipeline**: nodes and edges are laid out using a force-directed algorithm, then rendered to PNG (default) or SVG. The `--width`, `--height`, and `--dotwidth` flags control canvas size and edge thickness.
- Node annotation is driven by **GFA tags** present in segments. Tags like `LN`, `dp`, `kn`, and `cl` map to coverage, depth, k-mer count, and clean length respectively. When these tags are absent, related display options silently degrade to default values.
- The `--randomseed` flag initializes the force-directed layout randomizer. Identical graphs with the same seed and identical input will produce byte-for-byte identical layouts, enabling reproducible image generation.
- **Node length filtering** via `--minlength` and `--maxlength` acts as a pre-pass: nodes outside the range are excluded from both layout and rendering before the force-directed algorithm runs, reducing memory and compute for large graphs.

## Pitfalls

- Using GFA files from assemblers that produce **non-standard GFA variants** (e.g., Flye, Unicycler partial GFA) can cause missing or misnamed tags. If `--depth` colors are all uniform, inspect the GFA file for missing `LN` or `dp` tags.
- Specifying `--minlength` or `--maxlength` values that exclude all nodes produces a **blank/empty image** with no error or warning. Always visually verify output or check node count in the source GFA before filtering.
- The force-directed layout is **non-deterministic by default** across runs (no `--randomseed`). Re-running without fixing the seed on large graphs yields different node arrangements, making comparative visualization unreliable.
- Conflicting or overlapping constraints like `--minlength 1000 --maxlength 500` silently result in **no nodes processed**, producing an empty image without a diagnostic message.
- The `--distance` parameter (node context radius for display) combined with aggressive `--minlength` filtering can produce misleading visuals where only peripheral node halos are shown without their core connections.

## Examples

### Render an assembly graph from a GFA file to a PNG image
**Args:** `/path/to/assembly.gfa --output /path/to/graph.png`
**Explanation:** This is the basic visualization invocation, loading the GFA file and rendering the force-directed graph layout to a PNG image at default resolution.

### Render a graph as an SVG with wider canvas and thicker edges
**Args:** `/path/to/assembly.gfa --output /path/to/graph.svg --width 3000 --height 2000 --dotwidth 5`
**Explanation:** SVG output with enlarged canvas and increased edge thickness improves readability for publication-quality figures and dense assembly graphs.

### Color nodes by depth using the LN tag and set a fixed layout seed
**Args:** `/path/to/assembly.gfa --output /path/to/graph.png --depth --randomseed 42`
**Explanation:** The `--depth` flag activates coverage-based node coloring using the GFA `LN` tag, and `--randomseed 42` ensures the layout algorithm produces the same arrangement on repeated runs.

### Filter to show only long nodes (≥ 500 bp) to simplify a complex graph
**Args:** `/path/to/assembly.gfa --output /path/to/simplified.png --minlength 500`
**Explanation:** Setting a minimum node length threshold removes short bubble and tip nodes from both the layout computation and the rendered image, isolating high-confidence paths in large assemblies.

### Label node IDs on the image for manual curation
**Args:** `/path/to/assembly.gfa --output /path/to/labelled.png --labels --labelfont-size 6`
**Explanation:** Enabling `--labels` overlays GFA segment names directly on graph nodes, and reducing the font size to 6 prevents label overlap in graphs with hundreds of nodes.

### Increase node context radius to show surrounding connections
**Args:** `/path/to/assembly.gfa --output /path/to/local.png --distance 3`
**Explanation:** The `--distance 3` flag expands the display to include nodes reachable within 3 edges from each primary node, useful for inspecting local graph topology around specific loci.