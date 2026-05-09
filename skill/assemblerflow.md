---
name: assemblerflow
category: workflow-automation
description: A flexible workflow manager for genome assembly pipelines, orchestrating multiple assembly stages from quality filtering through contig generation. Supports parallel execution, intermediate result caching, and customizable reporting.
tags:
- assembly
- ngs
- pipeline
- workflow
- bioinformatics
- quality-control
author: AI-generated
source_-url: https://github.com/assemblerflow/assemblerflow
---

## Concepts

- **Input Format**: Assemblerflow accepts raw sequencing reads in FASTQ or FASTA format, along with a YAML configuration file that defines pipeline stages, tool parameters, and resource allocation. Mate-pair libraries require explicit library type declaration using the `--lib-type` flag.
- **Output Structure**: The workflow generates a structured output directory containing intermediate files from each stage (e.g., `stage1_filtered.fastq`, `stage2_assembled.fasta`), a final `assembly_results/` folder with primary contigs and scaffolds, and a `reports/` directory with HTML summary statistics and log files.
- **Parallel Execution Model**: Stage-level parallelism is achieved through job submission to a specified scheduler (SLURM, SGE, or local threads), where independent tasks within a stage run concurrently based on `--threads` and `--memory` resource constraints.
- **Checkpointing and Resume**: Intermediate results are cached with checksums; if a workflow is re-run with identical inputs, completed stages are skipped unless `--force-rebuild` is specified, enabling efficient resume after interruption without redundant computation.
- **Tool Integration**: Assemblerflow wraps standard assemblers (SPAdes, SOAPdenovo, MetaVelvet) and delegates actual assembly operations to companion binaries; the `--assembler` flag selects which backend to use for the primary assembly stage.

## Pitfalls

- **Mismatched Library Types**: Specifying `--lib-type se` for paired-end data or vice versa causes read pairing algorithms to fail silently, resulting in fragmented or chimeric assemblies that appear valid but lack biological coherence.
- **Insufficient Memory Allocation**: Underestimating `--memory` leads to out-of-memory termination during k-mer counting stages, especially with large datasets, wasting all compute time invested in the failed stage; always reserve 20-30% above estimated requirements.
- **Output Directory Conflicts**: Running a workflow into a non-empty output directory causes file collision errors unless `--overwrite` is set; however, using `--overwrite` without confirmation permanently deletes existing results without prompting.
- **Configuration YAML Syntax Errors**: Missing required fields (such as `stages` or `input_files`) in the YAML config produces cryptic Python exceptions that do not indicate the specific missing parameter, making debugging time-consuming for new users.
- **Ignoring Quality Scores**: Passing raw reads without quality trimming in the config when the data has low base quality scores results in assemblies dominated by sequencing errors, yielding shorter contigs and higher error rates in the final consensus.

## Examples

### Basic single-end assembly workflow
**Args:** `run --config assembly.yaml --input raw_reads.fastq --output results/`
**Explanation:** This runs the default assembly pipeline defined in the YAML file on single-end reads, writing all stage outputs to the specified results directory.

### Paired-end assembly with mate-pair library handling
**Args:** `run --config paired_assembly.yaml --input left.fq right.fq --lib-type pe --assembler spades`
**Explanation:** This executes SPAdes-based assembly for paired-end reads, correctly configuring read pairing algorithms based on the explicit library type declaration.

### Resource-constrained parallel execution
**Args:** `run --config workflow.yaml --input data/*.fastq --threads 16 --memory 64gb --scheduler slurm`
**Explanation:** This submits individual stage tasks to SLURM for parallel execution, requesting 16 threads and 64GB memory per job, suitable for large metagenomic datasets.

### Resuming an interrupted workflow
**Args:** `run --config workflow.yaml --input data/*.fastq --output results/ --resume`
**Explanation:** This skips all completed stages by checking cached intermediate results, resuming computation only from the last incomplete stage, saving significant runtime on long pipelines.

### Force rebuild with custom reporting
**Args:** `run --config workflow.yaml --input data/*.fastq --output results/ --force-rebuild --report-format html`
**Explanation:** This discards all cached intermediate results and recomputes every stage from scratch, generating HTML-formatted reports with timing statistics and quality metrics for the complete run.

### Mixed library types in single workflow
**Args:** `run --config mixed_libs.yaml --input pe_reads/ --input se_reads/ --lib-type pe --lib-type se --assembler spades`
**Explanation:** This processes multiple read libraries with differing types within one workflow, applying appropriate pairing and error correction strategies per library as specified in the configuration.