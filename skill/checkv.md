---
name: checkv
category: Bioinformatics/Viral Analysis
description: A tool for analyzing viral contigs from assembled metagenomes, including quality assessment, completeness estimation, taxonomy assignment, and integrated prophage detection.
tags: virology, metagenomics, viral contigs, genome-quality, prophage detection, viral taxonomy
author: AI-generated
source_url: https://bitbucket.org/bbertozzi/checkv/src/master/
---

## Concepts

- **Input Format**: CheckV takes FASTA files containing assembled viral contigs as input. Contigs should be nucleotide sequences in standard FASTA format, with headers that can be parsed for analysis.
- **Quality Assessment**: The tool estimates genome completeness by analyzing the presence of conserved viral marker genes from an internal database. It scores contigs as complete, high-quality, medium-quality, or low-quality based on gene content and genome length.
- **Database of Viral Markers**: CheckV uses a built-in database of ~50,000 viral marker proteins from known viral genomes to identify conserved genes. This database determines completeness estimates and taxonomic relationships.
- **Pairwise Comparison Mode**: The tool can perform all-vs-all comparisons of viral sequences to identify potential integrated prophages or related viral genomes, outputting BLAST/Diamond-style alignment results.

## Pitfalls

- **Short Contigs Produce Unreliable Results**: Input contigs shorter than 1 kb often lack sufficient marker genes for accurate completeness estimation. The tool may classify these as "unknown" or produce misleading quality scores.
- **Non-Viral Sequences Give False Posults**: Feeding bacterial or eukaryotic genomic contigs into CheckV will produce meaningless quality scores. The tool assumes viral input and will attempt to classify any sequence, leading to spurious predictions.
- **Insufficient Memory for Large Datasets**: When analyzing metagenomes with thousands of viral contigs, CheckV loads database indices into memory. Running on systems with less than 8 GB RAM may cause slowdowns or memory exhaustion errors.
- **Confusing Completeness with Accuracy**: CheckV's completeness score reflects the proportion of conserved viral genes detected, not the actual percentage of the original viral genome. Highly fragmented assemblies may overestimate completeness.

## Examples

### Analyze viral contigs for quality assessment

**Args:** `analyze -d /databases/checkv_db viral_contigs.fna output_dir/`

**Explanation:** This runs the main analysis pipeline using the built-in database to assess each viral contig for completeness and quality, outputting results to the specified directory.

### Detect integrated prophages in bacterial genomes

**Args:** `find_prophages viral_contigs.fna bacterial_genomes.fna output_prophages.tsv`

**Explanation:** This compares viral contigs against bacterial genomes to identify sequences that may be integrated prophages or viral elements integrated into bacterial chromosomes.

### Estimate completeness for a single contig

**Args:** `estimate_quality viral_contig.fna -d /databases/checkv_db`

**Explanation:** This outputs a completeness score for one or more viral contigs by scanning for conserved viral marker genes and comparing against the database.

### Assign taxonomy to viral contigs

**Args:** `taxify viral_contigs.fna -d /databases/checkv_db -o taxonomy_output.tsv`

**Explanation:** This assigns taxonomic labels to viral contigs based on marker gene matches, outputting phylum and family-level predictions for each sequence.

### Build a custom viral marker database

**Args:** `checkv_build /custom/viral_proteins.faa custom_db/`

**Explanation:** This creates a custom database from a FASTA file of viral protein sequences, which can then be used with the main analysis commands for specialized viral groups.