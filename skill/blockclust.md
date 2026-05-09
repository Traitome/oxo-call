---
name: blockclust
category: bioinformatics/clustering
description: A tool for clustering RNA-Seq data using ViennaRNA-based secondary structure predictions and sequence similarity. Assigns sequences to clusters based on local alignment against representative centroids and structural compatibility.
tags:
  - RNA-Seq
  - clustering
  - secondary-structure
  - sequence-analysis
  - bioinformatics
author: AI-Generated
source_url: https://github.com/ibestCSIL/blockclust
---

## Concepts

- blockclust clusters sequences by combining local alignment scores (computed with dlib) against pre-computed centroid sequences and structural compatibility (assessed via ViennaRNA). The clustering is robust against sequencing errors and length variations because it relies on local alignment rather than global alignment.
- Input files must be in FASTA format for both the centroid sequences (with associated secondary structure annotations in dot-bracket notation) and the query sequences to be clustered.
- Output is written as a text-based cluster file where each line corresponds to a query sequence ID followed by its assigned cluster identifier. The format is straightforward and can be easily parsed by downstream scripts.
- The companion binary `blockclust-build` constructs the centroid database from a set of labeled FASTA sequences with secondary structure annotations. This step must be completed before running `blockclust`.
- Clustering accuracy depends on two key parameters: the `-w` (alignment word size) and `-s` (structural weight). Increasing `-w` makes alignment stricter, while adjusting `-s` balances the contribution of structural compatibility versus sequence similarity.

## Pitfalls

- Using FASTA files with invalid or inconsistent secondary structure annotations (e.g., mismatched parentheses) causes blockclust to fail or produce unreliable clusters, because ViennaRNA parsers will reject malformed dot-bracket strings.
- Setting the `-w` word size too small (e.g., below 5) leads to an excessive number of false-positive cluster assignments, as short words match frequently by chance across unrelated sequences.
- Omitting the `-t` parameter when the query dataset is large causes all sequences to be loaded into memory at once, which may trigger out-of-memory errors on systems with limited RAM.
- Running `blockclust` without first executing `blockclust-build` results in a missing database error and the tool aborts, because clustering requires the pre-built centroid index to score alignments against.
- Confusing the order of positional arguments (centroid file vs. query file) produces swapped cluster assignments, meaning query sequences are scored against the wrong reference set.

## Examples

### Cluster query sequences using a pre-built centroid database

**Args:** centroids.fa query_sequences.fa output.clusters
**Explanation:** This runs blockclust using the default alignment word size (6) and equal weight for sequence and structural features, assigning each query sequence to its best-matching centroid cluster.

### Cluster with a custom alignment word size for stricter matching

**Args:** -w 8 centroids.fa query_sequences.fa output.clusters
**Explanation:** Increasing the alignment word size from the default to 8 requires longer exact matches during local alignment, which reduces incorrect cluster assignments in highly conserved sequence families.

### Cluster with increased structural compatibility weight

**Args:** -s 0.8 centroids.fa query_sequences.fa output.clusters
**Explanation:** Setting the structural weight to 0.8 biases clustering toward sequences with thermodynamically compatible secondary structures, which is useful for families where structure is more conserved than primary sequence.

### Cluster using a specific number of parallel threads

**Args:** -t 4 centroids.fa query_sequences.fa output.clusters
**Explanation:** Using four threads allows blockclust to process the query dataset in parallel, significantly speeding up clustering for large query sets at the cost of higher CPU utilization.

### Cluster with both custom word size and structural weight

**Args:** -w 7 -s 0.6 centroids.fa query_sequences.fa output.clusters
**Explanation:** Combining a word size of 7 with a structural weight of 0.6 provides a balanced trade-off between sequence specificity and structural conservation, suitable for datasets with moderate sequence diversity.

### Build a centroid database from labeled FASTA sequences

**Args:** blockclust-build training_sequences.fa centroids.db
**Explanation:** This companion binary constructs the centroid index required for clustering, extracting representative sequences and their secondary structure annotations from the training set into an efficient binary format.