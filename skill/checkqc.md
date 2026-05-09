---
name: checkqc
description: A bioinformatics tool for assessing and reporting quality control metrics on sequencing FASTQ files. Analyzes per-base quality scores, sequence content, GC bias, and generates comprehensive QC reports.
category: quality_control
tags:
  - fastq
  - quality_control
  - sequencing
  - ngs
  - bioinformatics
author: AI-generated
source_url: https://github.com/checkqc/checkqc
---

## Concepts

- **Input Format**: checkqc accepts raw FASTQ files (both single-end and paired-end). Files can be specified directly as arguments or via a samplesheet configuration file defining multiple samples and their associated read files.
- **Quality Metrics Analyzed**: The tool computes per-base quality score distributions, mean/median quality per position, N content percentages, sequence length distributions, GC content per read, and adapter contamination detection.
- **Output Reports**: Results are generated in multiple formats including human-readable HTML reports, machine-parseable JSON summaries, and pass/fail status based on configurable thresholds defined in a run configuration file.
- **Threshold-based Pass/Fail**: Each QC metric is evaluated against user-defined thresholds (e.g., minimum mean quality, maximum N percentage). The tool exits with a non-zero status if any metric fails QC, enabling integration into automated pipelines.
- **Paired-end Sample Handling**: When processing paired-end data, checkqc correctly associates read1 and read2 files as a single sample and reports combined metrics across both files.

## Pitfalls

- **Mismatched File Pairs**: Specifying read1 and read2 files incorrectly (e.g., swapping file order or mixing files from different samples) leads to corrupted QC summaries that don't reflect actual sample quality, causing false pass/fail calls.
- **Missing Run Configuration**: Running checkqc without a defined run configuration file uses default thresholds that may be too lenient or too strict for your specific sequencing platform, resulting in unreliable QC verdicts.
- **Compressed File Extensions**: Failing to specify the correct file extension (e.g., `.fq.gz` versus `.fastq.gz`) when input files are gzipped causes the tool to fail to detect input files or read them incorrectly.
- **Insufficient Disk Space for Reports**: Not allocating sufficient disk space before generating large HTML reports for high-throughput runs can result in truncated or corrupted report files that are unreadable.
- **Ignoring Warnings in Non-strict Mode**: Treating warnings as non-critical when running in non-strict mode may cause overlooking early-stage quality degradation that progresses to sample failure in later sequencing cycles.

## Examples

### Single FASTQ file quality assessment
**Args:** `/data/sample1_R1.fastq.gz --run-config config.yaml`
**Explanation:** Runs quality assessment on a single FASTQ file using custom thresholds from the specified run configuration file, producing a QC report for read 1 of sample 1.

### Paired-end sample with explicit pair specification
**Args:** `--read1 /data/sample1_R1.fastq.gz --read2 /data/sample1_R2.fastq.gz --run-config config.yaml`
**Explanation:** Processes both reads of a paired-end sample together, calculating combined metrics and ensuring both reads pass QC before marking the sample as passing.

### Batch processing using samplesheet
**Args:** `--samplesheet samples.csv --run-config config.yaml`
**Explanation:** Processes multiple samples defined in a CSV samplesheet file, where each row contains sample ID and associated FASTQ files, enabling high-throughput batch QC analysis.

### Generate JSON output for automated pipeline integration
**Args:** `/data/sample1_R1.fastq.gz --run-config config.yaml --output-format json --output results.json`
**Explanation:** Generates machine-parseable JSON output instead of default HTML report, enabling automated pipeline decision-making based on QC pass/fail status.

### Override individual threshold values
**Args:** `/data/sample1_R1.fastq.gz --run-config config.yaml --min-mean-quality 30 --max-n-proportion 0.01`
**Explanation:** Runs QC with custom threshold overrides without modifying the run configuration file, specifying stricter mean quality and N content requirements for this specific run.