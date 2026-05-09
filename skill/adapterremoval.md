---
name: adapterremoval
category: read-processing
description: Trims adapter sequences and low-quality bases from next-generation sequencing (NGS) reads. Supports FASTQ, SAM, BAM formats and both single-end and paired-end data.
tags: [fastq, trimming, adapters, quality-control, ngs, illumina]
author: AI-generated
source_url: https://github.com/MikkelSchubert/adapterremoval
---

## Concepts

- **Input formats**: AdapterRemoval accepts FASTQ files (gzipped or plain), SAM, and BAM formats. For paired-end data, provide both read files using `--read1` and `--read2` flags or specify them as positional arguments.
- **Adapter detection**: The tool automatically detects common Illumina adapter sequences (e.g., ATCTCGTATG, GATCTTCC) from the input files. Custom adapter sequences can be specified using `--adapter1` and `--adapter2` for read 1 and read 2 respectively.
- **Quality trimming**: Bases with quality scores below the threshold (default 2) are trimmed from the 3' end of reads. Use `--qualitythreshold` to adjust sensitivity. The `--minquality` flag removes entire reads falling below the specified quality floor.
- **Output formats**: Results can be written as FASTQ, collapsed FASTQ (with merged reads), SAM, or BAM. Use `--output` for read 1, `--output2` for read 2, and `--compressed` for gzip compression.
- **Read merging**: For paired-end overlapping reads, AdapterRemoval can merge them before output using `--collapse` (detects overlapping pairs and creates a consensus sequence), or `-- collapse--exact` for strict matching.

## Pitfalls

- **Missing adapter specification**: If your data uses non-standard adapters and you don't specify them with `--adapter1`/`--adapter2`, adapter sequences will remain in the reads, causing alignment issues and false variants in downstream analysis.
- **Inappropriate quality threshold**: Setting `--qualitythreshold` too high (e.g., 30) removes too much data including valid reads; setting it too low (e.g., 0) keeps low-quality bases that introduce sequencing errors.
- **Paired-end file mismatch**: Providing read files in the wrong order or using mismatched `--read1`/`--read2` causes the tool to incorrectly identify read pairs, resulting in lost or corrupted paired-end information.
- **Output format incompatibility**: Writing output in SAM format when downstream tools expect BAM, or vice versa, breaks subsequent pipelines. Always verify format requirements of your alignment or analysis tools.
- **Forgetting gzip compression**: For large datasets, not using `--compressed` produces enormous uncompressed FASTQ files that consume massive storage and slow down file I/O in downstream steps.

## Examples

### Trim adapters from single-end FASTQ reads
**Args:** `--qualitythreshold 15 --minlength 30 --output trimmed_sample.fastq.gz`
**Explanation:** Removes adapters and bases with quality below 15 from single-end reads, keeping only reads at least 30 bases long and compressing output.

### Remove adapters from paired-end reads with default detection
**Args:** read1.fq.gz read2.fq.gz --output1 trimmed_R1.fq.gz --output2 trimmed_R2.fq.gz --qualitythreshold 20
**Explanation:** Detects and removes Illumina adapters automatically from both paired-end files using default detection, trimming low-quality bases with threshold 20.

### Trim adapters and merge overlapping paired-end reads
**Args:** --collapse --collapse--exact --output1 merged.fq.gz --output2 collapsed_R2.fq.gz
**Explanation:** Enables strict read merging for overlapping paired-end reads, creating consensus sequences where reads overlap, reducing file size and improving alignment accuracy.

### Custom adapter sequences with quality filtering
**Args:** --adapter1 AAGGGCCCTT --adapter2 TCCGGAATTC --qualitythreshold 25 --minquality 20 --minlength 50
**Explanation:** Specifies custom adapter sequences for both reads, discarding bases below quality 25, removing entire reads below quality 20, and keeping only reads 50+ bases.

### Convert FASTQ toSAM format with adapter removal
**Args:** sample1.fastq --tosam --output sample1.sam
**Explanation:** Outputs results in SAM format after trimming adapters, useful for pipelines requiring SAM input.

### Batch trimming multiple files
**Args:** --fastqinput "*.fastq.gz" --outputdir ./trimmed/ --compressed
**Explanation:** Processes all FASTQ files matching the pattern in the current directory, writing compressed output to the trimmed subdirectory.