---
name: bigscape
category: Comparative genomics / Secondary metabolite analysis
description: BiG-SCERP (BiG-Sibling Cluster Enzyme Retrieval Protocol) is a bioinformatics tool for analyzing and visualizing similarity networks of biosynthetic gene clusters (BGCs). It takes antiSMASH or similar BGC prediction outputs and generates cluster similarity networks based on PFAM domain content, enabling comparative analysis of secondary metabolite potential across microbial genomes.
tags:
- biosynthetic gene clusters
- secondary metabolites
- natural products
- antiSMASH
- BGC similarity
- network analysis
- microbial genomics
author: AI-generated
source_url: https://github.com/medema-group/BiG-SCERP
---

## Concepts

- **Input format**: BiG-SCE accepts antiSMASH results in GenBank (.gbk) or JSON format. Each file should represent one predicted biosynthetic gene cluster from antiSMASH. The tool parses the PFAM domains annotated within each BGC to compute domain-based similarity.
- **Similarity calculation**: The tool computes all-vs-all pairwise similarity between BGCs using the Jaccard index of shared PFAM domains. A minimum overlap requirement (typically 2 PFAMs in common) must be met before similarity is calculated.
- **Clustering algorithms**: After computing the similarity matrix, BiG-SCE applies clustering algorithms—most commonly MCL (Markov Cluster Algorithm)—to group BGCs into families (BiG-SCE families) based on similarity cutoff thresholds.
- **Output generation**: Results include a similarity table (pairwise scores), a network file (edges representing similarities above cutoff), and clusters assigned to each BGC. The network can be visualized with tools like Cytoscape or included web viewers.

## Pitfalls

- **Using antiSMASH version mismatches**: Running BiG-SCE on BGC predictions from different antiSMASH versions can produce inconsistent PFAM annotations, leading to unreliable similarity networks. Always use antiSMASH v5 or v6 outputs consistently for comparative analyses.
- **Setting similarity cutoff too low**: A low cutoff (e.g., 0.1) creates overly broad clusters that merge functionally unrelated BGCs, obscuring real natural product families. Higher cutoffs (0.3–0.5) yield more specific groupings.
- **Ignoring the --include-ghosts flag**: BGCs that are too short or have few PFAM annotations (ghost clusters) are excluded by default, potentially removing legitimate small BGCs. If analyzing small peptide BGCs, verify inclusion.
- **Mixed input folder contents**: Placing GenBank files from different organisms or non-BGC files in the input directory causes cross-organism false positives. Ensure input folder contains only antiSMASH-predicted BGC files for the target analysis.
- **Memory usage on large datasets**: Running with thousands of BGCs without adjusting the `--cache_pfams` and batch processing can cause memory exhaustion. Process large datasets in batches or use the `--chunk` option to distribute computation.

## Examples

### Analyze a directory of antiSMASH GenBank files with default parameters
**Args:** -i input/gbk_files -o output_folder --families-gfx
**Explanation:** This runs BiG-SCE on all GenBank files in the input directory, computes similarity using default PFAM overlap, clusters using default MCL parameters, and generates network visualization graphics.

### Set a stricter similarity cutoff to get tighter BGC families
**Args:** -i input/gbk_files -o output_strict --cutoff 0.4 --families-gfx
**Explanation:** Raising the similarity cutoff to 0.4 ensures only BGCs sharing more PFAM domains are grouped together, producing narrower, more biologically relevant families.

### Run clustering using the Euclidean distance method
**Args:** -i input/gbk_files -o output_euclidean --clust-method euclidean --families-gfx
**Explanation:** Euclidean clustering uses continuous domain counts rather than binary presence/absence, which can reveal subtler relationships between BGCs with varying domain copies.

### Skip network visualization to speed up analysis for large datasets
**Args:** -i input/gbk_files -o output_fast --no-gfx
**Explanation:** Disabling the families-gfx step removes visualization generation, significantly reducing runtime when only cluster assignments or similarity matrices are needed.

### Use with BiG-SCERP advanced options for custom PFAM weighting
**Args:** -i input/gbk_files -o output_weighted --pfam-weight key-pfams.txt --families-gfx
**Explanation:** Providing a custom weight file allows prioritizing specific PFAM domains (like key tailoring enzymes) during similarity calculation, improving functional relevance of the resulting clusters.