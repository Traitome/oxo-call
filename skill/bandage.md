---
name: Bandage
category: Assembly Graph Visualization
description: A bioinformatics tool for visualising and exploring de novo assembly graphs. Bandage loads assembly graphs from various assemblers (SPAdes, Velvet, SOAPdenovo) in formats like GFA or FASTG, and provides interactive visualisation to identify misassemblies, repeats, and complex regions. It helps researchers understand assembly quality by visualising node connections, sequences, and graph structure.
tags: [assembly-graph, visualisation, GFA, FASTG, SPAdes, Velvet, bioinformatics, de-novo-assembly, graph-analysis]
author: AI-generated
source_url: https://github.com/rrwick/Bandage
---

## Concepts

- **Graph Format Support**: Bandage reads assembly graphs in GFA (Graphical Fragment Format), FASTG, and native formats from SPAdes, Velvet, VelvetOptimiser, and SOAPdenovo2. The graph consists of nodes (contigs) and edges (overlaps), where each node may contain sequence data and length information.
- **Node and Edge Data Model**: Each graph node represents a contig with properties including sequence string, length, coverage, and unique identifier. Edges represent overlap relationships between contigs, with orientation and overlap length. Bandage visualises this as an interactive network where users can click nodes to inspect their sequences.
- **Visualisation and Navigation**: Bandage renders the assembly graph as a layout where node positions are determined by graph topology (connected nodes are placed near each other). Users can zoom, pan, select nodes, and view detailed information including the full sequence, coverage values, and adjacent connections.
- **Info Output for Automation**: The `--info` option outputs graph statistics as text (node count, total length, N50, etc.) suitable for piping into other tools or scripts, enabling batch processing workflows without GUI interaction.

## Pitfalls

- **Large Graph Memory Usage**: Assembly graphs from complex datasets (e.g., metagenomes or large eukaryotic genomes) can contain thousands of nodes and edges. Loading such graphs into Bandage may consume significant RAM, and the GUI may become unresponsive during layout computation. Consequence: Application crash or system slowdown; consider filtering or simplifying the graph first.
- **Missing Sequence Data in Graph Files**: Some assemblers output graph files where nodes lack sequence information (only length is stored). Bandage can still visualise the topology, but users cannot view actual sequences or perform operations requiring sequence data. Consequence: Limited analysis capability; verify that your graph file contains sequencedata before expecting full functionality.
- **Format Compatibility Issues**: Not all graph formats are equally supported. FASTG files must follow the exactFASTG specification, and some SPAdes/Velvet output formats require specific versions. Using an incompatible or corrupted graph file results in a parsing error with no visualisation. Consequence: Wasted time troubleshooting; ensure the assembler version matches Bandage's supported formats.
- **Slow Initial Graph Layout**: Bandage computes node positions using a force-directed layout algorithm, which scales poorly with graph size. Graphs with >10,000 nodes may take minutes to hours to render initially. Consequence: Patience required or need to subset the graph; use Bandage's `--info` flag for quick statistics if visualisation is unnecessary.

## Examples

### Load and visualise an assembly graph from SPAdes
**Args:** input_graph.gfa
**Explanation:** This opens the GFA-formatted assembly graph in the Bandage GUI, allowing interactive exploration of contigs, overlaps, and graph structure.

### Generate a PNG image of the entire assembly graph
**Args:** --png output_image.png --width 2000 input_graph.gfa
**Explanation:** Renders the full graph to a 2000-pixel-wide PNG image without launching the interactive GUI, useful for reports or batch processing.

### Output graph statistics to the terminal
**Args:** --info input_graph.gfa
**Explanation:** Prints summary statistics (number of nodes, total length, longest contig, N50, etc.) and exits, enabling integration into pipelines.

### Visualise a specific region of the graph by node names
**Args:** --subset "node_1 node_5 node_12" input_graph.gfa
**Explanation:** Loads only the specified nodes and their immediate neighbours into Bandage, dramatically reducing memory usage and rendering time for focused analysis.

### Export sequences of all nodes to a FASTA file
**Args:** --outputfasta exported_contigs.fasta input_graph.gfa
**Explanation:** Extracts the sequence from every node in the graph and writes them to a FASTA file, useful for downstream analysis or validation.

### Specify a custom layout depth for subset visualization
**Args:** --subset "node_42" --depth 2 input_graph.gfa
**Explanation:** Loads node_42 and all nodes within 2 edges of distance, providing context around a region of interest without loading the entire graph.

### Set a shorter timeout for automated pipeline use
**Args:** --quitafterinfodump input_graph.gfa
**Explanation:** When combined with `--info`, automatically closes Bandage after outputting statistics, suitable for non-interactive scripts.