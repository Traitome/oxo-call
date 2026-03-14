---
name: trimmomatic
category: qc
description: Flexible read trimming tool for Illumina NGS data with adapter removal and quality filtering
tags: [trimming, adapter, quality-control, illumina, paired-end, ngs]
author: oxo-call built-in
source_url: "http://www.usadellab.org/cms/?page=trimmomatic"
---

## Concepts

- Trimmomatic processes reads in a pipeline of steps (ILLUMINACLIP, LEADING, TRAILING, SLIDINGWINDOW, MINLEN) applied in listed order.
- Use PE mode for paired-end data and SE mode for single-end; PE mode outputs 4 files (R1_paired, R1_unpaired, R2_paired, R2_unpaired).
- ILLUMINACLIP:<adapters.fa>:<seed_mismatches>:<palindrome_clip_threshold>:<simple_clip_threshold> — typical: ILLUMINACLIP:TruSeq3-PE.fa:2:30:10
- SLIDINGWINDOW:4:15 scans 4-base windows and cuts when average quality drops below 15 (Phred scale).
- MINLEN:36 discards reads shorter than 36 bp after trimming — always include to avoid very short fragments.
- Common adapter files: TruSeq3-PE.fa, TruSeq3-SE.fa, NexteraPE-PE.fa — found in Trimmomatic's adapters/ directory.
- Use -threads N for multi-threading; -phred33 or -phred64 to specify quality encoding (modern data is -phred33).

## Pitfalls

- PE mode requires exactly 2 input files and produces 4 output files — missing any output path causes an error.
- The adapter file path must be absolute or correct relative path — Trimmomatic does not search PATH for it.
- ILLUMINACLIP must come before LEADING/TRAILING/SLIDINGWINDOW in the step list for best results.
- Forgetting -phred33 on modern Illumina data may give incorrect quality scores (default assumes Phred64 in older versions).
- Trimmomatic is a Java tool — the command is 'trimmomatic' or 'java -jar trimmomatic.jar', not just the JAR name.
- MINLEN should be set based on your downstream tool requirements (e.g., ≥36 for STAR, ≥25 for BWA).

## Examples

### trim adapters and quality-filter paired-end Illumina reads
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** PE mode with adapter removal, quality trimming of leading/trailing bases, sliding window filter, and minimum length cutoff

### trim single-end reads with quality filtering
**Args:** `SE -threads 4 -phred33 reads.fastq.gz trimmed_reads.fastq.gz ILLUMINACLIP:TruSeq3-SE.fa:2:30:10 LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** SE mode for single-end reads with standard quality trimming steps

### trim Nextera adapters from paired-end reads
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:NexteraPE-PE.fa:2:30:10:8:true LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** NexteraPE-PE.fa adapter file for Nextera/Transposase library prep; :8:true enables palindrome mode

### aggressive quality trimming for low-quality paired-end data
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 LEADING:5 TRAILING:5 SLIDINGWINDOW:4:20 MINLEN:50`
**Explanation:** stricter quality thresholds (LEADING/TRAILING:5, window quality:20) for higher-confidence trimming
