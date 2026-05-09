---
name: cluster-picker
category: Phylogenetics
description: Identifies clusters in phylogenetic trees based on branch support values. Takes Newick format trees and outputs cluster assignments using configurable support thresholds.
tags: phylogenetics, cluster-analysis, tree-clustering, newick, bootstrap, bayesian
author: AI-generated
source_url: https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3930241/
---

## Concepts

- Input is a Newick format phylogenetic tree file with support values embedded as node labels (bootstrap percentages or Bayesian posterior probabilities). The tool reads each internal node and compares its support value against a user-specified threshold to determine cluster membership.

- The tool outputs tab-delimited files containing cluster assignments: one file lists which sequence/taxon belongs to which cluster, while an optional Newick output includes cluster annotations on the tree for visualization.

- Cluster identification works by traversing the tree top-down from the root, identifying well-supported monophyletic clades (those with branch support at or above the threshold), and recursively assigning taxa to clusters until all terminals are assigned.

- The algorithm handles both bootstrap values (0-100) and Bayesian posterior probabilities (0-1) by detecting the format of support values in the input tree, requiring appropriate threshold scaling.

---

## Pitfalls

- Using a support threshold that's too low (e.g., 50%) will create many spurious small clusters from tree noise, artificially fragmenting a monophyletic group into meaningless subclusters and complicating downstream interpretation.

- Using a threshold that's too high (e.g., 98%) will merge distinct evolutionary lineages together because insufficiently supported branches collapsed, causing you to miss important biological signals in the data.

- Failing to verify support value formats in your input Newick file will produce nonsensical clusters; bootstrap trees use integer percentages while BEAST/MrBayes trees use decimal probabilities, requiring different threshold inputs.

- Not ensuring your Newick file has support values as separate node labels will cause the tool to fail silently or produce a single cluster containing all taxa, as it cannot detect support from unlabeled nodes.

---

## Examples

### Identify clusters with default 70% bootstrap threshold
**Args:** input_tree.newick
**Explanation:** Runs cluster-picker on the input tree using the default 70% bootstrap threshold, writing cluster assignments to clusterPickerClusters.txt and annotated tree to clusterPickerTree.newick.

### Use a custom 90% bootstrap threshold for strict clustering
**Args:** input_tree.newick 0.9
**Explanation:** Applies a stringent 90% support threshold to identify only highly supported clusters, useful when analyzing well-resolved trees where you want to avoid weakly supported groupings.

### Specify explicit output filenames for cluster assignments
**Args:** input_tree.newick 0.85 my_clusters.txt my_annotated_tree.newick
**Explanation:** Writes cluster assignments to my_clusters.txt and saves the tree with cluster annotations to my_annotated_tree.newick for downstream visualization in tools like FigTree.

### Process tree with Bayesian posterior probabilities (0-1 scale)
**Args:** beast_tree.newick 0.95
**Explanation:** Reads posterior probability support values from a BEAST-generated tree; using 0.95 corresponds to 95% posterior probability support for cluster determination.

### Generate cluster output without annotated tree file
**Args:** input_tree.newick 0.8 clusters_only.txt
**Explanation:** Outputs only the cluster assignments table to clusters_only.txt without creating an annotated Newick file, useful when you need assignment data for scripting pipelines.

### Run with detailed logging to diagnose clustering issues
**Args:** input_tree.newick 0.7 output_clusters.txt output_tree.newick --log run_log.txt
**Explanation:** Creates a verbose log file run_log.txt containing the clustering process details,帮助你调试支撑值閾值的選擇問題。