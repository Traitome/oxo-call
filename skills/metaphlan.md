---
name: metaphlan
category: metagenomics
description: Profiling the composition of microbial communities from shotgun metagenomic sequencing data
tags: [metagenomics, microbiome, taxonomy, profiling, metaphlan, species-composition]
author: oxo-call built-in
source_url: "https://github.com/biobakery/MetaPhlAn"
---

## Concepts

- MetaPhlAn4 profiles microbial community composition (bacteria, archaea, eukaryotes, viruses) using unique clade-specific marker genes.
- MetaPhlAn uses the mpa_vJan21_CHOCOPhlAnSGB_202103 or newer database; download with metaphlan --install.
- Use --input_type fastq for raw reads; --input_type bowtie2out for pre-computed alignments.
- Output is a tab-delimited table with clade names and relative abundances at all taxonomic levels.
- Use --bowtie2db and --index to specify database path and index prefix.
- Use -t to specify analysis type: rel_ab (default, relative abundance), rel_ab_w_read_stats, marker_ab_table.
- Use --nproc N for multi-threading; --tmp_dir for intermediate files.
- merge_metaphlan_tables.py combines multiple sample outputs into a single matrix.

## Pitfalls

- MetaPhlAn4 database is NOT backward-compatible with MetaPhlAn3 — ensure database and version match.
- For large datasets, save the bowtie2 output (--bowtie2out) to avoid re-aligning when re-running.
- MetaPhlAn reports relative abundance (0-100%) — values sum to 100% per sample.
- The --bowtie2db path must be the directory containing the index, and --index the index prefix.
- Paired-end reads should be combined or passed separately — MetaPhlAn handles them as separate mate files.
- Without --nproc, MetaPhlAn uses 4 threads by default — increase for faster processing of large files.

## Examples

### profile microbial community from single-end FASTQ reads
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 reads.fastq.gz -o sample_profile.txt`
**Explanation:** --input_type fastq; --bowtie2db database directory; -o output profile table

### profile paired-end metagenomic reads
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 -o sample_profile.txt R1.fastq.gz,R2.fastq.gz`
**Explanation:** PE reads separated by comma as input; MetaPhlAn handles them as paired mates

### save bowtie2 alignments for faster re-runs and profile
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 --bowtie2out sample.bowtie2.bz2 -o sample_profile.txt reads.fastq.gz`
**Explanation:** --bowtie2out saves alignment for re-use; avoids re-aligning when re-running with different parameters

### merge multiple MetaPhlAn profiles into a single table
**Args:** `sample1_profile.txt sample2_profile.txt sample3_profile.txt > merged_profiles.txt`
**Explanation:** merge_metaphlan_tables.py merges profiles; run as: merge_metaphlan_tables.py *.txt > merged_profiles.txt
