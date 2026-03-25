---
name: fasttree
category: phylogenetics
description: Approximately-maximum-likelihood phylogenetic tree inference optimized for large alignments
tags: [phylogenetics, tree, maximum-likelihood, protein, nucleotide, large-dataset]
author: oxo-call built-in
source_url: "http://www.microbesonline.org/fasttree/"
---

## Concepts

- FastTree infers approximately-maximum-likelihood phylogenetic trees; much faster than IQ-TREE/RAxML for large datasets.
- FastTree uses GTR+CAT model for DNA and JTT+CAT for proteins; accepts aligned FASTA or PHYLIP input.
- Use FastTree (nucleotide, GTR) or FastTree (protein, JTT/WAG/LG) with appropriate flags.
- Use -nt for nucleotide alignment; default is protein.
- Use -gtr for GTR model (recommended for divergent sequences); default is JC for nucleotide.
- FastTree outputs Newick-format tree to stdout — redirect to a file.
- For reproducibility, use -seed N to set random seed.
- FastTreeMP (multi-threaded) is available as a separate binary for faster computation.

## Pitfalls

- FastTree outputs to stdout — always redirect: FastTree input.fasta > tree.nwk
- FastTree is faster than IQ-TREE but less accurate — for publications, prefer IQ-TREE with bootstrap.
- The -nt flag is required for nucleotide data — without it, FastTree treats DNA as protein.
- FastTree supports only FASTA and interleaved PHYLIP — not sequential PHYLIP.
- FastTree uses local nearest-neighbor interchange (NNI) — it may not find the globally optimal tree.
- Bootstrap is not a standard feature of FastTree; use -boot N for local support values.

## Examples

### infer phylogenetic tree from nucleotide alignment
**Args:** `-nt -gtr aligned_sequences.fasta > nucleotide_tree.nwk`
**Explanation:** -nt specifies nucleotide data; -gtr GTR substitution model; output Newick tree to file

### infer phylogenetic tree from protein alignment
**Args:** `aligned_proteins.fasta > protein_tree.nwk`
**Explanation:** protein data is default (no -nt flag); JTT model default; output Newick to file

### infer tree with WAG protein substitution model
**Args:** `-wag aligned_proteins.fasta > wag_tree.nwk`
**Explanation:** -wag uses WAG substitution model for proteins; commonly used for bacterial proteins

### infer tree with local support values
**Args:** `-nt -gtr -boot 1000 -seed 42 aligned_sequences.fasta > tree_with_support.nwk`
**Explanation:** -boot 1000 local support values; -seed 42 for reproducibility

### infer tree using multithreaded FastTreeMP
**Args:** `-nt -gtr aligned_sequences.fasta > tree.nwk`
**Explanation:** invoke as FastTreeMP (the multi-threaded binary) instead of FastTree; set OMP_NUM_THREADS=8 before the command to control thread count

### infer protein tree with LG substitution model
**Args:** `-lg aligned_proteins.fasta > lg_tree.nwk`
**Explanation:** -lg uses the LG model, which is more accurate than JTT/WAG for many protein families; recommended over default JTT for modern analyses

### run faster but less thorough tree search
**Args:** `-nt -gtr -fastest aligned_sequences.fasta > fast_tree.nwk`
**Explanation:** -fastest enables the fastest heuristic (reduces NNI rounds and neighbor comparisons); suitable for very large alignments where speed is critical

### infer tree with gamma-distributed rate variation
**Args:** `-nt -gtr -gamma aligned_sequences.fasta > gamma_tree.nwk`
**Explanation:** -gamma uses the exact gamma model instead of CAT approximation; more accurate rate variation modeling at higher computational cost

### infer tree from PHYLIP format input
**Args:** `-nt -gtr -n 1 alignment.phy > phylip_tree.nwk`
**Explanation:** FastTree accepts interleaved PHYLIP format; -n 1 specifies one alignment in the file; output is Newick format to stdout

### infer tree with more thorough nearest-neighbor interchange search
**Args:** `-nt -gtr -slownni aligned_sequences.fasta > thorough_tree.nwk`
**Explanation:** -slownni performs a more exhaustive NNI search at each step; improves tree accuracy at the cost of speed; recommended for smaller alignments where precision matters
