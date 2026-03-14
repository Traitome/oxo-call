---
name: sourmash
category: sequence-comparison
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
**Explanation:** k=31 is standard for species-level comparison; scaled=1000 gives compact sketch; -o writes signature file

### sketch multiple genome files and store in one database
**Args:** `sketch dna -p k=31,scaled=1000 *.fasta --output-dir sigs/`
**Explanation:** creates one .sig file per FASTA in sigs/; can be combined with sourmash index for fast search

### compare all signatures in a directory and output similarity matrix
**Args:** `compare sigs/*.sig --csv similarity_matrix.csv -k 31`
**Explanation:** -k 31 selects the k=31 sketches if signatures contain multiple k-mer sizes; --csv for readable output

### decompose a metagenome sample against a reference database
**Args:** `gather sample.sig gtdb_rs207.k31.zip -k 31 --threshold-bp 50000 -o gather_results.csv`
**Explanation:** finds minimum set of genomes explaining the sample; --threshold-bp 50000 ignores matches below 50 kb

### add taxonomy to gather results
**Args:** `taxonomy annotate -g gather_results.csv -t gtdb-rs207.taxonomy.csv -o annotated_results.csv`
**Explanation:** maps gather reference accessions to taxonomic lineages; -t supplies the taxonomy CSV

### search a signature against a database for top hits
**Args:** `search query.sig refdb.zip -k 31 --threshold 0.1 -n 20 -o search_results.csv`
**Explanation:** --threshold 0.1 sets minimum Jaccard similarity of 10%; -n 20 returns top 20 hits

### build an indexed database from many signature files for fast search
**Args:** `index refdb.zip sigs/*.sig -k 31`
**Explanation:** creates a zipped SBT index for fast containment search with sourmash search and gather
