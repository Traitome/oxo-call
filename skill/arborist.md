---
name: arborist
category: Phylogenetics
description: A tool for phylogenetic tree construction, manipulation, and analysis. Supports Newick and Nexus formats for input and output, with operations including bootstrapping, rooting, pruning, and label renaming.
tags: [phylogeny, tree-analysis, newick, nexus, bootstrap]
author: AI-generated
source_url: https://github.com/example/arborist
---

## Concepts

- **Tree Formats**: Arborist accepts input trees in Newick format (.nwk, .tree) and Nexus format (.nex, .nexus). Output is typically written in Newick format by default. The tool internally represents trees as bifurcating rooted or unrooted structures with nodes, edges, and branch lengths.
- **Bootstrapping Support**: When input contains multiple trees (e.g., from resampling), arborist can compute bootstrap consensus trees, assign bootstrap values to nodes, and filter clades by a minimum support threshold specified in percentage.
- **Tree Operations**: Core operations include re-rooting at a specified outgroup, pruning tips (removing selected taxa), ladderizing (ordering branches by size), extracting subtrees, and renaming or relabeling taxa using tab-delimited mapping files.

## Pitfalls

- **Outgroup Specification**: Specifying an outgroup that is not present in the tree causes the rooting operation to fail and abort. Always verify taxon labels match exactly, including case sensitivity, before specifying an outgroup.
- **Format Mismatches**: Providing a Nexus file with embedded comments or metadata without the `-F nexus` flag causes the parser to misread tree tokens, resulting in malformed or truncated output trees.
- **Bootstrap Threshold Interpretation**: A threshold of 50 means nodes appearing in fewer than 50% of bootstrap replicates are collapsed. Setting the threshold too high can overresolve the tree and produce multifurcations where bifurcations are expected.
- **Memory with Large Alignments**: Running arborist with very large multiple sequence alignments (thousands of sequences) can exhaust memory if the alignment is held entirely in RAM. Processing in chunks or using reduced taxon sets is recommended.

## Examples

### Build a neighbor-joining tree from a distance matrix
**Args:** `NJ -i sequences.fasta -o tree.nwk`
**Explanation:** The NJ subcommand computes a neighbor-joining tree from an input FASTA alignment, writing the resulting Newick tree to the specified output file.

### Compute bootstrap consensus tree from resampled alignments
**Args:** `bootstrap -i aln directory/ -o consensus.tree -t 75`
**Explanation:** This command reads all alignment files from the input directory, computes a majority-rule consensus tree with a 75% bootstrap threshold, and writes the result to consensus.tree.

### Root the tree at a designated outgroup taxon
**Args:** `root -i tree.nwk -o rooted.tree -g Homo sapiens`
**Explanation:** The root subcommand takes an unrooted input tree and re-roots it using Homo sapiens as the outgroup, placing that taxon at the base of the tree.

### Rename multiple taxa using a mapping file
**Args:** `relabel -i tree.nwk -o renamed.tree -m taxon map.tsv`
**Explanation:** This applies a bulk rename operation using the tab-separated mapping file, where each line contains original label and new label, writing the transformed tree to the output file.

### Prune tips to create a subtree matching a species list
**Args:** `prune -i full.tree -o subset.tree -k species list.txt`
**Explanation:** The prune operation removes all tips not present in the species list file, producing a subtree containing only the specified taxa.

### Ladderize the tree to improve visual presentation
**Args:** `ladderize -i unladdered.tree -o laddered.tree`
**Explanation:** This reorders branches so that the largest clade is on the left at each node, producing a ladderized tree better suited for display in tree viewers.