---
name: bpp-phyl
category: phylogenetics
description: Bio++ maximum likelihood phylogenetic reconstruction and model selection tool for nucleotide and amino acid alignments
tags:
  - phylogeny
  - maximum-likelihood
  - substitution-model
  - sequence-analysis
  - evolutionary-biology
  - bioinformatics
  - alignment
  - tree-building
  - bpp-suite
author: AI-generated
source_url: https://github.com/BioPP/bpp-phyl
---

## Concepts

- **Alignment input formats**: bpp-phyl reads sequence alignments from multiple formats (FASTA, PHYLIP, Nexus, Stockholm) and requires a rooted or unrooted phylogenetic tree as a starting topology; use `bpp-phy-seq` or `bpp-raxml` for prior sequence processing if your alignment is not in PHYLIP format.
- **Substitution model selection**: The tool supports a wide range of nucleotide models (JC69, K80, F81, HKY85, TN93, GTR) and amino acid models (PAM, WAG, LG, JTT); model specification is mandatory via the `--algebra` or `--model` flag and affects tree branch length optimization significantly.
- **Maximum likelihood optimization**: Branch length optimization and tree topology searches use iterative algorithms (Newton–Raphson or EM) driven by the `--opt` parameter; convergence criteria controlled by `--eps` (tolerance threshold) must be set appropriately for long alignments to avoid premature stopping or excessive runtime.
- **Output artifacts**: The tool produces a Newick-format tree file, a likelihood parameter log, and optionally an HTML/CSV report via `--output`; without explicit `--output` redirection, results are written to standard output and may be lost on piping mistakes.
- **Parallel execution**: On multi-core systems, enable thread-based parallelism with `--thread` or `--nb_threads` to accelerate likelihood computations on large datasets; single-threaded runs on deep alignments (hundreds of taxa) can take hours to converge.

## Pitfalls

- **Missing substitution model**: Omitting `--model` or `--algebra` causes the tool to abort with a cryptic error about an uninitialized parameter; this is particularly frustrating when the alignment is valid but the run produces no output.
- **Mismatched tree/alignment taxa**: Providing a starting tree whose tip labels do not match alignment sequence headers leads to silent taxon mismatches and produces a tree with zero likelihood or garbage branch lengths; always verify tip labels with `grep` before running.
- **Numerical instability with `JC69` on amino acids**: Specifying a simple nucleotide model like `JC69` for amino acid alignments causes the optimizer to fail or produce `NaN` likelihood values; use amino acid-specific models such as `WAG` or `LG` for protein sequences.
- **Unset tolerance causing non-convergence**: Using the default `--eps` value on alignments with >200 sequences often results in the optimizer terminating far from the maximum likelihood peak, yielding unreliable tree topologies; set `--eps 1e-6` or tighter for large datasets.
- **Overwriting output files without confirmation**: The `--output` flag writes results without prompting, silently overwriting any existing Newick file in the target path; use shell redirection `>` carefully to avoid accidental data loss.

## Examples

### Compute a maximum likelihood tree from a nucleotide alignment with the HKY85 model
**Args:** `--algebra HKY85 --inputAlignment alignment.fasta --inputTree guide.tree --output tree.nhx --eps 1e-6`
**Explanation:** This builds a rooted phylogenetic tree from a FASTA alignment using the HKY85 substitution model with tight optimization tolerance, writing the result to NHX format.

### Compute a tree for an amino acid alignment using the WAG model
**Args:** `--model WAG --sequence_type AA --inputAlignment protein_alignment.phylip --inputTree guide.tree --output protein_tree.newick`
**Explanation:** This runs maximum likelihood optimization on a protein alignment under the WAG amino acid substitution model and outputs a Newick tree file.

### Run with parallel threads on a multi-core system
**Args:** `--algebra GTR --inputAlignment dataset.phylip --inputTree starting.tree --output result.newick --nb_threads 8 --eps 1e-7`
**Explanation:** This uses 8 threads to parallelize branch length computations under the GTR model, with a stricter tolerance for higher precision on a large dataset.

### Perform rapid topology screening with a simple JC69 nucleotide model
**Args:** `--model JC69 --inputAlignment short_nuc.fasta --inputTree random.tree --output screened.newick --iter_max 10`
**Explanation:** This quickly screens multiple starting topologies under the simple JC69 model, limiting optimization to 10 iterations to test topology candidates before expensive full runs.

### Log detailed parameter estimates to a file
**Args:** `--algebra TN93 --inputAlignment seqs.phylip --inputTree guide.tree --output ml_tree.newick --log params.log --verbose`
**Explanation:** This runs TN93 model optimization with verbose logging enabled, writing both the tree and a detailed parameter log file containing branch lengths and substitution rates.