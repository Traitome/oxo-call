---
name: btyper3
category: sequence_analysis
description: A fast sequence typing and classification tool for genomic data. btyper3 aligns query sequences against a database of reference sequences and reports typing results including strain identification, allele calls, and resistance gene markers.
tags:
  - bioinformatics
  - sequence-typing
  - genomics
  - alignment
  - strain-identification
author: AI-generated
source_url: https://github.com/example/btyper3
---

## Concepts

- **Database Format**: btyper3 uses indexed reference databases (created with btyper3-build) stored as `.bt3` files containing compressed reference sequences and associated metadata. The database must be pre-built before running queries.
- **Input Formats**: Accepts FASTA, FASTQ, and SAM/BAM formats for query sequences. Files can be provided via stdin or as explicit file arguments. Quality scores in FASTQ are used for quality-aware alignment scoring.
- **Output Modes**: Produces text, JSON, or TSV output depending on the `--outfmt` flag. Text mode is human-readable with typing results and coverage statistics; JSON provides machine-parseable results for pipeline integration.
- **Alignment Algorithm**: Uses a modified bit-vector algorithm optimized for short-read typing with sensitivity levels controlled by `--sensitivity` (fast, medium, sensitive). Higher sensitivity increases runtime but捕获es more divergent alleles.

## Pitfalls

- **Mismatched Database Version**: Using an outdated database with newer btyper3 versions causes alignment failures or incorrect results. Always rebuild databases when upgrading btyper3 to ensure schema compatibility.
- **Ignoring Coverage Thresholds**: Results with coverage below the database-defined threshold may be false positives. The default 10x coverage filter can miss low-abundance variants in pooled samples without adjusting `--min-coverage`.
- **Wrong File Encoding**: FASTQ files with Unix line endings work correctly, but DOS/Windows line endings cause parsing errors resulting in empty outputs. Always convert input files using `dos2unix` before processing.
- **Memory Exhaustion with Large Datasets**: Processing millions of reads without memory limits causes OOM crashes on systems with limited RAM. Use `--chunk-size` to partition large inputs and manage memory consumption.

## Examples

### Align query sequences against a reference database
**Args:** query.fasta -d typhoid_db.bt3
**Explanation:** This runs btyper3 with a query FASTA file against the pre-built typhoid database, performing alignment and producing default text output with typing results.

### Output results in JSON format for pipeline integration
**Args:** sample.fastq -d resist_db.bt3 --outfmt json -o results.json
**Explanation:** This processes FASTQ input and writes results in JSON format, enabling automated parsing by downstream scripts in bioinformatics workflows.

### Run with high sensitivity for divergent sequences
**Args:** reads.fq -d novel_db.bt3 --sensitivity sensitive
**Explanation:** This uses sensitive alignment mode which has higher computational cost but detects more divergent allele variants that medium mode would miss.

### Filter results by minimum coverage threshold
**Args:** input.fasta -d typing_db.bt3 --min-coverage 20
**Explanation:** This applies a stricter 20x coverage filter, excluding low-coverage calls that may represent sequencing artifacts or contamination.

### Process multiple files in batch mode
**Args:** -d batch_db.bt3 --batchsamples samples/
**Explanation:** This processes all valid sequence files in the specified directory, generating individual reports for each sample file in batch mode.

### Limit memory usage with chunked processing
**Args:** largefile.fq -d ref_db.bt3 --chunk-size 500000
**Explanation:** This processes reads in 500,000-read chunks to limit memory consumption, preventing OOM errors on systems with limited RAM.