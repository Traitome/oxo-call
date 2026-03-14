---
name: cellranger
category: single-cell
description: 10x Genomics Cell Ranger pipeline for single-cell RNA-seq, ATAC-seq, and multiome data processing
tags: [single-cell, scrna-seq, 10x-genomics, cellranger, gene-expression, atac-seq]
author: oxo-call built-in
source_url: "https://github.com/10XGenomics/cellranger"
---

## Concepts

- Cell Ranger is 10x Genomics' official pipeline for single-cell data; main commands: count, arc, atac, multi.
- cellranger count processes scRNA-seq (10x 3' or 5' Gene Expression) from FASTQ files.
- Requires: --transcriptome (pre-built reference), --fastqs (FASTQ directory), --sample (sample name prefix).
- Build a custom reference with 'cellranger mkref --genome --fasta --genes' for non-standard genomes.
- Cell Ranger output: per_barcode_metrics.csv, molecule_info.h5, filtered_feature_bc_matrix/, web_summary.html.
- Use --localcores and --localmem to control CPU and RAM usage.
- Expect 1-2 hours per sample for human scRNA-seq on 8 cores with 64 GB RAM.
- The FASTQ files must follow 10x naming conventions: <sample>_S1_L001_R1_001.fastq.gz.

## Pitfalls

- Cell Ranger requires FASTQ files named with 10x conventions — check file naming before running.
- The --transcriptome must be a Cell Ranger compatible reference (built with cellranger mkref or pre-built).
- Cannot use generic STAR or HISAT2 indices — must use Cell Ranger-formatted references.
- Cell Ranger output directory (--id) must not already exist — it creates a fresh directory.
- Without --localcores and --localmem, Cell Ranger may use all available resources on shared systems.
- The --sample flag must match the sample prefix in FASTQ filenames exactly.

## Examples

### count gene expression from 10x scRNA-seq FASTQ files
**Args:** `count --id=sample_output --transcriptome=/path/to/refdata-gex-GRCh38-2020-A --fastqs=/path/to/fastqs/ --sample=sample_name --localcores=16 --localmem=64`
**Explanation:** --id output directory name; --transcriptome 10x reference; --fastqs directory; --sample FASTQ prefix

### process 10x multiome (RNA+ATAC) data with Cell Ranger ARC
**Args:** `arc count --id=multiome_output --reference=/path/to/arc_ref/ --libraries=libraries.csv --localcores=16 --localmem=128`
**Explanation:** cellranger-arc count for RNA+ATAC multiome; --libraries CSV specifies ATAC and GEX FASTQ paths

### build a custom Cell Ranger reference from genome FASTA and GTF
**Args:** `mkref --genome=custom_genome --fasta=genome.fa --genes=genes.gtf --nthreads=8`
**Explanation:** --genome output reference name; --fasta genome FASTA; --genes GTF annotation; creates compatible reference

### process 10x ATAC-seq data with Cell Ranger ATAC
**Args:** `atac count --id=atac_output --reference=/path/to/refdata-cellranger-arc-GRCh38-2020-A-2.0.0 --fastqs=/path/to/atac_fastqs/ --sample=atac_sample --localcores=16 --localmem=64`
**Explanation:** cellranger-atac count for single-cell ATAC-seq; requires arc reference
