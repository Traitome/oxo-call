---
name: clusterpicker
category: Phylogenetics::Clustering
description: A tool for identifying and extract clusters from phylogenetic trees based on branch support values and topological criteria
tags: [phylogeny, tree-clustering, newick, bootstrap, group-detection, bii]
author: AI-generated
source_url: https://github.com/phylo-tools/clusterpicker
---

## Concepts

- ClusterPicker operates on phylogenetic trees in Newick format (.nwk), using branch support values (bootstrap, posterior probabilities) to define cluster boundaries and assign membership
- The tool requires input trees to have annotated support values on internal nodes; these are typically extracted from bootstrap analysis or Bayesian MCMC runs
- Output consists of cluster assignments as a tab-delimited file with cluster IDs, member taxa, and support values for each identified group
- Cluster identification is governed by a minimum support threshold (default 70) and optional tree height or distance cutoffs to define clean clusters
- The tool can process multiple trees simultaneously, producing consensus clusters with frequency information across input trees

## Pitfalls

- Using trees without support value annotations causes all internal nodes to be treated as unsupported, resulting in no clusters being identified
- Setting the support threshold too low (e.g., below 50) produces overly inclusive clusters that contain divergent taxa which destabilize the group
- Input trees with polytomies (unresolved multifurcations) are not automatically resolved, leading to fragmented or missing cluster assignments
- Mixing trees from different gene alignments can produce artifactual clusters driven by recombination rather than shared evolutionary history
- Failing to specify the correct support value field (e.g., using internal node labels vs. bootstrap values in a different column) silently produces empty results

## Examples

### Identify clusters with 80% bootstrap support from a single tree
**Args:** --tree input.nwk --support 80 --output clusters.tsv
**Explanation:** Extracts monophyletic groups where internal node support meets or exceeds 80%, writing cluster membership to the output file

### Generate clusters from 50% support threshold
**Args:** --tree alignment_tree.nwk --support 50 --min-size 3
**Explanation:** Identifies clusters with at least 50% support containing a minimum of 3 taxa each, filtering out singleton or poorly supported groups

### Process multiple trees and compute consensus clusters
**Args:** --trees run1.nwk,run2.nwk,run3.nwk --support 75 --consensus --output consensus.tsv
**Explanation:** Analyzes three phylogenetic trees and outputs clusters present in at least two of three runs, with frequency statistics

### Extract clusters using a support value field named "bp"
**Args:** --tree rooted_tree.nwk --support-field bp --support 70 --output bp_clusters.tsv
**Explanation:** Parses the "bp" annotation from internal node labels rather than default bootstrap columns, useful for non-standard Newick variants

### Limit cluster size and save the filtered tree
**Args:** --tree gene_tree.nwk --support 85 --max-size 10 --prune-tree --output pruned.tre
**Explanation:** Identifies well-supported clusters of up to 10 taxa and outputs a Newick tree with filtered cluster members retained

### Run with verbose output for debugging
**Args:** --tree test.nwk --support 60 --verbose --log clusterpicker.log
**Explanation:** Enables detailed logging of parsing steps and cluster decisions, helpful when troubleshooting empty or unexpected output