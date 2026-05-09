---
name: atol-qc-raw-shortread
category: Quality Control
description: Quality control analysis tool for raw short-read sequencing data (FASTQ files). Computes per-base quality metrics, read length distributions, GC content statistics, adapter content, and overrepresented sequences to identify issues in sequencing data prior to downstream analysis.
tags:
  - bioinformatics
  - quality-control
  - short-read-sequencing
  - fastq
  - ngs
  - genomics
author: AI-generated
source_url: https://github.com/alequal/atol
---

## Concepts

- **Input Format**: Accepts raw FASTQ files (compressed `.fq.gz` or `.fastq.gz`, as well as uncompressed `.fq` or `.fastq`). The tool expects raw, unfiltered sequencing reads directly from the sequencer output. Reads should NOT be trimmed or quality-filtered prior to analysis.

- **Output Format**: Produces a multi-section quality report containing JSON summary statistics, HTML visualization reports, and text-based metric tables. Key metrics include per-base quality scores (Phred scale), read length distributions, GC content histograms, N content per position, and overrepresented k-mer identification.

- **Sequencing Metrics Tracked**: The tool calculates essential short-read QC metrics including mean/median quality by position, quality score distribution (boxplots), adapter content percentage, duplication rates, and sequence complexity scores. These metrics help identify pipeline failures, contamination, or low-quality samples.

- **Multi-Sample Support**: Can process multiple FASTQ files in a single run, either as a batch directory input or explicit file list. Reports are generated per-sample with optional aggregate summary comparisons across samples for cohort-level quality assessment.

## Pitfalls

- **Running on Pre-Processed Data**: Applying this tool to trimmed or quality-filtered FASTQ files produces misleading metrics that do not reflect the original sequencing quality. The tool's purpose is to assess raw data quality before any filtering steps; always use the raw, unprocessed FASTQ files.

- **Ignoring Warning Flags**: The tool exits with non-zero codes for quality warnings (e.g., excessive N bases, low quality scores below configurable thresholds). Ignoring these exit codes in automated pipelines leads to downstream analysis of degraded data with unreliable results.

- **Insufficient Disk Space**: QC reports with per-position metrics generate intermediate files proportional to read count and read length. Running on large datasets without checking available disk space results in incomplete reports or pipeline failure. Allocate at least 3× the input file size in free disk space.

- **Incorrect Encoding Specification**: Specifying the wrong quality encoding (Sanger vs. Illumina 1.8 vs. Illumina 1.3) produces systematically shifted quality scores. Always verify the sequencer's native encoding; Illumina NovaSeq and MiSeq output Illumina 1.8+ format by default.

- **Missing Reverse Complement Analysis**: For paired-end data, only analyzing forward reads misses adapter contamination in reverse reads. Always specify both read files when processing paired-end data to ensure complete adapter and quality assessment across both directions.

## Examples

### Generate a basic quality report for a single FASTQ file
**Args:** `--input sample_R1.fq.gz --output qc_report --format html`
**Explanation:** Runs quality control on a single forward read FASTQ file and generates an HTML report with visualizations of quality metrics, read length distributions, and GC content.

### Generate quality reports for multiple samples in a directory batch
**Args:** `--input /rawdata/sequencer_run/run123/ --output batch_report --format both`
**Explanation:** Processes all FASTQ files in the specified directory, generating individual reports for each sample plus an aggregate summary report comparing quality metrics across the batch.

### Analyze paired-end data with explicit forward and reverse files
**Args:** `--r1 sample_001_R1.fastq.gz --r2 sample_001_R2.fastq.gz --output paired_report --format json`
**Explanation:** Analyzes both read files of a paired-end sequencing library to assess quality, adapter content, and overrepresented sequences across both directions before read alignment.

### Generate a text summary with custom quality threshold warnings
**Args:** `--input sample.fq.gz --output summary --format text --min-quality 20 --max-n 0.05`
**Explanation:** Produces a condensed text summary report instead of full HTML, setting minimum quality threshold to Phred 20 and maximum allowable N content to 5% of bases.

### Export quality metrics to JSON for automated pipeline integration
**Args:** `--input run_data/*.fastq.gz --output metrics.json --format json --verbose`
**Explanation:** Processes multiple FASTQ files matching the wildcard pattern and outputs structured JSON metrics suitable for automated QC filtering pipelines, sample sheets, or dashboard ingestion.