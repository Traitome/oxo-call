---
name: cawlign
category: sequence-alignment
description: A fast codon-aware nucleotide and protein sequence alignment tool that optimizes for phylogenetic inference by preserving reading frame integrity and handling frame-shifted indels gracefully.
tags:
  - sequence-alignment
  - codon-alignment
  - phylogenetics
  - nucleotide-alignment
  - protein-alignment
  - bioinformatics
author: AI-generated
source_url: https://github.com/example/cawlign
---

## Concepts

- **Codon-aware alignment mode**: cawlign treats codons as atomic alignment units rather than individual nucleotides, preventing the insertion of frameshifts that would disrupt the reading frame in protein-coding sequences. Enable this with `--codon`; when active, gaps are inserted in multiples of three to maintain synchrony.

- **Progressive alignment with iterative refinement**: The tool uses a guide tree built from a distance matrix (by default, identity-based for nucleotides or BLOSUM62 for proteins) to guide progressive alignment of sequences, followed by iterative refinement rounds to improve the final score. The `--iterations` flag controls refinement depth.

- **Input formats and ambiguity handling**: cawlign accepts FASTA, Clustal, and PHYLIP formats for input via `--informat`. Ambiguous nucleotide codes (e.g., R, Y, N) are preserved during alignment rather than being expanded, which matters for population genetic datasets where IUPAC codes carry heterozygosity information.

- **Output scoring based on transversion weighting**: For nucleotide alignments, cawlign applies transversion-biased scoring (weight 2.0 for TV, 1.0 for TI) by default, reflecting the higher mutation cost of transversions. This improves evolutionary accuracy in AT/GC-rich genomes where transitions are saturated.

- **Pairwise alignment seed-and-extend**: For small numbers of sequences (≤4 by default, tunable with `--pairwisethresh`), the tool bypasses progressive alignment and performs an exact Needleman-Wunsch alignment, ensuring optimal results for benchmarking or when computational cost is not a concern.

## Pitfalls

- **Aligning coding sequences without `--codon`**: Running cawlign on protein-coding nucleotides without the `--codon` flag allows gaps to be inserted at any position, creating frameshift mutations that corrupt translations downstream. The resulting alignment will be meaningless for codon-based phylogenetic models (e.g., PAML, HyPhy) because codons are split across gap characters.

- **Misinterpreting `--gapopen` as a per-gap penalty**: Each gap-open event incurs the full `--gapopen` penalty plus `--gapextend` multiplied by gap length minus one. Users who set a low gapopen value expecting cheap indels will get unexpectedly high penalties for long insertions. For heavily indel-prone sequences (microsatellites, TEs), increase `--gapextend` and use `--opengap 15 --extgap 1`.

- **Using nucleotide scoring matrices for protein alignments**: Passing `--matrix DNA` for protein sequences ignores the actual amino acid substitution biology (conservative vs. radical substitutions are treated equally). Protein alignments will be less phylogenetically accurate. Always use `--matrix AA` or specific matrices like `--matrix BLOSUM62` for proteins.

- **Ignoring `--mincol` for downstream masking**: After alignment, columns with excessive gaps or ambiguity characters can mislead ModelTest or IQ-Tree. cawlign does not auto-mask these by default; set `--mincol 0.5` to remove columns where more than 50% of sequences have gaps or N characters before exporting for phylogenetics.

- **Specifying output format without checking downstream tool compatibility**: cawlign's native `.cwl` format (activated by `--format cwl`) is not recognized by RAxML, IQ-Tree, or BEAST. Converting to PHYLIP (`--format phylip`) or Nexus (`--format nexus`) is required for most phylogenetic software. Using the wrong format causes silent failures in downstream tools.

## Examples

### Align a small set of protein-coding DNA sequences using codon-aware mode

**Args:** `sequences.fasta --codon --outfmt fasta --output aligned_codons.fasta`
**Explanation:** The `--codon` flag forces triplet alignment units, preserving reading frames for protein-coding genes, and `--outfmt fasta` produces standard output readable by most tools.

### Generate a nucleotide alignment with transversion-biased scoring for a phylogeny

**Args:** `--input virus_seqs.fa --scoring TV --matrix DNA --gapopen 10 --gapextend 2 --iterations 3`
**Explanation:** Explicitly setting `--scoring TV` ensures transversions are weighted double relative to transitions, which is appropriate for viral序列 with high substitution rates.

### Align multiple sequences progressively with automatic format detection

**Args:** `--informat auto dataset.clw --codon --mincol 0.7 --output clean_aln.fasta`
**Explanation:** `--informat auto` detects Clustal input, `--mincol 0.7` removes columns with >30% missing data, producing a cleaner alignment for ModelTest or phylogenetic reconstruction.

### Produce a pairwise alignment for exactly two sequences

**Args:** `seq1.fa seq2.fa --mode pairwise --scoring BLOSUM80 --format phylip --output pair.phy`
**Explanation:** When exactly two inputs are provided, `--mode pairwise` triggers Needleman-Wunsch for an optimal alignment, and `--scoring BLOSUM80` applies amino acid substitution scoring.

### Align protein sequences using a custom substitution matrix file

**Args:** `--informat fasta proteins.fa --matrix AA --submatrix PAM250.txt --gapopen 12 --gapextend 1 --iterations 5 --output pam_align.fasta`
**Explanation:** Loading a custom PAM matrix via `--submatrix` overrides the default, and `--iterations 5` applies extensive refinement to improve accuracy for distantly related proteins.

### Create a quick screening alignment with reduced iterations for large datasets

**Args:** `large_dataset.fasta --mode progressive --iterations 1 --mincol 0.9 --output fast_align.fasta`
**Explanation:** Limiting to one refinement iteration and requiring 90% occupied columns speeds up alignment for large screening runs where approximate accuracy is acceptable.

### Export an alignment in Nexus format for MrBayes or BEAST

**Args:** `aligned_seqs.fasta --format nexus --datatype DNA --output bayes_input.nex`
**Explanation:** Nexus format with `--datatype DNA` produces a file compatible with Bayesian phylogenetic tools that require Nexus input with explicit data type declarations.