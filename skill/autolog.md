---
name: autolog
category: Pipeline Automation / Logging
description: A bioinformatics tool for automatic logging and tracking of computational analysis runs, capturing metadata, tool versions, parameters, and runtime statistics for reproducibility and auditability.
tags: [logging, reproducibility, provenance, workflow, tracking, metadata, audit]
author: AI-generated
source_url: https://github.com/autolog-project/autolog
---

## Concepts

- **Automatic Metadata Capture**: autolog automatically records system information (OS, hostname, CPU, memory), software versions (via `--version` parsing), and timestamps without requiring explicit specification for each field.
- **Structured Output Formats**: Logs are written in multiple formats (JSON, YAML, TSV) to `--output` or `--log-file`, enabling integration with downstream databases, pipelines, or visualization tools.
- **Sample and Run Association**: Use `--sample-id` and `--project-id` to link log entries to specific samples or projects, enabling queryable provenance across thousands of runs.
- **Nested Parameter Recording**: Nested parameters (e.g., tool-specific sub-options) are captured using dot-notation (e.g., `bwa.mem.min-score`) and stored hierarchically in the output format.
- **Checkpoint and Resume Support**: The `--checkpoint` flag saves state at defined intervals, allowing pipeline resume without re-execution of completed steps.

## Pitfalls

- **Overwriting Existing Logs Without Warning**: Running autolog with the same `--sample-id` and `--run-id` without `--append` will silently overwrite existing log entries, losing historical provenance data.
- **Missing Tool Version Detection**: If a tool in the pipeline does not support `--version` or prints to stderr instead of stdout, version information will be missing; always verify captured versions in the output log.
- **Incomplete Parameter Recording for Dynamic Tools**: Tools that generate dynamic parameters at runtime (e.g., variant callers with adaptive thresholds) may not be fully captured unless explicitly passed via `--additional-params` file.
- **Permission Errors on Shared Output Directories**: Writing logs to a shared NFS or network path without proper permissions will cause silent failures; always verify write access before large-scale deployments.
- **Incompatible Output Formats for Downstream Parsers**: Using YAML output for large-scale runs may cause parsing overhead; prefer JSON or TSV for high-throughput integration pipelines.

## Examples

### Log a single-sample analysis run with automatic metadata capture
**Args:** `--sample-id SMPL001 --project-id PROJ123 --tool bwa --version-output "bwa mem" --output logs/SMPL001_run.json`
**Explanation:** Captures system metadata, tool version via `bwa mem`, and writes structured JSON log for downstream parsing.

### Append log entries to an existing run instead of overwriting
**Args:** `--sample-id SMPL001 --run-id RUN456 --append --output logs/SMPL001 cumulative.json`
**Explanation:** Appends new log entries to an existing cumulative file, preserving full run history without data loss.

### Record tool parameters from a config file rather than command line
**Args:** `--config params.yml --output logs/project_run.json --log-format json`
**Explanation:** Reads all parameters from a YAML config file, ensuring complete and reproducible parameter capture for complex tools.

### Enable checkpointing to allow pipeline resume after interruption
**Args:** `--checkpoint /tmp/autolog_checkpoint.bin --interval 300 --sample-id SMPL002`
**Explanation:** Saves state every 5 minutes to a binary checkpoint file, enabling resume without re-running completed pipeline stages.

### Configure output in YAML format for human readability
**Args:** `--sample-id SMPL003 --output logs/SMPL003.yaml --log-format yaml`
**Explanation:** Writes log in YAML format for easy manual inspection and debugging of run configurations.

### Track multiple samples in a batch with shared project metadata
**Args:** `--batch-samples sample_list.txt --project-id PROJ456 --output logs/batch_run.json --log-format json`
**Explanation:** Processes a text file of sample IDs in batch, applying shared project metadata to all entries for large-scale studies.

### Explicitly record additional metadata fields not captured automatically
**Args:** `--sample-id SMPL004 --extra-field "sequencer:IlluminaNovaSeq" --extra-field "libraryprep:StrandedmRNA" --output logs/SMPL004.json`
**Explanation:** Adds custom key-value metadata fields that are not auto-detected, useful for LIMS integration or experimental notes.