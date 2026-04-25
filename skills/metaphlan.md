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
- Use --db_dir and --index to specify database path and index prefix (--bowtie2db is deprecated).
- Use -t to specify analysis type: rel_ab (default, relative abundance), rel_ab_w_read_stats, marker_ab_table.
- Use --nproc N for multi-threading; --tmp_dir for intermediate files.
- merge_metaphlan_tables.py combines multiple sample outputs into a single matrix.
- --tax_lev controls output taxonomic level: a (all), k (kingdom), p (phylum), c (class), o (order), f (family), g (genus), s (species).
- --ignore_eukaryotes, --ignore_bacteria, --ignore_archaea exclude specific domains from profiling.
- --CAMI_format_output produces CAMI-compliant output format for benchmarking.
- --biom_format_output generates BIOM format for downstream microbiome analysis tools.
- --stat_q sets quantile value for robust average (default 0.1); lower values are more conservative.

## Pitfalls
- MetaPhlAn4 database is NOT backward-compatible with MetaPhlAn3 — ensure database and version match.
- For large datasets, save the mapping output (--mapout) to avoid re-aligning when re-running (--bowtie2out is deprecated).
- MetaPhlAn reports relative abundance (0-100%) — values sum to 100% per sample.
- The --db_dir path must be the directory containing the index, and --index the index prefix.
- Paired-end reads should be combined or passed separately — MetaPhlAn handles them as separate mate files.
- Without --nproc, MetaPhlAn uses 4 threads by default — increase for faster processing of large files.
- --bowtie2db is deprecated; use --db_dir instead for specifying database location.
- --ignore_markers excludes specific marker genes from analysis; useful for custom databases.
- Long reads require --input_type fastq with --long_reads flag and minimap2 alignment.
- Default --min_mapq_val is 5 for short reads and 50 for long reads; adjust based on data quality.

## Examples

### profile microbial community from single-end FASTQ reads
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 reads.fastq.gz -o sample_profile.txt`
**Explanation:** metaphlan command; --input_type fastq input format; --bowtie2db /path/to/mpa_db database directory; --index mpa_vJan21_CHOCOPhlAnSGB_202103 index name; --nproc 8 threads; reads.fastq.gz input FASTQ; -o sample_profile.txt output profile table

### profile paired-end metagenomic reads
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 -o sample_profile.txt R1.fastq.gz,R2.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --bowtie2db database directory; --index index name; --nproc 8 threads; -o sample_profile.txt output; R1.fastq.gz,R2.fastq.gz paired-end reads separated by comma

### save bowtie2 alignments for faster re-runs and profile
**Args:** `--input_type fastq --bowtie2db /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 --bowtie2out sample.bowtie2.bz2 -o sample_profile.txt reads.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --bowtie2db database directory; --index index name; --nproc 8 threads; --bowtie2out sample.bowtie2.bz2 saves alignment for re-use; -o sample_profile.txt output; reads.fastq.gz input

### merge multiple MetaPhlAn profiles into a single table
**Args:** `sample1_profile.txt sample2_profile.txt sample3_profile.txt > merged_profiles.txt`
**Explanation:** merge_metaphlan_tables.py command; sample1_profile.txt sample2_profile.txt sample3_profile.txt input profiles; > merged_profiles.txt output merged table

### profile at genus level only
**Args:** `--input_type fastq --db_dir /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 --tax_lev g -o genus_profile.txt reads.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --db_dir /path/to/mpa_db database directory; --index index name; --nproc 8 threads; --tax_lev g limits output to genus level; -o genus_profile.txt output; reads.fastq.gz input

### profile bacteria only, ignoring eukaryotes and archaea
**Args:** `--input_type fastq --db_dir /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 --nproc 8 --ignore_eukaryotes --ignore_archaea -o bacteria_only.txt reads.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --db_dir database directory; --index index name; --nproc 8 threads; --ignore_eukaryotes --ignore_archaea excludes non-bacterial domains; -o bacteria_only.txt output; reads.fastq.gz input

### save mapping output for reuse
**Args:** `--input_type fastq --db_dir /path/to/mpa_db --index latest --nproc 8 --mapout sample.map.bz2 -o profile.txt reads.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --db_dir database directory; --index latest index name; --nproc 8 threads; --mapout sample.map.bz2 saves alignment results; -o profile.txt output; reads.fastq.gz input

### profile from existing mapping file
**Args:** `--input_type mapout --db_dir /path/to/mpa_db --index mpa_vJan21_CHOCOPhlAnSGB_202103 -o reprofile.txt sample.map.bz2`
**Explanation:** metaphlan command; --input_type mapout uses pre-computed alignment; --db_dir database directory; --index index name; -o reprofile.txt output; sample.map.bz2 input map file

### generate BIOM format for QIIME2
**Args:** `--input_type fastq --db_dir /path/to/mpa_db --index latest --nproc 8 --biom_format_output -o profile.biom reads.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --db_dir database directory; --index latest index name; --nproc 8 threads; --biom_format_output produces BIOM format; -o profile.biom output; reads.fastq.gz input

### profile long reads with minimap2
**Args:** `--input_type fastq --db_dir /path/to/mpa_db --index latest --nproc 8 --long_reads -o longread_profile.txt nanopore.fastq.gz`
**Explanation:** metaphlan command; --input_type fastq input format; --db_dir database directory; --index latest index name; --nproc 8 threads; --long_reads enables minimap2 alignment; -o longread_profile.txt output; nanopore.fastq.gz input
