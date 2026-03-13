# Workflow Builder Tutorial

This tutorial teaches you how to use the oxo-call native workflow engine to build, validate, and run reproducible multi-sample bioinformatics pipelines. You will convert the manual RNA-seq steps from the previous tutorial into a single automated `.oxo.toml` file.

**Time to complete:** 20–30 minutes
**Prerequisites:** oxo-call configured, [RNA-seq walkthrough](./rnaseq-walkthrough.md) completed (recommended)
**You will learn:** `.oxo.toml` format, wildcards, dependencies, dry-run, DAG visualization

---

## Why Use the Workflow Engine?

Running commands manually works for a single sample. For a real experiment with 10–100 samples, you need:
- **Reproducibility**: every sample processed identically
- **Parallelism**: independent samples run at the same time
- **Caching**: skip steps whose outputs already exist
- **Auditability**: a single file describes the entire pipeline

The native `.oxo.toml` workflow engine provides all of this with no external dependencies.

---

## The Workflow File Format

A `.oxo.toml` file has four sections:

```toml
[workflow]       # name and description
[wildcards]      # variables that expand per sample
[params]         # shared configuration values
[[step]]         # repeated for each pipeline step
```

### A minimal example

```toml
[workflow]
name        = "my-pipeline"
description = "A simple two-step pipeline"

[wildcards]
sample = ["sample1", "sample2"]

[params]
threads = "4"

[[step]]
name    = "qc"
cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz \
           --out1 trimmed/{sample}_R1.fq.gz --out2 trimmed/{sample}_R2.fq.gz \
           --thread {params.threads} --html qc/{sample}.html"
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["trimmed/{sample}_R1.fq.gz", "qc/{sample}.html"]

[[step]]
name       = "align"
depends_on = ["qc"]
cmd        = "STAR --genomeDir /data/star_index \
              --readFilesIn trimmed/{sample}_R1.fq.gz trimmed/{sample}_R2.fq.gz \
              --readFilesCommand zcat \
              --outSAMtype BAM SortedByCoordinate \
              --outFileNamePrefix aligned/{sample}/ \
              --runThreadN {params.threads}"
inputs     = ["trimmed/{sample}_R1.fq.gz", "trimmed/{sample}_R2.fq.gz"]
outputs    = ["aligned/{sample}/Aligned.sortedByCoord.out.bam"]
```

When you run this with `sample = ["sample1", "sample2"]`:
- `qc` runs for both samples in parallel
- `align` runs for each sample after its `qc` step completes

---

## Step 1: Explore the Built-in RNA-seq Template

Start by examining what a production-ready template looks like:

```bash
oxo-call workflow show rnaseq
```

This prints the full `.oxo.toml` for the built-in RNA-seq template. Notice:

- `[wildcards]` with `sample = [...]`
- `[params]` for `threads`, `star_index`, and `gtf`
- Steps: `fastp` → `star` → `multiqc` (gather) → `featurecounts`
- The `multiqc` step has `gather = true` — it runs once after all samples finish

Visualize the dependency graph:

```bash
oxo-call workflow vis rnaseq
```

Output:
```
◆ workflow 'rnaseq' — 4 step(s), 1 wildcard(s)

Phase 1 (parallel):
  fastp  [per-sample: sample1, sample2, sample3]

Phase 2 (parallel):
  star  [per-sample: sample1, sample2, sample3]

Phase 3 (gather):
  multiqc  [gather across all samples]

Phase 4 (parallel):
  featurecounts  [per-sample: sample1, sample2, sample3]
```

---

## Step 2: Customize a Template for Your Data

Save the template to a file and edit it:

```bash
oxo-call workflow show rnaseq > my_rnaseq.toml
```

Open `my_rnaseq.toml` and edit the wildcards and params sections:

```toml
[wildcards]
sample = ["ctrl_1", "ctrl_2", "treat_1", "treat_2"]   # your sample names

[params]
threads    = "8"
star_index = "/data/star_hg38"                          # your STAR index
gtf        = "/data/gencode.v44.gtf"                    # your GTF file
```

Also update the `inputs` paths in each step to match your data layout. For example, if your data is in `/data/fastq/{sample}_R1.fq.gz`:

```toml
[[step]]
name   = "fastp"
cmd    = "fastp --in1 /data/fastq/{sample}_R1.fq.gz ..."
inputs = ["/data/fastq/{sample}_R1.fq.gz", "/data/fastq/{sample}_R2.fq.gz"]
```

---

## Step 3: Validate Before Running

Always validate your workflow file before running it:

```bash
oxo-call workflow verify my_rnaseq.toml
```

This checks for:
- Malformed TOML
- References to undefined wildcards or params
- Unknown `depends_on` steps
- Step ordering violations (depending on a step defined later)
- DAG cycles

Example valid output:
```
◆ workflow 'rnaseq' — 4 step(s), 1 wildcard(s)
✓ No issues found — workflow is valid
```

