---
name: cellranger
category: single-cell
description: 10x Genomics Cell Ranger pipeline for single-cell RNA-seq, ATAC-seq, and multiome data processing
tags: [single-cell, scrna-seq, 10x-genomics, cellranger, gene-expression, atac-seq, multiome, cell-multiplexing, aggregation, mkref]
author: oxo-call built-in
source_url: "https://www.10xgenomics.com/support/software/cell-ranger"
---

## Concepts

- Cell Ranger is 10x Genomics' official pipeline for single-cell data; main commands: count, multi, aggr, mkref, arc, atac, vdj.
- cellranger count processes scRNA-seq (10x 3' or 5' Gene Expression) from FASTQ files.
- cellranger multi handles Cell Multiplexing (CellPlex) and multi-modal data (GEX + VDJ) via a CSV config file.
- cellranger aggr aggregates outputs from multiple count runs for combined analysis.
- Requires: --transcriptome (pre-built reference), --fastqs (FASTQ directory), --sample (sample name prefix).
- Build a custom reference with 'cellranger mkref --genome --fasta --genes' for non-standard genomes.
- Cell Ranger output: per_barcode_metrics.csv, molecule_info.h5, filtered_feature_bc_matrix/, web_summary.html, cloupe.cloupe.
- Use --localcores and --localmem to control CPU and RAM usage.
- Expect 1-2 hours per sample for human scRNA-seq on 8 cores with 64 GB RAM.
- The FASTQ files must follow 10x naming conventions: <sample>_S1_L001_R1_001.fastq.gz.
- cellranger-arc is a separate binary for multiome (RNA + ATAC) data processing.
- cellranger-atac is a separate binary for ATAC-seq only data.
- cellranger-vdj is a separate binary for immune repertoire profiling (BCR/TCR).

## Pitfalls

- Cell Ranger requires FASTQ files named with 10x conventions — check file naming before running.
- The --transcriptome must be a Cell Ranger compatible reference (built with cellranger mkref or pre-built).
- Cannot use generic STAR or HISAT2 indices — must use Cell Ranger-formatted references.
- Cell Ranger output directory (--id) must not already exist — it creates a fresh directory.
- Without --localcores and --localmem, Cell Ranger may use all available resources on shared systems.
- The --sample flag must match the sample prefix in FASTQ filenames exactly.
- cellranger-arc and cellranger-atac are separate binaries from cellranger — they are not subcommands.
- cellranger aggr requires a CSV file listing molecule_info.h5 paths from previous count runs.
- cellranger multi requires a CSV config file with specific columns — check 10x documentation for format.
- Different Cell Ranger versions may produce incompatible outputs — always use the same version for samples to be aggregated.
- Memory requirements scale with cell count — large datasets (>50k cells) may require >128GB RAM.

## Examples

### count gene expression from 10x scRNA-seq FASTQ files
**Args:** `count --id=sample_output --transcriptome=/path/to/refdata-gex-GRCh38-2020-A --fastqs=/path/to/fastqs/ --sample=sample_name --localcores=16 --localmem=64`
**Explanation:** count subcommand; --id=sample_output output directory name; --transcriptome=/path/to/refdata-gex-GRCh38-2020-A 10x reference; --fastqs=/path/to/fastqs/ FASTQ directory; --sample=sample_name FASTQ prefix; --localcores=16 CPU limit; --localmem=64 RAM limit (GB)

### process 10x multiome (RNA+ATAC) data with Cell Ranger ARC
**Args:** `arc count --id=multiome_output --reference=/path/to/arc_ref/ --libraries=libraries.csv --localcores=16 --localmem=128`
**Explanation:** arc count subcommand for cellranger-arc; --id=multiome_output output directory; --reference=/path/to/arc_ref/ multiome reference; --libraries=libraries.csv CSV specifies ATAC and GEX FASTQ paths; --localcores=16 CPU limit; --localmem=128 RAM limit

### build a custom Cell Ranger reference from genome FASTA and GTF
**Args:** `mkref --genome=custom_genome --fasta=genome.fa --genes=genes.gtf --nthreads=8`
**Explanation:** mkref subcommand; --genome=custom_genome output reference name; --fasta=genome.fa genome FASTA input; --genes=genes.gtf GTF annotation input; --nthreads=8 threads; creates Cell Ranger compatible reference

### process 10x ATAC-seq data with Cell Ranger ATAC
**Args:** `atac count --id=atac_output --reference=/path/to/refdata-cellranger-arc-GRCh38-2020-A-2.0.0 --fastqs=/path/to/atac_fastqs/ --sample=atac_sample --localcores=16 --localmem=64`
**Explanation:** atac count subcommand for cellranger-atac; --id=atac_output output directory; --reference=/path/to/refdata-cellranger-arc-GRCh38-2020-A-2.0.0 ATAC reference; --fastqs=/path/to/atac_fastqs/ FASTQ directory; --sample=atac_sample FASTQ prefix; --localcores=16 CPU limit; --localmem=64 RAM limit

