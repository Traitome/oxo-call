---
name: bactopia-qc
category: bioinformatics/quality-control
description: A tool for performing quality control on bacterial whole genome sequencing data, part of the Bactopia pipeline. Provides integrated QC metrics including read quality, coverage estimation, and contamination screening for bacterial genomes.
tags: [bacterial-genomics, qc, fastq, quality-control, sequencing, bactopia]
author: AI-generated
source_url: https://github.com/bactopia/bactopia
---

## Concepts

- bactopia-qc processes paired-end FASTQ files to generate comprehensive quality control reports for bacterial WGS data, using tools like FastQC and fastp under the hood to assess read quality, per-base sequences, and GC content distribution.
- The tool requires either a directory containing FASTQ files (paired or single-end) or explicit input parameters specifying R1/R2 read files, and outputs HTML/PDF reports along with JSON summary data for integration into larger pipelines.
- bactopia-qc supports parallel execution for multiple samples through the `--samples` parameter, making it suitable for batch processing of bacterial isolates in high-throughput genomics studies.
- The tool automatically detects read type (single-end vs paired-end) based on input files and can apply adapters trimming using the `--trim` flag to remove low-quality bases and sequencing artifacts before QC evaluation.
- QC results are structured with standardized metrics including coverage estimation, average read depth, and species-specific contamination checks that are critical for downstream bacterial genome assembly.

## Pitfalls

- Running bactopia-qc without specifying an output directory with `--outdir` causes results to be written to the current working directory, potentially overwriting previous QC reports if filenames collide.
- Using the `--threads` parameter with values exceeding available CPU cores leads to resource contention and may cause the process to hang or crash, especially when running on shared compute nodes.
- Failing to provide proper adapter sequences with `--adapters` when processing library preps with non-standard indexing results in failed trimming and inflated QC metrics for the specified sequences.
- Specifying the wrong genome size with `--genome-size` leads to incorrect coverage calculations, which then propagates errors to all downstream estimates including assembly depth recommendations.
- Running on insufficient disk space causes partial output files that are difficult to detect, leading to incomplete QC reports that may be mistakenly trusted for downstream decisions.

## Examples

### Run basic QC on paired-end FASTQ files
**Args:** `--fastq R1.fastq.gz --fastq2 R2.fastq.gz --outdir qc_results`
**Explanation:** Processes paired-end reads with default parameters, outputting QC reports to the specified directory for standard bacterial WGS quality assessment.

### Run QC with adapters trimming enabled
**Args:** `--fastq R1.fastq.gz --fastq2 R2.fastq.gz --trim --outdir qc_trimmed`
**Explanation:** Enables built-in adapter trimming and quality filtering before QC calculation, recommended for libraries prepared with standard Illumina adapters.

### Batch process multiple samples from a directory
**Args:** --samples /path/to/fastq_dir --outdir batch_qc --threads 8
**Explanation:** Processes all FASTQ file pairs in the specified directory in parallel using 8 threads, efficient for processing multiple bacterial isolates simultaneously.

### Run QC with custom genome size for coverage estimation
**Args:** --fastq R1.fastq.gz --fastq2 R2.fastq.gz --genome-size 4.5m --outdir qc_results
**Explanation:** Specifies a 4.5 Megabase genome size for accurate coverage calculations, essential for accurate depth estimates in organisms with non-standard genome sizes.

### Run QC with detailed JSON output for pipeline integration
**Args:** --fastq R1.fastq.gz --fastq2 R2.fastq.gz --json --outdir qc_results
**Explanation:** Generates machine-readable JSON summary alongside standard reports, enabling automated parsing and integration into larger bacterial genomics workflows.

### Run QC with custom adapter sequences
**Args:** --fastq R1.fastq.gz --fastq2 R2.fastq.gz --adapters custom_adapters.fasta --outdir qc_results
**Explanation:** Uses custom adapter sequences from a FASTA file for trimming, necessary when processing non-standard library preps or custom indexing primers.