---
name: centrifuge
category: metagenomics
description: Rapid and memory-efficient taxonomic classification of sequencing reads using FM-index
tags: [metagenomics, taxonomy, classification, reads, 16S, wgs, kraken]
author: oxo-call built-in
source_url: "https://ccb.jhu.edu/software/centrifuge/manual.shtml"
---

## Concepts

- Centrifuge classifies reads by aligning them to a reference database using an FM-index; it is faster and more memory-efficient than Kraken for large DBs.
- Index building uses companion binary 'centrifuge-build'; Kraken-style report conversion uses companion binary 'centrifuge-kreport' — both are detected automatically.
- centrifuge outputs two files: a per-read classification TSV and a summary report TSV; centrifuge-kreport converts these to Kraken-style reports.
- Paired-end reads are passed with -1 and -2; single-end with -U; reads can be FASTQ or FASTA, gzipped or uncompressed.
- --report-file writes a per-taxon abundance summary during the run; this is a quick overview but the Kraken-style report has more detail.
- Centrifuge databases are available pre-built from the JHU website (nt, bacteria, archaea, viruses, human); building custom DBs is possible but time-consuming.

## Pitfalls

- Not specifying -x (index prefix) causes centrifuge to fail — always provide the full path prefix to the index files.
- Centrifuge index files (.1.cf, .2.cf, .3.cf) must all be present; a missing file causes a cryptic error at startup.
- --min-hitlen default is 22; raising it (e.g., 30) increases precision but reduces recall for shorter reads.
- Centrifuge does not handle interleaved FASTQ natively; split into separate R1/R2 files before using -1/-2.
- The summary report from centrifuge is not in Kraken format by default; use centrifuge-kreport to get Pavian/Krona-compatible output.
- Memory-mapped index loading (default) can be slow on network filesystems; copy indexes to local SSD before running.

## Examples

### classify paired-end reads against a pre-built bacterial/viral database
**Args:** `-x /databases/bv_bacteria -1 R1.fastq.gz -2 R2.fastq.gz -S classifications.tsv --report-file report.tsv -p 16`
**Explanation:** -x sets the database prefix; -S is the classification output; --report-file gives per-taxon summary; -p 16 for threads

### classify single-end reads against the NT database
**Args:** `-x /databases/nt -U reads.fastq.gz -S classifications.tsv --report-file report.tsv -p 16`
**Explanation:** -U for single-end reads; -x NT database prefix; outputs per-read classifications and summary report

### build a custom centrifuge index from bacterial reference genomes
**Args:** `centrifuge-build -p 16 --taxonomy-tree nodes.dmp --name-table names.dmp --conversion-table seqid2taxid.map genomes.fasta custom_db`
**Explanation:** centrifuge-build companion binary; --taxonomy-tree and --name-table are NCBI taxonomy files; --conversion-table maps sequence IDs to taxids

### classify reads with increased sensitivity for viral detection
**Args:** `-x /databases/viral -U reads.fastq.gz -S viral_hits.tsv --report-file viral_report.tsv -p 8 --min-hitlen 16`
**Explanation:** --min-hitlen 16 increases sensitivity for short viral reads at the cost of specificity

### convert centrifuge output to Kraken-style report for Pavian/Krona
**Args:** `centrifuge-kreport -x /databases/bv_bacteria classifications.tsv > kraken_report.txt`
**Explanation:** centrifuge-kreport companion binary; converts centrifuge output to Kraken-compatible format for Pavian/Krona visualization

### remove human reads by classifying against human genome and excluding matches
**Args:** `-x /databases/hg38 -1 R1.fastq.gz -2 R2.fastq.gz -S human_classifications.tsv -p 16 --un-conc non_human_%.fastq.gz`
**Explanation:** --un-conc writes paired reads that did NOT classify against the human database; useful for host depletion

### build centrifuge index from viral reference sequences
**Args:** `centrifuge-build -p 8 --taxonomy-tree nodes.dmp --name-table names.dmp --conversion-table seqid2taxid.map viral_sequences.fasta viral_db`
**Explanation:** centrifuge-build companion binary; creates viral_db index files for rapid viral read classification

### classify reads and save unclassified reads for downstream assembly
**Args:** `-x /databases/bv_bacteria -1 R1.fastq.gz -2 R2.fastq.gz -S classifications.tsv --report-file report.tsv -p 16 --un-conc unclassified_%.fastq.gz`
**Explanation:** --un-conc saves unclassified paired reads; useful for recovering novel organisms for de novo assembly

### use high minimum hit length for precision metagenomic classification
**Args:** `-x /databases/nt -U reads.fastq.gz -S classifications.tsv --report-file report.tsv -p 16 --min-hitlen 30`
**Explanation:** --min-hitlen 30 reduces false positives; useful for long reads or when database contamination is a concern

### classify paired-end metagenome against custom host-depleted database
**Args:** `-x /databases/custom_microbiome -1 R1.fastq.gz -2 R2.fastq.gz -S classifications.tsv --report-file report.tsv -p 16 -k 5`
**Explanation:** -k 5 reports top 5 assignments per read; useful for reads that map to multiple taxa with similar scores
