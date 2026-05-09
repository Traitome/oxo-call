---
name: btrim
category: Sequence Quality Control
description: A bioinformatics tool for trimming low-quality bases and adapter sequences from FASTQ sequencing reads. Supports quality-score based trimming, adapter removal, and length filtering.
tags: [trimming, fastq, quality-control, adapters, bioinformatics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/btrim
---

## Concepts

- **Input Format:** btrim accepts FASTQ files (single-end or paired-end) and supports gzipped (.gz) input. The tool reads quality scores (Phred+33 or Phred+64 encoding) to determine trimming endpoints.
- **Quality Trimming Algorithm:** Uses a sliding window approach with configurable window size and quality threshold. Bases are trimmed from the 3' end when the average quality in the window drops below the threshold.
- **Adapter Detection:** Supports known adapter sequences (Illumina, TruSeq) and allows custom adapter specification. Uses exact matching or allows user-defined mismatch tolerance.
- **Output Modes:** Generates trimmed FASTQ files and optional summary statistics reports. Can output separate files for kept reads and discarded reads based on length thresholds.

## Pitfalls

- **Ignoring Quality Score Encoding:** Failing to specify the correct quality score encoding (Phred+33 vs Phred+64) causes incorrect trimming decisions and may delete valid reads or retain low-quality data.
- **Insufficient Adapter Sequences:** Using incomplete or incorrect adapter sequences leaves residual adapter artifacts, compromising downstream analysis like assembly or variant calling.
- **Overly Aggressive Trimming:** Setting quality thresholds too high or minimum length thresholds too strict results in excessive data loss, reducing coverage and statistical power in downstream analyses.
- **Paired-End File Mismatch:** Not preserving read ordering between read1 and read2 files when trimming paired-end data creates misaligned pairs, breaking tools that expect synchronized mate pairs.

## Examples

### Trim low-quality bases from 3' ends using a quality threshold of 20
**Args:** `-i input.fastq -q 20 -o trimmed.fastq`
**Explanation:** Removes bases from the 3' end where the sliding window quality drops below Phred score 20, improving overall read quality.

### Remove Illumina universal adapters from paired-end reads
**Args:** `-i read1.fastq -i2 read2.fastq -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC -a2 AGATCGGAAGAGCGTCGTGTAGGGAAAGAGTGT -o read1_trimmed.fastq -o2 read2_trimmed.fastq`
**Explanation:** Strips the specified Illumina adapter sequences from both ends of read1 and read2 files in paired-end mode.

### Trim reads to a minimum length of 50 bases after quality trimming
**Args:** `-i input.fastq -q 20 -m 50 -o output.fastq`
**Explanation:** Applies quality-based trimming then discards any reads shorter than 50 bases, ensuring consistent input for downstream applications.

### Use a sliding window of 4 bases with quality threshold 25
**Args:** `-i input.fastq -q 25 -w 4 -o trimmed.fastq`
**Explanation:** Calculates the average quality across 4-base windows and trims from the 3' end when the average falls below 25.

### Specify custom adapter sequence with 2 allowed mismatches
**Args:** `-i input.fastq -a MYCUSTOMADAPTER -m 2 -o output.fastq`
**Explanation:** Removes the custom adapter sequence while allowing up to 2 mismatches, accommodating sequencing errors in the adapter.