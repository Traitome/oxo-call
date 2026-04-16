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
- --alrt N performs SH-aLRT branch test (N replicates, use 0 for parametric aLRT); faster than bootstrap.
- --sprta computes SPRTA branch supports for genomic epidemiology; --pathogen enables CMAPLE algorithm.
- --date FILE infers time tree with tip dates (e.g., for virus phylogenies); supports YYYY-MM-DD format.
- -m MF+MERGE performs PartitionFinder to merge similar partitions and reduce over-parameterization.
- --gcf and --scfl compute gene concordance factor and site concordance factor for assessing gene tree discordance.
- --redo resumes interrupted runs; checkpoint files (.ckp.gz) enable recovery from crashes.

## Pitfalls

- Bootstrap values <70 are generally unreliable — use ≥1000 ultrafast bootstrap replicates for stability.
- Without -m TEST, IQ-TREE uses GTR+G by default which may not be the best model for your data.
- IQ-TREE output files can be large for many bootstrap replicates — check disk space.
- --bnni after -B is strongly recommended to optimize ultrafast bootstrap trees.
- For proteins, use -m TEST but also specify -st AA (amino acid sequences) if not auto-detected.
- Outgroup (-o) must be a taxon name exactly as it appears in the alignment.
- --alrt is much faster than bootstrap but less thorough; combine with -B for comprehensive support values.
- Date file format for --date: one line per taxon with taxon name and date separated by space/tab.
- Partition files (-p) use RAxML or NEXUS format; ensure partition names match alignment gene boundaries.
- --mem option controls RAM usage; default may not use all available memory on HPC systems.
- Sequence names cannot contain special characters (except _ - . / |); others are converted to underscores.
- -m MF+MERGE can take long time for many partitions; consider -m MFP for faster analysis.

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

### infer tree with SH-aLRT test (faster than bootstrap)
**Args:** `-s alignment.fasta -m MFP --alrt 1000 --prefix alrt_tree -T 8`
**Explanation:** --alrt 1000 performs SH-aLRT branch test with 1000 replicates; much faster than bootstrap for quick assessment

### infer time tree with tip dates for virus phylogeny
**Args:** `-s virus_alignment.fasta -m MFP --date dates.txt --prefix timetree -T 8`
**Explanation:** --date dates.txt provides sampling dates; infers time tree with dated ancestral nodes; dates.txt format: taxon_name YYYY-MM-DD

### run PartitionFinder to optimize partitioning scheme
**Args:** `-s alignment.fasta -p partitions.nex -m MF+MERGE --prefix merged_partitions -T 8`
**Explanation:** -m MF+MERGE merges similar partitions to reduce over-parameterization; -p specifies partition file

### infer tree with both SH-aLRT and ultrafast bootstrap
**Args:** `-s alignment.fasta -m MFP --alrt 1000 -B 1000 --bnni --prefix comprehensive_tree -T 8`
**Explanation:** combines SH-aLRT (fast) and UFBoot (thorough) for comprehensive branch support; recommended for publication

### resume interrupted analysis from checkpoint
**Args:** `-s alignment.fasta -m MFP -B 1000 --bnni --prefix resumed_tree -T 8 --redo`
**Explanation:** --redo restarts analysis; checkpoint files (.ckp.gz) allow recovery from crashes without starting over

### infer tree with SPRTA branch supports for pathogen analysis
**Args:** `-s pathogen_alignment.fasta -m MFP --sprta --pathogen --prefix sprta_tree -T 8`
**Explanation:** --sprta computes SPRTA supports; --pathogen enables CMAPLE algorithm optimized for low-divergence pathogen data

### infer tree with memory limit for HPC systems
**Args:** `-s alignment.fasta -m MFP -B 1000 --bnni --prefix memory_limited_tree -T 8 --mem 32G`
**Explanation:** --mem 32G limits RAM usage to 32GB; useful on shared HPC clusters with memory constraints
