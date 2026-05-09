---
name: cell2cell
category: Single-Cell Analysis
description: A tool for inferring cell-cell communication networks from single-cell RNA-seq data using ligand-receptor interaction analysis. It calculates communication probabilities between cell populations and generates network visualizations.
tags: [single-cell, cell-cell-communication, ligand-receptor, networks, scrna-seq]
author: AI-generated
source_url: https://github.com/ventolab/cell2cell
---

## Concepts

- **Input Data Model**: cell2cell accepts gene expression matrices (CSV or Market Matrix format) paired with cell metadata containing cell-type annotations. The expression matrix must have genes as rows and cells as columns, with gene symbols in a dedicated column or as row names.
- **Ligand-Receptor Database**: The tool uses a built-in database of ligand-receptor interaction pairs (human, mouse, and other species). For custom analysis, users can provide a custom ligand-receptor pair file in CSV format with ligand and receptor columns.
- **Communication Scoring Algorithm**: cell2cell computes communication probabilities using the average expression of ligand genes in sender cells and receptor genes in receiver cells, applying a permutation test to assess significance of interactions.
- **Output Formats**: Results include CSV files with interaction scores between each cell-type pair, a Docker-friendly JSON summary, and optional visualization outputs (heatmaps, network graphs in GraphML format).

## Pitfalls

- **Column Name Mismatch**: Using incorrect column names in the metadata file (e.g., "cell_type" instead of "CellType") causes the tool to fail silently, producing empty results without error messages.
- **Gene Symbol Version Inconsistency**: Mismatched gene symbols between expression data and the ligand-receptor database (e.g., alias names like "CD44R" vs. official symbols) lead to missing interactions, drastically reducing detected communication events.
- **Unfiltered Low-Expression Genes**: Running analysis without removing lowly expressed genes introduces noise, causing false-positive interactions and inflated network complexity that obscures meaningful biological signals.
- **Memory Limits with Large Datasets**: For datasets with >50,000 cells, failing to split analysis by cell-type groups causes memory overflow, resulting in crashed sessions and lost computation time.

## Examples

### Building a custom ligand-receptor database
**Args:** build --lr-pairs custom_lr.csv --database customDB.csv --species human
**Explanation:** This creates a custom interaction database from user-provided ligand-receptor pairs for human cells, allowing analysis with non-standard interaction hypotheses.

### Running communication analysis on a CSV expression matrix
**Args:** run --expression expt_matrix.csv --metadata cell_metadata.csv --output out/ --format csv
**Explanation:** This executes the full communication analysis pipeline using CSV-formatted expression data and cell-type annotations, outputting results to the specified directory.

### Analyzing with permutation testing for significance
**Args:** run --expression expr.h5ad --metadata meta.csv --method permutations --n-perms 1000 --output results/
**Explanation:** This runs cell-cell communication analysis with 1000 permutation tests to compute statistical significance of each ligand-receptor interaction between cell types.

### Filtering genes before analysis
**Args:** run --expression raw_counts.csv --metadata meta.csv --min-cells 10 --min-expression 1.0 --output filtered_out/
**Explanation:** This filters genes expressed in fewer than 10 cells or with mean expression below 1.0 before running communication inference, reducing noise in final results.

### Generating network visualizations
**Args:** run --expression expr.csv --metadata meta.csv --visualize --net-format graphml --output viz_out/
**Explanation:** This produces network graph files in GraphML format alongside the numerical results, enabling downstream network analysis and custom visualization in tools like Cytoscape.

### Running analysis on mouse data
**Args:** run --expression mouse_scRNA.csv --metadata mouse_meta.csv --species mouse --database lr_mouse.csv --output mouse_out/
**Explanation:** This runs cell-cell communication analysis specifically for mouse data using a mouse ligand-receptor database, ensuring correct gene symbol matching.