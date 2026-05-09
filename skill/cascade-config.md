---
name: cascade-config
category: Bioinformatics Pipeline Configuration
description: Configuration management tool for bioinformatics cascade pipelines, allowing users to create, validate, and manage pipeline configuration files for high-throughput genomic analysis workflows.
tags: [pipeline, configuration, workflow, genomics, bioinformatics]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cascade-config
---

## Concepts

- **Configuration File Format**: cascade-config uses YAML format for pipeline definitions, supporting nested parameters for tools, reference genomes, and runtime resources. The config file uses `.yaml` or `.yml` extensions and must follow a hierarchical structure with sections for `global_params`, `steps`, and `output_settings`.
- **Pipeline Step Chaining**: Each step in the configuration defines a bioinformatics tool or script to execute in sequence. Steps are ordered numerically and can reference outputs from previous steps using the `${step.N.output}` placeholder syntax. This enables complex multi-stage workflows like alignment → variant calling → annotation.
- **Reference Data Management**: Configuration files include paths to reference genomes, annotation databases, and indexed files. The tool verifies that referenced files exist and have correct extensions (e.g., `.fa` for FASTA, `.vcf` for VCF files). Relative paths are resolved relative to the configuration file location.
- **Validation Mode**: Running `cascade-config validate` checks configuration syntax, tool availability in system PATH, file existence, and parameter compatibility. Exit code 0 indicates valid config, non-zero indicates errors with detailed messages printed to stderr.

## Pitfalls

- **Incorrect YAML indentation**: Using tabs instead of spaces for indentation causes silent parsing failures. YAML requires spaces exclusively—using tabs will result in a cryptic "mapping values are not allowed here" error during configuration parsing.
- **Missing required global parameters**: Forgetting to specify `sample_name` or `output_dir` in the global_params section causes the pipeline to fail at runtime with an unclear "required parameter missing" message, requiring manual debugging of the config file.
- **Mismatched file extensions**:Specifying a `.bam` file path when the actual file is `.sam` leads to downstream tool errors since many bioinformatics tools strictly check extensions before processing. The validation step only checks existence, not format compatibility.
- **Circular step dependencies**: Creating a circular reference where step N references `${step.N.output}` or where two steps reference each other causes an infinite loop during pipeline execution and requires killing the process manually.
- **Unquoted special characters**: Not quoting strings that contain colons (e.g., `my:file`) causes YAML to interpret the colon as a key-value separator, corrupting the parameter value and leading to unexpected behavior.

## Examples

### Validate a pipeline configuration file
**Args:** `validate pipeline.yaml`
**Explanation:** Checks the syntax, file existence, tool availability, and parameter compatibility of the pipeline configuration before execution, ensuring the workflow can run without crashing.

### Generate a default configuration template
**Args:** `init --template variant_calling --output my_pipeline.yaml`
**Explanation:** Creates a pre-filled YAML configuration file with recommended parameters for variant calling pipelines, including steps for alignment, sorting, and variant detection.

### List all available pipeline templates
**Args:** `list-templates`
**Explanation:** Displays every supported pipeline template (e.g., RNA-seq, DNA-seq, metagenomics) stored in the tool's template directory for users to choose from.

### Check required tools are installed
**Args:** `check-deps pipeline.yaml`
**Explanation:** Verifies that all bioinformatics tools referenced in the configuration (like bwa, samtools, gatk) are available in the system PATH and meet version requirements.

### Set a custom output directory
**Args:** `init --template rna_seq --output_dir /data/results --output rna_pipeline.yaml`
**Explanation:** Creates an RNA-seq pipeline configuration that writes all output files to the specified `/data/results` directory instead of the default ./output folder.

### Override a template parameter
**Args:** `init --template dna_seq --extra_params "threads: 16 memory: 32G" --output custom_pipeline.yaml`
**Explanation:** Generates a DNA-seq pipeline with custom thread and memory allocation settings that override the default values in the template.