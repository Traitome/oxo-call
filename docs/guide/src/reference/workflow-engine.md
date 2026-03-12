# Workflow Engine

## Overview

oxo-call includes a native Rust workflow engine that executes `.oxo.toml` pipeline files. Unlike traditional workflow managers, it requires no external dependencies — no Snakemake, Nextflow, or Conda.

## Architecture

### DAG Execution

The engine builds a Directed Acyclic Graph (DAG) from step dependencies:

1. Parse workflow definition from `.oxo.toml`
2. Expand wildcards (`{sample}`, `{params.*}`)
3. Resolve dependencies between steps
4. Topological sort for correct execution order
5. Execute with `tokio::task::JoinSet` for parallelism
6. Track output freshness to skip completed steps

### Wildcard System

- **`{sample}`** — Expands to each sample name defined in the workflow
- **`{params.key}`** — Substitutes values from the `[params]` section
- **`{input}`** / **`{output}`** — References step I/O files

## File Format (.oxo.toml)

```toml
[workflow]
name = "my-pipeline"
samples = ["sample1", "sample2", "sample3"]

[params]
threads = 8
reference = "genome.fa"

[[steps]]
name = "qc"
tool = "fastp"
args = "-i {sample}_R1.fq -I {sample}_R2.fq -o {sample}_R1.clean.fq -O {sample}_R2.clean.fq"
output = "{sample}_R1.clean.fq"

[[steps]]
name = "align"
tool = "bwa"
args = "mem -t {params.threads} {params.reference} {sample}_R1.clean.fq {sample}_R2.clean.fq"
output = "{sample}.sam"
depends_on = ["qc"]
```

## Compatibility Export

### Snakemake

```bash
oxo-call workflow export rnaseq --format snakemake -o Snakefile
```

### Nextflow (DSL2)

```bash
oxo-call workflow export wgs --format nextflow -o main.nf
```

## Built-in Templates

Use `oxo-call workflow list` to see all available templates. Each template defines a complete analysis pipeline with proper step dependencies, sample expansion, and parameter substitution.
