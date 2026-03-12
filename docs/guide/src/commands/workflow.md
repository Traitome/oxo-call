# workflow

Native workflow engine with Snakemake and Nextflow compatibility export.

## Synopsis

```
oxo-call workflow run    <FILE> [--data-dir <DIR>] [--dry-run]
oxo-call workflow show   <TEMPLATE> [--format native|snakemake|nextflow]
oxo-call workflow list
oxo-call workflow export <TEMPLATE> --format <FORMAT> [-o <FILE>]
oxo-call workflow infer  --data-dir <DIR>
```

## Description

The `workflow` command provides a lightweight native Rust workflow engine that executes `.oxo.toml` pipeline files directly — no Snakemake, Nextflow, or Conda required. Snakemake and Nextflow are supported as compatibility export targets.

## Native Engine Features

- **DAG-based execution** — steps run in dependency order with full parallelism via `tokio::task::JoinSet`
- **Wildcard expansion** — `{sample}` automatically expands per sample; `{params.KEY}` for shared parameters
- **Output caching** — steps whose outputs are newer than their inputs are skipped
- **Parallel execution** — independent steps run concurrently

## Subcommands

### `workflow run`

Execute a workflow file:

```bash
oxo-call workflow run pipeline.oxo.toml --data-dir ./data/
oxo-call workflow run pipeline.oxo.toml --dry-run  # Preview only
```

### `workflow show`

Display a built-in template in different formats:

```bash
oxo-call workflow show rnaseq
oxo-call workflow show wgs --format snakemake
oxo-call workflow show metagenomics --format nextflow
```

### `workflow list`

List all available built-in templates:

```bash
oxo-call workflow list
```

### `workflow export`

Export a template to Snakemake or Nextflow format:

```bash
oxo-call workflow export rnaseq --format snakemake -o Snakefile
oxo-call workflow export wgs --format nextflow -o main.nf
```

### `workflow infer`

Automatically detect the appropriate workflow for a data directory:

```bash
oxo-call workflow infer --data-dir ./fastq_data/
```

## Built-in Templates

| Template | Domain | Steps |
|----------|--------|-------|
| rnaseq | RNA-Seq | fastp → STAR → featureCounts → MultiQC |
| wgs | Whole Genome | fastp → BWA-MEM2 → GATK BQSR → HaplotypeCaller |
| atacseq | ATAC-Seq | fastp → Bowtie2 → MACS2 → deepTools |
| metagenomics | Metagenomics | fastp → Kraken2 → Bracken → MultiQC |
| chipseq | ChIP-Seq | fastp → BWA-MEM2 → MACS2 → deepTools |
| scrnaseq | Single-cell | STARsolo → velocyto → MultiQC |
| methylseq | Bisulfite-Seq | fastp → Bismark → MethylDackel |
| longreads | Long-read | Dorado → Minimap2 → Sniffles |
| amplicon16s | 16S Amplicon | DADA2-based pipeline |

## .oxo.toml Format

```toml
[workflow]
name = "my-pipeline"
samples = ["sample1", "sample2"]

[params]
threads = 8
reference = "hg38.fa"

[[steps]]
name = "align"
tool = "bwa"
args = "mem -t {params.threads} {params.reference} {sample}_R1.fq {sample}_R2.fq"
output = "{sample}.sam"

[[steps]]
name = "sort"
tool = "samtools"
args = "sort -@ {params.threads} -o {sample}.sorted.bam {sample}.sam"
input = "{sample}.sam"
output = "{sample}.sorted.bam"
depends_on = ["align"]
```
