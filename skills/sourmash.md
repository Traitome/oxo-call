---
name: sourmash
category: sequence-utilities
description: K-mer sketching, genome comparison, and taxonomic classification using MinHash and FracMinHash
tags: [kmer, sketching, taxonomy, metagenomics, comparison, minhash, gather]
author: oxo-call built-in
source_url: "https://sourmash.readthedocs.io/"
---

## Concepts

- sourmash sketch creates FracMinHash (scaled) or MinHash sketches from DNA or protein sequences for fast comparison.
- The scaled parameter controls sketch resolution: scaled=1000 keeps 1/1000 k-mers; smaller scaled = larger sketch = more precise.
- sourmash gather performs metagenome decomposition: it finds the minimum set of reference genomes that explain the k-mers in a sample.
- sourmash compare computes a pairwise similarity matrix; sourmash plot renders it as a heatmap (requires matplotlib).
- sourmash taxonomy annotates gather results with taxonomic lineages using a prepared taxonomy CSV or SQL database.
- Multiple k-mer sizes and molecule types (DNA, protein, dayhoff, hp) can be stored in one signature file using --param-string.

## Pitfalls

- Sketches created with different k-mer sizes or scaled values cannot be compared — parameters must match exactly.
- sourmash gather requires a database sketched at the same or lower scaled value as the query; higher scaled databases miss content.
- Protein sketches use k=10 by default (amino acids), not k=31; mixing DNA and protein sketches in a search causes errors.
- sourmash compare outputs a numpy matrix by default; use --csv to get a human-readable CSV.
- Large databases (e.g., GTDB) require significant RAM for sourmash search; use sourmash gather with --threshold-bp to limit memory.
- Not using --singleton when sketching individual FASTA records causes all sequences in a file to be merged into one signature.

## Examples

### sketch a genome FASTA file at default parameters
**Args:** `sketch dna -p k=31,scaled=1000 genome.fasta -o genome.sig`
**Explanation:** sourmash sketch subcommand; dna DNA mode; -p k=31,scaled=1000 sketch parameters; genome.fasta input FASTA; -o genome.sig output signature file; k=31 standard for species-level comparison; scaled=1000 gives compact sketch

### sketch multiple genome files and store in one database
**Args:** `sketch dna -p k=31,scaled=1000 *.fasta --output-dir sigs/`
**Explanation:** sourmash sketch subcommand; dna DNA mode; -p k=31,scaled=1000 parameters; *.fasta multiple input FASTA files; --output-dir sigs/ output directory; creates one .sig file per FASTA; can combine with sourmash index for fast search

### compare all signatures in a directory and output similarity matrix
**Args:** `compare sigs/*.sig --csv similarity_matrix.csv -k 31`
**Explanation:** sourmash compare subcommand; sigs/*.sig input signature files; --csv similarity_matrix.csv output CSV; -k 31 selects k=31 sketches if signatures contain multiple k-mer sizes; outputs similarity matrix

### decompose a metagenome sample against a reference database
**Args:** `gather sample.sig gtdb_rs207.k31.zip -k 31 --threshold-bp 50000 -o gather_results.csv`
**Explanation:** sourmash gather subcommand; sample.sig input sample signature; gtdb_rs207.k31.zip reference database; -k 31 k-mer size; --threshold-bp 50000 ignores matches below 50 kb; -o gather_results.csv output CSV; finds minimum set of genomes explaining the sample

### add taxonomy to gather results
**Args:** `taxonomy annotate -g gather_results.csv -t gtdb-rs207.taxonomy.csv -o annotated_results.csv`
**Explanation:** sourmash taxonomy subcommand; annotate annotates gather results; -g gather_results.csv input gather results; -t gtdb-rs207.taxonomy.csv taxonomy CSV; -o annotated_results.csv output CSV; maps reference accessions to taxonomic lineages

### search a signature against a database for top hits
**Args:** `search query.sig refdb.zip -k 31 --threshold 0.1 -n 20 -o search_results.csv`
**Explanation:** sourmash search subcommand; query.sig input signature; refdb.zip reference database; -k 31 k-mer size; --threshold 0.1 minimum 10% Jaccard similarity; -n 20 returns top 20 hits; -o search_results.csv output CSV

### build an indexed database from many signature files for fast search
**Args:** `index refdb.zip sigs/*.sig -k 31`
**Explanation:** sourmash index subcommand; refdb.zip output database file; sigs/*.sig input signature files; -k 31 k-mer size; creates a zipped SBT index for fast containment search with sourmash search and gather

### sketch protein sequences instead of DNA
**Args:** `sketch protein -p k=10,scaled=100 proteins.fasta -o proteins.sig`
**Explanation:** sourmash sketch subcommand; protein protein mode; -p k=10,scaled=100 parameters; proteins.fasta input protein FASTA; -o proteins.sig output signature; k=10 for amino acids; protein sketches enable comparison at protein level

### compare signatures using containment instead of Jaccard
**Args:** `compare sigs/*.sig --containment --csv containment_matrix.csv -k 31`
**Explanation:** sourmash compare subcommand; sigs/*.sig input signatures; --containment uses asymmetric containment index; --csv containment_matrix.csv output CSV; -k 31 k-mer size; better for comparing genomes of different sizes

### gather with protein sketches against protein database
**Args:** `gather protein_sample.sig protein_db.zip -k 10 --threshold-bp 50000 -o protein_gather.csv`
**Explanation:** sourmash gather subcommand; protein_sample.sig protein input signature; protein_db.zip protein reference database; -k 10 k-mer size for amino acids; --threshold-bp 50000 threshold; -o protein_gather.csv output CSV; uses protein sketches for taxonomic classification

### sketch with multiple k-mer sizes in one file
**Args:** `sketch dna -p k=21,k=31,k=51,scaled=1000 genome.fasta -o multi_k.sig`
**Explanation:** sourmash sketch subcommand; dna DNA mode; -p k=21,k=31,k=51,scaled=1000 multiple k-mer sizes; genome.fasta input FASTA; -o multi_k.sig output signature; creates signature with multiple k-mer sizes; allows comparison at different resolutions

### calculate pairwise distances between signatures
**Args:** `compare sigs/*.sig --distance-matrix -o distances.npy`
**Explanation:** sourmash compare subcommand; sigs/*.sig input signatures; --distance-matrix outputs numpy distance matrix; -o distances.npy output file; suitable for clustering and phylogenetic analysis
