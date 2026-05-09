---
name: cdhit-reader
category: Sequence Clustering Analysis
description: Parse and extract data from CD-HIT cluster files (.clstr) including representative sequences, cluster membership, and identity metrics. Useful for downstream analysis of sequence clustering results from CD-HIT or CD-HIT-EST.
tags:
  - cdhit
  - clustering
  - fasta
  - sequence-analysis
  - representatives
  - bioinformatics
author: AI-generated
source_ url: https://github.com/weizhongli/cdhit
---

## Concepts

- **Cluster File Format (.clstr)**: CD-HIT outputs a `.clstr` file where each cluster is separated by a blank line, and each sequence entry contains the cluster ID, sequence length, and percentage identity to the representative. Lines begin with `>Cluster 0`, `>Cluster 1`, etc., making them parseable by line-based scanning.

- **Representative Selection**: In CD-HIT clustering, the first sequence added to a cluster becomes the representative (centroid). When using `-r 1` mode, only representatives are extracted; otherwise all sequences are processed. This matters for workflows needing unique sequences vs. full cluster membership.

- **Sequence Database Linking**: cdhit-reader typically requires both the `.clstr` file and the original FASTA sequence database to extract sequence data. The sequence IDs in the cluster file must match those in the FASTA file for proper linking.

- **Identity Threshold Context**: The identity values reported (e.g., `100%`) refer to local alignment identity within the alignment overlap region, not global sequence identity. This distinction is critical when interpreting cluster quality.

## Pitfalls

- **Mismatched Sequence IDs**: If sequence headers in the FASTA file have been modified (e.g., by adding prefixes or truncating) after CD-HIT clustering, cdhit-reader will fail to link cluster assignments to actual sequences, resulting in missing or empty output.

- **Ignoring the `.bak` File Warning**: CD-HIT creates a `.bak` backup file when run. If you accidentally use the backup file as input instead of the actual cluster file, you may reprocess outdated clustering results or encounter format errors.

- **Assuming Global Identity**: Reporting 97% identity in a CD-HIT cluster does not mean two sequences are 97% identical across their full length. It reflects the identity within the alignment region, which can be misleading for downstream evolutionary analysis.

- **Memory Issues with Large Clusters**: Processing extremely large CD-HIT output files (millions of sequences) without streaming or chunked processing may cause memory exhaustion, especially when extracting full sequences.

## Examples

### Extract all cluster representatives from a cluster file
**Args:** `-i clusters.clstr -d refdb.faa -o representatives.faa -r 1`
**Explanation:** The `-r 1` flag instructs cdhit-reader to extract only representative sequences (first sequence per cluster), writing them to the output FASTA file linked via the input sequence database.

### Parse cluster membership with full length information
**Args:** `-i clusters.clstr -d refdb.faa -o members.txt -c -L`
**Explanation:** Using the cluster parsing flag combined with length output produces a tabular file showing each cluster ID, member sequence, and sequence length for downstream statistical analysis.

### Filter clusters by minimum size before extraction
**Args:** `-i clusters.clstr -d refdb.faa -o large_clusters.faa -s 10`
**Explanation:** The `-s 10` flag filters to extract only representatives from clusters containing at least 10 members, useful for focusing on conserved or abundant sequence families.

### Generate a cluster-to-sequence mapping table
**Args:** `-i clusters.clstr -o cluster_map.tsv -m mapping`
**Explanation:** This outputs a tab-separated mapping file where each row links cluster IDs to their member sequence identifiers, enabling integration with other bioinformatics pipelines or visualization tools.

### Extract non-representative sequences for variant analysis
**Args:** `-i clusters.clstr -d refdb.faa -o variants.faa -r 0 -e`
**Explanation:** Using `-r 0` excludes representatives while `-e` includes all non-centroid sequences, providing the variant members for each cluster for downstream mutation or conservation analysis.