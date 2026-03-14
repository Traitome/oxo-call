---
name: fastqc
category: qc
description: Quality control analysis tool for high-throughput sequencing data producing HTML and zip reports
tags: [qc, quality-control, fastq, ngs, illumina, report, sequencing]
author: oxo-call built-in
source_url: "https://www.bioinformatics.babraham.ac.uk/projects/fastqc/"
---

## Concepts

- FastQC generates HTML and ZIP reports per FASTQ file; reports include per-base quality, sequence quality distribution, GC content, and adapter content.
- Use -o to specify output directory; FastQC creates <input_name>_fastqc.html and <input_name>_fastqc.zip in that directory.
- Use -t N for parallel processing of multiple files (one thread per file, not internal threading).
- FastQC accepts FASTQ (.fastq, .fq), gzipped FASTQ (.fastq.gz, .fq.gz), BAM, SAM, and BFAST formats.
- The --noextract flag keeps the zip file without extracting it (useful when only the HTML is needed).
- Key modules: Per Base Sequence Quality, Per Sequence Quality, Per Base N Content, Sequence Duplication, Adapter Content.
- FastQC works in batch mode — pass multiple files at once: fastqc file1.fq file2.fq -o qc_results/

## Pitfalls

- FastQC output goes to the same directory as the input by default — always use -o to control output location.
- FastQC is single-threaded per file; -t only helps when processing multiple files simultaneously.
- FastQC adapts thresholds for short reads but may give misleading WARN/FAIL on amplicon or very short-read data — interpret in context.
- FastQC does not trim reads — it only evaluates quality and generates reports.
- For BAM input, FastQC works on the sequences as stored — ensure BAM is not empty.
- Using --extract together with -o creates subdirectories — check that the output path has write permission.

## Examples

### run quality control on a single FASTQ file
**Args:** `reads.fastq.gz -o qc_results/`
**Explanation:** generates reads_fastqc.html and reads_fastqc.zip in qc_results/; create the output directory first

### run quality control on paired-end FASTQ files using 4 threads
**Args:** `-t 4 -o qc_results/ R1.fastq.gz R2.fastq.gz`
**Explanation:** -t 4 processes both files in parallel (2 files × 2 threads for I/O); output goes to qc_results/

### run quality control on multiple samples and keep zip files without extracting
**Args:** `--noextract -t 8 -o qc_output/ sample1_R1.fastq.gz sample1_R2.fastq.gz sample2_R1.fastq.gz sample2_R2.fastq.gz`
**Explanation:** --noextract keeps only HTML and zip; -t 8 processes up to 8 files in parallel

### run fastqc on a BAM file
**Args:** `-t 4 -o qc_results/ aligned.bam`
**Explanation:** FastQC can process BAM files; useful for QC of already-aligned data

### run fastqc with custom adapter sequences and format specification
**Args:** `-f fastq -a adapters.txt -t 4 -o qc_results/ reads.fastq.gz`
**Explanation:** -f specifies format; -a provides custom adapter sequences for adapter content module
