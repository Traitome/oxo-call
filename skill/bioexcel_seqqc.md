---
name: bioexcel_seqqc
category: Quality Control
description: A high-throughput sequencing data quality control tool that analyzes FASTQ files and generates detailed quality reports including base quality scores, GC content distribution, sequence length metrics, overrepresented sequences, and adapter content detection. Designed for parallel processing of large genomic datasets.
tags:
  - sequencing
  - quality-control
  - fastq
  - genomics
  - bioinformatics
  - ngs
author: AI-generated
source_url: https://github.com/bioexcel/seqqc
---

## Concepts

- **Input Format**: BioExcel SEqqc processes standard FASTQ files (with optional gzipped compression) containing raw or trimmed sequencing reads. The tool automatically detects quality encoding schemes (Sanger, Illumina 1.8+, and legacy Illumina formats) based on quality score ranges in the data.

- **Output Reports**: The tool generates per-sample quality reports in both HTML (interactive web-based) and JSON (machine-readable) formats. Reports include: per-base quality boxplots, GC content bias plots, sequence length distribution histograms, overrepresented k-mer lists with genomic origin annotations, and adapter/contamination screening results.

- **Parallel Processing Model**: BioExcel SEqqc supports multi-threaded execution via the `-t/--threads` flag, allowing simultaneous analysis of multiple FASTQ files or chunked processing of single large files. Thread count should match available CPU cores for optimal throughput on high-performance computing environments.

- **Quality Threshold Defaults**: The tool uses configurable quality thresholds: bases below Q20 (Phred score 20) are flagged as low-quality, samples with >30% overall low-quality bases trigger a warning, and overrepresented sequences appearing in >0.1% of reads are reported.

## Pitfalls

- **Mismatched Thread Configuration**: Setting `-t/--threads` higher than available CPU cores causes thread contention and segfaults, especially on shared HPC nodes. Always verify core availability with `nproc` or `cat /proc/cpuinfo` before execution.

- **Ignoring Adapter Content Warnings**: Failing to supply known adapter sequences via `-a/--adapter-file` results in undetected contamination, leading to false overrepresentation calls and compromised downstream assembly or mapping results.

- **Output Directory Overwriting**: Running analysis with the same `-o/--outdir` path multiple times overwrites existing reports without warning. Previous per-sample data is irretrievably lost after subsequent runs.

- **Mixed Quality Encoding in Input**: Feeding a single FASTQ file with inconsistent quality score encodings (e.g., mixed Illumina 1.3 and Illumina 1.8+) produces misleading per-base quality visualizations and incorrect QC pass/fail determinations.

- **Insufficient Disk Space for Large Genomes**: For reference-sized genomes (>3 GB), the temporary SQLite database created during k-mer analysis requires 2-3× the input file size on disk. Running without checking available space causes incomplete k-mer tables and missing overrepresentation annotations.

## Examples

### Run standard QC analysis on a single FASTQ file
**Args:** `-i sample_R1.fastq.gz -o seqqc_report`
**Explanation:** This executes a complete quality control analysis on the input FASTQ file, producing HTML and JSON reports in the specified output directory with default thresholds and single-threaded processing.

### Analyze paired-end reads with multi-threaded execution
**Args:** `-i left.fastq.gz -i right.fastq.gz -o paired_qc -t 8`
**Explanation:** Enables parallel processing of both read files across 8 threads for faster analysis of typical paired-end Illumina datasets, generating side-by-side comparison reports.

### Supply custom adapter sequences for contamination screening
**Args:** `-i raw_reads.fastq -o adapter_check -a TruSeq_adapters.fa`
**Explanation:** Triggers thorough adapter detection by providing the known adapter sequence file, preventing false overrepresentation flags from adapter-derived k-mers.

### Generate machine-readable JSON output for automated pipelines
**Args:** `-i reads.fastq -o json_report --format json --quiet`
**Explanation:** Produces only JSON-formatted metrics suitable for integration into automated quality control pipelines, suppressing console output and HTML report generation.

### Run with relaxed quality thresholds for highly fragmented input
**Args:** `-i lowqual_reads.fastq -o lenient_qc --min-quality 15 --max-failed-bases 50000`
**Explanation:** Adjusts quality thresholds to accommodate degraded or highly fragmented input libraries where default Q20 filtering would incorrectly flag acceptable reads as failures.

### Analyze multiple files in batch mode using a sample manifest
**Args:** `-m sample_manifest.tsv -o batch_qc -t 16`
**Explanation:** Reads sample configuration from a tab-delimited manifest file containing multiple FASTQ paths and metadata, executing parallel analysis across 16 threads for high-throughput cohort processing.

### Export k-mer overrepresentation data with genomic origin mapping
**Args:** `-i input.fastq -o kmer_analysis --kmer-size 5 --kmer-depth 50`
**Explanation:** Performs detailed 5-mer frequency analysis with mapped genomic origin annotations for contaminant or adapter source identification, reporting only k-mers appearing above 50× expected depth.