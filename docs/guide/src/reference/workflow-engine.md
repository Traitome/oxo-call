# Workflow Engine

## Overview

oxo-call includes a native Rust workflow engine that executes `.oxo.toml` pipeline files. Unlike traditional workflow managers, it requires no external dependencies — no Snakemake, Nextflow, or Conda. Only the bioinformatics tools themselves need to be installed.

## Architecture

### DAG Execution

The engine builds a Directed Acyclic Graph (DAG) from step dependencies:

1. **Parse** — Load workflow definition from `.oxo.toml` (TOML format)
2. **Expand** — Expand wildcards (`{sample}`) across all step definitions
3. **Resolve** — Build explicit dependency edges between concrete tasks
4. **Phase** — Group tasks into execution phases (sets of independent tasks)
5. **Execute** — Run tasks with maximum parallelism via `tokio::task::JoinSet`
6. **Cache** — Track output freshness to skip completed steps automatically

### Wildcard System

- **`{sample}`** — Expands to each value in the `[wildcards]` section
- **`{params.key}`** — Substitutes values from the `[params]` section
- **Gather steps** — Steps with `gather = true` run once after ALL wildcard instances of their dependency steps complete

### Execution Phases

The engine automatically computes execution phases — groups of tasks that can run in parallel. Tasks within a phase have no mutual dependencies and execute concurrently.

Example for the RNA-seq template with 3 samples:

```
Phase 1: fastp[s1]  fastp[s2]  fastp[s3]          (3 tasks in parallel)
    ↓
Phase 2: star[s1]   star[s2]   star[s3]            (3 tasks in parallel)
    ↓
Phase 3: samtools_index[s1]  samtools_index[s2]  samtools_index[s3]
    ↓
Phase 4: featurecounts[s1]  featurecounts[s2]  featurecounts[s3]
    ↓
Phase 5: multiqc [gather]                          (1 task, aggregates all)
```

### Progress Display

During execution, the engine displays:

- **DAG phase diagram** — shows the pipeline structure with parallel groups
- **Step counter** — `[N/M]` progress indicator for each completed task
- **Status symbols** — `▶` running, `✓` success, `↷` skipped (up to date)
- **Elapsed time** — total wall-clock time at completion

### Output Freshness Caching

The engine automatically skips tasks whose outputs are already up to date:

- A task is **skipped** if all its outputs exist AND are newer than all its inputs
- A task **always runs** if any output is missing or any input is newer than the oldest output
- Tasks without declared outputs always run

### MultiQC Aggregation Pattern

All built-in templates follow a consistent pattern where **MultiQC is always the final step**:

1. MultiQC is configured as a `gather = true` step
2. It depends on the **leaf analysis steps** (the last per-sample analysis step in each branch)
3. This ensures all per-sample QC, alignment, and analysis outputs are complete before aggregation
4. The MultiQC command scans all relevant output directories with `--force` for consistent reruns

## File Format (.oxo.toml)

```toml
[workflow]
name        = "my-pipeline"
description = "Pipeline description"
version     = "1.0"

[wildcards]
sample = ["sample1", "sample2", "sample3"]

[params]
threads    = "8"
reference  = "/path/to/genome.fa"
gtf        = "/path/to/annotation.gtf"

[[step]]
name    = "qc"
cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz --out1 trimmed/{sample}_R1.fq.gz --out2 trimmed/{sample}_R2.fq.gz --json qc/{sample}_fastp.json"
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz", "qc/{sample}_fastp.json"]

[[step]]
name       = "align"
depends_on = ["qc"]
cmd        = "STAR --genomeDir {params.reference} --readFilesIn trimmed/{sample}_R1.fq.gz trimmed/{sample}_R2.fq.gz --outFileNamePrefix aligned/{sample}/"
inputs     = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]
outputs    = ["aligned/{sample}/Aligned.sortedByCoord.out.bam"]

[[step]]
name       = "aggregate"
gather     = true
depends_on = ["align"]
cmd        = "multiqc qc/ aligned/ -o results/multiqc/ --force"
outputs    = ["results/multiqc/multiqc_report.html"]
```

## Compatibility Export

### Snakemake

```bash
oxo-call workflow export rnaseq --to snakemake -o Snakefile
```

The generated Snakefile includes:
- `rule all` collecting leaf outputs
- Individual rules with `input`, `output`, `log`, and `shell` blocks
- `expand()` for wildcard substitution
- `configfile: "config.yaml"` with parameter template

### Nextflow (DSL2)

```bash
oxo-call workflow export wgs --to nextflow -o main.nf
```

The generated Nextflow file includes:
- `nextflow.enable.dsl = 2`
- Individual `process` blocks with `input`, `output`, and `script` sections
- `workflow` block chaining processes via channels
- Gather steps use `.collect()` for channel aggregation

## Built-in Templates

Use `oxo-call workflow list` to see all available templates. Each template provides:

- **Native** (`.oxo.toml`) — primary format for the built-in engine
- **Snakemake** (`.smk`) — hand-optimized Snakefile with container directives
- **Nextflow** (`.nf`) — DSL2 with process emit labels and channel operators

All templates include container image references for reproducible execution and follow bioinformatics best practices for tool parameter defaults.
