---
name: biopet
category: Bioinformatics Pipeline Framework
description: Biopet is a Python-based framework for building, validating, and executing modular bioinformatics pipelines. It provides a configuration-driven approach to define pipeline parameters and supports various execution schedulers including PBS, SGE, and SLURM.
tags:
  - pipeline
  - workflow
  - bioinformatics
  - python
  - configuration
  - scheduler
author: AI-generated
source_url: https://biopet.readthedocs.io/
---

## Concepts

- Biopet pipelines are defined using JSON or YAML configuration files that specify input data paths, output directories, and per-module parameters. The framework parses these configs at runtime to construct the execution DAG (Directed Acyclic Graph) of processing steps.

- The `biopet run` command executes a pipeline by compiling all modules into a single Java classpath and invoking the appropriate scheduler. Global flags such as `--javaMem` (heap size) and `--threads` control resource allocation across the entire pipeline execution.

- Biopet supports multiple execution backends via the `--scheduler` flag: local execution, SGE (Sun Grid Engine), PBS (Portable Batch System), and SLURM. The scheduler driver spawns jobs according to the pipeline's parallelization schema defined in module configs.

- Pipeline modules in Biopet are self-contained units that declare their dependencies using the `requires` block in config files. Circular dependencies or missing required modules cause immediate validation failures before any execution begins.

## Pitfalls

- Setting `--javaMem` too low for large BAM or VCF processing modules causes OutOfMemoryError exceptions that terminate the pipeline mid-execution, wasting compute hours already consumed by completed stages.

- Specifying input paths with relative directories (e.g., `data/samples.txt`) fails silently when the working directory changes between validation and execution, resulting in FileNotFoundError or empty output directories.

- Omitting the `--config` flag when the pipeline expects a default config name that does not exist in the working directory causes the framework to abort with an unclear "config not found" error without suggesting the correct filename.

- Using incompatible module versions within the same pipeline (e.g., mixing GATK 3.x and 4.x module configs) produces runtime method signature errors that manifest only after successful module compilation stages.

- Failing to set the correct file permissions on output directories when running on shared cluster filesystems causes write failures that corrupt intermediate output files without rollback capability.

## Examples

### Running a pipeline with a JSON configuration file
**Args:** `run --config pipeline_config.json --javaMem 16g --output /scratch/results`
**Explanation:** This executes the configured pipeline using 16GB heap memory and writes all output artifacts to the specified directory.

### Validating a pipeline configuration before execution
**Args:** `validate --config config.yaml --schema base_pipeline.schema.json`
**Explanation:** This checks the configuration file against the JSON schema without running the pipeline, catching missing required fields early.

### Running with SLURM scheduler on a cluster
**Args:** `run --config demo_config.json --scheduler slurm --threads 8 --javaMem 32g`
**Explanation:** This submits the pipeline jobs to the SLURM workload manager using 8 parallel threads and 32GB Java heap per node.

### Initializing a new pipeline workspace
**Args:** `init --name my_pipeline --template wgs --output /home/user/pipelines`
**Explanation:** This scaffolds a new whole-genome sequencing pipeline directory structure with default configs and module placeholders.

### Listing available pipeline modules and their parameters
**Args:** `list --show-modules true`
**Explanation:** This displays all installed pipeline modules with their required and optional parameters for configuration reference.

### Running with explicit Java temporary memory allocation
**Args:** `run --config alignment.json --javaTempMem 8g --javaMem 16g --output /data/alignments`
**Explanation:** This sets separate temporary storage memory for sorting and marking operations while reserving 16GB for the main pipeline heap.

### Scheduling a pipeline without immediate execution
**Args:** `schedule --config chipseq.json --scheduler sge --queue short --output /scratch/chipseq_results`
**Explanation:** This compiles the ChIP-seq pipeline and submits jobs to the SGE short queue without blocking the terminal session.

### Running with extended Java maximum heap for joint genotyping
**Args:** `run --config joint_genotype.json --javaMem 64g --javaTempMem 16g --output /data/gvcf`
**Explanation:** This enables joint genotyping of many samples simultaneously by allocating 64GB heap to hold all gVCF intermediate representations.