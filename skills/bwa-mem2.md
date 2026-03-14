---
name: bwa-mem2
category: alignment
description: Faster version of BWA-MEM2 with 2-3x speedup for short read alignment using SIMD acceleration
tags: [alignment, mapping, short-read, ngs, illumina, bwa, reference, simd]
author: oxo-call built-in
source_url: "https://github.com/bwa-mem2/bwa-mem2"
---

## Concepts

- BWA-MEM2 is a drop-in replacement for BWA-MEM with 2-3x faster alignment using AVX512 SIMD instructions.
- All BWA-MEM flags and parameters are compatible with BWA-MEM2 — use the same arguments.
- Index build: bwa-mem2 index ref.fa; creates .bwt.2bit.64, .ann, .amb, .pac, .0123 files.
- Use -t N for threads; -R for read group (required by GATK); outputs SAM to stdout.
- BWA-MEM2 requires more disk space for the index than BWA (about 6x genome size).
- BWA-MEM2 automatically selects the best SIMD instruction set (AVX512, AVX2, SSE4.1) for your CPU.
- For GATK best practices, add read group: -R '@RG\tID:id\tSM:sample\tLB:lib\tPL:ILLUMINA'.

## Pitfalls

- BWA-MEM2 index is NOT interchangeable with BWA index — must re-index with bwa-mem2 index.
- On older CPUs without AVX2/SSE4.1 support, BWA-MEM2 may not run or may fall back to generic mode.
- BWA-MEM2 uses more RAM than BWA during alignment due to pre-loaded index (65+ GB for human genome).
- Output is SAM to stdout — always pipe to samtools view -b or redirect to a file.
- For paired-end data with GATK downstream, always include read groups with -R flag.

## Examples

### build BWA-MEM2 index from reference genome
**Args:** `index reference.fa`
**Explanation:** creates reference.fa.* index files; may take 30-60 min and ~60 GB RAM for human genome

### align paired-end reads to reference using 16 threads
**Args:** `mem -t 16 reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -@ 4 -o sorted.bam`
**Explanation:** BWA-MEM2 mem has same flags as BWA-MEM; pipe to samtools sort for sorted BAM

### align paired-end reads with GATK read group
**Args:** `mem -t 16 -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA' reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o aligned.bam`
**Explanation:** -R adds read group required for GATK downstream; same syntax as BWA
