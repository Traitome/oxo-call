---
name: alv
category: Bioinformatics - Genome Assembly Visualization
description: A viewer for Velvet de novo genome assembler output graphs, enabling visualization and analysis of assembly contigs, nodes, and edges in various export formats.
tags: [velvet, assembler, graph-viewer, assembly-analysis, de-novo, bioinformatics]
author: AI-generated
source_url: https://github.com/wkentaro/alv
---

## Concepts

- `alv` reads Velvet assembler's output files (Graph, sequences, and Log files) to reconstruct the assembly graph containing nodes (contigs) and edges (connections between contigs).
- The tool supports multiple output formats including AGV (ASCII Graph Visualization), DOT (GraphViz), GDL (Graph Description Language), and XDot (interactive viewer), allowing integration with graph visualization pipelines.
- Nodes in the assembly graph represent assembled sequences with coverage information, while edges represent read connections detected during the Velvet assembly process.
- `alv` can filter the graph by different criteria such as coverage thresholds, minimum sequence length, or specific contig identifiers to focus analysis on high-confidence regions.
- The tool maintains compatibility with Velvet's node/edge numbering scheme, enabling direct correlation between the visualized graph and Velvet's numeric contig IDs.

## Pitfalls

- **Specifying an incorrect or missing Velvet output directory**: Without the correct path to Velvet's Graph file (created by velveth and velvetg), alv cannot reconstruct the assembly graph and will fail with an unclear error about missing input files.
- **Setting overly restrictive coverage filters**: Using high minimum coverage thresholds may exclude legitimate low-coverage but valid contigs, leading to a fragmented or empty graph view that misrepresents the actual assembly.
- **Confusing alv with velveth/velvetg**: alv is a visualization tool only—it cannot perform assembly itself. Attempting to use alv without first running the Velvet assembler will result in missing input files.
- **Exporting to unsupported formats for downstream analysis**: Choosing an output format incompatible with the intended downstream tool (e.g., using AGV when GraphViz DOT format is needed) will require re-running alv with different flags.
- **Ignoring the log file warnings**: Velvet's log file contains important assembly statistics and warnings that alv may not fully display; ignoring these can lead to misinterpretation of graph elements that show low confidence.

## Examples

### View the assembly graph interactively

**Args:** `-w`

**Explanation:** Opens alv in interactive window mode (XDot), allowing you to zoom, pan, and click on nodes to inspect individual contig details and connections.

### Export the graph for GraphViz visualization

**Args:** `-f dot -o assembly_graph.dot`

**Explanation:** Exports the assembly graph to DOT format, enabling further styling and layout customization using GraphViz tools like dot, neato, or fdp.

### Filter out low-coverage contigs

**Args:** `-c 10`

**Explanation:** Removes nodes with coverage below 10x from visualization, helping focus on high-confidence assembly regions and reducing graph clutter.

### Show only contigs longer than a minimum length

**Args:** `-m 500`

**Explanation:** Filters the graph to display only contigs with sequence lengths of 500 base pairs or greater, highlighting the core assembly scaffold.

### Display edges with read count information

**Args:** `-e`

**Explanation:** Includes edge labels showing the number of reads connecting adjacent contigs, providing insight into assembly connectivity strength.

### Export to AGV text format for documentation

**Args:** `-f agv -o assembly.agv`

**Explanation:** Writes the graph in AGV (ASCII Graph Visualization) format, producing a plain-text representation suitable for inclusion in reports or raw inspection.

### Use a specific Velvet graph directory

**Args:** `--graph-dir /path/to/velvet/output`

**Explanation:** Specifies a non-default directory containing Velvet's output files (Graph, sequences, Log), necessary when working with assemblies stored outside the current working directory.