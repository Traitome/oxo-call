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
- `-with-dag` generates a workflow diagram in various formats (DOT, HTML, SVG, PNG).
- `-with-trace` creates a detailed execution trace file with task metrics.
- `NXF_OPTS` sets JVM options for the Nextflow process; useful for memory tuning.

## Pitfalls
- deleting `work/` breaks `-resume` for all past runs; keep `work/` until you are sure you no longer need to resume.
- Not setting `NXF_SINGULARITY_CACHEDIR` on HPC causes Nextflow to pull Singularity images into `work/` repeatedly, wasting space and quota.
- Memory expressions: `'8.GB'` is a Nextflow `MemoryUnit`; `'8g'` is a string (for Java flags) — do NOT confuse them in `nextflow.config`.
- `process.executor = 'slurm'` alone is insufficient on some clusters; also set `clusterOptions`, `queue`, and `time` for proper job submission.
- Stale asset cache: if a pipeline runs an old version unexpectedly, run `nextflow pull nf-core/<pipeline> -revision main` to force update.
- DSL1 vs DSL2 syntax incompatibility: old community pipelines may use `process.output` channel syntax that fails in modern Nextflow; check `nextflow.enable.dsl` setting.
- Running `nextflow run` as root is not recommended; prefer a dedicated service account.
- `-with-dag` requires Graphviz (dot) to be installed for PNG/SVG output.
- `NXF_OPTS` values must be valid JVM options; invalid settings prevent Nextflow from starting.

## Examples

### run an nf-core pipeline with Singularity on a Slurm cluster
**Args:** `run nf-core/rnaseq -profile singularity,slurm --input samplesheet.csv --genome GRCh38 -resume`
**Explanation:** nextflow run subcommand; nf-core/rnaseq pipeline; -profile singularity,slurm activates profiles; --input samplesheet.csv samplesheet; --genome GRCh38 reference; -resume skips completed tasks

### run a pipeline with a custom work directory
**Args:** `run main.nf -work-dir /scratch/$USER/nxf-work`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -work-dir /scratch/$USER/nxf-work custom work directory

### pull a specific pipeline version from nf-core
**Args:** `pull nf-core/sarek -revision 3.4.0`
**Explanation:** nextflow pull subcommand; nf-core/sarek pipeline; -revision 3.4.0 pins to specific version; caches in ~/.nextflow/assets/

### resume a failed pipeline run
**Args:** `run main.nf -resume`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -resume reuses cached tasks from work/ directory

### run pipeline with a custom config file
**Args:** `run nf-core/chipseq -c custom.config --input samplesheet.csv`
**Explanation:** nextflow run subcommand; nf-core/chipseq pipeline; -c custom.config additional config; --input samplesheet.csv input samplesheet

### show the list of cached pipeline assets
**Args:** `list`
**Explanation:** nextflow list subcommand; lists all cached pipelines in ~/.nextflow/assets/ with revision and update time

### clean up work directory keeping only the last run's intermediate files
**Args:** `clean -but last`
**Explanation:** nextflow clean subcommand; -but last removes work/ entries except most recent run

### run a pipeline with Singularity image cache set
**Args:** `run nf-core/rnaseq -profile singularity --singularity.cacheDir /shared/singularity-cache --input samplesheet.csv`
**Explanation:** nextflow run subcommand; nf-core/rnaseq pipeline; -profile singularity; --singularity.cacheDir /shared/singularity-cache image cache path; --input samplesheet.csv samplesheet

### check Nextflow version and environment
**Args:** `-version`
**Explanation:** nextflow -version flag; prints Nextflow version, Java version, and home directory

### generate a run report and timeline
**Args:** `run main.nf -with-report report.html -with-timeline timeline.html`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -with-report report.html HTML execution report; -with-timeline timeline.html Gantt chart

### generate workflow DAG visualization
**Args:** `run main.nf -with-dag flowchart.png`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -with-dag flowchart.png workflow diagram output

### create detailed execution trace
**Args:** `run main.nf -with-trace trace.txt`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -with-trace trace.txt detailed task metrics output

### run with custom JVM options
**Args:** `NXF_OPTS="-Xms2g -Xmx8g" run main.nf`
**Explanation:** NXF_OPTS="-Xms2g -Xmx8g" JVM heap size; nextflow run subcommand; main.nf pipeline script

### run specific process only
**Args:** `run main.nf --step process_name -resume`
**Explanation:** nextflow run subcommand; main.nf pipeline script; --step process_name specific process; -resume skips completed tasks

### dry run to validate workflow without execution
**Args:** `run main.nf -preview`
**Explanation:** nextflow run subcommand; main.nf pipeline script; -preview validates workflow without execution
