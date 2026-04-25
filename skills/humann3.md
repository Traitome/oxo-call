---
name: humann3
category: metagenomics
description: HUMAnN3 — functional profiling of metagenomes for pathway and gene family abundance
tags: [metagenomics, functional-profiling, pathway, gene-family, microbiome, uniref, metacyc]
author: oxo-call built-in
source_url: "https://github.com/biobakery/humann"
---

## Concepts

- HUMAnN3 profiles microbial community function (metabolic pathways, gene families) from metagenomic reads.
- HUMAnN3 pipeline: (1) MetaPhlAn4 for taxonomy → (2) nucleotide search (ChocoPhlAn) → (3) protein search (UniRef).
- Use --input for FASTQ input; --output for output directory; --threads for parallelism.
- Output files: <sample>_genefamilies.tsv (RPK), <sample>_pathabundance.tsv, <sample>_pathcoverage.tsv.
- Gene families (UniRef90/UniRef50) represent protein coding sequences; pathways from MetaCyc.
- Use humann_renorm_table to normalize gene families to RPM or relative abundance.
- Use humann_join_tables to merge multiple sample outputs into a single table.
- Databases: ChocoPhlAn (nucleotide), UniRef90 (protein) — download with humann_databases command.
- --bypass-translated-search skips protein search (faster, less sensitive); --bypass-nucleotide-search skips nucleotide search.
- --bypass-prescreen skips MetaPhlAn4 and uses full ChocoPhlAn database (slower but more comprehensive).
- humann_regroup_table regroups gene families to other functional categories (e.g., GO, KEGG, EC).
- humann_split_stratified_table separates taxonomic contributions from combined pathway/gene family tables.
- humann_rename_table converts UniRef IDs to human-readable gene names.

## Pitfalls

- HUMAnN3 requires both ChocoPhlAn and UniRef databases — download before first use.
- HUMAnN3 is slow without multi-threading; recommend --threads 16+ for large files.
- For concatenated PE reads, merge R1 and R2 with cat before running HUMAnN3.
- The output is in RPK units by default — normalize with humann_renorm_table for comparisons.
- HUMAnN3 uses MetaPhlAn4 internally — ensure MetaPhlAn4 is installed and its database is available.
- Large UniRef databases (20+ GB) significantly increase memory usage during protein search.
- --bypass-translated-search is faster but misses novel genes not in ChocoPhlAn; use for quick profiling.
- --bypass-prescreen is much slower but necessary when MetaPhlAn4 fails to detect expected taxa.
- humann_join_tables requires consistent column names; renormalize before joining if units differ.
- UniRef50 is smaller and faster than UniRef90 but less sensitive; choose based on computational resources.

## Examples

### run HUMAnN3 functional profiling on metagenomic reads
**Args:** `--input reads.fastq.gz --output humann3_output/ --threads 16 --nucleotide-database /path/to/chocophlan --protein-database /path/to/uniref90`
**Explanation:** humann3 subcommand; --input reads.fastq.gz input FASTQ; --output humann3_output/ output directory; --threads 16 threads; --nucleotide-database /path/to/chocophlan nucleotide database; --protein-database /path/to/uniref90 protein database

### run HUMAnN3 on concatenated paired-end reads
**Args:** `--input merged_R1R2.fastq.gz --output humann3_output/ --threads 16 --bypass-nucleotide-index`
**Explanation:** humann3 subcommand; --input merged_R1R2.fastq.gz concatenated paired-end reads; --output humann3_output/ output directory; --threads 16 threads; --bypass-nucleotide-index for faster run

### normalize HUMAnN3 gene family output to relative abundance
**Args:** `--input sample_genefamilies.tsv --output sample_genefamilies_relab.tsv --units relab`
**Explanation:** humann_renorm_table subcommand; --input sample_genefamilies.tsv input gene families; --output sample_genefamilies_relab.tsv output file; --units relab normalizes to relative abundance

### join multiple HUMAnN3 gene family tables into one matrix
**Args:** `--input humann3_results/ --output merged_genefamilies.tsv`
**Explanation:** humann_join_tables subcommand; --input humann3_results/ input directory; --output merged_genefamilies.tsv output table; merges all *_genefamilies.tsv files

### run HUMAnN3 with only nucleotide search (skip protein search)
**Args:** `--input reads.fastq.gz --output humann3_output/ --threads 16 --bypass-translated-search`
**Explanation:** humann3 subcommand; --input reads.fastq.gz input FASTQ; --output humann3_output/ output directory; --threads 16 threads; --bypass-translated-search skips UniRef protein search

### run HUMAnN3 without MetaPhlAn4 prescreen
**Args:** `--input reads.fastq.gz --output humann3_output/ --threads 16 --bypass-prescreen`
**Explanation:** humann3 subcommand; --input reads.fastq.gz input FASTQ; --output humann3_output/ output directory; --threads 16 threads; --bypass-prescreen uses full ChocoPhlAn database

### regroup gene families to KEGG Orthology
**Args:** `--input sample_genefamilies.tsv --output sample_genefamilies_ko.tsv --groups uniref90_ko`
**Explanation:** humann_regroup_table subcommand; --input sample_genefamilies.tsv input gene families; --output sample_genefamilies_ko.tsv output file; --groups uniref90_ko converts to KEGG Orthology

### split stratified table into unstratified and taxonomic contributions
**Args:** `--input sample_pathabundance.tsv --output sample_pathabundance_split/`
**Explanation:** humann_split_stratified_table subcommand; --input sample_pathabundance.tsv input pathway abundance; --output sample_pathabundance_split/ output directory

### rename UniRef IDs to gene names
**Args:** `--input sample_genefamilies.tsv --output sample_genefamilies_named.tsv --names uniref90`
**Explanation:** humann_rename_table subcommand; --input sample_genefamilies.tsv input gene families; --output sample_genefamilies_named.tsv output file; --names uniref90 converts UniRef90 IDs to gene names

### download HUMAnN3 databases
**Args:** `humann_databases --download chocophlan full /path/to/databases --update-config yes`
**Explanation:** humann_databases subcommand; --download chocophlan full downloads full ChocoPhlAn database; /path/to/databases target directory; --update-config yes updates humann config

### normalize pathway abundance to copies per million
**Args:** `--input sample_pathabundance.tsv --output sample_pathabundance_cpm.tsv --units cpm`
**Explanation:** humann_renorm_table subcommand; --input sample_pathabundance.tsv input pathway abundance; --output sample_pathabundance_cpm.tsv output file; --units cpm normalizes to copies per million
