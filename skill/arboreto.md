---
name: arboreto
category: gene-regulatory-network-inference
description: Python tool for inferring gene regulatory networks (GRN) from gene expression data using tree-based ensemble methods such as GENIE3 and GRNBoost2.
tags: [grn, gene-expression, network-inference, bioinformatics, genie3, machine-learning]
author: AI-generated
source_url: https://arboreto.readthedocs.io/
---

## Concepts

- **Input Format**: Arboreto accepts gene expression matrices in tab-delimited (.tsv) or comma-separated (.csv) format where rows represent genes and columns represent samples/conditions. The first column must contain gene names or IDs.
- **Output Format**: The tool produces a weighted edge list (network) where each line contains a source gene, target gene, and an importance score indicating the likelihood of regulatory interaction. Higher scores suggest stronger regulatory relationships.
- **Inference Algorithms**: Arboreto implements two primary algorithms—GENIE3 (Genetic Network Inference with Ensemble of Trees) which uses Random Forest, and GRNBoost2 which uses XGBoost. Both rank gene pairs by their inferred regulatory potential.
- **Parallel Processing**: The `--method` argument selects the algorithm, while `--n-trees` controls the number of trees per ensemble (more trees give better accuracy but increase runtime), and `--cv` sets cross-validation folds for stability estimation.
- **Custom Prior Networks**: The `arboreto-build` companion tool constructs custom gene regulatory network priors from database annotations (e.g., TF-target mappings) that can guide inference via the `--network` argument.

## Pitfalls

- **Missing or Malformed Input**: Supplying a matrix with missing values (NaN), non-numeric characters in the expression matrix, or without gene names in the first column will cause silent failures or biologically meaningless networks. Always validate your expression matrix before running.
- **Insufficient Samples**: Using expression matrices with fewer than 10 samples severely underdetermines the inference problem, leading to spurious edges. GRN inference quality degrades dramatically with small sample sizes; aim for ≥50 samples when possible.
- **Oversized Forests**: Setting `--n-trees` too high (e.g., >5000) without proportional sample counts creates overfitting and generates dense, artefactual networks. Start with 1000–2000 trees for typical datasets.
- **Memory Exhaustion**: Large gene matrices (>10,000 genes) require substantial RAM because the algorithm computes pairwise importance for all gene pairs. Process large matrices in chunks or reduce gene lists to candidate regulators.
- **Misinterpreting Directionality**: Arboreto infers regulatory *potential* but cannot definitively determine causality (source → target). Edge weights represent correlation strength, not proof of direct regulation; experimental validation is always required.

## Examples

### Running basic GRN inference with GENIE3 on an expression matrix

**Args:** `--expmat /data/expression_matrix.tsv --output /output/network.tsv --method genie3`
**Explanation:** This command reads the gene expression matrix, infers regulatory relationships using the GENIE3 algorithm (Random Forest ensemble), and writes the top-ranked gene pairs with importance scores to the output file.

### Using GRNBoost2 algorithm for faster inference

**Args:** `--expmat /data/expression.tsv --output /output/grnboost_network.tsv --method grnboost2`
**Explanation:** This uses the GRNBoost2 (XGBoost-based) algorithm which typically runs faster than GENIE3 while producing comparable network quality, suitable for initial exploratory analysis.

### Adjusting the number of trees to improve accuracy

**Args:** `--expmat /data/expr.tsv --output /output/high_conf_network.tsv --method genie3 --n-trees 2000`
**Explanation:** Increasing the number of trees from the default (1000) to 2000 reduces variance in importance estimates, yielding more stable and reproducible network rankings at the cost of longer runtime.

### Limiting inference to candidate transcription factors only

**Args:** `--expmat /data/expr.tsv --output /output/tf_network.tsv --method genie3 --network /data/tf_prior.tsv`
**Explanation:** This directs the algorithm to prioritize edges where the source gene is a known transcription factor (from the prior network file), reducing the search space and improving biological relevance of inferred edges.

### Running with cross-validation to estimate edge stability

**Args:** `--expmat /data/expr.tsv --output /output/cv_network.tsv --method genie3 --cv 5`
**Explanation:** This performs 5-fold cross-validation to estimate the stability of each inferred edge, adding confidence scores to the output that reflect how consistently each gene pair is ranked across validation folds.