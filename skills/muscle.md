---
name: muscle
category: phylogenetics
description: Fast and accurate multiple sequence alignment for proteins and DNA
tags: [multiple-sequence-alignment, msa, protein, nucleotide, phylogenetics, alignment]
author: oxo-call built-in
source_url: "https://drive5.com/muscle/"
---

## Concepts
- MUSCLE (v5) performs progressive and iterative multiple sequence alignment for proteins and DNA.
- MUSCLE v5 has a different command syntax than v3: use '-align' for alignment, '-super5' for large datasets.
- Use -align for standard alignment; -super5 for >1000 sequences (uses less memory).
- Use -threads N for parallelism in MUSCLE v5.
- Output is aligned FASTA by default; use -output to specify output file.
- MUSCLE v5 produces replicate ensembles by default — use -replicates 1 for a single alignment.
- MUSCLE v3 (older, widely used): muscle -in input.fasta -out aligned.fasta
- MUSCLE is generally faster than MAFFT for large datasets but may be less accurate for divergent sequences.
- -stratified generates ensemble with guide tree permutations (abc, acb, bca); 4 replicates per permutation.
- -diversified generates ensemble with random perturbations; better for confidence estimation.
- -disperse calculates dispersion of ensemble; zero dispersion indicates robust alignment.
- -maxcc extracts replicate with highest column confidence (CC) from ensemble.
- -letterconf calculates per-position letter confidence (LC) values from ensemble.
- EFA (Ensemble FASTA) format contains multiple alignments; use -efa_explode to extract individual FASTAs.

## Pitfalls
- MUSCLE v3 and v5 have completely different command syntax — check version before using.
- MUSCLE v5 outputs ensemble alignments by default; use -replicates 1 for a single alignment.
- For very divergent sequences (<20% identity), MAFFT with --localpair may give better results.
- MUSCLE output is to stdout by default in v3; use -out for file output.
- MUSCLE v5 is not compatible with MUSCLE v3 parameters — update commands when switching versions.
- -super5 does not support .efa output; use multiple runs with -perm and -perturb for ensembles.
- High dispersion (>0.05) indicates alignment uncertainty; review ensemble before downstream analysis.
- -stratified produces 4 replicates per guide tree permutation; total replicates = 4 x N.
- -maxgapfract 0.5 (default) filters columns with >50% gaps; increase for gappy alignments.
- -minconf 0.5 (default) filters columns with <50% confidence; decrease to retain more columns.

## Examples

### align multiple protein sequences with MUSCLE v5
**Args:** `-align proteins.fasta -output aligned_proteins.fasta -threads 8`
**Explanation:** muscle command; -align proteins.fasta input FASTA; -output aligned_proteins.fasta output FASTA; -threads 8 parallel

### align a large dataset with MUSCLE v5 super5 mode
**Args:** `-super5 large_dataset.fasta -output large_aligned.fasta -threads 16`
**Explanation:** muscle command; -super5 for >1000 sequences; large_dataset.fasta input FASTA; -output large_aligned.fasta output FASTA; -threads 16 parallel

### align sequences with MUSCLE v3 syntax (legacy)
**Args:** `-in sequences.fasta -out aligned.fasta`
**Explanation:** muscle command (v3 syntax); -in sequences.fasta input; -out aligned.fasta output

### generate multiple alignment replicates for uncertainty estimation
**Args:** `-align sequences.fasta -output aligned.fasta -replicates 5 -threads 8`
**Explanation:** muscle command; -align sequences.fasta input FASTA; -output aligned.fasta output FASTA; -replicates 5 alternative alignments; -threads 8 parallel

### create stratified ensemble for confidence assessment
**Args:** `-align sequences.fasta -stratified -output ensemble.efa -threads 8`
**Explanation:** muscle command; -align sequences.fasta input FASTA; -stratified generates ensemble with guide tree permutations; -output ensemble.efa output EFA; -threads 8 parallel

### calculate dispersion to assess alignment quality
**Args:** `-disperse ensemble.efa`
**Explanation:** muscle command; -disperse measures variation; ensemble.efa input EFA file

### extract best replicate by column confidence
**Args:** `-maxcc ensemble.efa -output best_alignment.afa`
**Explanation:** muscle command; -maxcc extracts highest column confidence replicate; ensemble.efa input EFA; -output best_alignment.afa output FASTA

### calculate letter confidence for each position
**Args:** `-letterconf ensemble.efa -ref best_alignment.afa -output letterconf.afa`
**Explanation:** muscle command; -letterconf calculates per-position confidence; ensemble.efa input EFA; -ref best_alignment.afa reference; -output letterconf.afa output

### generate HTML visualization with confidence colors
**Args:** `-letterconf ensemble.efa -ref best_alignment.afa -html alignment.html`
**Explanation:** muscle command; -letterconf calculates confidence; ensemble.efa input EFA; -ref reference alignment; -html alignment.html output HTML

### create diversified ensemble with perturbations
**Args:** `-align sequences.fasta -diversified -output diversified.efa -threads 8`
**Explanation:** muscle command; -align sequences.fasta input FASTA; -diversified generates ensemble with random perturbations; -output diversified.efa output EFA; -threads 8 parallel
