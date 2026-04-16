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
- mash paste combines multiple sketch files into one without re-sketching; useful for building reference databases incrementally.
- mash info displays sketch file metadata including k-mer size, sketch size, and number of sequences.
- -n (non-canonical) preserves strand information; default uses canonical k-mers (min of forward/reverse).
- -r indicates read set input for accurate genome size estimation from k-mer content.
- -b uses Bloom filter for memory-efficient unique k-mer filtering in large metagenomes.
- -c sets target coverage for early termination when sketching high-coverage read sets.

## Pitfalls
- mash ARGS must start with a subcommand (sketch, dist, screen, info, paste, bounds, taxscreen) — never with flags like -o, -k, -s. The subcommand ALWAYS comes first.
- K-mer size (-k) must match between sketch and query; mixing k-mer sizes gives undefined results — default is 21 for genomes.
- mash sketch on FASTQ sets a minimum copy number (-m) to filter low-frequency k-mers (errors); -m 2 or -m 3 is typical for reads.
- mash dist output has p-values; filter results by p-value column to exclude statistically non-significant distances.
- Sketching compressed files (.gz) works directly, but very short sequences (<k) produce empty sketches without warning.
- mash screen reports containment, not mutual similarity — a plasmid will appear contained in a chromosome, not vice versa.
- Combining sketches from different k-mer sizes with -l causes silent incorrect results; always use consistent parameters.
- -n (non-canonical) doubles sketch size and changes distance estimates; don't mix canonical and non-canonical sketches.
- Bloom filter (-b) may allow some false positives; use -m for exact filtering if memory permits.
- Very small sketch sizes (-s < 100) give poor distance estimates; use at least 400 for meaningful results.
- mash triangle output is lower-triangular; use -E for full matrix if needed by downstream tools.
- Taxscreen requires pre-built taxonomy database; not available in standard Mash installation.

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

### combine multiple sketch files into a single database
**Args:** `paste -o combined_db.msh genome1.msh genome2.msh genome3.msh`
**Explanation:** mash paste merges existing sketches without re-sketching; useful for incrementally building reference databases

### display information about a sketch file
**Args:** `info refseq_db.msh`
**Explanation:** shows k-mer size, sketch size, number of sequences, and other metadata; verify compatibility before comparisons

### sketch reads with Bloom filter for large metagenomes
**Args:** `sketch -r -b 100M -o metagenome.msh reads.fastq.gz`
**Explanation:** -r for read set; -b 100M uses 100MB Bloom filter for memory-efficient k-mer filtering in large datasets

### sketch with target coverage for early termination
**Args:** `sketch -r -c 50 -o highcov.msh reads.fastq.gz`
**Explanation:** -c 50 stops sketching after reaching 50x estimated coverage; saves time for ultra-high coverage datasets

### sketch individual sequences within a multi-FASTA file
**Args:** `sketch -i -o plasmids.msh plasmid_collection.fasta`
**Explanation:** -i sketches each sequence individually rather than the whole file; useful for comparing plasmids in a collection

### compute full distance matrix instead of lower-triangular
**Args:** `triangle -E -p 16 genomes_db.msh > full_matrix.tsv`
**Explanation:** -E outputs full symmetric matrix; some clustering tools require full matrix format

### screen with minimum identity threshold
**Args:** `screen -i 0.9 -p 8 refdb.msh metagenome.fastq.gz`
**Explanation:** -i 0.9 only reports hits with ≥90% identity; filters out low-confidence matches in contamination screening

### list input mode for batch processing
**Args:** `dist -l refdb.msh query_list.txt > distances.tsv`
**Explanation:** -l indicates query_list.txt contains file paths (one per line); useful for processing many queries efficiently
