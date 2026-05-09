---
name: cd-hit
category: Sequence Clustering and Redundancy Removal
description: Clustering tool for protein and nucleotide sequences that identifies redundant sequences above a user-specified identity threshold and builds representative sequence sets. Used for creating non-redundant databases, removing duplicates, and analyzing sequence diversity.
tags: clustering, redundancy removal, fasta, sequence identity, bioinformatics, nucleotide sequences, protein sequences
author: AI-generated
source_url: https://github.com/weizhongli/cdhit
---

## Concepts

- **Greedy Incremental Clustering Algorithm**: CD-HIT processes sequences sorted by length (longest first), comparing each new sequence against existing cluster representatives using a fast k-mer based滤. When a sequence exceeds the identity threshold (-c), it joins that cluster; otherwise, it starts a new cluster as a representative. This greedy approach is fast but may not find globally optimal clusters.
- **Sequence Identity Definitions**: The `-c` parameter controls clustering similarity but has different meanings depending on the word size (`-n`). For proteins with `-n 10` or `-n 11`, identity is calculated as the number of identical columns divided by the shorter sequence length (global identity). For nucleotides with `-n 10,11`, identity is the number of identical bases divided by the alignment length (local identity). Using `-n 9` or lower changes to a coverage-based formula.
- **Input and Output Formats**: Input must be in FASTA or FASTG format (sequences must be shorter than 10,000 aa or 100,000 nt by default). Output includes: (1) a FASTA file of cluster representatives/consensus sequences, and (2) a `.clstr` file describing cluster membership with sequence IDs and alignment lengths to the representative. The `-o` flag specifies the base name for both output files; adding `.fab` produces a FASTA file with all sequences labeled by cluster.
- **Memory and Threading Control**: The `-M` option sets memory limits in RAM (e.g., `-M 8000` for 8 GB), preventing out-of-memory crashes on large datasets. The `-T` option controls the number of parallel threads (e.g., `-T 8`). Without `-M` or `-T` limits, CD-HIT may consume excessive resources or fail on clusters with restricted allocation.

## Pitfalls

- **Overly Aggressive Identity Threshold**: Setting `-c` too high (e.g., 0.99 or 1.0) removes sequences with minor variations, potentially discarding biologically meaningful variants or mutations. For most analyses, a threshold of 0.90–0.97 balances redundancy removal with sequence retention.
- **Incompatible Word Size and Sequence Type**: Using `-n 10` or `-n 11` with nucleotide sequences or `-n 4`/`-n 5` with protein sequences produces unreliable clustering because these word sizes are optimized for specific sequence types. The word size determines the scoring matrix; mismatched settings lead to false clusters or missed similarities.
- **Ignoring Sequence Length Sorting**: CD-HIT assumes input sequences are sorted by length (longest first). Providing unsorted input causes shorter variants to be rejected as new clusters rather than grouped with their longer counterparts, reducing clustering accuracy. Pre-sort sequences using `awk` or `sort` before running CD-HIT.
- **Memory Exhaustion on Large Datasets**: Running without `-M` on datasets with millions of sequences causes the process to be killed by the system scheduler. Always set `-M` to a value below the available RAM, and consider using `-T 1` if system memory is constrained to reduce per-thread memory overhead.
- **Confusing Global vs. Local Identity**: The same `-c` value may produce different results for proteins vs. nucleotides due to different identity calculations. Proteins use global alignment identity (shorter sequence as denominator), while nucleotides often use local alignment. Validate clustering results by inspecting the `.clstr` file to ensure biologically expected groupings.

## Examples

### Cluster protein sequences at 90% identity to remove redundancy
**Args:** `-i proteins.fasta -o clusters90 -c 0.90 -n 10 -M 8000`
**Explanation:** Clusters protein sequences at 90% identity using a word size of 10 (optimized for proteins), limiting memory to 8 GB. The output creates `clusters90` (representatives) and `clusters90.clstr` (cluster assignments).

### Exact sequence clustering for identical sequence removal
**Args:** `-i sequences.fasta -o dedup -c 1.0 -n 11 -M 4000 -T 4`
**Explanation:** Removes completely identical sequences using global identity threshold (1.0) with word size 11 (most stringent for proteins), allocating 4 GB memory and using 4 threads for parallel processing.

### Nucleotide sequence clustering at 97% identity using cd-hit-est
**Args:** `-i nucleotides.fasta -o noclusters97 -c 0.97 -n 10 -M 16000`
**Explanation:** Clusters nucleotide sequences at 97% identity suitable for removing duplicate reads or very similar variants, using the EST/nucleotide-specific word size and 16 GB memory limit.

### Compare sequences between two databases to find cross-cluster matches
**Args:** `-i database1.fasta -i2 database2.fasta -o comparison -c 0.95 -n 9 -M 8000`
**Explanation:** Compares sequences in database1 against database2, outputting clusters where database2 sequences match database1 clusters above 95% identity, useful for finding orthologs or shared sequences.

### Find duplicate sequences within a single database
**Args:** `-i input.fasta -o duplicates_found -c 1.0 -n 11 -M 4000 -aS 0.9`
**Explanation:** Identifies duplicate sequences by requiring perfect identity (1.0) with 90% minimum alignment coverage (`-aS 0.9`), outputting to `duplicates_found.clstr` for downstream duplicate handling.

### Process large dataset with limited computational resources
**Args:** `-i huge_dataset.fasta -o filtered -c 0.80 -n 5 -M 2000 -T 2`
**Explanation:** Clusters a large dataset using a lower 80% identity threshold with reduced word size (5) and strict resource limits (2 GB memory, 2 threads), trading clustering precision for feasibility on constrained systems.