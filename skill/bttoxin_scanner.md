---
name: bttoxin_scanner
category: genomics
description: Scan nucleotide or protein sequences for Bacillus thuringiensis (Bt) toxin genes and domains using HMM profiles and motif-based detection
tags: [bt-toxin, toxin-detection, genomics, hmm-profile, pathogen-analysis, gene-scanner]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bttoxin_scanner
---

## Concepts

- bttoxin_scanner operates on FASTA (nucleotide/protein) or GenBank input files and can detect over 15 different Bt toxin families including Cry, Cyt, and Vip protein families
- Output formats include TSV (default), JSON, and HTML reports with domain diagrams and e-value significance scores
- The tool uses HMM profiles built from the Bt toxin database along with PROSITE motifs for detection, enabling adjustable sensitivity via --evalue and --bit-score flags
- Three operation modes are available: --mode scan performs fast homology searches, --mode annotate provides detailed domain architecture, and --mode align generates multi-sequence alignments of detected toxin homologs

## Pitfalls

- Setting the e-value threshold too high (e.g., >0.01) causes false positives, as non-toxin sequences with marginal similarity will be reported in the output
- Using nucleotide input without the --translate flag results in missed protein-specific toxin domains that require translation before HMM searching
- Overwriting the existing output file without --force fails silently, leaving users believing their results were saved when they were not
- Running scans without a recent database update (--update-db) can miss newly characterized toxin variants that differ from older HMM profiles

## Examples

### Detect Bt toxin genes in a FASTA file
**Args:** input.fasta --mode scan --evalue 0.001 --format tsv --out toxins.tsv
**Explanation:** Scans the input FASTA for toxin genes using default HMM profiles and outputs significant hits as a tab-separated file

### Search protein sequences for Cry toxin domains with high stringency
**Args:** proteins.fasta --mode annotate --protein --bit-score 20.0 --format html --out annotation.html
**Explanation:** Annotates protein sequences to identify Cry toxin domain architecture with strict bit-score filtering and HTML visualization

### Output results in JSON format for pipeline integration
**Args:** query.fasta --mode scan --format json --out results.json
**Explanation:** Exports detection results in JSON format suitable for integration with automated bioinformatics pipelines

### Update the local Bt toxin HMM database before scanning
**Args:** --update-db /path/to/hmmdb --force
**Explanation:** Downloads and installs the latest Bt toxin HMM profiles from the database repository to ensure comprehensive detection

### Generate multi-sequence alignment of detected toxin homologs
**Args:** gene_collection.fasta --mode align --format fasta --out alignments.fasta
**Explanation:** Produces a multiple sequence alignment of all detected toxin homologs for phylogenetic analysis

### Scan GenBank files for toxin gene clusters
**Args:** genome.gb --mode scan --evalue 0.0001 --format tsv --out genome_toxins.tsv
**Explanation:** Parses GenBank files to identify toxin genes and their genomic context within gene clusters

### Run quick screening with nucleotide translation
**Args:** reads.fasta --translate --mode scan --evalue 0.01 --out quick_scan.tsv
**Explanation:** Performs rapid in-silico translation of nucleotide reads followed by toxin screening with relaxed e-value for catch-all detection