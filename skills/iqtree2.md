---
name: iqtree2
category: phylogenetics
description: Fast and versatile maximum-likelihood phylogenetic tree inference with model selection
tags: [phylogenetics, maximum-likelihood, tree, evolution, bootstrap, substitution-model]
author: oxo-call built-in
source_url: "http://www.iqtree.org/"
---

## Concepts

- IQ-TREE 2 infers phylogenetic trees by maximum-likelihood; use -s for input alignment (FASTA, PHYLIP, or NEXUS).
- Use -m TEST to perform automatic model selection (ModelFinder); -m MFP for model selection + tree inference.
- Bootstrap support: -b N for standard bootstrap (slow); --bnni -B N for ultrafast bootstrap (faster, N ≥ 1000 recommended).
- Use -T N for threads (auto-detect optimal with -T AUTO); -o to specify outgroup taxa.
- Output files: <prefix>.treefile (main tree), <prefix>.iqtree (full report), <prefix>.log (run log).
- Use --prefix to name all output files; default prefix is the input file name.
- IQ-TREE supports partition models (-p partition.txt) for multi-gene datasets.
- Use -nt N (IQ-TREE v1 syntax) vs -T N (IQ-TREE v2); v2 preferred for all new analyses.

## Pitfalls

- Bootstrap values <70 are generally unreliable — use ≥1000 ultrafast bootstrap replicates for stability.
- Without -m TEST, IQ-TREE uses GTR+G by default which may not be the best model for your data.
- IQ-TREE output files can be large for many bootstrap replicates — check disk space.
- --bnni after -B is strongly recommended to optimize ultrafast bootstrap trees.
- For proteins, use -m TEST but also specify -st AA (amino acid sequences) if not auto-detected.
- Outgroup (-o) must be a taxon name exactly as it appears in the alignment.

## Examples

### infer maximum-likelihood tree with automatic model selection
**Args:** `-s alignment.fasta -m MFP --prefix my_tree -T AUTO`
**Explanation:** -m MFP: model selection + tree inference; --prefix names output files; -T AUTO detects CPU count

### infer tree with ultrafast bootstrap and model selection
**Args:** `-s alignment.fasta -m MFP -B 1000 --bnni --prefix bootstrap_tree -T 8`
**Explanation:** -B 1000 ultrafast bootstrap replicates; --bnni optimizes each bootstrap tree; standard approach

### infer phylogenetic tree for protein sequences
**Args:** `-s protein_alignment.fasta -st AA -m TEST -B 1000 --bnni --prefix protein_tree -T 8`
**Explanation:** -st AA specifies amino acid data type; -m TEST selects best protein substitution model

### infer concordance factor analysis for assessing gene tree discordance
**Args:** `-s alignment.fasta -m MFP -B 1000 --prefix main_tree -T 8 --gcf gene_trees.txt --scfl 100`
**Explanation:** --gcf gene tree concordance factor; --scfl site concordance factor; requires per-gene trees

### infer tree with standard bootstrap and specified outgroup
**Args:** `-s alignment.fasta -m MFP -b 100 -o outgroup_taxon --prefix rooted_tree -T 8`
**Explanation:** -b 100 standard bootstrap (slower); -o outgroup_taxon roots the tree at specified taxon
