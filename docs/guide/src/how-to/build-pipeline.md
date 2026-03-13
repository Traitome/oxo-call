# How-to: Build a Production Pipeline

This guide covers advanced patterns for building robust, production-ready bioinformatics pipelines with the oxo-call workflow engine.

---

## Pipeline Design Checklist

Before writing a `.oxo.toml` file for a real project:

- [ ] Define all samples in `[wildcards]`
- [ ] Extract shared configuration into `[params]`
- [ ] Specify `inputs` and `outputs` for every step (enables caching)
- [ ] Use `depends_on` to express all dependencies explicitly
- [ ] Use `gather = true` for aggregation steps
- [ ] Run `oxo-call workflow verify` before any run
- [ ] Run `oxo-call workflow dry-run` to inspect every expanded command
- [ ] Run `oxo-call workflow vis` to confirm the DAG looks correct

---

## Pattern 1: Per-Sample Steps with Shared Parameters

The most common pattern: each sample runs through the same steps, sharing configuration:

```toml
[workflow]
name        = "wgs-variant-calling"
description = "WGS variant calling pipeline: alignment → BQSR → HaplotypeCaller"

[wildcards]
sample = ["sample_A", "sample_B", "sample_C", "sample_D"]

[params]
threads  = "16"
ref      = "/data/hg38/hg38.fa"
known    = "/data/hg38/dbsnp_146.hg38.vcf.gz"
intervals = "/data/hg38/wgs_calling_regions.hg38.interval_list"

[[step]]
name    = "bwa_align"
cmd     = "bwa-mem2 mem -t {params.threads} {params.ref} \
           data/{sample}_R1.fq.gz data/{sample}_R2.fq.gz \
           | samtools sort -@ 4 -o aligned/{sample}.bam && \
           samtools index aligned/{sample}.bam"
inputs  = ["data/{sample}_R1.fq.gz", "data/{sample}_R2.fq.gz"]
outputs = ["aligned/{sample}.bam", "aligned/{sample}.bam.bai"]

[[step]]
name       = "mark_duplicates"
depends_on = ["bwa_align"]
cmd        = "picard MarkDuplicates \
              I=aligned/{sample}.bam \
              O=dedup/{sample}.bam \
              M=dedup/{sample}_metrics.txt && \
              samtools index dedup/{sample}.bam"
inputs     = ["aligned/{sample}.bam"]
outputs    = ["dedup/{sample}.bam", "dedup/{sample}.bam.bai"]

[[step]]
name       = "bqsr"
depends_on = ["mark_duplicates"]
cmd        = "gatk BaseRecalibrator \
              -I dedup/{sample}.bam \
              -R {params.ref} \
              --known-sites {params.known} \
              -O bqsr/{sample}.recal.table && \
              gatk ApplyBQSR \
              -I dedup/{sample}.bam \
              -R {params.ref} \
              --bqsr-recal-file bqsr/{sample}.recal.table \
              -O bqsr/{sample}.bam"
inputs     = ["dedup/{sample}.bam"]
outputs    = ["bqsr/{sample}.bam"]

[[step]]
name       = "haplotypecaller"
depends_on = ["bqsr"]
cmd        = "gatk HaplotypeCaller \
              -I bqsr/{sample}.bam \
              -R {params.ref} \
              -L {params.intervals} \
              -O gvcf/{sample}.g.vcf.gz \
              -ERC GVCF \
              --native-pair-hmm-threads 4"
inputs     = ["bqsr/{sample}.bam"]
outputs    = ["gvcf/{sample}.g.vcf.gz"]
```

---

## Pattern 2: Gather Steps

Gather steps aggregate results across all samples and run exactly once after all instances of their dependencies complete:

```toml
[[step]]
name       = "combine_gvcfs"
gather     = true
depends_on = ["haplotypecaller"]
cmd        = "gatk CombineGVCFs \
              -R {params.ref} \
              $(ls gvcf/*.g.vcf.gz | sed 's/^/-V /') \
              -O combined/cohort.g.vcf.gz"
inputs     = ["gvcf/{sample}.g.vcf.gz"]
outputs    = ["combined/cohort.g.vcf.gz"]

[[step]]
name       = "genotype_gvcfs"
depends_on = ["combine_gvcfs"]
cmd        = "gatk GenotypeGVCFs \
              -R {params.ref} \
              -V combined/cohort.g.vcf.gz \
              -O final/cohort.vcf.gz"
inputs     = ["combined/cohort.g.vcf.gz"]
outputs    = ["final/cohort.vcf.gz"]
```

