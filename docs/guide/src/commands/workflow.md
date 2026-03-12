# workflow

Native workflow engine with Snakemake and Nextflow compatibility export.

## Synopsis

```
oxo-call workflow run      <FILE|TEMPLATE>
oxo-call workflow dry-run  <FILE|TEMPLATE>
oxo-call workflow export   <FILE|TEMPLATE> --to <snakemake|nextflow> [-o <FILE>]
oxo-call workflow generate <TASK> [--engine native|snakemake|nextflow] [-o <FILE>]
oxo-call workflow infer    <TASK> --data <DIR> [--engine native|snakemake|nextflow] [-o <FILE>] [--run]
oxo-call workflow list
oxo-call workflow show     <TEMPLATE> [--engine native|snakemake|nextflow]
```

## Description

The `workflow` command (alias: `wf`) provides a lightweight native Rust workflow engine that executes `.oxo.toml` pipeline files directly — no Snakemake, Nextflow, or Conda required. Snakemake and Nextflow are supported as compatibility export targets.

## Native Engine Features

- **DAG-based execution** — steps run in dependency order with maximum parallelism via `tokio::task::JoinSet`
- **Wildcard expansion** — `{sample}` automatically expands per sample; `{params.KEY}` for shared parameters
- **Output caching** — steps whose outputs are newer than their inputs are automatically skipped
- **Parallel execution** — independent steps within the same DAG phase run concurrently
- **Gather steps** — steps with `gather = true` run once after all wildcard instances of their dependency steps complete (e.g., MultiQC aggregation)
- **Progress display** — step counter `[N/M]`, elapsed time, and DAG phase visualization
- **Cycle detection** — dependency cycles are detected and reported as errors

## Subcommands

### `workflow run`

Execute a workflow file or built-in template:

```bash
oxo-call workflow run pipeline.oxo.toml
oxo-call workflow run rnaseq              # Run a built-in template directly
```

### `workflow dry-run`

Preview the expanded task graph without executing any commands:

```bash
oxo-call workflow dry-run pipeline.oxo.toml
oxo-call workflow dry-run rnaseq
```

The dry-run shows the DAG phase diagram, step-by-step expansion with wildcard bindings, commands, dependencies, inputs, and outputs.

### `workflow export`

Export a native `.oxo.toml` workflow to Snakemake or Nextflow format:

```bash
oxo-call workflow export rnaseq --to snakemake -o Snakefile
oxo-call workflow export wgs --to nextflow -o main.nf
```

### `workflow generate`

Generate a workflow from a natural-language description using the configured LLM:

```bash
oxo-call workflow generate "RNA-seq analysis of mouse samples"
oxo-call workflow generate "Variant calling from WGS data" --engine snakemake -o Snakefile
```

### `workflow infer`

Detect data files in a directory and generate an appropriate workflow:

```bash
oxo-call workflow infer "RNA-seq QC and alignment" --data ./fastq_data/
oxo-call workflow infer "16S analysis" --data ./amplicon_reads/ --run  # Generate and run
```

### `workflow list`

List all available built-in templates:

```bash
oxo-call workflow list
```

### `workflow show`

Display a built-in template in different formats:

```bash
oxo-call workflow show rnaseq
oxo-call workflow show wgs --engine snakemake
oxo-call workflow show metagenomics --engine nextflow
```

## Built-in Templates

| Template | Domain | Pipeline Steps |
|----------|--------|----------------|
| `rnaseq` | Transcriptomics | fastp → STAR → samtools index → featureCounts → MultiQC |
| `wgs` | Genomics | fastp → BWA-MEM2 → MarkDuplicates → BQSR → HaplotypeCaller → MultiQC |
| `atacseq` | Epigenomics | fastp → Bowtie2 → Picard dedup → blacklist filter → MACS3 → MultiQC |
| `chipseq` | Epigenomics | fastp → Bowtie2 → MarkDup → filter → MACS3 + bigWig → MultiQC |
| `metagenomics` | Metagenomics | fastp → host removal → Kraken2 → Bracken → MultiQC |
| `amplicon16s` | Metagenomics | cutadapt → fastp → DADA2 (gather) → MultiQC |
| `scrnaseq` | Single-cell | fastp → STARsolo (10x v3) → samtools index + cell QC → MultiQC |
| `longreads` | Genomics | NanoQ → NanoStat + Flye (parallel) → Medaka → QUAST → MultiQC |
| `methylseq` | Epigenomics | Trim Galore → Bismark → dedup → sort → methylation extract → bedGraph → MultiQC |

### MultiQC Aggregation

All templates follow a consistent pattern: **MultiQC always runs as the final step** after all per-sample analysis steps complete. It is configured as a `gather` step that depends on all leaf analysis steps, ensuring:

1. All QC, alignment, and analysis outputs are available before aggregation
2. MultiQC scans all relevant output directories (qc/, aligned/, peaks/, etc.)
3. A single comprehensive report is generated across all samples

## .oxo.toml Format

```toml
[workflow]
name        = "my-pipeline"
description = "Pipeline description"
version     = "1.0"

# Wildcards: {sample} expands for each value
[wildcards]
sample = ["sample1", "sample2", "sample3"]

# Parameters: accessible as {params.KEY} in step commands
[params]
threads    = "8"
reference  = "/path/to/genome.fa"

# Steps: each [[step]] runs for every wildcard combination
[[step]]
name    = "align"
cmd     = "bwa mem -t {params.threads} {params.reference} data/{sample}_R1.fq.gz data/{sample}_R2.fq.gz | samtools sort -o aligned/{sample}.bam"
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["aligned/{sample}.bam"]

[[step]]
name       = "index"
depends_on = ["align"]
cmd        = "samtools index aligned/{sample}.bam"
inputs     = ["aligned/{sample}.bam"]
outputs    = ["aligned/{sample}.bam.bai"]

# Gather step: runs ONCE after all wildcard instances of deps complete
[[step]]
name       = "multiqc"
gather     = true
depends_on = ["index"]
cmd        = "multiqc aligned/ -o results/multiqc/ --force"
outputs    = ["results/multiqc/multiqc_report.html"]
```

### Step Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique step identifier (used in `depends_on`) |
| `cmd` | string | Shell command with `{wildcard}` and `{params.KEY}` substitution |
| `depends_on` | list | Names of steps that must complete first |
| `inputs` | list | Input file patterns for freshness checking |
| `outputs` | list | Output file patterns for freshness checking and skip-if-fresh logic |
| `gather` | bool | When `true`, runs once after ALL wildcard instances of dependency steps complete |

## Progress Display

During execution, the engine shows:

```
◆ oxo workflow — 13 task(s)
────────────────────────────────────────────────────────────────

  Pipeline DAG (5 phases, 13 tasks)

    Phase 1  fastp[sample=s1]  │  fastp[sample=s2]  │  fastp[sample=s3]
             ↓
    Phase 2  star[sample=s1]  │  star[sample=s2]  │  star[sample=s3]
             ↓
    Phase 3  samtools_index[sample=s1]  │  … +2 more
             ↓
    Phase 4  featurecounts[sample=s1]  │  … +2 more
             ↓
    Phase 5  multiqc [gather]

────────────────────────────────────────────────────────────────
  ▶ fastp[sample=s1]
  ✓ [1/13] fastp[sample=s1]
  ▶ fastp[sample=s2]
  ✓ [2/13] fastp[sample=s2]
  ...
  ✓ [13/13] multiqc
────────────────────────────────────────────────────────────────

✓ Workflow complete — 13 task(s) run, 0 up to date  (2m 34s)
```
