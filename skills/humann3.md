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

## Pitfalls

- HUMAnN3 requires both ChocoPhlAn and UniRef databases — download before first use.
- HUMAnN3 is slow without multi-threading; recommend --threads 16+ for large files.
- For concatenated PE reads, merge R1 and R2 with cat before running HUMAnN3.
- The output is in RPK units by default — normalize with humann_renorm_table for comparisons.
- HUMAnN3 uses MetaPhlAn4 internally — ensure MetaPhlAn4 is installed and its database is available.
- Large UniRef databases (20+ GB) significantly increase memory usage during protein search.

## Examples

### run HUMAnN3 functional profiling on metagenomic reads
**Args:** `--input reads.fastq.gz --output humann3_output/ --threads 16 --nucleotide-database /path/to/chocophlan --protein-database /path/to/uniref90`
**Explanation:** --input FASTQ; --output directory; --threads 16; explicit database paths for reproducibility

### run HUMAnN3 on concatenated paired-end reads
**Args:** `--input merged_R1R2.fastq.gz --output humann3_output/ --threads 16 --bypass-nucleotide-index`
**Explanation:** merge PE reads: cat R1.fq.gz R2.fq.gz > merged.fq.gz; --bypass-nucleotide-index for faster run

### normalize HUMAnN3 gene family output to relative abundance
**Args:** `--input sample_genefamilies.tsv --output sample_genefamilies_relab.tsv --units relab`
**Explanation:** humann_renorm_table normalizes to relative abundance (relab) or copies per million (cpm)

### join multiple HUMAnN3 gene family tables into one matrix
**Args:** `--input humann3_results/ --output merged_genefamilies.tsv`
**Explanation:** humann_join_tables merges all *_genefamilies.tsv files from the directory into one table
