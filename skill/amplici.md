---
name: amplici
category: Amplicon Sequencing Analysis
description: A bioinformatics tool for processing and analyzing amplicon sequencing data, including quality filtering, primer trimming, clustering, and taxonomic classification of 16S rRNA, ITS, and other targeted gene sequences.
tags:
- amplicon-seq
- 16S-rRNA
- ITS-analysis
- quality-control
- otu-clustering
- taxonomic-assignment
- pcr-amplicons
author: AI-generated
source_url: https://github.com/amplici/amplici
---

## Concepts

- **Amplicon Sequencing Data Model**: amplici processes paired-end or single-end reads from targeted amplicon sequencing (16S rRNA, ITS, or custom gene panels). The primary input is FASTQ files containing raw sequencing reads, with quality scores encoded in ASCII-33 or ASCII-64 format. The tool expects demultiplexed data where each sample has been previously separated by index/barcode.

- **Quality Filtering and Primer Trimming**: The tool performs per-read quality filtering using sliding window approaches (default: 4 bp window, Q-score threshold 20). Primer sequences are automatically detected and removed from both ends of reads using exact matching or configurable mismatch tolerance (default: 2 mismatches allowed). Trimmed reads below minimum length threshold (default: 50 bp) are discarded.

- **OTU Clustering and Denoising Workflows**: amplici supports two primary analysis modes: de novo clustering (identifies operational taxonomic units using sequence similarity thresholds, default: 97% identity) and amplicon sequence variant (ASV) inference using the Expectation-Maximization algorithm. ASV mode is recommended for marker gene studies requiring high resolution and rarefaction support.

- **Output Formats and Downstream Compatibility**: Generated outputs include: FASTA files of representative sequences, BIOM-formatted abundance tables (for QIIME2 compatibility), and TSV taxonomy reports with confidence scores. Intermediate files (trimmed reads, merged pairs) are retained in a configurable output directory for manual inspection or reprocessing.

## Pitfalls

- **Using Paired-End Data Without Proper Merging**: Specifying only `--input-r1` without corresponding `--input-r2` for paired-end libraries produces collapsed results with artificial chimeras, leading to inflated diversity estimates and false positive OTUs in downstream statistical analysis.

- **Incorrect Primer Orientation**: Supplying reverse complement primers when reads are in forward orientation (or vice versa) results in zero-length reads after trimming, causing complete data loss and error messages about empty input files during clustering steps.

- **Ignoring Sample Metadata Requirements**: Running taxonomic assignment without providing a properly formatted mapping file (TSV/QIIME format) prevents inclusion of sample-level covariates in output reports, limiting downstream differential abundance and association testing.

- **Setting Inconsistent Similarity Thresholds**: Using different clustering thresholds for OTU picking (e.g., 97%) and downstream phylogenetic analysis (e.g., 95% for tree building) creates mismatched sequence sets where representative sequences do not correspond to actual cluster members.

- **Memory Overflow with Large Datasets**: Processing datasets exceeding available RAM without the `--memory-safe` flag causes crashes during the alignment step, potentially corrupting intermediate files and requiring complete reprocessing from raw data.

## Examples

### Basic paired-end amplicon quality filtering and merging

**Args:** `--input-r1 sample_R1.fastq.gz --input-r2 sample_R2.fastq.gz --output-dir filtered_reads --merge-pairs`
**Explanation:** This command performs quality filtering on both read files using default parameters and merges overlapping paired-end reads to produce consensus sequences for downstream analysis.

### Taxonomic classification using SILVA database

**Args:** `--input rep_set.fasta --taxonomy SILVA_138_SSURef_NR99 --output taxonomy.tsv --置信度阈值 0.80`
**Explanation:** Assigns taxonomic labels to representative sequences using the SILVA reference database with a minimum confidence threshold of 80%, outputting results in TSV format.

### ASV inference with DADA2-style algorithm

**Args:** `--input filtered.fastq.gz --asv-mode --min-reads 5 --output asv_table.biom`
**Explanation:** Infers amplicon sequence variants rather than OTUs, removing sequences appearing in fewer than 5 reads across all samples, and outputs results in BIOM format for QIIME2 compatibility.

### Full workflow with custom primer trimming

**Args:** `--input-r1 R1.fastq.gz --input-r2 R2.fastq.gz --forward-primer GTGCCAGCMGCCGCGGTAA --reverse-primer GGACTACHVGGGTWTCTAAT --mismatches 1 --output-dir analysis`
**Explanation:** Trims exact 16S V4 region primers allowing 1 mismatch, then proceeds through filtering, merging, and clustering in a single pipeline execution.

### Parallel processing on multiple samples with metadata

**Args:** `--input-map mapping_file.tsv --parallel --threads 8 --output-dir batch_analysis`
**Explanation:** Processes multiple samples in parallel using 8 threads, with sample names and covariates read from the mapping file for organized output and downstream statistical integration.