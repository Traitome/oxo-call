---
name: bismark
category: epigenomics
description: Bisulfite sequencing alignment and methylation extraction tool for WGBS and RRBS data
tags: [methylation, bisulfite, wgbs, rrbs, epigenomics, cpg, dna-methylation]
author: oxo-call built-in
source_url: "https://github.com/FelixKrueger/Bismark"
---

## Concepts

- Bismark performs bisulfite-aware alignment (C→T and G→A converted references) and extracts CpG/CHG/CHH methylation.
- Three-step pipeline: (1) bismark_genome_preparation to build index; (2) bismark to align; (3) bismark_methylation_extractor.
- Bismark uses Bowtie2 (default) or HISAT2 as the underlying aligner; genome index must match the aligner.
- Use --non_directional for libraries prepared without strand selection (PBAT, scBS-seq, Swift libraries).
- Output BAM from bismark includes methylation information in XM tag; always deduplicate before extraction.
- bismark_methylation_extractor outputs CpG, CHG, and CHH context methylation in bedGraph and coverage files.
- Use --comprehensive to extract all cytosine contexts; --CX_context for CHG and CHH in addition to CpG.
- MBias plots from bismark_methylation_extractor reveal end-of-read bias; use --ignore/--ignore_3prime to trim.

## Pitfalls

- Bismark genome index must be in a directory, not pointing to the FASTA file directly.
- RRBS data requires '--rrbs' flag during alignment and extraction to handle MspI restriction site bias.
- Deduplication (deduplicate_bismark) MUST be run before methylation extraction for WGBS data.
- For paired-end bisulfite sequencing, use -1/-2 for reads; single-end uses positional argument.
- The --non_directional flag is critical for Swift/PBAT libraries — using directional mode loses 50% of reads.
- bismark_methylation_extractor needs the original aligned BAM before deduplication OR after — check protocol.

## Examples

### prepare bisulfite genome index for alignment
**Args:** `bismark_genome_preparation /path/to/genome_directory/`
**Explanation:** creates CT_conversion and GA_conversion subfolders in the genome directory; only run once per genome

### align paired-end WGBS reads to bisulfite genome
**Args:** `bismark --genome /path/to/genome_dir/ -1 R1.fastq.gz -2 R2.fastq.gz --output_dir bismark_output/ -p 4`
**Explanation:** --genome points to prepared genome directory; -1/-2 paired-end reads; -p 4 parallel cores

### deduplicate bismark-aligned paired-end BAM file
**Args:** `deduplicate_bismark --paired --bam sample_bismark_bt2_pe.bam`
**Explanation:** deduplicate_bismark removes PCR duplicates from bismark BAM; --paired for PE data

### extract methylation information from deduplicated BAM
**Args:** `bismark_methylation_extractor --paired-end --comprehensive --CX_context --genome_folder /path/to/genome_dir/ --output_dir methylation/ sample_deduplicated.bam`
**Explanation:** --comprehensive extracts all contexts; --CX_context includes CHG and CHH; outputs bedGraph and coverage files

### align RRBS data with MspI site handling
**Args:** `bismark --genome /path/to/genome_dir/ --rrbs -1 R1.fastq.gz -2 R2.fastq.gz --output_dir rrbs_output/ -p 4`
**Explanation:** --rrbs adjusts for MspI-digested RRBS libraries; trims methylation-invariant positions
