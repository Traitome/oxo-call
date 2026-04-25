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
- CROP:<length> cuts reads to specified length from start; HEADCROP:<length> removes N bases from start.
- TAILCROP:<length> removes N bases from end; MAXINFO:<target_length>:<strictness> for adaptive quality trimming.
- AVGQUAL:<quality> drops reads with average quality below threshold; MAXLEN:<length> drops reads longer than threshold.
- TOPHRED33/TOPHRED64 converts quality encoding; useful for downstream tool compatibility.

## Pitfalls
- trimmomatic ARGS must start with 'PE' or 'SE' (mode selector for paired-end or single-end) — never with input files or flags like -threads. The mode (PE/SE) ALWAYS comes first.
- PE mode requires exactly 2 input files and produces 4 output files — missing any output path causes an error.
- The adapter file path must be absolute or correct relative path — Trimmomatic does not search PATH for it.
- ILLUMINACLIP must come before LEADING/TRAILING/SLIDINGWINDOW in the step list for best results.
- Forgetting -phred33 on modern Illumina data may give incorrect quality scores (default assumes Phred64 in older versions).
- Trimmomatic is a Java tool — the command is 'trimmomatic' or 'java -jar trimmomatic.jar', not just the JAR name.
- MINLEN should be set based on your downstream tool requirements (e.g., ≥36 for STAR, ≥25 for BWA).
- CROP cuts from end to reach target length; HEADCROP removes from start regardless of quality.
- MAXINFO is adaptive — balances read length vs quality; strictness 0.8 favors correctness, 0.2 favors length.
- AVGQUAL filters entire reads based on average quality, different from LEADING/TRAILING which trim ends.
- Steps are applied in order — place MINLEN last to ensure final length filtering.

## Examples

### trim adapters and quality-filter paired-end Illumina reads
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz input paired reads; R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz four output files; ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 adapter removal; LEADING:3 trim low-quality from start; TRAILING:3 trim low-quality from end; SLIDINGWINDOW:4:15 sliding window quality filter; MINLEN:36 minimum length cutoff

### trim single-end reads with quality filtering
**Args:** `SE -threads 4 -phred33 reads.fastq.gz trimmed_reads.fastq.gz ILLUMINACLIP:TruSeq3-SE.fa:2:30:10 LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** trimmomatic SE mode; -threads 4 parallelism; -phred33 quality encoding; reads.fastq.gz input reads; trimmed_reads.fastq.gz output file; ILLUMINACLIP:TruSeq3-SE.fa:2:30:10 adapter removal; LEADING:3 TRAILING:3 quality trimming; SLIDINGWINDOW:4:15 sliding window; MINLEN:36 minimum length

### trim Nextera adapters from paired-end reads
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:NexteraPE-PE.fa:2:30:10:8:true LEADING:3 TRAILING:3 SLIDINGWINDOW:4:15 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; ILLUMINACLIP:NexteraPE-PE.fa:2:30:10:8:true Nextera adapter file with palindrome mode; LEADING:3 TRAILING:3 quality trimming; SLIDINGWINDOW:4:15 sliding window; MINLEN:36 minimum length

### aggressive quality trimming for low-quality paired-end data
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 LEADING:5 TRAILING:5 SLIDINGWINDOW:4:20 MINLEN:50`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 adapter removal; LEADING:5 TRAILING:5 stricter thresholds; SLIDINGWINDOW:4:20 higher window quality; MINLEN:50 longer minimum length

### trim reads to fixed length (e.g., for uniformity)
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz CROP:100 HEADCROP:10 MINLEN:50`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; HEADCROP:10 removes 10 bp from start; CROP:100 keeps first 100 bp; MINLEN:50 minimum length; useful for removing UMIs/barcodes

### remove low-quality reads by average quality
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz AVGQUAL:20 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; AVGQUAL:20 drops reads with average quality <20; MINLEN:36 minimum length; filters poor-quality reads entirely

### adaptive quality trimming with MAXINFO
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz MAXINFO:40:0.8 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; MAXINFO:40:0.8 adaptive trimming (target 40bp, strictness 0.8 favors correctness); MINLEN:36 minimum length

### trim from 3' end only (TAILCROP)
**Args:** `PE -threads 8 -phred33 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz TAILCROP:10 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred33 quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; TAILCROP:10 removes 10 bp from 3' end; MINLEN:36 minimum length; useful for systematic 3' issues

### convert quality encoding to Phred-33
**Args:** `PE -threads 8 -phred64 R1.fastq.gz R2.fastq.gz R1_paired.fastq.gz R1_unpaired.fastq.gz R2_paired.fastq.gz R2_unpaired.fastq.gz TOPHRED33 ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 MINLEN:36`
**Explanation:** trimmomatic PE mode; -threads 8 parallelism; -phred64 input quality encoding; R1.fastq.gz R2.fastq.gz inputs; four output files; TOPHRED33 converts Phred-64 to Phred-33; ILLUMINACLIP:TruSeq3-PE.fa:2:30:10 adapter removal; MINLEN:36 minimum length

### filter by maximum read length
**Args:** `SE -threads 4 -phred33 reads.fastq.gz trimmed.fastq.gz MAXLEN:150 MINLEN:36`
**Explanation:** trimmomatic SE mode; -threads 4 parallelism; -phred33 quality encoding; reads.fastq.gz input reads; trimmed.fastq.gz output file; MAXLEN:150 drops reads longer than 150 bp; MINLEN:36 minimum length; useful for removing chimeric reads
