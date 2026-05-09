---
name: cgatcore
category: Workflow Execution / Pipeline Management
description: cgatcore is the core execution engine for CGAT (Computational Genomics Analysis Toolkit) pipelines. It parses pipeline definitions, resolves parameter tables, manages task dependencies, and dispatches individual bioinformatics tasks to a local or HPC job scheduler (SGE, SLURM, PBS, or local mode). It reads configuration from YAML or INI files and produces structured analysis reports.
tags: [cgat, pipeline, workflow, task-scheduler, hpc, bioinformatics, yaml-config]
author: AI-generated
source_url: https://github.com/CSB5/cgat
---

## Concepts

- **Pipeline-driven architecture**: cgatcore does not run a single tool; it discovers a registered pipeline class (e.g., `pipeline_rnaseq`), loads its parameter table (`.params.tsv`), constructs a directed acyclic graph (DAG) of tasks, and executes tasks in dependency order. The pipeline name is passed as the first positional argument and maps to a Python class under `cgat.Pipeline.*`.
- **Parameter table format (TSV)**: Each pipeline expects a parameter file in `.params.tsv` format — a tab-separated table with columns `section`, `parameter`, `value`, and optionally `force`. Sections group parameters (e.g., `[data]`, `[clouds]`, `[rmark]`) and are referenced by pipeline tasks. A missing or malformatted `.params.tsv` causes silent parameter fallback to defaults, producing incorrect results.
- **Execution backends**: cgatcore supports `--local` mode (foreground single-process), `--jobs` with an SGE/SLURM/PBS cluster (background submission via `qsub`/`sbatch`), and `--max-jobs` to limit concurrency. In `--local` mode, stdout/stderr are multiplexed per task; on a cluster, individual task logs go to the working directory. Mixing `--jobs` and `--local` in the same run leads to undefined scheduler behaviour.
- **Report generation**: The `--report` flag triggers report-building using CGATReport, which walks the working directory for cached results (stored under `csvdb` SQLite databases and per-task output files) and generates HTML/Markdown summary documents. Reports are only complete if all upstream tasks succeeded; a failed task leaves stale caches that corrupt report links.
- **Working directory and paths**: cgatcore creates and operates within a configured working directory. Relative paths in `.params.tsv` are resolved from the working directory, not from the pipeline source directory. Absolute paths are respected. Moving or renaming the working directory after a run breaks `--report` and any subsequent `--rerun` attempts.

## Pitfalls

- **Forgetting --local or --jobs**: Running cgatcore without specifying `--local` or `--jobs` causes it to default to a cluster scheduler check. On a machine without scheduler access, tasks hang indefinitely with no error message until killed by the shell timeout. Always verify the intended execution context before launching.
- **Stale task caches after interruption**: If a run is killed (SIGINT, node failure, or walltime expiry), the `csvdb` cache and per-task marker files are left in an inconsistent state. A subsequent `cgatcore pipeline_name --local --rerun` will silently skip tasks marked as "done" even though their output is incomplete, producing corrupted downstream results. Always inspect `csvdb` with `SELECT * FROM task WHERE runid = 'latest'` or delete stale cache files before rerunning.
- **Incorrect .params.tsv section grouping**: Parameters tied to a specific task must appear under the correct `[section]` name. Placing a parameter under `[data]` when the task reads `[load]` causes the pipeline to silently use the default value instead of the intended one, because section-to-task matching is strict. Always cross-reference the parameter table against the pipeline source code's `getValue()` calls.
- **Path resolution surprises**: Relative paths in `.params.tsv` are resolved from the current working directory at the time of invocation, not from the location of the pipeline module. Running `cgatcore` from a different directory than expected yields "file not found" errors or, worse, silently processes the wrong input file if a file with the same name exists in the invocation directory.
- **Overwriting outputs with --force**: Using `--force` re-runs all tasks regardless of cache status, which is useful for reprocessing but destroys all per-task outputs and `csvdb` entries. There is no partial force option; downstream tasks that did not need recomputing are also reset. This is especially dangerous on shared working directories where collaborators rely on cached results.

## Examples

### Running a pipeline locally in foreground mode
**Args:** `pipeline_rnaseq --local --jobs 0`
**Explanation:** `--local` activates single-process foreground execution and `--jobs 0` signals that no cluster job submissions should occur, ensuring the entire pipeline runs synchronously in the current terminal session.

### Running a pipeline on a SLURM cluster with job submission
**Args:** `pipeline_rnaseq --jobs 200 --max-jobs 40`
**Explanation:** `--jobs 200` submits individual tasks as SLURM batch jobs (up to 200 queued) and `--max-jobs 40` caps the concurrent active jobs to 40, preventing overload of the cluster scheduler.

### Performing a dry-run to inspect the task DAG before execution
**Args:** `pipeline_rnaseq --local --dry-run`
**Explanation:** `--dry-run` prints the ordered list of tasks that would be executed without running any of them, revealing dependency chains and helping identify misconfigured parameters before time is wasted on a full run.

### Rerunning only failed or incomplete tasks after a partial failure
**Args:** `pipeline_rnaseq --local --jobs 0 --rerun`
**Explanation:** `--rerun` scans the `csvdb` cache for tasks marked as failed or incomplete (based on exit code records) and re-executes only those tasks and their downstream dependents, avoiding redundant recomputation of successful steps.

### Generating an HTML analysis report after a successful run
**Args:** `pipeline_rnaseq --makehtml`
**Explanation:** `--makehtml` triggers CGATReport to parse all `csvdb` SQLite caches and per-task outputs under the working directory and produce a navigable HTML report summarizing metrics, figures, and QC tables for downstream review.