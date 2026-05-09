---
name: argutils
category: bioinformatics-utilities
description: Command-line utilities for managing, validating, and transforming bioinformatics arguments, configurations, and data format specifications. Provides subcommands for parsing, conversion, and validation of CLI arguments and parameter files.
tags: command-line, arguments, validation, configuration, bioinformatics-utilities, parsing, yaml, json
author: AI-generated
source_url: https://github.com/ncbi/argutils
---

## Concepts

- **Subcommand Architecture**: argutils follows a subcommand pattern where each operation (validate, parse, convert, generate) is invoked as a separate subcommand, similar to version control tools like git.
- **Configuration File Support**: argutils reads and writes configuration files in YAML, JSON, and TOML formats, enabling pipeline parameter management and reproducible workflows.
- **Schema Validation**: The tool can validate arguments against predefined schemas, ensuring type safety and required field compliance before downstream bioinformatics tools execute.
- **Standard Input/Output Streams**: Many argutils subcommands can read from stdin and write to stdout, enabling piping with other bioinformatics tools like samtools, bcftools, and bedtools.
- **Exit Code Semantics**: argutils returns exit code 0 for successful operations, 1 for validation errors, and 2 for usage errors, facilitating integration into scripting and pipeline frameworks.

## Pitfalls

- **Missing Required Arguments Silently Proceeding**: When required arguments are omitted, some subcommands may proceed with default values or empty strings rather than failing explicitly, leading to incomplete or invalid output files.
- **Incorrect Configuration File Format**: Specifying a JSON file when YAML is expected produces parsing errors; always verify the configuration file matches the expected format using the `--format` flag or file extension.
- **Schema Mismatch With Tool Version**: Using a schema designed for an older argutils version may cause validation failures or silently accept deprecated fields that newer tools reject.
- **Overwriting Output Files Without Confirmation**: The default behavior may overwrite existing output files without prompting, causing data loss in collaborative or multi-step pipeline environments.
- **Case-Sensitive Argument Names**: Argument names are case-sensitive; using `SampleID` versus `sampleid` in configurations will be treated as different parameters, potentially causing validation failures.

## Examples

### Validate a YAML configuration file against a schema
**Args:** `validate --schema schema.json config.yaml`
**Explanation:** Validates the configuration file `config.yaml` against the JSON schema `schema.json`, reporting any missing required fields or type mismatches before running downstream bioinformatics analyses.

### Parse arguments from a YAML file and output as JSON
**Args:** `parse --input pipeline_params.yaml --output-format json`
**Explanation:** Reads parameter definitions from the YAML input file and converts them to JSON format, useful for interoperability with tools that require JSON configuration.

### Check if a configuration has all required pipeline arguments
**Args:** `check --required name=sample_id,input_file,output_dir config.yaml`
**Explanation:** Verifies that the configuration contains the three required keys (name, input_file, output_dir), exiting with error code 1 if any are missing, preventing incomplete pipeline runs.

### Generate a boilerplate argument parser template
**Args:** `generate template --tool my_pipeline --output parser_template.yaml`
**Explanation:** Creates a template YAML file with standard arguments (input, output, threads, verbose) that can be customized for a new bioinformatics tool called "my_pipeline".

### Convert a legacy INI format configuration to YAML
**Args:** `convert --from ini --to yaml legacy_config.ini > modern_config.yaml`
**Explanation:** Reads the legacy INI-format configuration and outputs equivalent YAML to stdout, enabling migration from older pipeline configurations to the current argutils format.

### Validate FASTQ input arguments for a read processing tool
**Args:** `validate-inputs --files R1.fastq.gz,R2.fastq.gz --format fastq --require-paired`
**Explanation:** Ensures both FASTQ input files exist, are readable, and are in proper paired-end format before proceeding with tools like BWA or Bowtie.

### List all supported configuration schema versions
**Args:** `list-schemas --available`
**Explanation:** Displays all schema versions available in the installed argutils package, helping users select the appropriate schema for their pipeline version.

### Dry-run a configuration validation without executing
**Args:** `validate --dry-run --verbose config.yaml`
**Explanation:** Performs a dry-run validation of the configuration, printing all checks that would be performed without actually executing, useful for debugging configuration issues.