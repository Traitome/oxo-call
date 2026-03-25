---
name: nextflow
category: workflow-manager
description: Nextflow dataflow workflow manager; DSL2 pipeline language for bioinformatics, supports HPC schedulers, cloud, and containers
tags: [nextflow, dsl2, pipeline, nf-core, workflow, slurm, aws, docker, singularity, work-dir]
author: oxo-call built-in
source_url: "https://www.nextflow.io/docs/latest/"
---

## Concepts
- Nextflow workflows are written in DSL2 (default since 22.03); enable with `nextflow.enable.dsl=2` (legacy) or omit in modern versions.
- **Work directory**: `./work/` by default; stores all intermediate files; can be overridden with `-work-dir /path/to/work` or the `NXF_WORK` env var.
- **Nextflow home**: `~/.nextflow/` stores the asset cache (downloaded pipelines), executors, and config; override with `NXF_HOME`.
- **Global config**: `~/.nextflow/config` — loaded for every run; set default executor, CPUs, memory, and profiles here.
- **Project config**: `nextflow.config` in the pipeline directory — overrides global config; also loads `conf/` subdirectory configs.
- **Profiles** (`-profile`): named groups of config settings (e.g. `docker`, `singularity`, `slurm`); define in `nextflow.config` under `profiles { }`.
- **nf-core**: community pipeline collection at `nf-co.re`; run with `nextflow run nf-core/<pipeline> -profile singularity,slurm`.
- `-resume` flag reuses cached results from previous runs (stored in `work/`); only re-runs tasks whose inputs changed.
- Process labels in `nextflow.config` assign resources: `process { withLabel:'big_mem' { memory = '64.GB'; cpus = 16 } }`.
- Executors: `local` (default), `slurm`, `pbs`, `sge`, `lsf`, `aws`, `google-cloud-batch`, `k8s`; set with `process.executor`.
- `NXF_SINGULARITY_CACHEDIR` — directory where Nextflow caches pulled Singularity images; critical to set on HPC shared filesystems.
- Log file: `.nextflow.log` in the launch directory (most recent); `.nextflow.log.1`, `.nextflow.log.2` for previous runs.
- Pipeline assets cache: `~/.nextflow/assets/<org>/<pipeline>/` — cloned or updated with `nextflow pull` or on first run.

## Pitfalls
- deleting `work/` breaks `-resume` for all past runs; keep `work/` until you are sure you no longer need to resume.
- Not setting `NXF_SINGULARITY_CACHEDIR` on HPC causes Nextflow to pull Singularity images into `work/` repeatedly, wasting space and quota.
- Memory expressions: `'8.GB'` is a Nextflow `MemoryUnit`; `'8g'` is a string (for Java flags) — do NOT confuse them in `nextflow.config`.
- `process.executor = 'slurm'` alone is insufficient on some clusters; also set `clusterOptions`, `queue`, and `time` for proper job submission.
- Stale asset cache: if a pipeline runs an old version unexpectedly, run `nextflow pull nf-core/<pipeline> -revision main` to force update.
- DSL1 vs DSL2 syntax incompatibility: old community pipelines may use `process.output` channel syntax that fails in modern Nextflow; check `nextflow.enable.dsl` setting.
- Running `nextflow run` as root is not recommended; prefer a dedicated service account.

## Examples

### run an nf-core pipeline with Singularity on a Slurm cluster
**Args:** `run nf-core/rnaseq -profile singularity,slurm --input samplesheet.csv --genome GRCh38 -resume`
**Explanation:** -profile activates singularity+slurm config; --input is the nf-core samplesheet; --genome selects reference; -resume skips completed tasks

### run a pipeline with a custom work directory
**Args:** `run main.nf -work-dir /scratch/$USER/nxf-work`
**Explanation:** -work-dir overrides ./work/; use scratch storage on HPC to avoid filling home quota; -resume still works with this path

### pull a specific pipeline version from nf-core
**Args:** `pull nf-core/sarek -revision 3.4.0`
**Explanation:** downloads and caches the pipeline in ~/.nextflow/assets/; -revision pins to a specific tag or branch

### resume a failed pipeline run
**Args:** `run main.nf -resume`
**Explanation:** reuses cached tasks from the most recent run's work/ directory; only re-runs tasks whose inputs or code changed

### run pipeline with a custom config file
**Args:** `run nf-core/chipseq -c custom.config --input samplesheet.csv`
**Explanation:** -c loads an additional config file on top of nextflow.config; useful for site-specific cluster settings without modifying the pipeline

### show the list of cached pipeline assets
**Args:** `list`
**Explanation:** lists all pipelines stored in ~/.nextflow/assets/; includes the revision (branch/tag) and last update time

### clean up work directory keeping only the last run's intermediate files
**Args:** `clean -but last`
**Explanation:** removes work/ entries from all runs except the most recent; frees disk while preserving the ability to resume the latest run

### run a pipeline with Singularity image cache set
**Args:** `run nf-core/rnaseq -profile singularity --singularity.cacheDir /shared/singularity-cache --input samplesheet.csv`
**Explanation:** --singularity.cacheDir prevents re-pulling images; use a shared HPC path so all users benefit from cached images

### check Nextflow version and environment
**Args:** `-version`
**Explanation:** prints Nextflow version, Java version, and home directory; use before reporting issues or upgrading

### generate a run report and timeline
**Args:** `run main.nf -with-report report.html -with-timeline timeline.html`
**Explanation:** -with-report creates an HTML execution report with resource usage; -with-timeline shows a Gantt chart of task execution order