### aggregate multiple samples with cellranger aggr
**Args:** `aggr --id=combined_analysis --csv=aggregation.csv --localcores=16 --localmem=64`
**Explanation:** aggr subcommand; --id=combined_analysis output directory; --csv=aggregation.csv CSV lists sample_id,molecule_h5 columns; --localcores=16 CPU limit; --localmem=64 RAM limit; generates unified feature-barcode matrix

### process cell multiplexing data with cellranger multi
**Args:** `multi --id=multiplexed_output --csv=config.csv --localcores=16 --localmem=64`
**Explanation:** multi subcommand; --id=multiplexed_output output directory; --csv=config.csv CSV config file; --localcores=16 CPU limit; --localmem=64 RAM limit; processes CellPlex multiplexed data where multiple samples share a GEM well; config.csv defines libraries and sample assignments

### process VDJ immune repertoire data
**Args:** `vdj count --id=vdj_output --reference=/path/to/refdata-cellranger-vdj-GRCh38-alts-ensembl-7.0.0 --fastqs=/path/to/vdj_fastqs/ --sample=vdj_sample --localcores=8 --localmem=32`
**Explanation:** vdj count subcommand for cellranger-vdj; --id=vdj_output output directory; --reference=/path/to/refdata-cellranger-vdj-GRCh38-alts-ensembl-7.0.0 VDJ reference; --fastqs=/path/to/vdj_fastqs/ FASTQ directory; --sample=vdj_sample FASTQ prefix; --localcores=8 CPU limit; --localmem=32 RAM limit; separate binary for BCR/TCR repertoire analysis

### run cellranger with BAM output disabled
**Args:** `count --id=sample_output --transcriptome=/path/to/ref --fastqs=/path/to/fastqs/ --sample=sample_name --create-bam=false --localcores=16 --localmem=64`
**Explanation:** count subcommand; --id=sample_output output directory; --transcriptome=/path/to/ref 10x reference; --fastqs=/path/to/fastqs/ FASTQ directory; --sample=sample_name FASTQ prefix; --create-bam=false skips BAM generation; --localcores=16 CPU limit; --localmem=64 RAM limit; saves ~50% disk space

### test cellranger count with dry-run
**Args:** `count --id=test_run --transcriptome=/path/to/ref --fastqs=/path/to/fastqs/ --sample=sample_name --dry`
**Explanation:** count subcommand; --id=test_run output directory; --transcriptome=/path/to/ref 10x reference; --fastqs=/path/to/fastqs/ FASTQ directory; --sample=sample_name FASTQ prefix; --dry performs dry run to validate inputs and estimate resource requirements without executing full pipeline

### count with custom feature-barcode matrix output
**Args:** `count --id=sample_output --transcriptome=/path/to/ref --fastqs=/path/to/fastqs/ --sample=sample_name --feature-ref=features.csv --localcores=16 --localmem=64`
**Explanation:** count subcommand; --id=sample_output output directory; --transcriptome=/path/to/ref 10x reference; --fastqs=/path/to/fastqs/ FASTQ directory; --sample=sample_name FASTQ prefix; --feature-ref=features.csv CSV defines feature barcodes and types; --localcores=16 CPU limit; --localmem=64 RAM limit; enables Feature Barcoding technology (CITE-seq, CellPlex)

### count with force cells parameter for low-quality data
**Args:** `count --id=sample_output --transcriptome=/path/to/ref --fastqs=/path/to/fastqs/ --sample=sample_name --force-cells=5000 --localcores=16 --localmem=64`
**Explanation:** count subcommand; --id=sample_output output directory; --transcriptome=/path/to/ref 10x reference; --fastqs=/path/to/fastqs/ FASTQ directory; --sample=sample_name FASTQ prefix; --force-cells=5000 overrides automatic cell detection; --localcores=16 CPU limit; --localmem=64 RAM limit; useful for samples with low RNA content or high background

### build reference with custom reference genome and pre-mRNA
**Args:** `mkref --genome=custom_genome --fasta=genome.fa --genes=genes.gtf --ref-version=1.0.0 --nthreads=8`
**Explanation:** mkref subcommand; --genome=custom_genome output reference name; --fasta=genome.fa genome FASTA input; --genes=genes.gtf GTF annotation input; --ref-version=1.0.0 reference version; --nthreads=8 threads; Cell Ranger includes pre-mRNA sequences by default for intron counting
