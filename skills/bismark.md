---
name: bismark
category: epigenomics
description: Bisulfite sequencing alignment and methylation extraction tool for WGBS, RRBS, and SLAM-seq data
tags: [methylation, bisulfite, wgbs, rrbs, epigenomics, cpg, dna-methylation, slam-seq, minimap2]
author: oxo-call built-in
source_url: "https://github.com/FelixKrueger/Bismark"
---

## Concepts

- Bismark performs bisulfite-aware alignment (C→T and G→A converted references) and extracts CpG/CHG/CHH methylation.
- Three-step pipeline: (1) bismark_genome_preparation to build index; (2) bismark to align; (3) bismark_methylation_extractor to extract methylation.
- All companion binaries (bismark_genome_preparation, bismark_methylation_extractor, deduplicate_bismark, bismark2report, bismark2summary, coverage2cytosine, bam2nuc) are detected automatically — use them as the first ARGS token.
- Bismark supports three aligners: Bowtie2 (default, short reads), HISAT2 (splice-aware), minimap2 (long reads: PacBio/Nanopore BS-Seq). Genome index must match the aligner.
- Use --non_directional for libraries prepared without strand selection (PBAT, scBS-seq, Swift libraries).
- --slam mode enables SLAM-seq/TUC-seq time-resolved experiments (T→C and A→G conversions instead of bisulfite).
- Output BAM from bismark includes methylation information in XM tag; always deduplicate before extraction.
- bismark_methylation_extractor outputs CpG, CHG, and CHH context methylation in bedGraph and coverage files.
- Use --comprehensive to extract all cytosine contexts; --CX_context for CHG and CHH in addition to CpG.
- MBias plots from bismark_methylation_extractor reveal end-of-read bias; use --ignore/--ignore_3prime to trim.
- --parallel/--multicore N runs N instances of Bismark concurrently (near-linear speedup up to 8); resource-hungry (N× memory/CPU).
- bismark2report generates HTML reports from alignment logs; bismark2summary aggregates multiple reports.
- coverage2cytosine converts coverage files to per-cytosine methylation reports; bam2nuc computes nucleotide composition of the reference.

## Pitfalls

- CRITICAL: Bismark is a multi-binary suite. The main alignment command is just 'bismark' — ARGS starts with flags like --genome, -1, -2, NOT with a subcommand. The companion tools are separate binaries: bismark_genome_preparation, bismark_methylation_extractor, deduplicate_bismark, bismark2bedGraph, bismark2report, bismark2summary, coverage2cytosine, bam2nuc. Use the correct binary name for each step.
- Bismark genome index must be in a directory, not pointing to the FASTA file directly.
- Index building uses companion binary 'bismark_genome_preparation <genome_dir>'; provide the directory path, not the FASTA file.
- RRBS data requires '--rrbs' flag during alignment and extraction to handle MspI restriction site bias.
- Deduplication (deduplicate_bismark) is NOT recommended for RRBS data — RRBS naturally produces duplicated positions.
- For WGBS, deduplicate_bismark MUST be run before methylation extraction.
- For paired-end deduplication, the BAM must be sorted by read name (samtools sort -n), NOT by position — position-sorted BAMs will produce incorrect deduplication.
- For paired-end bisulfite sequencing, use -1/-2 for reads; single-end uses positional argument.
- The --non_directional flag is critical for Swift/PBAT libraries — using directional mode loses 50% of reads.
- Bowtie1 support was removed in Bismark v0.25; only Bowtie2, HISAT2, and minimap2 are supported.
- --parallel increases memory linearly (--parallel 4 = 4× memory); monitor system resources.

## Examples

### prepare bisulfite genome index for alignment
**Args:** `bismark_genome_preparation /path/to/genome_directory/`
**Explanation:** bismark_genome_preparation companion binary; creates CT_conversion and GA_conversion subfolders; only run once per genome

### align paired-end WGBS reads to bisulfite genome
**Args:** `--genome /path/to/genome_dir/ -1 R1.fastq.gz -2 R2.fastq.gz --output_dir bismark_output/ -p 4`
**Explanation:** --genome points to prepared genome directory; -1/-2 paired-end reads; -p 4 parallel cores

### deduplicate bismark-aligned paired-end BAM file
**Args:** `deduplicate_bismark --paired --bam sample_bismark_bt2_pe.bam`
**Explanation:** deduplicate_bismark companion binary; removes PCR duplicates from bismark BAM; --paired for PE data; NOT recommended for RRBS

### extract methylation information from deduplicated BAM
**Args:** `bismark_methylation_extractor --paired-end --comprehensive --CX_context --genome_folder /path/to/genome_dir/ --output_dir methylation/ sample_deduplicated.bam`
**Explanation:** bismark_methylation_extractor companion binary; --comprehensive extracts all contexts; outputs bedGraph and coverage files

### align RRBS data with MspI site handling
**Args:** `--genome /path/to/genome_dir/ --rrbs -1 R1.fastq.gz -2 R2.fastq.gz --output_dir rrbs_output/ -p 4`
**Explanation:** --rrbs adjusts for MspI-digested RRBS libraries; trims methylation-invariant positions; do NOT deduplicate RRBS data

### align single-end WGBS reads with HISAT2 aligner
**Args:** `--genome /path/to/genome_dir/ --hisat2 reads.fastq.gz --output_dir bismark_output/ -p 4`
**Explanation:** --hisat2 uses HISAT2 instead of Bowtie2; genome index must have been prepared for HISAT2

### align PBAT or scBS-seq (non-directional) library
**Args:** `--genome /path/to/genome_dir/ --non_directional -1 R1.fastq.gz -2 R2.fastq.gz --output_dir pbat_output/ -p 4`
**Explanation:** --non_directional required for PBAT/Swift/scBS-seq libraries; aligns all four strands

### align long-read bisulfite data (PacBio/Nanopore) with minimap2
**Args:** `--genome /path/to/genome_dir/ --minimap2 reads.fastq.gz --output_dir nanopore_output/ -p 4`
**Explanation:** --minimap2 for long-read BS-Seq (PacBio/Nanopore); genome must be prepared with bismark_genome_preparation --minimap2

### align SLAM-seq time-resolved experiment data
**Args:** `--genome /path/to/genome_dir/ --slam -1 R1.fastq.gz -2 R2.fastq.gz --output_dir slam_output/ -p 4`
**Explanation:** --slam mode uses T→C and A→G conversions for SLAM-seq/TUC-seq experiments instead of bisulfite

### generate HTML alignment report from bismark output
**Args:** `bismark2report --output_dir reports/`
**Explanation:** bismark2report companion binary; reads bismark mapping report files from current directory; generates HTML visualization of alignment stats

### generate multi-sample summary report
**Args:** `bismark2summary --output_dir reports/ sample1_bismark_bt2/ sample2_bismark_bt2/`
**Explanation:** bismark2summary companion binary; aggregates multiple sample reports into a single HTML summary with comparison charts

### convert coverage file to per-cytosine methylation report
**Args:** `coverage2cytosine --genome_folder /path/to/genome_dir/ sample_CpG_report.txt`
**Explanation:** coverage2cytosine companion binary; converts coverage output to individual cytosine methylation levels across all contexts

### prepare bisulfite index for minimap2-based long-read alignment
**Args:** `bismark_genome_preparation --minimap2 /path/to/genome_directory/`
**Explanation:** bismark_genome_preparation companion binary; --minimap2 prepares the index for minimap2-based long-read alignment
