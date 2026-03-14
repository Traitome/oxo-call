---
name: megahit
category: assembly
description: Ultra-fast and memory-efficient de novo metagenome assembler for large and complex metagenomes
tags: [assembly, metagenome, de-novo, memory-efficient, ngs, illumina, microbiome]
author: oxo-call built-in
source_url: "https://github.com/voutcn/megahit"
---

## Concepts

- MEGAHIT is designed for large metagenomic datasets; it's faster and uses less memory than metaSPAdes.
- Use -1/-2 for paired-end reads; -r for single-end reads; comma-separate multiple files per end.
- Use -o for output directory; --num-cpu-threads for parallelism; --memory as fraction of RAM or bytes.
- MEGAHIT automatically selects k-mer values; use --k-min, --k-max, --k-step to customize.
- Output: final.contigs.fa in the output directory — filter contigs by length for downstream analysis.
- Use --min-contig-len to set minimum contig length in the output (default: 200 bp).
- MEGAHIT supports both metagenomic and single-species genome assembly.

## Pitfalls

- MEGAHIT output directory must not already exist — use a new directory each run or delete the old one.
- For complex metagenomes, use --presets meta-large for improved assembly of complex communities.
- MEGAHIT is fast but SPAdes --meta may produce better assemblies for simpler or well-sequenced metagenomes.
- Reads should be quality-trimmed before MEGAHIT for better assembly results.
- The --memory flag accepts both fraction (0.9 for 90% of RAM) and absolute values (200e9 for 200 GB).
- Very short contigs (<500 bp) are often not useful — increase --min-contig-len for cleaner output.

## Examples

### assemble a metagenome from paired-end reads
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o megahit_output/ --num-cpu-threads 16 --min-contig-len 500`
**Explanation:** -1/-2 paired-end input; -o output directory; --min-contig-len 500 removes very short contigs

### assemble a large complex metagenome with meta-large preset
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o large_meta/ --num-cpu-threads 32 --presets meta-large --min-contig-len 500`
**Explanation:** --presets meta-large uses k-mer range optimized for highly complex communities

### assemble metagenome from multiple samples combined
**Args:** `-1 s1_R1.fq.gz,s2_R1.fq.gz -2 s1_R2.fq.gz,s2_R2.fq.gz -o coassembly/ --num-cpu-threads 32 --min-contig-len 500`
**Explanation:** comma-separated input files for co-assembly; combines reads from multiple samples

### assemble with custom k-mer range for specific data type
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o custom_k/ --num-cpu-threads 16 --k-min 27 --k-max 127 --k-step 10`
**Explanation:** --k-min, --k-max, --k-step customize the k-mer iteration range
