---
name: brockman-pipeline
category: pipeline-orchestration
description: A flexible bioinformatics pipeline framework for orchestrating multi-stage genomic data processing workflows. Supports iterative stages, streaming data between steps, checkpoint-based resumption, and parallel execution across compute nodes.
tags: [pipeline, ngs, workflow, genomics, processing, parallel, hpc]
author: AI-generated
source_url: https://github.com/brockman/pipeline
---

## Concepts

- **Stage-based Workflow Model**: brockman-pipeline organizes bioinformatics tasks into discrete stages (e.g., align, filter, call) that execute sequentially. Each stage receives input from the previous stage's output, enabling modular pipeline construction. Stages are defined in a YAML configuration file using the `stages` key, where each stage specifies its module, parameters, and dependencies.

- **Streaming and Checkpointing**: The tool supports streaming mode (`--stream`) to pass data directly between stages without writing intermediate files to disk, reducing I/O overhead. Checkpointing (`--checkpoint`) allows resumption from the last completed stage if the pipeline fails, avoiding redundant computation. Checkpoints are stored in a `.brockman` directory in the output folder.

- **Multi-Format I/O**: brockman-pipeline handles common bioinformatics formats natively: FASTQ (input), BAM/CRAM (alignment), VCF (variants), and BED (regions). The `--input-format` and `--output-format` flags specify conversion between formats across pipeline stages. Format auto-detection is supported when file extensions are standard (.fastq.gz, .bam, .vcf.gz).

- **Parallel Execution**: The pipeline distributes stages across multiple threads or compute nodes using the `--threads` and `--nodes` flags. Intra-stage parallelization leverages POSIX threads, while inter-stage parallelization runs independent stages concurrently. Resource allocation is controlled via a `resources` section in the stage configuration.

## Pitfalls

- **Missing Stage Dependencies**: If stage dependencies are not explicitly declared in the configuration, brockman-pipeline may execute stages out of order, producing incorrect results. Always specify the `requires` field for each stage to define input dependencies. Consequence: downstream stages receive malformed input, causing pipeline failure or silent data corruption.

- **Insufficient Disk Space for Checkpoints**: Checkpointing requires disk space to store intermediate outputs. Running on a full partition causes checkpoint写入 to fail mid-pipeline, forcing a complete restart. Monitor output directory size beforehand using `df -h`. Consequence: wasted compute time and potential data loss for long-running pipelines.

- **Mismatched Input Formats**: Attempting to process a BAM file with a stage configured for FASTQ input without conversion will fail. brockman-pipeline performs format conversion only when explicitly requested via `--input-format` or a conversion stage. Check input format compatibility before pipeline execution. Consequence: stage failure, pipeline abortion, and wasted resources.

- **Ignoring Resource Limits**: Over-allocating threads or nodes can cause job rejection on shared HPC schedulers (SLURM, PBS) or trigger OOM kills. brockman-pipeline does not automatically detect scheduler limits. Consequence: job failure, node eviction, and potential account penalties on HPC systems.

## Examples

### Run a basic three-stage variant calling pipeline

**Args:** --config pipeline.yaml --input sample1.fastq.gz --output /results/sample1 --threads 8

**Explanation:** This executes a pipeline defined in pipeline.yaml with three sequential stages (align, filter, call) using 8 threads for parallelizable steps. The output directory stores all intermediate and final results.

### Resume a failed pipeline from the last checkpoint

**Args:** --config pipeline.yaml --input sample1.fastq.gz --output /results/sample1 --resume --checkpoint

**Explanation:** The `--resume` flag detects the last completed stage from the `.brockman` checkpoint directory and restarts from the subsequent stage, avoiding recomputation of already-processed data.

### Run with streaming between stages to reduce I/O

**Args:** --config pipeline.yaml --input sample1.fastq.gz --output /results/sample1 --stream --threads 4

**Explanation:** The `--stream` flag enables in-memory data passing between pipeline stages, eliminating intermediate file writes. Useful for fastq-to-bam workflows where disk I/O is the bottleneck.

### Specify explicit input and output formats

**Args:** --config pipeline.yaml --input sample1.fastq.gz --output /results/sample1 --input-format fastq.gz --output-format vcf.gz

**Explanation:** Explicit format flags ensure proper codec selection and extension handling. brockman-pipeline uses this to select appropriate tools (e.g., bgzip for .vcf.gz output) and validate stage compatibility.

### Execute with distributed compute across multiple nodes

**Args:** --config pipeline.yaml --input sample1.fastq.gz --output /results/sample1 --nodes 4 --threads 16

**Explanation:** The `--nodes` flag distributes independent pipeline stages across 4 compute nodes, while `--threads` allocates 16 threads per node. Suitable for large-scale cohort processing on HPC clusters.