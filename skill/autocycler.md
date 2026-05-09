---
name: autocycler
category: Oxford Nanopore Data Processing
description: A bioinformatics tool for processing and filtering Oxford Nanopore sequencing reads from SLOW5/FAST5 files, supporting read selection, basecalling metadata extraction, and quality-based filtering operations.
tags:
  - nanopore
  - oxford-nanopore
  - slow5
  - fast5
  - sequencing
  - quality-control
  - read-filtering
author: AI-generated
source_url: https://github.com/nanopore/slow5tools
---

## Concepts

- **Input formats:** autocycler processes Oxford Nanopore reads from SLOW5 files (binary format) or FAST5 files, extracting sequence data, quality scores, and basecalling metadata such as channel information and timestamps.

- **Read selection and filtering:** The tool supports filtering reads based on quality scores, read length thresholds, and strandedness (template/complement reads), enabling selective extraction of high-quality reads for downstream analysis.

- **Output formats:** Processed reads can be exported to FASTQ, SAM/BAM, or remain in SLOW5 format, with optional duplication or splitting by strand orientation for specialized pipelines like genome assembly.

- **Companion index builder:** autocycler-build creates indices for SLOW5 files to enable rapid random access, significantly reducing I/O time when processing large nanopore datasets.

- **Batch processing:** The tool handles multiple SLOW5 files concurrently through directory scanning, aggregating reads across runs while maintaining read group metadata for traceability.

## Pitfalls

- **Mismatched quality thresholds:** Setting the minimum quality (`--qscore`) too high may filter out valid reads that could contribute to assembly or variant calling, while setting it too low includes poor-quality reads that introduce errors; always validate threshold settings against downstream tool requirements.

- **Forgetting to build indices before processing:** Running autocycler on large SLOW5 files without pre-built indices causes excessive I/O overhead as the tool scans the entire file sequentially, easily adding hours to processing time for multi-GB datasets.

- **Incorrect strand handling in duplex reads:** When processing duplex reads, selecting the wrong strand orientation (`--strand`) discards half of the available sequence data, leading to reduced coverage and potentially biased results in applications like variant detection.

- **Output format incompatibility:** Generating output in BAM format without specifying appropriate headers (`--header`) results in files that downstream tools like samtools cannot properly parse, causing pipeline failures.

- **Ignoring read grouping metadata:** Disabling read group preservation (`--no-read-group`) loses critical sample provenance information, making it impossible to trace back mixed-sample datasets to their original source.

## Examples

### Filter reads by minimum quality score

**Args:** `--qscore 12 --output high_quality.slow5 input.slow5`

**Explanation:** This filters the input SLOW5 file to retain only reads with a minimum quality score of 12, outputting to a new SLOW5 file for downstream analysis with higher-confidence sequences.

### Convert SLOW5 to FASTQ format

**Args:** `--fastq --output reads.fastaq input.slow5`

**Explanation:** Converts nanopore reads from binary SLOW5 to plain-text FASTQ format for compatibility with standard bioinformatics tools like minimap2 or Flye assembler.

### Extract complementary strand reads only

**Args:** `--strand complement --output duplex_complement.slow5 input.slow5`

**Explanation:** Extracts only the complementary strand reads from duplex sequencing, useful when investigating modification detection or increasing per-base accuracy for variant calling.

### Build index for faster random access

**Args:** `input.slow5`

**Explanation:** Uses autocycler-build (the companion binary) to create an index file adjacent to the SLOW5 file, enabling rapid seeking to specific reads during iterative filtering operations.

### Filter by minimum read length

**Args:** `--minlen 1000 --output filtered_long.slow5 input.slow5`

**Explanation:** Retains only reads with minimum length of 1000 bases, removing short reads that often represent low-complexity regions or failed basecalls in nanopore sequencing data.

### Process all SLOW5 files in a directory

**Args:** `--directory ./run_data --output all_filtered.slow5`

**Explanation:** Recursively finds all SLOW5 files within the specified directory and processes them collectively, combining reads from multiple runs into a single output file.