---
name: arborator
category: Phylogenetics and Evolutionary Analysis
description: A phylogenetic tree annotation and visualization tool for creating, editing, and analyzing evolutionary trees with support for multiple file formats and metadata annotation.
tags: [phylogeny, tree-annotation, evolutionary-biology, visualization, newick, nexus]
author: AI-generated
source_url: https://arborator.github.io/
---

## Concepts

- **Tree Data Model**: Arborator works with phylogenetic trees represented in Newick, Nexus, or Phylip formats, where each node can carry metadata such as species names, evolutionary events, and custom annotations.
- **Input/Output Formats**: The tool accepts multiple input formats (`.nwk`, `.nex`, `.phy`) and can export annotated trees in standard formats alongside JSON metadata files for downstream analysis.
- **Annotation System**: Nodes in the tree can be annotated with categorical or numeric attributes, supporting evolutionary events like gene duplications, transfers, and losses via the `–annotate` flag.
- **Batch Processing**: Arborator supports processing multiple tree files in a directory using the `–batch` flag, enabling high-throughput phylogenetic analyses across large datasets.
- ** Visualization Options**: The `--output-format` flag controls whether output is written as a static image (PDF, PNG, SVG) or as an annotated text file for further bioinformatic processing.

## Pitfalls

- **Missing Root Node**: Failing to specify a rooted tree (with an explicit root node using `--root` or an outgroup) causes incorrect interpretation of evolutionary direction, leading to misleading phylogenetic conclusions.
- ** Mismatched File Extensions**: Providing an input file with an incorrect extension (e.g., `.txt` instead of `.nwk`) without specifying `--format` explicitly may cause the parser to fail or misread the tree structure entirely.
- **Annotation Key Typos**: Specifying an annotation key that does not match any defined attribute (e.g., typing `--annotate=speciees` instead of `--annotate=species`) results in silent failure where no annotation is applied to the tree.
- **Insufficient Memory for Large Trees**: Attempting to visualize trees with thousands of nodes without increasing the memory limit (`--memory`) causes the tool to hang or crash, losing unsaved work.
- **Overwriting Output Without Confirmation**: Using the `--force` flag on an existing output file without backing up the original results in irreversible data loss of previous annotations.

## Examples

### Convert a Newick tree to Nexus format
**Args:** `input.tre --output=output.nex --format=nexus`
**Explanation:** This converts a tree from Newick format to Nexus format, enabling compatibility with other phylogenetic software that requires Nexus input.

### Annotate nodes with species names and export to PDF
**Args:** `tree.nwk --annotate=species --output=annotated_tree.pdf --format=pdf`
**Explanation:** This reads a Newick file, applies species name annotations to tree nodes, and exports the result as a PDF image for publication or presentation.

### Process all trees in a directory batchwise
**Args:** `./trees/ --batch --output=./annotated/ --format=newick`
**Explanation:** This applies the default annotation pipeline to every tree file in the specified directory, writing results to the output folder in Newick format.

### Specify an outgroup to root the tree
**Args:** `tree.nwk --root=outgroup_species --output=rooted_tree.nwk`
**Explanation:** This defines an outgroup species to root the phylogenetic tree, ensuring correct evolutionary direction for downstream analyses like divergence time estimation.

### Set memory limit for large tree visualization
**Args:** `large_tree.nwk --memory=4096M --output=large_tree.png`
**Explanation:** This increases the memory allocation to 4GB, allowing proper rendering of trees with thousands of taxa without crashing.