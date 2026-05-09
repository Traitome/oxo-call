---
name: bactopia-gather
category: Microbial Genomics
description: A tool to gather and combine individual Bactopia results from multiple samples into a unified dataset for downstream comparative genomics analysis. Collects assemblies, annotations, variants, and other outputs from completed Bactopia runs.
tags: [bactopia, ngs, wgs, bacterial-genomics, data-aggregation, variant-calling, comparative-genomics]
author: AI-generated
source_url: https://github.com/bactopia/bactopia
---

## Concepts

- **Input Directory Structure**: bactopia-gather recursively scans a parent directory containing multiple sample-specific Bactopia output folders (typically named by sample ID), each with standardized subdirectories for assemblies (GFA/FASTA), annotations (GBK/JSON), variants (VCF), and quality reports.
- **Sample Filtering Capabilities**: The tool supports include/exclude filters using glob patterns and sample lists, as well as quantitative filters on assembly quality metrics (e.g., minimum genome completion, maximum contamination) and coverage depth thresholds to curate which samples enter the combined dataset.
- **Output Formats**: Gathered data is organized into a consolidated directory with sample-named subfolders preserving original file extensions, plus a tab-delimited samplesheet (samples.txt) listing all included samples with their metadata, allowing compatibility with downstream Bactopia analysis modules like bactopia-dereplicate-genomes and bacter-combine-snps.
- **Lazy Evaluation Mode**: When running in `--lazy` mode, the tool creates symbolic links instead of copying files to save disk space and execution time; this is useful when the original Bactopia outputs remain accessible on the same filesystem.
- **Version Compatibility**: The tool verifies that all input Bactopia datasets were generated with compatible pipeline versions before merging; mismatched versions trigger warnings or errors to prevent analysis artifacts from mixed software versions.

## Pitfalls

- **Omitting the `--output-dir` Flag**: Without specifying an output directory, the gathered dataset may be written to the current working directory, causing filesystem clutter and potential overwrites of existing files; always explicitly define `--output-dir` to a clean, dedicated location.
- **Using Conflicting Filter Combinations**: Applying both `--include` and `--exclude` flags with overlapping sample patterns can yield empty results without warning; review which samples pass filters using `--dry-run` before executing the full gather operation.
- **Ignoring Disk Space Requirements**: When not using `--lazy` mode, bactopia-gather copies all selected files which can consume substantial disk space for large cohorts; a 100-sample dataset with 500MB average assembly files easily requires 50GB+ of free space.
- **Failing to Check Sample Name Collisions**: If multiple input directories contain samples with identical names but different origins, the tool may silently overwrite or mix data; use `--prefix` to add unique identifiers to disambiguate sample sources.
- **Running Without Sufficient Permissions**: The tool requires read access to all input Bactopia directories and write access to the output directory; permission errors mid-process can leave partial output that corrupts subsequent analyses.

## Examples

### Gather all Bactopia results from a parent directory
**Args:** `--bactopia-dir /data/bactopia-runs --output-dir /data/combined-dataset`
**Explanation:** Recursively finds all sample subdirectories within the parent Bactopia output directory and copies their complete datasets into a unified combined directory for pan-genome or population-level analysis.

### Filter samples by minimum coverage threshold
**Args:** `--bactopia-dir /data/cohort-run --output-dir /data/high-quality --min-coverage 30`
**Explanation:** Only includes samples with overall genome coverage of at least 30x, filtering out low-quality datasets that could introduce noise into downstream variant calling or phylogenetic analysis.

### Include only specific samples using a list file
**Args:** `--bactopia-dir /data/bactopia-runs --output-dir /data/subset --samples sample-list.txt`
**Explanation:** Reads the sample IDs from a line-separated text file and gathers only those matching samples, useful for focusing analysis on clinically relevant or study-specific subsets.

### Create symbolic links to save disk space
**Args:** `--bactopia-dir /data/all-runs --output-dir /data/lazy-gathered --lazy`
**Explanation:** Uses symlinks instead of copying files when building the combined dataset, dramatically reducing execution time and disk usage when the original files remain accessible.

### Add unique prefix to disambiguate sample sources
**Args:** `--bactopia-dir /data/hospital-a --output-dir /data/merged --prefix hospitalA_`
**Explanation:** Prepends "hospitalA_" to all sample names from this input directory, preventing name collisions when merging datasets from multiple origins in multi-center studies.