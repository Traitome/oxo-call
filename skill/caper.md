---
name: caper
category: Workflow Runner
description: A workflow management tool for running CWL and WDL bioinformatics pipelines on local and distributed computing environments. Caper handles resource allocation, job scheduling, output management, and provides debugging tools for workflow execution.
tags:
- workflow
- CWL
- WDL
- pipeline
- bioinformatics
- job-scheduler
- HPC
author: AI-generated
source_url: https://github.com/ENCODE-DCC/caper
---

## Concepts

- Caper executes workflows written in either Common Workflow Language (CWL) or Workflow Description Language (WDL) on backends including local (direct execution), SGE, PBS, SLURM, and cloud environments. The `--backend` flag selects the execution backend, and `--default-resources` defines baseline resource allocation for all tasks within a workflow.

- Input parameters are passed via a JSON or YAML file using the `--input` flag, where each key corresponds to an input identifier defined in the workflow document. Caper automatically localizes input files (copies remote URLs to the working directory) and handles glob patterns for output collection.

- Output files are written to a specified directory (default: `caper_out` in the working directory) with a structured naming scheme that preserves workflow and task information. Intermediate files can be configured for cleanup or retention based on `--keep-docker-cache` and `--keep-intermediate-files` settings.

- Resource requirements per task are specified using `cores`, `memory`, and `time` directives within the workflow file or through `--default-resources` and `--override-wf-defaults` CLI flags. These values directly influence the scheduler's resource allocation when running on distributed backends.

- Caper maintains a SQLite database (`caper_metadata.db`) tracking all workflow runs, including status, timestamps, input/output file locations, and runtime metrics. This database powers the `caper list` and `caper report` commands for monitoring and auditing pipeline executions.

## Pitfalls

- Specifying an incompatible backend for the HPC environment causes all tasks to fail with scheduler submission errors. For example, using `--backend sge` on a SLURM-managed cluster results in job rejection; always verify the cluster scheduler with `qstat` or `sinfo` before execution.

- Omitting required input parameters in the JSON/YAML input file produces silent failures where tasks complete without producing expected outputs. The workflow may appear to finish successfully but downstream steps receive empty files, corrupting downstream results.

- Underestimating memory requirements causes out-of-memory kills, particularly for Java-based tools (GATK, Picard) that require heap space matching the container's memory allocation. Always set `--default-resources memory` to a value exceeding the tool's base memory request by at least 20%.

- Output directory conflicts arise when running multiple workflows in the same working directory without specifying unique `--outdir` paths. Caper overwrites previous outputs with the same filename pattern, making it impossible to compare results across runs.

- Using relative paths for input files when running on distributed backends causes file-not-found errors because worker nodes cannot access the paths from the submit host. Always use absolute paths or ensure input files are accessible from all compute nodes.

## Examples

### Run a WDL workflow locally with default resources
**Args:** `run pipeline.wdl --input inputs.json --backend local`
**Explanation:** Executes the workflow file locally using direct process spawning, suitable for testing small workflows on a local machine before submitting to an HPC cluster.

### Run a CWL workflow on a SLURM cluster with custom resources
**Args:** `run workflow.cwl --input params.yaml --backend slurm --default-resources 'mem_mb=8192'` `--outdir slurm_output`
**Explanation:** Submits the workflow to a SLURM-managed cluster while overriding default memory allocation to 8GB per task, writing outputs to a dedicated directory for organization.

### Debug a workflow locally with verbose output
**Args:** `debug workflow.wdl --input inputs.json --verbose`
**Explanation:** Runs the workflow locally with detailed logging, capturing stdout/stderr from all tasks to help identify failures or unexpected behavior before cluster submission.

### List all completed workflows with metadata
**Args:** `list --field id --field start_time --field status --field outdir`
**Explanation:** Queries the internal SQLite database to display a tabular summary of workflow runs, filtering by specified columns for monitoring and audit purposes.

### Generate a HTML report for a specific workflow run
**Args:** `report workflow_id --format html --output run_report.html`
**Explanation:** Creates a self-contained HTML document containing execution statistics, task timings, resource usage, and file locations for sharing results with collaborators or including in publications.

### Abort a running workflow by ID
**Args:** `abort 12345678-1234-5678-1234-567812345678`
**Explanation:** Terminates all pending and running tasks associated with the specified workflow UUID, clearing SLURM/SGE job arrays and releasing allocated compute resources.

### Merge outputs from multiple workflow runs into a single directory
**Args:** `coalesce --dir output1 --dir output2 --dir output3 --outdir merged_results`
**Explanation:** Consolidates output files from separate workflow executions into a unified directory structure, simplifying downstream analysis that requires combining results from multiple runs.

### Initialize a new analysis workspace with recommended settings
**Args:** `init analysis_workspace --backend slurm --default-db-path ./workflow_db`
**Explanation:** Creates a workspace directory with configuration files and a local database, pre-configured for SLURM execution and ensuring all workflow metadata is stored within the project directory for version control.

### Override workflow-defined defaults for a specific task
**Args:** `run pipeline.wdl --input inputs.json --override-wf-defaults 'runtime.memory=16384'`
**Explanation:** Forces a specific task within the workflow to use 16GB of memory instead of its original specification, useful for adjusting resource-intensive steps identified during testing without modifying the workflow source file.

### Run workflow with custom resource monitoring database
**Args:** `run pipeline.wdl --input inputs.json --db database.sqlite --outdir custom_output`
**Explanation:** Stores all workflow metadata in a custom-named SQLite database rather than the default `caper_metadata.db`, enabling parallel workflow tracking or integration with external database systems.