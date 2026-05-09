---
name: ashleys-qc
category: bioinformatics/qc
description: A comprehensive quality control analysis tool for high-throughput sequencing data, providing detailed reports on read quality, base composition, GC content, adapter contamination, and sequence complexity across FASTQ and BAM inputs.
tags:
  - qc
  - sequencing
  - fastq
  - bam
  - quality-control
  - bioinformatics
  - ngs
author: AI-generated
source_url: https://github.com/example/ashleys-qc
---

## Concepts

- **Input format handling**: ashleys-qc accepts both single-end and paired-end FASTQ files (compressed or uncompressed) as well as BAM/SAM alignment files. For paired-end data, provide both files sequentially—the tool automatically detects read relationships from filename patterns (e.g., `_R1_` and `_R2_` suffixes).

- **Module-based analysis architecture**: The tool runs independent analysis modules including Per Base Sequence Quality, Per Sequence Quality Scores, GC Content Distribution, Sequence Length Distribution, Adapter Content Detection, and K-mer Complexity. Each module produces a pass/warn/fail flag based on thresholds defined in the default or custom configuration.

- **Output report formats**: ashleys-qc generates three output types: an interactive HTML report (`qc_report.html`) containing static charts and summary tables, a machine-readable JSON summary (`summary.json`) with per-module status codes and statistics, and a plain-text log file (`qc_log.txt`) with detailed per-tile or per-cycle metrics for downstream processing.

- **Resource requirements**: The tool processes files in streaming mode to minimize memory footprint, requiring approximately 1 GB RAM for typical human WGS FASTQ files. Memory scaling is linear with unique k-mer storage when K-mer Complexity analysis is enabled; disable this module for samples exceeding 500 million reads to prevent excessive allocation.

- **Configuration inheritance**: Global default thresholds are defined in `~/.ashleys-qc/limits.default`. User-specified `--limits` files override defaults for specific organisms or sequencing technologies. When analyzing data from multiple sequencing runs, use the `--adapters` flag with a pre-built adapter library to ensure consistent contamination detection across batches.

## Pitfalls

- **Mismatched input specification for paired-end data**: Providing only one file of a paired-end pair causes the tool to analyze single reads as if single-end, producing inflated Per Sequence GC Content estimates and incorrect Duplication Level calculations. Always specify both files when working with paired reads.

- **Ignoring per-tile warnings on patterned flow cells**: If per-tile quality analysis flags warnings for specific tiles (e.g., tiles 21xx on Illumina NovaSeq), proceeding without investigation results in systematic coverage gaps near flow cell edges. Review the `--tile-filter` option and regenerate reports subsetting problematic tiles before downstream analysis.

- **Confusing pass/warn/fail status interpretation**: A "warn" status in any module does not invalidate the sample but indicates potential concerns requiring biological context. Treating all warnings as failures leads to unnecessary sample exclusion—review warn thresholds in the configuration and compare against sample-specific expectations (e.g., lower complexity expected for amplicon data).

- **Compressed input without proper extension**: ashleys-qc requires `.gz`, `.bz2`, or `.zip` extensions to auto-detect decompression. Files with non-standard compression markers but missing extensions are processed as raw text, producing corrupted base calls and false positive N content flags in the quality reports.

- **Overwriting existing reports without backup**: Running ashleys-qc on the same output directory without `--outdir` or `--no-overwrite` flag automatically replaces previous reports. When comparing pre- and post-processing QC (e.g., before and after trimming), use dated subdirectories to preserve historical reports for reproducibility.

## Examples

### Basic single-end FASTQ quality assessment

**Args:** `sample_R1.fastq.gz`

**Explanation:** This runs the complete QC analysis pipeline on a single compressed FASTQ file, producing default HTML and JSON outputs in the current directory, suitable for initial quality evaluation before downstream processing decisions.

### Paired-end sequencing analysis with custom output directory

**Args:** `sample_L001_R1_001.fastq.gz sample_L001_R2_001.fastq.gz --outdir ./qc_run_2024 --threads 8`

**Explanation:** Analyzing paired-end data with 8 parallel threads and directing all outputs to a dedicated directory prevents file clutter and enables easy comparison across multiple samples by organizing results per run.

### BAM file quality control with adapter specification

**Args:** `aligned.bam --format bam --adapters illumina_adapters.fa --no-kmer`

**Explanation:** When analyzing pre-aligned BAM files, specifying adapter sequences explicitly improves contamination detection accuracy, and disabling K-mer analysis reduces processing time and memory usage for alignment data where adapter sequences are typically already removed.

### Batch processing multiple samples with JSON output

**Args:** `*.fastq.gz --outdir batch_qc --format json --quiet --limits sensitive_limits.txt`

**Explanation:** Processing multiple files in batch mode with strict threshold configuration and JSON-only output enables automated high-throughput screening and integrates with sample tracking databases for large project quality monitoring workflows.

### Generating summary comparison across samples

**Args:** `sample1.fastq.gz sample2.fastq.gz sample3.fastq.gz --compare --outdir comparison_report`

**Explanation:** The comparison mode aggregates per-sample statistics into a single table, enabling rapid identification of outlier samples based on quality metrics, GC bias, and sequence complexity across a sample set.