With 4 samples, the execution order is:
1. `bwa_align` × 4 (parallel)
2. `mark_duplicates` × 4 (parallel)
3. `bqsr` × 4 (parallel)
4. `haplotypecaller` × 4 (parallel)
5. `combine_gvcfs` × 1 (gather — waits for all 4)
6. `genotype_gvcfs` × 1 (sequential after combine)

---

## Pattern 3: Mixed Wildcards

Some steps may process pairs of conditions rather than individual samples:

```toml
[wildcards]
sample    = ["ctrl_1", "ctrl_2", "treat_1", "treat_2"]
condition = ["ctrl", "treat"]
```

Steps using `{sample}` expand per sample. Steps using `{condition}` expand per condition.

> **Note:** Mixed wildcards are advanced and require care. Use `workflow verify` to check for expansion errors.

---

## Pattern 4: Conditional Output Paths

Use parameter substitution to control output organization:

```toml
[params]
outdir  = "results/v2"
threads = "8"

[[step]]
name    = "fastp"
cmd     = "fastp --in1 data/{sample}_R1.fq.gz --in2 data/{sample}_R2.fq.gz \
           --out1 {params.outdir}/trimmed/{sample}_R1.fq.gz \
           --out2 {params.outdir}/trimmed/{sample}_R2.fq.gz \
           --thread {params.threads}"
outputs = ["{params.outdir}/trimmed/{sample}_R1.fq.gz"]
```

Changing `outdir` in `[params]` moves all output paths to a new versioned directory.

---

## Restarting a Failed Pipeline

If a step fails mid-run, fix the issue and re-run:

```bash
oxo-call workflow run my_pipeline.toml
```

The engine automatically skips steps whose outputs are already newer than their inputs. Only failed and downstream steps will re-run.

To force a specific step to re-run, delete its output files:

```bash
rm aligned/sample_A.bam aligned/sample_A.bam.bai
oxo-call workflow run my_pipeline.toml
# Only bwa_align for sample_A and its downstream steps will re-run
```

---

## HPC/Cluster Submission

The native engine runs on the current machine. For HPC cluster execution, export to Snakemake with a cluster profile:

```bash
oxo-call workflow export my_pipeline.toml --to snakemake -o Snakefile

# Run on SLURM cluster
snakemake --cluster "sbatch -p short -c {threads} --mem=16G" --jobs 50

# Run with Singularity containers (if workflow has container: directives)
snakemake --use-singularity --cluster "sbatch ..." --jobs 50
```

Or export to Nextflow for cloud execution:

```bash
oxo-call workflow export my_pipeline.toml --to nextflow -o main.nf

# Run locally
nextflow run main.nf

# Run on AWS
nextflow run main.nf -profile aws
```

---

## Troubleshooting Pipeline Issues

### Verify first

```bash
oxo-call workflow verify my_pipeline.toml
```

Fix all reported errors before running.

### Check the DAG

```bash
oxo-call workflow vis my_pipeline.toml
```

Confirm the phases and dependencies look correct.

### Dry-run to inspect commands

```bash
oxo-call workflow dry-run my_pipeline.toml 2>&1 | grep -A3 "sample_A"
```

### Common errors

| Error | Cause | Fix |
|-------|-------|-----|
| `step 'X' depends on 'Y' which is not defined` | Typo in `depends_on` | Check step names |
| `{params.X} is used but 'X' is not in [params]` | Missing param key | Add to `[params]` |
| `forward reference to step 'Y'` | Step ordering | Move the referenced step before the current step |
| `DAG cycle detected` | Circular dependency | Break the cycle |
| Step always re-runs | No `outputs` defined | Add `outputs` list |

---

## Related

- [Workflow Builder tutorial](../tutorials/workflow-builder.md) — step-by-step for beginners
- [Workflow Engine reference](../reference/workflow-engine.md) — complete format specification
- [workflow command reference](../commands/workflow.md) — all subcommands
