---
name: abromics_galaxy_json_extractor
category: Bioinformatics / Data Extraction
description: A command-line tool for extracting metrics, statistics, and workflow information from Galaxy JSON output files. Parses execution reports from Galaxy workflow runs to extract tool inputs, outputs, runtimes, and quality metrics.
tags:
  - galaxy
  - json
  - bioinformatics
  - workflow
  - data extraction
  - metrics
  - ngs
author: AI-generated
source_url: https://github.com/abromics/abromics_galaxy_json_extractor
---

## Concepts

- **JSON Input Model**: The tool parses Galaxy workflow execution JSON files containing nested structures with tool invocations, input datasets, output datasets, parameter values, and runtime metrics. Each workflow step is represented as an object with unique identifiers, job states, and associated file paths.

- **Extraction Targets**: It can extract specific data types including tool names, input/output dataset paths, execution times (wall clock and CPU), parameter settings, error messages, and quality control metrics from sequencing pipelines (e.g., FASTQC, MultiQC reports embedded in JSON).

- **Output Formats**: Extracted data can be output in multiple formats including tab-delimited text, CSV, JSON structures, or summary reports. The tool supports filtering by workflow step, tool name, or dataset type to focus extraction on relevant records.

- **Companion Binary**: The suite includes `abromics_galaxy_json_extractor-build` for indexing multiple Galaxy JSON files into a searchable database or combined report, enabling comparative analysis across multiple workflow executions.

## Pitfalls

- **Invalid JSON Structure**: Providing a malformed or non-Galaxy JSON file will cause parsing failures. The tool requires specifically formatted Galaxy workflow history exports, not arbitrary JSON files—consequence: extraction fails with unclear error messages.

- **Missing Required Fields**: Some Galaxy JSON files may not include all fields (e.g., runtime metrics may be absent for failed jobs). The tool will skip missing fields silently, potentially leading to incomplete extractions without warning—consequence: downstream analysis may have gaps.

- **Encoding Issues with Unicode**: Galaxy JSON files containing non-ASCII characters in dataset names or tool parameters (e.g., Chinese, Arabic text in metadata) may cause output encoding problems if the terminal locale is not properly configured—consequence: garbled text in extracted reports.

- **Memory Usage with Large Files**: Processing very large Galaxy JSON files (e.g., from batched workflow runs with hundreds of datasets) can consume significant memory. The tool may become unresponsive or abort on systems with limited RAM—consequence: extraction never completes.

## Examples

### Extract all tool names from a Galaxy workflow run

**Args:** `input.json --select-tool-names`

**Explanation:** This extracts a list of all unique tool names that were executed in the workflow run, useful for generating a summary of the analysis pipeline used.

### Extract input and output dataset paths for a specific tool

**Args:** `input.json --tool-name "fastp" --output-format tsv`

**Explanation:** Returns tab-separated paths of FASTQ files processed by the fastp tool, including both input reads and output trimmed reads for quality control analysis.

### Extract runtime information for all workflow steps

**Args:** `input.json --select-runtimes --output-format csv`

**Explanation:** Exports execution times (wall clock and CPU time where available) for each workflow step into a CSV file suitable for performance analysis or bottleneck identification.

### Build a combined index from multiple Galaxy JSON files

**Args:** `abromics_galaxy_json_extractor-build --input-dir ./json_archives --output combined_index.db`

**Explanation:** The companion binary indexes multiple Galaxy JSON files from a directory into a SQLite database, enabling queries across all workflow runs for comparative analysis.

### Filter extraction to failed jobs only

**Args:** `input.json --job-state failed --select-error-messages`

**Explanation:** Extracts only error messages from failed workflow steps, making it easy to identify which tools or inputs caused workflow failures for debugging purposes.

### Extract quality metrics from a sequencing QC workflow

**Args:** `input.json --tool-name "fastqc" --select-metrics --output-format json`

**Explanation:** Extracts FASTQC quality metrics (read counts, per-base quality scores, sequence length distribution) embedded in the Galaxy JSON, outputting them as structured JSON for integration with other analysis pipelines.

### Generate a summary report of a completed workflow

**Args:** `input.json --summary-report --output workflow_summary.txt`

**Explanation:** Produces a human-readable text summary including workflow name, total duration, datasets produced, and any warnings or errors encountered during execution.