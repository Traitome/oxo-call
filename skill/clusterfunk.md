---
name: clusterfunk
category: Bioinformatics/Functional Analysis
description: A command-line tool for clustering biological sequences or functional annotations. Clusterfunk groups similar sequences or gene functions into clusters based on sequence similarity, enabling downstream analysis of gene families, functional groups, or metagenomic bins.
tags:
  - clustering
  - sequence-analysis
  - functional-annotation
  - bioinformatics
  - genomics
author: AI-generated
source_url: https://github.com/clusterfunk/clusterfunk
---

## Concepts

- **Input Format:** Clusterfunk accepts FASTA or FASTQ files containing biological sequences (nucleotide or amino acid). Sequences are compared pairwise or via hash-based methods to determine similarity scores.
- **Clustering Algorithm:** The tool implements several clustering methods including hierarchical clustering, CD-HIT-like sequence identity clustering, and graph-based clustering. Users can specify identity thresholds (e.g., `--identity 0.97` for 97% sequence identity).
- **Output Formats:** Generated clusters are written to file formats including clustr format (cluster per line), matrix format (pairwise similarity matrix), or JSON for downstream programmatic analysis. The default output is a cluster assignment file mapping each sequence to a cluster ID.
- **Parallelization:** Clusterfunk supports multithreading via the `--threads` flag to accelerate pairwise comparisons on multi-core systems, significantly improving performance on large datasets (>10,000 sequences).

## Pitfalls

- **Low Identity Threshold:** Setting `--identity` too low (e.g., below 0.5) produces overly broad clusters that merge functionally unrelated sequences, obscuring biological signal in downstream analyses.
- **Memory Exhaustion:** Running without the `--chunk-size` parameter on very large datasets (>100,000 sequences) can cause memory overflow, as the tool stores a similarity matrix in RAM during clustering.
- **Missing Sequence Headers:** Providing FASTA files without unique identifiers causes cluster assignments to map to blank or duplicate headers, making downstream interpretation impossible. Always ensure unique sequence names.
- **Sorted Input Requirement:** Some clustering modes require pre-sorted sequences by length (longest to shortest). Running on unsorted input produces non-reproducible or suboptimal clusterings.

## Examples

### Cluster sequences at 95% identity threshold
**Args:** `--input sequences.fasta --identity 0.95 --output clusters.txt`
**Explanation:** Groups sequences sharing at least 95% identity into clusters, appropriate for clustering highly similar variants or close homologs.

### Use hierarchical clustering with average linkage
**Args:** `--input genes.fasta --method hierarchical --linkage average --distance 0.1 --output clusters.txt`
**Explanation:** Performs hierarchical clustering using average linkage and a distance cutoff of 0.1, producing a dendrogram-style cluster structure.

### Enable multithreaded processing for large datasets
**Args:** `--input large_set.fasta --identity 0.90 --threads 8 --output clusters.txt`
**Explanation:** Runs clustering with 8 threads to accelerate computations on datasets with more than 10,000 sequences.

### Output clusters in JSON format for programmatic use
**Args:** `--input seqs.fasta --identity 0.85 --format json --output clusters.json`
**Explanation:** Produces machine-readable JSON output listing sequence-to-cluster mappings, suitable for integration into pipelines.

### Limit memory usage with chunking
**Args:** `--input huge_dataset.fasta --identity 0.80 --threads 4 --chunk-size 5000 --output clusters.txt`
**Explanation:** Processes sequences in chunks of 5,000 to prevent memory overflow while maintaining parallel processing capability.