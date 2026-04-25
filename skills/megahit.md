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
- --presets meta-sensitive uses more k-mers for sensitive detection of low-abundance species.
- --presets meta-large uses larger k-mer range optimized for complex soil metagenomes.
- --bubble-level controls bubble merging intensity (0-2); bubbles occur from heterozygosity or repeats.
- --prune-level controls low-depth pruning strength (0-3); higher values remove more low-coverage regions.
- --no-mercy disables mercy k-mer recovery; reduces memory but may lose low-abundance sequences.

## Pitfalls
- MEGAHIT output directory must not already exist — use a new directory each run or delete the old one.
- For complex metagenomes, use --presets meta-large for improved assembly of complex communities.
- MEGAHIT is fast but SPAdes --meta may produce better assemblies for simpler or well-sequenced metagenomes.
- Reads should be quality-trimmed before MEGAHIT for better assembly results.
- The --memory flag accepts both fraction (0.9 for 90% of RAM) and absolute values (200e9 for 200 GB).
- Very short contigs (<500 bp) are often not useful — increase --min-contig-len for cleaner output.
- --no-mercy reduces memory but loses low-abundance sequences; use only for high-coverage datasets.
- --bubble-level 0 disables bubble merging; may fragment contigs in polymorphic regions.
- Odd k-mer sizes required; even values cause errors. Maximum k-mer is 255.
- --continue resumes interrupted runs; useful for large assemblies that may crash.

## Examples

### assemble a metagenome from paired-end reads
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o megahit_output/ --num-cpu-threads 16 --min-contig-len 500`
**Explanation:** megahit command; -1 R1.fastq.gz -2 R2.fastq.gz paired-end input; -o megahit_output/ output directory; --num-cpu-threads 16 threads; --min-contig-len 500 removes very short contigs

### assemble a large complex metagenome with meta-large preset
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o large_meta/ --num-cpu-threads 32 --presets meta-large --min-contig-len 500`
**Explanation:** megahit command; -1/-2 paired-end input; -o large_meta/ output directory; --num-cpu-threads 32 threads; --presets meta-large k-mer range optimized for highly complex communities; --min-contig-len 500 minimum contig length

### assemble metagenome from multiple samples combined
**Args:** `-1 s1_R1.fq.gz,s2_R1.fq.gz -2 s1_R2.fq.gz,s2_R2.fq.gz -o coassembly/ --num-cpu-threads 32 --min-contig-len 500`
**Explanation:** megahit command; -1/-2 comma-separated input files for co-assembly; -o coassembly/ output directory; --num-cpu-threads 32 threads; --min-contig-len 500 minimum contig length

### assemble with custom k-mer range for specific data type
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o custom_k/ --num-cpu-threads 16 --k-min 27 --k-max 127 --k-step 10`
**Explanation:** megahit command; -1/-2 paired-end input; -o custom_k/ output directory; --num-cpu-threads 16 threads; --k-min 27 --k-max 127 --k-step 10 customize k-mer iteration range

### assemble with meta-sensitive preset for low-abundance detection
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o sensitive_out/ --num-cpu-threads 16 --presets meta-sensitive --min-contig-len 500`
**Explanation:** megahit command; -1/-2 paired-end input; -o sensitive_out/ output directory; --num-cpu-threads 16 threads; --presets meta-sensitive uses more k-mers (21-141) for detecting low-abundance species; --min-contig-len 500 minimum contig length

### reduce memory usage with no-mercy for high-coverage data
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o low_mem/ --num-cpu-threads 16 --no-mercy --memory 0.5 --min-contig-len 500`
**Explanation:** megahit command; -1/-2 paired-end input; -o low_mem/ output directory; --num-cpu-threads 16 threads; --no-mercy disables mercy k-mers; --memory 0.5 limits RAM to 50%; --min-contig-len 500 minimum contig length

### resume interrupted assembly with continue option
**Args:** `-o resumed_out/ --continue`
**Explanation:** megahit command; -o resumed_out/ output directory; --continue resumes from last checkpoint

### assemble single-end reads only
**Args:** `-r reads.fastq.gz -o se_out/ --num-cpu-threads 16 --min-contig-len 500`
**Explanation:** megahit command; -r reads.fastq.gz single-end reads; -o se_out/ output directory; --num-cpu-threads 16 threads; --min-contig-len 500 minimum contig length

### assemble interleaved paired-end reads
**Args:** `--12 interleaved.fastq.gz -o interleaved_out/ --num-cpu-threads 16 --min-contig-len 500`
**Explanation:** megahit command; --12 interleaved.fastq.gz interleaved paired-end format; -o interleaved_out/ output directory; --num-cpu-threads 16 threads; --min-contig-len 500 minimum contig length

### assemble with bubble-level adjustment for polymorphic data
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o bubble_adj/ --num-cpu-threads 16 --bubble-level 1 --min-contig-len 500`
**Explanation:** megahit command; -1/-2 paired-end input; -o bubble_adj/ output directory; --num-cpu-threads 16 threads; --bubble-level 1 moderate bubble merging; --min-contig-len 500 minimum contig length

### assemble with prune-level for low-depth regions
**Args:** `-1 R1.fastq.gz -2 R2.fastq.gz -o pruned/ --num-cpu-threads 16 --prune-level 2 --min-contig-len 500`
**Explanation:** megahit command; -1/-2 paired-end input; -o pruned/ output directory; --num-cpu-threads 16 threads; --prune-level 2 removes low-coverage regions; --min-contig-len 500 minimum contig length

### calculate assembly statistics after MEGAHIT
**Args:** `awk '/^>/{if(l!="") print l; l=0; next}{l+=length($0)}END{print l}' megahit_output/final.contigs.fa | sort -n | awk '{sum+=$1; count++}END{print "Total:", sum, "Count:", count, "N50:", NR%2?$0:a[(NR+1)/2]}'`
**Explanation:** calculate total length, contig count, and N50 from assembly output; essential quality assessment

### filter contigs by length after assembly
**Args:** `seqkit seq -m 1000 megahit_output/final.contigs.fa > filtered_contigs.fa`
**Explanation:** use seqkit to filter contigs ≥1000bp; removes short contigs that may be assembly artifacts
