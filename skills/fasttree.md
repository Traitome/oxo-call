---
name: fasttree
category: phylogenetics
description: Approximately-maximum-likelihood phylogenetic tree inference optimized for large alignments
tags: [phylogenetics, tree, maximum-likelihood, protein, nucleotide, large-dataset, newick, bootstrap]
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
- -pseudo uses pseudocounts for highly gapped sequences; recommended for sparse alignments.
- -spr enables subtree-prune-regraft moves; -sprlength sets maximum SPR move length (default 10).
- -mlnni sets number of rounds of maximum-likelihood NNIs; -mlacc 2/3 optimizes all 5 branches at each NNI.
- -constraints constrains topology search using an alignment with 0/1/- values defining splits.
- -log saves intermediate trees, settings, and model details to a log file.
- -nosupport disables support value computation for faster execution when supports are not needed.

## Pitfalls

- fasttree has NO subcommands. ARGS starts directly with flags (e.g., -nt, -gtr, -wag, -lg) or the input alignment file. Do NOT put a subcommand like 'build' or 'infer' before flags.
- FastTree outputs to stdout — always redirect: FastTree input.fasta > tree.nwk
- FastTree is faster than IQ-TREE but less accurate — for publications, prefer IQ-TREE with bootstrap.
- The -nt flag is required for nucleotide data — without it, FastTree treats DNA as protein.
- FastTree supports only FASTA and interleaved PHYLIP — not sequential PHYLIP.
- FastTree uses local nearest-neighbor interchange (NNI) — it may not find the globally optimal tree.
- Bootstrap is not a standard feature of FastTree; use -boot N for local support values.
- -constraints may not be fully satisfied; check stderr for 'violating constraints' messages.
- -pseudo adds pseudocounts which can affect branch lengths; interpret results carefully.
- -fastest reduces accuracy significantly; only use for very large datasets (>50,000 sequences).
- FastTreeMP requires OMP_NUM_THREADS environment variable; without it, may use all available cores.

## Examples

### infer phylogenetic tree from nucleotide alignment
**Args:** `-nt -gtr aligned_sequences.fasta > nucleotide_tree.nwk`
**Explanation:** -nt specifies nucleotide data; -gtr GTR substitution model; aligned_sequences.fasta input alignment; output Newick tree to nucleotide_tree.nwk file

### infer phylogenetic tree from protein alignment
**Args:** `aligned_proteins.fasta > protein_tree.nwk`
**Explanation:** aligned_proteins.fasta input alignment; protein data is default (no -nt flag); JTT model default; output Newick to protein_tree.nwk file

### infer tree with WAG protein substitution model
**Args:** `-wag aligned_proteins.fasta > wag_tree.nwk`
**Explanation:** -wag uses WAG substitution model for proteins; aligned_proteins.fasta input alignment; output Newick to wag_tree.nwk file; commonly used for bacterial proteins

### infer tree with local support values
**Args:** `-nt -gtr -boot 1000 -seed 42 aligned_sequences.fasta > tree_with_support.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR substitution model; -boot 1000 local support values; -seed 42 for reproducibility; aligned_sequences.fasta input alignment; output Newick tree to tree_with_support.nwk

### infer tree using multithreaded FastTreeMP
**Args:** `-nt -gtr aligned_sequences.fasta > tree.nwk`
**Explanation:** invoke as FastTreeMP (the multi-threaded binary) instead of FastTree; -nt nucleotide data; -gtr GTR model; aligned_sequences.fasta input alignment; output Newick tree to tree.nwk; set OMP_NUM_THREADS=8 before the command to control thread count

### infer protein tree with LG substitution model
**Args:** `-lg aligned_proteins.fasta > lg_tree.nwk`
**Explanation:** -lg uses the LG model, which is more accurate than JTT/WAG for many protein families; aligned_proteins.fasta input alignment; output Newick to lg_tree.nwk file; recommended over default JTT for modern analyses

### run faster but less thorough tree search
**Args:** `-nt -gtr -fastest aligned_sequences.fasta > fast_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -fastest enables the fastest heuristic (reduces NNI rounds and neighbor comparisons); aligned_sequences.fasta input alignment; output Newick to fast_tree.nwk; suitable for very large alignments where speed is critical

### infer tree with gamma-distributed rate variation
**Args:** `-nt -gtr -gamma aligned_sequences.fasta > gamma_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -gamma uses the exact gamma model instead of CAT approximation; aligned_sequences.fasta input alignment; output Newick to gamma_tree.nwk; more accurate rate variation modeling at higher computational cost

### infer tree from PHYLIP format input
**Args:** `-nt -gtr -n 1 alignment.phy > phylip_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -n 1 specifies one alignment in the file; alignment.phy interleaved PHYLIP format input; output Newick to phylip_tree.nwk

### infer tree with more thorough nearest-neighbor interchange search
**Args:** `-nt -gtr -slownni aligned_sequences.fasta > thorough_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -slownni performs a more exhaustive NNI search at each step; aligned_sequences.fasta input alignment; output Newick to thorough_tree.nwk; improves tree accuracy at the cost of speed; recommended for smaller alignments where precision matters

### infer tree from highly gapped alignment using pseudocounts
**Args:** `-nt -gtr -pseudo aligned_sequences.fasta > pseudo_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -pseudo uses pseudocounts for highly gapped sequences; aligned_sequences.fasta input alignment; output Newick to pseudo_tree.nwk; recommended for sparse alignments to improve numerical stability

### infer tree with constrained topology
**Args:** `-nt -gtr -constraints constraint_alignment.fasta aligned_sequences.fasta > constrained_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -constraints constraint_alignment.fasta specifies alignment with 0/1/- defining splits; aligned_sequences.fasta input alignment; output Newick to constrained_tree.nwk; tree search tries to satisfy constraints (may not be fully met)

### infer tree without support values for faster execution
**Args:** `-nt -gtr -nosupport aligned_sequences.fasta > nosupport_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -nosupport disables support value computation; aligned_sequences.fasta input alignment; output Newick to nosupport_tree.nwk; significantly faster when supports are not needed for downstream analysis

### infer tree with detailed log file
**Args:** `-nt -gtr -log tree_log.txt aligned_sequences.fasta > tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -log tree_log.txt saves intermediate trees, model parameters, and settings; aligned_sequences.fasta input alignment; output Newick to tree.nwk; useful for troubleshooting and reproducibility

### optimize branch lengths on fixed topology
**Args:** `-nt -gtr -nome -mllen -intree input_tree.nwk aligned_sequences.fasta > optimized_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -nome -mllen optimizes branch lengths without changing topology; -intree input_tree.nwk provides starting tree; aligned_sequences.fasta input alignment; output Newick to optimized_tree.nwk; useful for refining approximate trees

### infer tree with custom SPR settings
**Args:** `-nt -gtr -spr -sprlength 15 -mlnni 4 aligned_sequences.fasta > spr_tree.nwk`
**Explanation:** -nt nucleotide data; -gtr GTR model; -spr enables SPR moves; -sprlength 15 allows longer moves; -mlnni 4 increases ML NNI rounds; aligned_sequences.fasta input alignment; output Newick to spr_tree.nwk; more thorough search for better accuracy
