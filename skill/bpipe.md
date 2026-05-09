---
name: bpipe
category: Pipeline Automation
description: A Groovy-based domain-specific language for defining and executing bioinformatics data processing pipelines with automatic file dependency tracking, incremental re-runs, and parallel stage execution.
tags: pipeline, workflow, groovy, bioinformatics, automation, parallel-execution, dependency-tracking
author: AI-generated
source_url: https://github.com/ssadedin/bpipe
---

## Concepts

- **Stage-based pipeline model**: Pipelines are composed of named stages that read input files matching glob patterns and produce output files. Each stage runs as an independent shell command block, and bpipe automatically tracks which files were produced to enable smart re-running when dependencies change.
- **Implicit and explicit file variables**: Inside stage blocks, input files matching the stage's `in` pattern are bound to a `reads` list variable, and output files should be assigned to `forward` or written to a location matching the `out` pattern so downstream stages can find them.
- **Branch and merge parallelism**: The `branch` directive splits a stage into parallel sub-pipelines, one per sample or branch item, and the results automatically merge when all branches complete. The `parallel` block explicitly runs multiple stages concurrently when they have no interdependencies.
- **Configuration and templates**: Global settings (threads, memory, environment variables) live in a `pipeline.config` closure or separate configuration file. Stage templates can be defined with `template` blocks to reuse common command patterns across multiple stages.
- **Incremental execution**: Bpipe stores a `.bpipe/db` directory tracking stage outputs and their inputs. When you re-run a pipeline, stages with unchanged inputs are skipped unless you use the `-r` or `--rerun` flag to force full re-execution.

## Pitfalls

- **Omitting the `out` file pattern**: Without a declared output pattern, bpipe cannot register what files the stage produced, breaking dependency tracking and causing downstream stages to fail with "no input files found" errors.
- **Naming collisions across stages**: If two different stages write to the same output filename, bpipe may use stale results from an unintended earlier stage, silently producing incorrect downstream output.
- **Unquoted shell variables in stage blocks**: Variables like `$sample` expand in the Groovy scope before the shell runs, not inside the shell command. Always wrap shell variables in single quotes or escape them properly so Groovy does not substitute them prematurely.
- **Misaligned input glob patterns**: If a stage's `in` pattern does not match any actual files, the stage receives an empty file list and fails at runtime. Patterns must exactly reflect the filenames produced by preceding stages.
- **Assuming `forward` persists across stages**: The `forward` variable holds only the current stage's outputs. It resets after each stage transition; if you need to carry data between non-consecutive stages, use a global variable or write an intermediate file.
- **Neglecting to check the `.bpipe/db` state**: When debugging why a stage skipped or re-ran unexpectedly, the database may contain stale entries. Deleting the database or using `--rerun` is necessary to force clean execution.

## Examples

### Run a simple two-stage FASTQ to BAM pipeline
**Args:** `run "pipeline.groovy"`
**Explanation:** Executes the pipeline defined in `pipeline.groovy`, which contains stages for aligning sequencing reads and converting the output to BAM format.

### Force re-running all stages from the beginning
**Args:** `-r`
**Explanation:** Discards cached results and re-runs every stage from the first stage in the pipeline definition, regardless of whether input files have changed.

### Specify a non-default pipeline configuration file
**Args:** `-c my_config.groovy run pipeline.groovy`
**Explanation:** Loads configuration settings from `my_config.groovy` instead of the embedded default, allowing per-project or per-environment tuning of threads, memory, and environment variables.

### Execute a single named stage for testing
**Args:** `-p stageName=align run pipeline.groovy`
**Explanation:** Overrides the pipeline to begin execution from the `align` stage alone, passing a synthetic input so you can test a stage without running the full upstream pipeline.

### List all stages that would run without executing
**Args:** `check pipeline.groovy`
**Explanation:** Parses the pipeline definition and reports which stages are defined and their order, without running any commands or modifying the database.

### Pass a sample variable to a stage using the samples directive
**Args:** `-s "samples.txt" run pipeline.groovy`
**Explanation:** Loads sample identifiers and associated file paths from `samples.txt` and makes the `samples` variable available within stage blocks so each sample gets its own branch in the pipeline.