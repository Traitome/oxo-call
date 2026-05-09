---
name: clearcut
category: phylogenetics
description: Relaxed neighbor-joining tool for rapid phylogenetic tree construction from multiple sequence alignments, producing Newick-format output trees.
tags:
  - phylogenetics
  - tree-building
  - sequence-alignment
  - neighbor-joining
  - bioinformatics
author: AI-generated
source_url: https://github.com/refresh-bio/clearcut
---

## Concepts

- **Input requires a pre-aligned multiple sequence alignment.** Clearcut does not perform alignment itself; it consumes FASTA, Nexus, or PHYLIP alignment files. Raw unaligned sequences will produce incorrect or failed results. The alignment quality directly determines tree reliability.
- **Output is Newick format written to stdout by default.** The resulting tree can be redirected to a file or piped to visualization tools like FigTree, iTOL, or DendroPy. Newick format encodes both topology and branch lengths, making it interoperable with most phylogenetic analysis software.
- **The relaxed neighbor-joining algorithm scales to large alignments.** Unlike standard neighbor-joining with O(N³) complexity, the relaxed approach handles thousands of sequences efficiently. This makes clearcut suitable for whole-genome or metagenomic surveys where fast approximation is acceptable.
- **Substitution model flags control distance calculations.** Models such as Kimura two-parameter (kimura), Jukes-Cantor (jukes-cantor), or custom matrices affect branch length estimates. Mismatched models for the data type (e.g., nucleotide vs. protein) produce biased trees with incorrect evolutionary distances.

## Pitfalls

- **Feeding unaligned sequences causes silent failure or garbage output.** Clearcut assumes column-wise positional homology. Passing raw FASTA sequences without prior alignment produces a tree that reflects random similarity rather than evolutionary relationships, rendering downstream analysis meaningless.
- **Specifying an incorrect sequence type (nucleotide vs. protein) corrupts branch lengths.** The `--protein` flag switches the substitution matrix to amino-acid models. Forgetting this flag for protein alignments or including it for nucleotide data yields systematically wrong distance estimates that cannot be fixed by re-rooting.
- **Output overwriting without confirmation loses previous results.** The `-o`/`--output` flag writes directly without prompting. If the same filename exists from a prior run, it is replaced unconditionally, and no automatic backup is created.
- **Ignoring gap handling settings inflates topological uncertainty.** Gap-rich regions treated with default settings may cause the algorithm to skip entire columns, altering the effective alignment length and producing inconsistent trees across runs. This is especially problematic for PCR amplicon data with variable trimming.

## Examples

### Build a phylogenetic tree from a FASTA alignment
**Args:** `-i alignment.fasta -o tree.nwk`
**Explanation:** The `-i` flag specifies the input multiple sequence alignment file, and `-o` redirects the Newick output to a named file for downstream use.

### Build a tree using the Kimura two-parameter nucleotide model
**Args:** `--model kimura -i alignment.fasta`
**Explanation:** The `--model kimura` flag applies a transition/transversion correction appropriate for nucleotide alignments, yielding more realistic branch lengths than simple distance models.

### Construct a tree from a Nexus-formatted alignment
**Args:** `--format nexus -i alignment.nex -o tree.nwk`
**Explanation:** The `--format nexus` flag explicitly declares the input format, ensuring correct parsing of Nexus-specific charactersets or partition blocks that FASTA cannot represent.

### Build a tree for a protein alignment with Dayhoff substitution model
**Args:** `--protein --model dayhoff -i protein_alignment.fasta -o protein_tree.nwk`
**Explanation:** Both `--protein` and `--model dayhoff` must be specified together to apply the correct amino-acid substitution matrix; omitting `--protein` causes the tool to misread codons as nucleotides.

### Suppress output of branch lengths for topology-only analysis
**Args:** `--no-lengths -i alignment.fasta`
**Explanation:** The `--no-lengths` flag outputs topology without branch lengths, which is useful for downstream bipartition support or consensus tree calculations where branch length scaling is irrelevant.