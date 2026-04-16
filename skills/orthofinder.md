---
name: orthofinder
category: comparative-genomics
description: Phylogenetic orthology inference for comparative genomics across multiple species proteomes
tags: [orthology, gene-families, phylogenomics, proteome, evolution, comparative-genomics]
author: oxo-call built-in
source_url: "https://github.com/davidemms/OrthoFinder"
---

## Concepts

- OrthoFinder infers orthogroups, orthologs, gene duplication events, and species trees from a set of proteome FASTA files.
- Input is a directory of protein FASTA files, one per species; file names become species identifiers in the output.
- OrthoFinder uses DIAMOND for all-vs-all protein search by default; -S blast or -S mmseqs2 can substitute other search tools.
- Results are written to a timestamped OrthoFinder/Results_* directory inside the input FASTA directory by default.
- -og reports orthogroups only (faster, no gene trees); -M msa builds multiple sequence alignments for each orthogroup.
- The Orthogroups/Orthogroups.tsv output maps each orthogroup to member genes per species; Orthogroup_Statistics.tsv provides counts.
- -I controls the MCL inflation parameter; higher values (e.g., 5.0) produce smaller, tighter orthogroups.
- --assign adds new species to existing orthogroups without re-running the full analysis.
- -d flag indicates DNA sequence input instead of protein sequences.
- -X prevents adding species names to sequence IDs; useful for maintaining original IDs.

## Pitfalls

- Protein FASTA files must use unique sequence IDs across all files; duplicate IDs cause incorrect gene tree inference.
- Species with very different proteome sizes can bias OG size statistics; check that all FASTAs contain complete proteomes.
- OrthoFinder requires DIAMOND, MCL, and FastME (or MAFFT/IQ-TREE for -M msa) to be in PATH; missing tools fail with non-obvious errors.
- Restarting OrthoFinder after partial completion requires -b (results directory) not -f; using -f restarts from scratch.
- Gene tree inference (-M msa) can be very slow for large orthogroups with many paralogs; set --max-msa-genes to cap size.
- Results directory names include timestamps; scripting downstream analysis should use -o to set a fixed output path.
- -I inflation parameter default (1.2) may be too permissive for some analyses; adjust based on desired orthogroup granularity.
- --assign requires the core orthogroup directory to be from a completed OrthoFinder run with compatible versions.

## Examples

### run OrthoFinder on a directory of species proteomes
**Args:** `-f proteomes/ -t 32 -a 8`
**Explanation:** -f points to directory of protein FASTAs; -t 32 threads for DIAMOND search; -a 8 threads for orthogroup analysis

### run OrthoFinder with MSA-based gene trees using MAFFT and IQ-TREE
**Args:** `-f proteomes/ -M msa -S diamond -A mafft -T iqtree -t 32 -a 8`
**Explanation:** -M msa builds gene trees from MSA; -A mafft for alignment; -T iqtree for tree inference; more accurate but slower

### infer orthogroups only without gene trees for fast proteome comparison
**Args:** `-f proteomes/ -og -t 32`
**Explanation:** -og stops after orthogroup assignment and statistics; much faster than full analysis with gene trees

### restart OrthoFinder from existing DIAMOND results (add a new species)
**Args:** `-b proteomes/OrthoFinder/Results_Jan01/ -f new_species/ -t 32 -a 8`
**Explanation:** -b provides existing results; -f provides new species FASTA directory; OrthoFinder re-runs only new comparisons

### use MMseqs2 instead of DIAMOND for faster all-vs-all search
**Args:** `-f proteomes/ -S mmseqs2 -t 32 -a 8`
**Explanation:** -S mmseqs2 substitutes MMseqs2 for the all-vs-all search step; faster for very large proteome sets

### run OrthoFinder with a fixed output directory name
**Args:** `-f proteomes/ -o results/orthofinder_run -t 32 -a 8`
**Explanation:** -o sets the output directory explicitly instead of using a timestamped directory inside the input folder

### assign new species to existing orthogroups
**Args:** `--assign new_species/ --core proteomes/OrthoFinder/Results_Jan01/ -t 32 -a 8`
**Explanation:** --assign adds new species to existing orthogroups; --core points to previous OrthoFinder results; faster than full re-run

### run with higher MCL inflation for tighter orthogroups
**Args:** `-f proteomes/ -I 5.0 -t 32 -a 8`
**Explanation:** -I 5.0 increases MCL inflation; produces smaller, more specific orthogroups; useful for fine-grained analysis

### run OrthoFinder on DNA sequences
**Args:** `-f dna_sequences/ -d -t 32 -a 8`
**Explanation:** -d flag indicates DNA input; OrthoFinder will handle nucleotide sequences instead of protein

### run with FAMSA for fast MSA (default)
**Args:** `-f proteomes/ -M msa -A famsa -t 32 -a 8`
**Explanation:** -A famsa uses FAMSA for MSA; faster than MAFFT for large datasets; default for -M msa mode
