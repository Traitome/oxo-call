---
name: aastk
category: sequence-analysis
description: A toolkit for read alignment, filtering, and quality control of sequencing data with support for SAM/BAM formats.
tags:
- alignment
- sequencing
- quality-control
- sam-bam
- bioinformatics
author: AI-generated
source_url: https://github.com/aastk/aastk
---

## Concepts

- aastk processes sequencing reads in FASTQ format and produces alignments in SAM/BAM format, supporting both single-end and paired-end workflows with configurable alignment parameters.
- The core data model uses a reference genome index (created with aastk-build) and read input files, with alignment results stored as coordinate-sorted BAM files by default.
- The toolkit is organized as subcommands: build for index creation, align for read mapping, filter for alignment refinement, and qc for quality metric generation.
- Output formats vary by operation: aastk-align produces BAM/SAM, aastk-filter generates filtered subsets, and aastk-qc outputs JSON/TSV reports for downstream analysis.

## Pitfalls

- Running aastk-align without an existing index causes the command to fail with a reference not found error; always execute aastk-build prior to alignment operations.
- Specifying incorrect quality score encoding (phred33 vs phred64) corrupts base quality values in the output, leading to inflated or deflated quality scores throughout the BAM file.
- Specifying more threads than available system memory causes out-of-memory termination; begin with thread counts that fit within available RAM and scale incrementally.
- Overwriting existing output files without confirmation silently replaces previous results; use the --force flag only when intentional replacement is required.
- Filtering with conflicting parameters (e.g., --mapq threshold exceeding 60 with --include-multimaps) produces empty output files with no warning, wasting computation time.

## Examples

### Build a reference genome index for alignment
**Args:** build GRCh38.fa --index GRCh38 --threads 16
**Explanation:** Creating the index with 16 threads enables faster k-mer lookup during subsequent alignment operations against the human reference.

### Align single-end FASTQ reads to a reference genome
**Args:** align --input reads_R1.fastq.gz --index GRCh38 --output aligned.bam --phred33
**Explanation:** Specifying phred33 quality encoding ensures correct interpretation of quality scores for modern Illumina sequencing data.

### Align paired-end reads with proper mate pairing
**Args:** align --input reads_R1.fastq.gz reads_R2.fastq.gz --index GRCh38 --output paired.bam --paired --fr
**Explanation:** The --fr flag sets the correct orientation for Illumina paired-end libraries where R1 and R2 face outward toward the insert ends.

### Filter alignments to retain high-quality uniquely mapped reads
**Args:** filter aligned.bam --output high_quality.bam --mapq 30 --exclude-secondary
**Explanation:** Removing secondary alignments and reads with mapping quality below 30 ensures a clean set of confident mappings for variant calling.

### Generate alignment statistics report in JSON format
**Args:** qc aligned.bam --output qc_report.json --format json
**Explanation:** Exporting quality metrics as JSON enables programmatic parsing and integration into automated bioinformatics pipelines.