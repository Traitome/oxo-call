---
name: mash
category: sequence-utilities
description: MinHash-based rapid genome distance estimation and sketching
tags: [minhash, genome-distance, sketching, ANI, metagenomics, clustering]
author: oxo-call built-in
source_url: "https://mash.readthedocs.io/en/latest/"
---

## Concepts

- Mash uses MinHash sketches to estimate Jaccard similarity and Mash distance between sequences in sub-second time, even for large genomes.
- mash sketch converts sequences (FASTA or FASTQ) into compact sketch files (.msh); sketches can be combined into a sketch database.
- Mash distance approximates 1 - ANI (average nucleotide identity); a distance of 0.05 corresponds to ~95% ANI.
- The sketch size (-s) controls precision: larger sketches are more accurate but bigger; default 1000 is good for species-level, use 10000 for strain-level.
- mash screen estimates the containment of a genome in a sequencing dataset (e.g., identifying organisms in a metagenome) without assembly.
- mash triangle computes an all-vs-all distance matrix for phylogenetic clustering; output is lower-triangular by default.

## Pitfalls

- K-mer size (-k) must match between sketch and query; mixing k-mer sizes gives undefined results — default is 21 for genomes.
- mash sketch on FASTQ sets a minimum copy number (-m) to filter low-frequency k-mers (errors); -m 2 or -m 3 is typical for reads.
- mash dist output has p-values; filter results by p-value column to exclude statistically non-significant distances.
- Sketching compressed files (.gz) works directly, but very short sequences (<k) produce empty sketches without warning.
- mash screen reports containment, not mutual similarity — a plasmid will appear contained in a chromosome, not vice versa.
- Combining sketches from different k-mer sizes with -l causes silent incorrect results; always use consistent parameters.

## Examples

### sketch a collection of genome FASTA files into a single database
**Args:** `sketch -o genomes_db *.fasta`
**Explanation:** creates genomes_db.msh containing all genome sketches; subsequent queries use this single file

### compute pairwise distances between two genome sketches
**Args:** `dist genome1.fasta genome2.fasta`
**Explanation:** sketches both on the fly and reports distance, p-value, and shared hashes

### query all genomes in a database against a query genome
**Args:** `dist -p 16 genomes_db.msh query.fasta | sort -k3 -n | head -20`
**Explanation:** -p 16 uses 16 threads; sorting by column 3 (distance) shows closest matches first

### sketch raw sequencing reads with error filtering
**Args:** `sketch -m 2 -s 10000 -o reads_sketch reads.fastq.gz`
**Explanation:** -m 2 requires k-mers to appear at least twice (filters sequencing errors); -s 10000 for higher precision

### screen a metagenome for known reference genomes
**Args:** `screen -w -p 8 refdb.msh metagenome.fastq.gz | sort -gr -k1 > screen_results.txt`
**Explanation:** -w uses winner-takes-all to remove redundant hits; sort by identity (col 1) descending

### compute all-vs-all distance triangle for genome clustering
**Args:** `triangle -p 16 genomes_db.msh > distances.tsv`
**Explanation:** outputs lower-triangular distance matrix suitable for clustering tools like mashtree or R
