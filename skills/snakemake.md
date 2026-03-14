---
name: snakemake
category: workflow-manager
description: Snakemake Python-based workflow manager; rule-based pipelines for bioinformatics with support for conda, singularity, and HPC clusters
tags: [snakemake, workflow, pipeline, rules, wildcards, conda, singularity, slurm, bioinformatics]
author: oxo-call built-in
source_url: "https://snakemake.readthedocs.io/"
---

## Concepts
- Snakemake workflows are defined in a `Snakefile` (default) or any `.smk` file loaded with `--snakefile` (`-s`).
- **Workflow directory**: Snakemake looks for the Snakefile in the current directory or the directory given by `-s`; `workflow/Snakefile` is the modern convention.
- **Working directory**: Snakemake treats the directory containing the Snakefile as the working dir; all relative paths in rules are relative to it.
- **Metadata directory**: `.snakemake/` in the workflow working directory stores rule logs, conda envs, conda package caches, and job metadata.
- **Config file**: loaded with `--configfile config/config.yaml`; variables accessible via `config["key"]` inside rules.
- **Profile directory**: `~/.config/snakemake/<profile_name>/config.yaml` — define default flags (executor, cores, mem) here; activate with `--profile <name>`.
- Wildcards in rule outputs (`{sample}`, `{unit}`) expand via `expand()` in `all:` input and are resolved by Snakemake's DAG engine.
- **conda integration**: `rule: conda: "envs/tool.yaml"` creates and activates per-rule conda envs; use `--use-conda` to enable.
- **Singularity/containers**: `rule: container: "docker://biocontainers/samtools"` pulls and runs the container; enable with `--use-singularity`.
- HPC execution: `--executor slurm` (plugin) or `--cluster "sbatch ..."` (classic) submit rules as cluster jobs; `--jobs N` limits concurrent jobs.
- `--cores N` controls the number of local CPUs; `--jobs N` limits simultaneous cluster jobs; both can be used together.
- `--rerun-incomplete` re-runs rules that produced incomplete output files (e.g. from a crashed run).
- `--dry-run` (`-n`) shows what would be executed without running anything; pair with `-p` to also print shell commands.

## Pitfalls
- DANGER: `snakemake --delete-all-output` removes all output files defined in the Snakefile; irreversible if outputs are not backed up.
- Not specifying `--cores` causes Snakemake to use only 1 core; always set `--cores all` for local runs or `--jobs N` for cluster runs.
- Relative paths in rules are resolved relative to the Snakefile directory, not the working directory where `snakemake` is called; this causes confusion when calling from a different directory.
- `expand()` with multiple wildcards creates a Cartesian product; use `zip()` inside `expand()` to pair samples with their units instead.
- Conda env recreation: `.snakemake/conda/` caches built envs; delete this directory to force a rebuild if an env becomes corrupted.
- `--use-conda` without `--conda-frontend mamba` is slow for complex environments; add `--conda-frontend mamba` for faster dependency resolution.
- The `params:` section values are not tracked for re-running; change a shell command or output file to force re-execution.

## Examples

### run a workflow using all available cores
**Args:** `--cores all --use-conda`
**Explanation:** --cores all uses all CPUs; --use-conda creates per-rule conda environments as specified in rule conda: directives

### dry-run to see what would be executed
**Args:** `--dry-run --printshellcmds`
**Explanation:** -n/--dry-run shows the execution plan without running; -p/--printshellcmds prints the actual shell commands; safe for debugging

### run a workflow on a Slurm cluster
**Args:** `--executor slurm --jobs 50 --default-resources mem_mb=4096 runtime=60 --use-conda`
**Explanation:** --executor slurm requires snakemake-executor-plugin-slurm; --jobs caps concurrent cluster jobs; --default-resources sets per-job defaults

### run with a configuration file
**Args:** `--configfile config/config.yaml --cores 8`
**Explanation:** --configfile loads key-value pairs accessible as config["key"] in rules; overrides values in the Snakefile's configfile: directive

### use a named profile for cluster execution
**Args:** `--profile slurm`
**Explanation:** loads ~/.config/snakemake/slurm/config.yaml for default executor/jobs/resource settings; keeps the command short

### force re-run of specific rules
**Args:** `--forcerun trimming alignment --cores 16`
**Explanation:** --forcerun re-runs named rules and all their downstream dependencies regardless of output timestamps

### unlock a workflow after a crash
**Args:** `--unlock`
**Explanation:** removes the .snakemake/locks/ lock file left by a crashed run; required before restarting the workflow

### generate a rule dependency graph (DAG)
**Args:** `--dag | dot -Tpng > dag.png`
**Explanation:** --dag outputs a DOT-format rule graph; pipe to Graphviz dot to render a PNG; helps visualise complex pipeline structures

### clean up incomplete output files and restart
**Args:** `--rerun-incomplete --cores all`
**Explanation:** --rerun-incomplete detects and re-runs rules that left behind incomplete outputs from a previous failed run

### run with Singularity containers
**Args:** `--use-singularity --singularity-args '--bind /scratch' --cores 8`
**Explanation:** --use-singularity executes each rule inside the container specified by container: directive; --singularity-args passes bind mounts
