---
name: biopet-sampleconfig
category: Pipeline Configuration
description: A Biopet toolkit utility for generating, validating, and managing sample-level configuration files in JSON or YAML format for downstream bioinformatics pipeline processing.
tags:
  - biopet
  - configuration
  - sample-sheets
  - json
  - yaml
  - pipeline-setup
author: AI-generated
source_url: https://biopet.readthedocs.io/en/latest/
---

## Concepts

- **Sample Configuration Data Model**: The tool operates on a data model representing samples as JSON or YAML objects containing fields such as sample ID, read group information, input file paths, sequencing platform, and sample metadata (e.g., tissue type, treatment condition). Each sample entry maps a unique identifier to its associated attributes.
- **I/O Formats**: Input can be provided as a pre-existing config file (JSON or YAML), a tab-delimited sample sheet, or command-line arguments. Output is generated as a validated configuration file in the chosen format (JSON by default, YAML when requested). The tool enforces schema compatibility between input and output formats.
- **Validation Behavior**: On generation or update operations, the tool validates all fields against the Biopet schema—checking that sample IDs contain no spaces or special characters, that referenced file paths exist on the filesystem, and that required fields (e.g., sample_id) are present. Validation errors cause the tool to exit with a non-zero status and print the specific field failures.
- **Companion Binary**: The companion tool `biopet-sampleconfig-validate` reads a configuration file and reports schema compliance without modifying the file, enabling pre-flight checks in CI/CD pipelines or before launching expensive compute jobs.

## Pitfalls

- **Missing Required Fields**: Omitting mandatory fields such as `sample_id` during inline configuration causes silent failures where the tool generates a partial config file that downstream Biopet tools will reject, wasting compute time on pipelines that fail at sample initialization.
- **Mismatched Format Versions**: Specifying YAML input to a tool invocation that defaults to JSON output (or vice versa) without explicit format flags results in silent format conversion that may truncate fields containing complex nested objects, leading to incomplete sample metadata in the pipeline configuration.
- **Relative File Paths**: Using relative paths for input files (e.g., `data/samples.csv`) when the tool is invoked from a different working directory produces "file not found" errors, and worse, if the tool partially succeeds, it may write the config file to an unexpected location.
- **Overwriting Configurations Unintentionally**: The tool may overwrite existing config files with the same output path without prompting when run non-interactively, causing irreversible loss of manually curated sample metadata if not backed up beforehand.

## Examples

### Create a basic JSON configuration file for a single sample
**Args:** `-o sample_config.json --sample-id SRR123456 --input-files /data/fastq/SRR123456_R1.fastq.gz /data/fastq/SRR123456_R2.fastq.gz --platform Illumina`
**Explanation:** This generates a JSON configuration file with a single sample entry, specifying the sample identifier, paired-end FASTQ input files, and sequencing platform.

### Generate a YAML configuration file for multiple samples from a sample sheet
**Args:** `-o multi_sample.yaml --format yaml --sample-sheet samples.tsv --platform ILLUMINA --validate`
**Explanation:** This reads a tab-delimited sample sheet containing multiple sample rows and produces a validated YAML configuration file, ensuring schema compliance before pipeline execution.

### Validate an existing configuration file without modification
**Args:** `biopet-sampleconfig-validate sample_config.json`
**Explanation:** The companion binary reads the existing JSON configuration file and reports any schema violations or missing required fields without generating or modifying any output.

### Add a new sample to an existing configuration file
**Args:** `-i existing_config.json -o updated_config.json --add-sample --sample-id SRR789012 --input-files /data/fastq/SRR789012_R1.fastq.gz --platform ONT`
**Explanation:** This loads an existing configuration file, adds a new sample entry with the provided attributes, and writes the merged result to a separate output file to preserve the original.

### Generate a configuration with custom metadata tags
**Args:** `-o annotated_config.json --sample-id SRR456789 --input-files /data/bam/SRR456789.bam --platform PACBIO --metadata tissue=lung --metadata treatment=control`
**Explanation:** This creates a configuration entry that includes custom key-value metadata tags for downstream pipeline steps that require sample-specific annotation beyond standard fields.