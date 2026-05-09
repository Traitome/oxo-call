---
name: andi
category: Phylogenetics / Evolutionary Distance Estimation
description: Estimates evolutionary distances between genomic or proteomic sequences using maximum-likelihood or correction methods. Accepts unaligned FASTA input and computes pairwise distance matrices for phylogenetic reconstruction.
tags:
  - phylogeny
  - evolutionary-distance
  - distance-matrix
  - sequence-analysis
  - whole-genome
  - substitution-model
author: AI-Generated
source_url: https://github.com/evobuild化石/andi
---

## Concepts

- **Input Format**: `andi` accepts FASTA-formatted sequences (nucleotide or amino acid). Sequences may be unaligned; the tool internally performs rapid alignment using an anchor-based method. Each sequence must be on its own line or split across multiple lines, and headers must begin with `>`.

- **Distance Estimation Models**: The tool implements several substitution models. For nucleotides, common models include `JC69` (Jukes-Cantor), `K80` (Kimura 2-parameter), and `F84`. For amino acids, models include `PMB` (Poisson/Motor Beijing) and `WAG`. The correct model depends on your sequences: use nucleotide models for DNA/RNA, amino acid models for proteins.

- **Output Modes**: Distance estimation produces either a **pairwise distance list** (one distance per line) or a **distance matrix** (square format suitable for tools like `rapidnj` or `PHYLIP`). The output format is controlled by flags; a distance matrix is required by most downstream phylogenetic reconstruction programs.

- **Gap and Missing Data Handling**: Sequences with excessive gaps or `N` characters may distort distance estimates. `andi` ignores alignment columns with gaps in both sequences when computing distances, but columns where one sequence has a gap and the other has a character are treated differently depending on the model selected.

- **Computation Scaling**: Runtime scales roughly quadratically with the number of sequences (O(n²) pairwise comparisons). For large datasets (>500 sequences), consider computing distances on sequence subsets or using approximate methods to avoid long runtimes.

## Pitfalls

- **Running on Unaligned Sequences Without Verification**: While `andi` handles unaligned input, it assumes sequences are homologous. Mixing non-homologous sequences (e.g., from different genomic regions) produces meaningless distances that can mislead phylogenetic reconstruction.

- **Mismatching Sequence Type and Substitution Model**: Specifying a nucleotide substitution model (e.g., `JC69`) for amino acid sequences produces errors or silently incorrect distances. Always match `-m` (model) to your sequence alphabet: nucleotide sequences need nucleotide models; protein sequences need amino acid models.

- **Omitting the `-s` Flag for Large Alignments**: When input sequences are already aligned and longer than ~10,000 sites, using the default alignment mode without `-s` may consume excessive memory or time. The `-s` flag enables faster gap-stripping that is appropriate for pre-aligned inputs.

- **Ignoring Sequence Order in Distance Matrix Output**: The distance matrix output lists sequences in the order they appear in the input file. If downstream tools assume alphabetical ordering, distances will be assigned to the wrong sequence pairs, corrupting phylogenetic trees.

- **Assuming Output Units Are Substitutions Per Site Without Checking**: Distance values depend on the model. For `JC69` and `K80`, distances are in units of substitutions per site. For other models, distances may represent expected number of substitutions per character. Always verify units against your downstream tool's requirements.

## Examples

### Compute pairwise evolutionary distance between two FASTA sequences

**Args:** `-m K80 -o pairwise seq1.fasta seq2.fasta`
**Explanation:** The `-m K80` flag selects the Kimura 2-parameter nucleotide model, and `-o pairwise` directs output to a simple two-column list. This is the fastest way to get a single distance estimate for a sequence pair.

### Estimate a full distance matrix for multiple unaligned nucleotide sequences

**Args:** `-m JC69 -o matrix -O fasta input_seqs.fasta`
**Explanation:** `-o matrix` generates a square distance matrix in PHYLIP format, and `-O fasta` ensures the input is interpreted as FASTA (the default for `.fasta` extensions). The Jukes-Cantor model is appropriate for short divergences where transition/transversion bias is negligible.

### Calculate amino acid distances using the WAG substitution model

**Args:** `-m WAG -o matrix protein_seqs.fasta`
**Explanation:** Specifying `WAG` (a codon-based amino acid model) with `-m` computes distances appropriate for protein-coding sequences. The output matrix can be piped directly into `rapidnj -Dd` for neighbor-joining tree reconstruction.

### Generate distances with verbose logging suppressed for scripting

**Args:** `-m F84 -o pairwise -q long_seqs.fasta`
**Explanation:** The `-q` flag suppresses progress messages and version information, producing clean output suitable for parsing in automated pipelines. This is essential when integrating `andi` into workflow managers like Snakemake or Nextflow.

### Compute distances from a pre-aligned FASTA alignment with gap-aware settings

**Args:** `-m K80 -s 0.5 -o matrix aligned_input.fasta`
**Explanation:** The `-s 0.5` flag sets the gap tolerance threshold (columns with >50% gaps are excluded) and enables efficient pre-aligned mode. This reduces memory usage and runtime for alignments already processed by tools like `Mafft` or `ClustalW`.