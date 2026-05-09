---
name: bcbio-nextgen
category: variant_calling
description: A distributed pipeline for automated next-generation sequencing analysis, including alignment, variant calling, annotation, and quality control reporting. bcbio-nextgen provides a configurable workflow manager that orchestrates multiple bioinformatics tools into cohesive analysis pipelines.
tags: [ngs, variant-calling, sequencing, alignment, pipeline, automation, wgs, rna-seq]
author: AI-generated
source_url: https://bcbio-nextgen.readthedocs.io/
---

## Concepts

- **YAML Configuration-Driven Workflow**: bcbio-nextgen uses a project YAML file (`config.yaml`) to define sample information, sequencing chemistry, analysis workflow type (variantcall, rnaseq, smallrna), and algorithmic parameters. The tool parses this configuration to generate the complete analysis pipeline, eliminating the need for manual per-sample scripting.

- **Multi-Tool Pipeline Orchestration**: The pipeline automatically coordinates execution of upstream tools including aligners (BWA-mem, Bowtie2), quality trimmers (fastp, cutadapt), variant callers (GATK HaplotypeCaller, FreeBayes, DeepVariant), and annotation tools (VEP, SnpEff). Each stage receives standardized input and produces compatible output for the next stage.

- **Flexible Execution Backends**: bcbio-nextgen supports running locally, on HPC clusters using LSF/SGE/Slurm, or in cloud environments. The `resources` section in configuration specifies per-stage parallelization, memory requirements, and queue specifications that the scheduler interprets.

- **Standardized Input/Output Formats**: Pipeline inputs accept FASTQ (single or paired-end), BAM, or CRAM files. Outputs include aligned BAM files, variant calls in VCF/BCF format, coverage BED files, and multi-sample comparison reports. The variant calling workflow produces a ready-to-annotate VCF file with standard INFO/FORMAT fields.

## Pitfalls

- **Misconfigured Sample YAML Structure**: Incorrectly formatted sample YAML (missing `description`, misnamed `files`, or improperly nested `metadata`) causes silent sample skipping during initialization. The pipeline proceeds with only samples it can parse, leaving unexplained missing output files.

- **Incompatible Tool Version Dependencies**: Specifying algorithm options for a variant caller not installed (or an incompatible version) results in runtime failures mid-pipeline. bcbio-nextgen does not validate tool availability before starting, potentially wasting compute hours on partially completed analyses.

- **Insufficient Memory Specification for Large Genomes**: When analyzing large genomes (like wheat or maize) with aligners or variant callers, underspecified memory in the resources section causes OOM kills. The process terminates without error notification if the scheduler terminates the job externally.

- **Mixed FASTQ Encoding in Input Files**: Providing a mix of Sanger-scaled and Illumina-scaled FASTQ files within the same sample batch causes base quality score confusion during alignment and variant calling. This produces false positive variant calls in low-complexity regions and inflates variant quality scores incorrectly.

## Examples

### Run a variant calling analysis from a configuration file
**Args:** `run -n analysis.yaml`
**Explanation:** The `-n` flag specifies the configuration file containing sample metadata and pipeline parameters, initiating the complete analysis workflow from inputs defined in that YAML.

### Create a template project directory with sample configuration
**Args:** `template -n myproject`
**Explanation:** Generates a new project directory with template configuration files and a samples.csv template, providing a starting point for customizing the analysis setup.

### Upgrade bcbio-nextgen installation to latest version
**Args:** `upgrade -u stable`
**Explanation:** The `-u stable` flag pulls and installs the latest stable release, updating all bundled pipelines and dependencies to the current version.

### Run analysis with a custom algorithm parameters file
**Args:** `run -n config.yaml -t algorithm.yaml`
**Explanation:** Provides an additional YAML file specifying algorithm-specific parameters (like variant caller thresholds, quality filters) that override defaults from the main configuration.

### Scale the pipeline to use 16 parallel cores
**Args:** `run -n analysis.yaml -c 16`
**Explanation:** The `-c` flag specifies the number of parallel cores allocated for multi-threaded stages like alignment and variant calling, enabling efficient multi-core utilization.

### Generate a summary report without rerunning analysis
**Args:** `summary -n analysis.yaml`
**Explanation:** Generates or updates the project's HTML summary report using existing analysis results, displaying coverage statistics, variant call metrics, and sample comparisons.