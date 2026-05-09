---
name: apu-label-propagation
category: Graph-based label propagation / Semi-supervised learning
description: A bioinformatics tool for propagating functional labels (e.g., gene ontology terms, disease associations) across biological networks using label propagation algorithms. Takes an interaction network and partially labeled nodes, then computes label scores for unlabeled nodes based on network structure and known labels.
tags: [label-propagation, graph-algorithms, network-analysis, gene-function-prediction, semi-supervised-learning, bioinformatics]
author: AI-generated
source_url: https://github.com/apu-tools/apu-label-propagation
---

## Concepts

- **Network Input Format**: The tool accepts weighted or unweighted graphs in edge-list format (tab-separated, two columns: source target [weight]) representing biological relationships such as protein-protein interactions, gene co-expression correlations, or metabolic pathways.
- **Label File Format**: Labels are provided as a two-column tab-separated file where the first column contains node identifiers and the second column contains numeric label IDs or categorical annotation tags. Nodes without a label in the second column are treated as unlabeled and will receive propagated scores.
- **Propagation Modes**: The tool implements two algorithms - 'random-walk' (propagates labels based on stationary distribution of random walks starting from labeled nodes) and 'harmonic' (solves the harmonic function equation for semi-supervised learning). Mode is specified via the `--method` flag.
- **Output Scores**: For each unlabeled node, the tool outputs a score matrix where rows are nodes and columns are label probabilities/scores, enabling downstream thresholding or ranking of candidate annotations.

## Pitfalls

- **Missing Node ID Mismatch**: If node IDs in the label file do not exactly match node IDs in the network file (including whitespace, case, or formatting differences), those nodes will be treated as unlabeled rather than seed nodes, leading to invalid propagation results. Always verify ID consistency between files before running.
- **Disconnected Network Components**: Label propagation only affects nodes reachable from labeled nodes within the same connected component. Nodes in isolated components with no labeled seeds will receive zero scores for all labels, producing empty or misleading output files.
- **Weight Interpretation Errors**: For unweighted graphs, default edge weights are treated as 1.0; for weighted graphs, the tool normalizes weights internally. Using binary interaction presence as weights when actual confidence scores are available will significantly alter propagation results.
- **Memory Consumption on Large Networks**: Networks with >100,000 nodes can consume significant RAM during matrix construction. The tool constructs a sparse representation internally, but very dense or highly interconnected graphs may exhaust available memory.

## Examples

### Propagate gene function labels across a protein-protein interaction network using random walk

**Args:** --network ppi_network.tsv --labels known_annotations.tsv --method random-walk --output propagated_scores.tsv
**Explanation:** This runs label propagation on a PPI network using the random walk method, outputting probability scores for each unlabeled gene receiving each known function annotation.

### Use harmonic propagation with a confidence-weighted metabolic network

**Args:** --network metabolic_edges_weighted.tsv --labels enzyme_hmm_terms.tsv --method harmonic --normalized --output predicted_enzyme_roles.tsv
**Explanation:** This applies the harmonic (graph-based semi-supervised) propagation method on a weighted metabolic network where edge weights represent reaction confidence, producing predicted enzyme role scores.

### Limit propagation to 3 iterations for fast preliminary results

**Args:** --network coexpression_network.tsv --labels disease_genes.tsv --method random-walk --max-iter 3 --output quick_predictions.tsv
**Explanation:** This restricts label propagation to 3 iterations instead of the default convergence, useful for quick exploratory analysis when computational time is limited.

### Specify output format as CSV with posterior probabilities

**Args:** --network signaling_graph.tsv --labels signaling_components.tsv --method harmonic --format csv --scores-as posterior --output signal_predictions.csv
**Explanation:** This outputs results in CSV format with probabilities representing posterior label distributions rather than raw log-likelihood scores, simplifying downstream interpretation.

### Set a convergence threshold of 1e-5 for high-precision propagation

**Args:** --network Regululon_network.tsv --labels tf_targets.tsv --method random-walk --tol 1e-5 --output precise_prop.tsv
**Explanation:** This sets a tighter convergence tolerance (1e-5 instead of default 1e-3), causing the algorithm to run more iterations but producing more precisely converged label probability distributions.

### Provide a pre-computed degree-normalized network for homogeneous propagation

**Args:** --network PPInorm_weights.tsv --labels pathways.tsv --method harmonic --use-prenormalized --output norm_propagated.tsv
**Explanation:** This uses the network as already degree-normalized, skipping internal normalization steps and ensuring the edge weights are applied directly as provided, which is critical when external normalization has already been applied.

### Propagate multiple labels simultaneously with multi-class output

**Args:** --network interactome.tsv --labels multi_class.tsv --method random-walk --multi-label --output multi_class_scores.tsv
**Explanation:** This enables multi-label propagation mode where each node can receive multiple simultaneous label predictions (e.g., a gene annotated to multiple functions), outputting comprehensive multi-class score matrices.