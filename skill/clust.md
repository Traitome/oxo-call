---
name: clust
category: sequence_analysis
description: A bioinformatics tool for clustering biological sequences based on pairwise identity thresholds. Groups similar sequences into clusters, keeping a representative sequence for each cluster. Commonly used for dereplicating sequence datasets, analyzing diversity, and reducing redundancy in FASTA/Q files.
tags:
- clustering
- sequence-analysis
- bioinformatics
- dereplication
- cd-hit
- identity
author: AI-generated
source_url: https://github.com/bioinformatics-tools/clust
---

## Concepts

- **Input Format**: Accepts FASTA and FASTQ files containing nucleotide or amino acid sequences. The tool reads sequences line-by-line, computes pairwise identities, and clusters sequences exceeding the identity threshold.
- **Identity Threshold**: The core parameter controlling cluster granularity. A 0.97 threshold means sequences with ≥97% identity are grouped together. Lower thresholds create fewer, larger clusters; higher thresholds produce more, smaller clusters.
- **Representative Selection**: For each cluster, one sequence serves as the representative (often the longest or first encountered). The representative retains annotations while member sequences are discarded in standard output modes.
- **Output Modes**: Produces cluster assignments (mapping sequence to cluster ID), cluster centroids (representative sequences), and optionally both. The clustered file contains representatives with new identifiers encoding cluster membership.
- **Algorithm Modes**: Word-based (fast, uses k-mer matching for pre-clustering) and alignment-based (accurate, performs full Needleman-Wunsch or local alignment). Word mode is suitable for whole-genome assembly clustering; alignment mode for gene or marker sequences.

## Pitfalls

- **Setting threshold too high (≥0.99)**: Clusters become trivially small or singletons, defeating the purpose of dereplication. This produces minimal file size reduction and may retain near-identical variants that should be merged.
- **Using incompatible word lengths**: Setting word length shorter than appropriate for sequence complexity causes false clustering of unrelated sequences. Too long misses true matches, especially in highly variable regions.
- **Forcing alignment mode on large datasets**: Full alignment mode scales poorly (O(n²) worst case). A 100,000 sequence input can take hours or crash. Use word-based pre-clustering first or subsample.
- **Ignoring directionality**: By default, clust treats sequences as untemplated. For oriented sequences (e.g., from stranded RNA-seq), failing to set strand-specific flags mixes forward and reverse complements into wrong clusters.
- **Not preserving sequence annotations**: Default output overwrites original headers with cluster IDs. Annotations in description lines (taxonomy, abundance, sample info) are lost unless the full header preservation mode is enabled.

## Examples

### Clustering a FASTA file at 97% identity using word-based algorithm
**Args:** `-i input.fasta -o clustered.fasta -t 0.97 -c 8 -d`
**Explanation:** This clusters sequences at 97% identity using word-based algorithm with 8 threads and discards member sequences, outputting only representatives.

### Clustering reads and saving both representatives and cluster mapping
**Args:** `-i reads.fq -o clusters.fasta -t 0.95 --outmap cluster_map.txt -M`
**Explanation:** Runs clustering at 95% identity and outputs cluster representatives plus a mapping file listing which read belongs to which cluster.

### Using alignment-based clustering with word size 12
**Args:** `-i sequences.fasta -o clustered.fasta -t 0.98 -a -w 12 -g 0.5`
**Explanation:** Uses alignment mode with 12-mer word size and 0.5 gap penalty for accurate clustering of closely related sequences.

### Clustering amino acid sequences with longer coverage requirement
**Args:** `-i proteins.faa -o protein_clusters.fasta -t 0.90 -L 0.8 -F protein`
**Explanation:** Clusters protein sequences requiring 90% identity and 80% coverage (alignment length fraction), suitable for protein families.

### Parallel clustering with 16 threads and verbose logging
**Args:** `-i dataset.fasta -o output.fasta -t 0.96 -c 16 -v -s 1000`
**Explanation:** Runs clustering with 16 CPUs, shows progress, and uses 1000 sequence buffer for memory-efficient processing of large files.

### Clustering and keeping full headers with cluster statistics
**Args:** `-i input.fasta -o clustered_out.fasta -t 0.94 --preserve-headers -S stats.txt`
**Explanation:** Preserves original sequence headers in output and saves a statistics file with cluster sizes and distributions.

### Dereplicating FASTQ files while preserving quality scores
**Args:** `-i reads.fastq -o derep.fastq -t 0.99 -q -c 4`
**Explanation:** Performs near-exact (99%) dereplication on FASTQ, keeping quality scores intact and using 4 CPU threads.