Example error output:
```
◆ workflow 'rnaseq' — 4 step(s), 1 wildcard(s)
✗ Step 'star' depends on 'qc' which is not defined
✗ {params.star_index} is used but 'star_index' is not in [params]
```

Fix any errors before proceeding.

---

## Step 4: Preview with Dry-Run

Do a full dry-run to see every expanded command before executing:

```bash
oxo-call workflow dry-run my_rnaseq.toml
```

This shows:
- DAG phase diagram
- Every expanded command (with wildcards substituted)
- Dependencies and output paths
- Which steps would be cached (outputs already newer than inputs)

Example dry-run output:
```
◆ Workflow: rnaseq (4 steps, 4 samples)

Phase 1 — fastp [ctrl_1]
  Command: fastp --in1 /data/fastq/ctrl_1_R1.fq.gz ...
  Inputs:  /data/fastq/ctrl_1_R1.fq.gz, /data/fastq/ctrl_1_R2.fq.gz
  Outputs: trimmed/ctrl_1_R1.fq.gz, qc/ctrl_1.html

Phase 1 — fastp [ctrl_2]
  Command: fastp --in1 /data/fastq/ctrl_2_R1.fq.gz ...
  ...

[SKIP] Phase 2 — star [ctrl_1]  (outputs up-to-date)
```

The `[SKIP]` lines tell you which steps will be cached.

---

## Step 5: Format for Readability

Auto-format the workflow file for consistent style:

```bash
oxo-call workflow fmt my_rnaseq.toml
```

This normalizes key alignment and quoting. Use `--stdout` to preview changes without modifying the file:

```bash
oxo-call workflow fmt my_rnaseq.toml --stdout
```

---

## Step 6: Run the Workflow

Once everything looks correct, execute:

```bash
oxo-call workflow run my_rnaseq.toml
```

The engine will:
1. Expand wildcards for all samples
2. Build the DAG
3. Run Phase 1 steps (fastp) in parallel across all samples
4. When all Phase 1 steps finish, run Phase 2 (STAR) in parallel
5. After STAR finishes, run MultiQC as a gather step (once)
6. Run featureCounts in parallel for all samples

Progress output:
```
[1/16] fastp ctrl_1        ... done (12.3s)
[2/16] fastp ctrl_2        ... done (11.8s)
[3/16] fastp treat_1       ... done (13.1s)
[4/16] fastp treat_2       ... done (12.7s)
[5/16] star ctrl_1         ... done (4m 12s)
...
[13/16] multiqc            ... done (3.2s)
[14/16] featurecounts ctrl_1  ... done (45.2s)
...
✓ Workflow complete in 18m 32s
```

---

## Step 7: Export to Snakemake or Nextflow

If your HPC cluster requires Snakemake or Nextflow:

```bash
# Export to Snakemake
oxo-call workflow export my_rnaseq.toml --to snakemake -o Snakefile

# Export to Nextflow DSL2
oxo-call workflow export my_rnaseq.toml --to nextflow -o main.nf
```

The exported files preserve all sample wildcards and dependency structure.

---

## Generate a New Workflow with LLM

You can also ask the LLM to generate a workflow from scratch:

```bash
oxo-call workflow generate \
  "ChIP-seq pipeline for H3K27ac, paired-end, with bowtie2 alignment, \
   picard duplicate marking, and macs3 peak calling against input control" \
  -o chipseq_h3k27ac.toml
```

Always validate and dry-run LLM-generated workflows before executing:

```bash
oxo-call workflow verify chipseq_h3k27ac.toml
oxo-call workflow dry-run chipseq_h3k27ac.toml
```

---

## Workflow Design Tips

### Keep steps focused

Each `[[step]]` should do one thing. Avoid chaining multiple tools with `&&` unless they are tightly coupled (e.g., `samtools sort && samtools index`).

### Always specify inputs and outputs

The engine uses `inputs` and `outputs` for cache checking. A step without `outputs` will always re-run.

### Use `gather = true` for aggregation steps

Steps that aggregate across all samples (MultiQC, count matrix merging) should have `gather = true` to ensure they run after all sample instances complete.

### Step order matters

Steps must be defined in order — a step can only reference dependencies that appear **before it** in the file.

```toml
# ✓ CORRECT: align is defined after qc
[[step]]
name = "qc"
...

[[step]]
name       = "align"
depends_on = ["qc"]
...

# ✗ WRONG: align references qc which is defined after it
[[step]]
name       = "align"
depends_on = ["qc"]
...

[[step]]
name = "qc"
...
```

---

## What You Learned

- How to write a `.oxo.toml` workflow file from scratch
- How wildcards expand per-sample commands
- How `gather = true` enables aggregation steps like MultiQC
- How to validate, visualize, dry-run, and execute a workflow
- How to export to Snakemake or Nextflow
- How to generate a workflow from natural language

**Next steps:**
- [Build pipeline how-to](../how-to/build-pipeline.md) — advanced pipeline patterns
- [Workflow Engine reference](../reference/workflow-engine.md) — complete format specification
- [workflow command reference](../commands/workflow.md) — all subcommands
