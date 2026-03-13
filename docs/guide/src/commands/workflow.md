# workflow

Native workflow engine with Snakemake and Nextflow compatibility export.

## Synopsis

```
oxo-call workflow run      <FILE|TEMPLATE>
oxo-call workflow dry-run  <FILE|TEMPLATE>
oxo-call workflow verify   <FILE|TEMPLATE>        # alias: check
oxo-call workflow fmt      <FILE|TEMPLATE> [--stdout]  # alias: format
oxo-call workflow vis      <FILE|TEMPLATE>        # alias: dag
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
- **Verification** — validate your workflow file before running with `workflow verify`
- **Auto-formatting** — canonical style with `workflow fmt`
- **DAG visualization** — text phase diagram with `workflow vis`

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

### `workflow verify`

Validate a workflow file or built-in template for correctness before running. Checks for:

- Parse errors (malformed TOML)
- Empty step names or commands
- Duplicate step names
- References to unknown `depends_on` steps
- `{params.key}` references to undefined parameters
- `{wildcard}` references to undefined wildcards
- Forward-ordering violations (depending on a step defined later in the file)
- DAG expansion failures (cycles, unresolvable dependencies)

```bash
oxo-call workflow verify pipeline.oxo.toml    # Exit 0 if valid, 1 if errors
oxo-call workflow check pipeline.oxo.toml     # alias
```

Example output:

```
◆ workflow 'rnaseq' — 6 step(s), 1 wildcard(s)
✓ No issues found — workflow is valid
```

### `workflow fmt`

Auto-format a `.oxo.toml` workflow file to canonical aligned style:

```bash
oxo-call workflow fmt pipeline.oxo.toml         # Edit in-place
oxo-call workflow fmt pipeline.oxo.toml --stdout  # Print to stdout
oxo-call workflow format pipeline.oxo.toml      # alias
```

The formatter normalizes key alignment, sorts wildcards and params alphabetically, and ensures consistent quoting. Parse the file first with `verify` if you are unsure whether it is valid TOML.

### `workflow vis`

Visualize the workflow as a DAG phase diagram. Shows parallel execution groups, step dependency table, and wildcard expansion summary:

```bash
oxo-call workflow vis pipeline.oxo.toml   # From file
oxo-call workflow vis rnaseq              # Built-in template
oxo-call workflow dag rnaseq              # alias
```

Example output:

```
◆ Workflow: rnaseq  (6 steps, 13 tasks, 4 phases)
  RNA-seq bulk transcript quantification pipeline

  Wildcards:
    sample       = [s1, s2, s3]

────────────────────────────────────────────────────────────────
  Pipeline DAG (4 phases, 13 tasks)

    Phase 1  fastp[sample=s1]  │  fastp[sample=s2]  │  fastp[sample=s3]
             ↓
    Phase 2  multiqc [gather]  │  star[sample=s1]  │  … +2 more
             ↓
    Phase 3  samtools_index[sample=s1]  │  … +2 more
             ↓
    Phase 4  featurecounts[sample=s1]  │  … +2 more

────────────────────────────────────────────────────────────────

  Step details:
  Step               Gather   Tasks    Depends on
  ────────────────────────────────────────────────────────
  fastp                       3        (none)
  multiqc            yes      1        fastp
  star                        3        fastp
  samtools_index              3        star
  featurecounts               3        samtools_index
```

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
| `rnaseq` | Transcriptomics | fastp → MultiQC + STAR → samtools index → featureCounts |
| `wgs` | Genomics | fastp → MultiQC + BWA-MEM2 → MarkDuplicates → BQSR → HaplotypeCaller |
| `atacseq` | Epigenomics | fastp → MultiQC + Bowtie2 → Picard dedup → blacklist filter → MACS3 |
| `chipseq` | Epigenomics | fastp → MultiQC + Bowtie2 → MarkDup → filter → MACS3 + bigWig |
| `metagenomics` | Metagenomics | fastp → MultiQC + host removal → Kraken2 → Bracken |
| `amplicon16s` | Metagenomics | cutadapt → fastp → MultiQC + DADA2 (gather) |
| `scrnaseq` | Single-cell | fastp → MultiQC + STARsolo (10x v3) → samtools index + cell QC |
| `longreads` | Genomics | NanoQ → NanoStat → MultiQC + Flye (parallel) → Medaka → QUAST |
| `methylseq` | Epigenomics | Trim Galore → MultiQC + Bismark → dedup → sort → methylation extract → bedGraph |

### MultiQC Aggregation

All templates follow a consistent pattern: **MultiQC runs as an upstream QC aggregation step** right after the QC/preprocessing step (e.g., fastp, trim_galore, or nanostat). It is configured as a `gather` step that depends only on the QC step, enabling it to run in parallel with downstream analysis:

1. MultiQC only needs QC results (fastp JSON/HTML reports) to generate its report
2. MultiQC scans the QC output directory (e.g., qc/ or trimmed/)
3. A single comprehensive QC report is generated across all samples without blocking downstream steps

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
name    = "qc"
cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz --out1 trimmed/{sample}_R1.fq.gz --out2 trimmed/{sample}_R2.fq.gz --json qc/{sample}_fastp.json"
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz", "qc/{sample}_fastp.json"]

# MultiQC runs right after QC, in parallel with alignment
[[step]]
name       = "multiqc"
gather     = true
depends_on = ["qc"]
cmd        = "multiqc qc/ -o results/multiqc/ --force"
outputs    = ["results/multiqc/multiqc_report.html"]

[[step]]
name    = "align"
depends_on = ["qc"]
cmd     = "bwa mem -t {params.threads} {params.reference} trimmed/{sample}_R1.fq.gz trimmed/{sample}_R2.fq.gz | samtools sort -o aligned/{sample}.bam"
inputs  = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]
outputs = ["aligned/{sample}.bam"]

[[step]]
name       = "index"
depends_on = ["align"]
cmd        = "samtools index aligned/{sample}.bam"
inputs     = ["aligned/{sample}.bam"]
outputs    = ["aligned/{sample}.bam.bai"]
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

  Pipeline DAG (4 phases, 13 tasks)

    Phase 1  fastp[sample=s1]  │  fastp[sample=s2]  │  fastp[sample=s3]
             ↓
    Phase 2  multiqc [gather]  │  star[sample=s1]  │  … +2 more
             ↓
    Phase 3  samtools_index[sample=s1]  │  … +2 more
             ↓
    Phase 4  featurecounts[sample=s1]  │  … +2 more

────────────────────────────────────────────────────────────────
  ▶ fastp[sample=s1]
  ✓ [1/13] fastp[sample=s1]
  ▶ fastp[sample=s2]
  ✓ [2/13] fastp[sample=s2]
  ...
  ✓ [13/13] featurecounts[sample=s3]
────────────────────────────────────────────────────────────────

✓ Workflow complete — 13 task(s) run, 0 up to date  (2m 34s)
```
