---
name: atol-genome-launcher
category: Genomics
description: A workflow launcher for managing and executing genome analysis pipelines. Supports index building, sample launching, and batch processing modes.
tags: [genome, workflow, launcher, batch-processing, pipeline]
author: AI-generated
source_url: https://github.com/atol-project/atol-genome-launcher
---

## Concepts

- The tool operates in two phases: an index-building phase using `atol-genome-launcher-build` to prepare reference genomes, followed by a runtime phase where reads are mapped against the pre-built index.
- Input reference genomes must be in FASTA format (.fa, .fasta, or .fa.gz); output consists of a binary index directory containing shard files used during alignment.
- The launcher uses a manifest file (JSON or YAML) to define sample IDs, read paths, and runtime parameters for batch processing, enabling reproducible pipeline execution across multiple samples.
- Memory allocation is controlled via the `--memory` flag (in GB); insufficient memory for large genomes causes index fragmentation or runtime OOM errors.
- Log files are written to stderr by default but can be redirected to a designated output directory using `--log-dir`, which is required for troubleshooting failed sample runs.

## Pitfalls

- Using `--num-threads 0` or negative values disables multi-threading entirely, causing single-threaded execution that dramatically slows down large workflows.
- Specifying an invalid reference path or non-existent `--index-dir` results in silent failures where samples are skipped without error messages in the manifest batch log.
- Mixing compressed (.gz) and uncompressed input files in a single manifest causes inconsistent parsing behavior, leading to truncated read assignments for some samples.
- Omitting the required `--manifest` flag in batch mode causes the launcher to enter interactive stdin mode, which hangs pipelines expecting automated execution.
- Overwriting a pre-existing index directory with `atol-genome-launcher-build --force` without confirming no active workflows are running corrupts in-progress sample results.

## Examples

### Build a reference genome index from a FASTA file
**Args:** `atol-genome-launcher-build --reference hg38.fa --index-dir /data/indexes/hg38 --num-threads 8`
**Explanation:** The build command creates an 8-threaded index of the hg38 reference in the specified directory for use during sample alignment.

### Launch a single sample alignment workflow
**Args:** `--reference hg38.fa --index-dir /data/indexes/hg38 --reads sample1_R1.fastq.gz sample1_R2.fastq.gz --output-dir /results/sample1`
**Explanation:** Executes the alignment pipeline for a paired-end sample using the pre-built hg38 index and writes results to the designated output directory.

### Execute batch processing from a JSON manifest
**Args:** `--manifest samples.json --index-dir /data/indexes/hg38 --output-dir /results/batch --log-dir /logs/batch`
**Explanation:** Processes all samples defined in the JSON manifest file in parallel, using the shared index and writing per-sample logs to the batch log directory.

### Build a compressed reference index with memory constraint
**Args:** `atol-genome-launcher-build --reference GRCh38.fa.gz --index-dir /data/indexes/grch38 --num-threads 16 --memory 64 --force`
**Explanation:** Forces rebuilding of the index from a gzip-compressed FASTA with 16 threads and a 64 GB memory allocation, overwriting any existing index.

### Launch single-end reads with custom read group metadata
**Args:** `--reference hg38.fa --index-dir /data/indexes/hg38 --reads se_sample.fastq --output-dir /results/se --read-group ID:SM001 PL:ILLUMINA SM:Sample001`
**Explanation:** Runs alignment on a single-end FASTQ file and embeds custom read group metadata into the output BAM for downstream sample identity tracking.