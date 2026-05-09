---
name: biobb_common
category: Workflow Management
description: BioExcel Building Blocks common utilities and base functions shared across all BioBB modules. Provides core classes, configuration management, file utilities, and a CLI wrapper for launching and managing molecular dynamics workflows.
tags:
  - biobb
  - molecular-dynamics
  - workflow
  - gromacs
  - python
  - building-blocks
  - common-utilities
  - cli
author: AI-generated
source_url: https://biobb.readthedocs.io/en/latest/biobb_common.html
---

## Concepts

- **Base Classes (`CmdSSH`, `CmdConfig`)**: biobb_common provides reusable Python base classes that wrap shell commands and manage configuration across all BioBB modules. All building block classes inherit from these bases, meaning understanding `CmdConfig` options (config file paths, output directories, logging levels) applies uniformly to every derived tool.
- **File and Path Utilities (`manage_output`, `folders_checkam`)**: The library centralizes file I/O handling including automatic creation of output directories, SHA256 checksum verification for downloaded reference files, and glob/nlst pattern matching. All paths can be absolute or relative to the current working directory.
- **CLI Launch Mode (`biobb_common launch`)**: The CLI entry point accepts a YAML or JSON workflow definition file and executes building block steps sequentially. Inputs and outputs between steps are connected via named channels in the workflow definition, not via shell piping.
- **Input/Output Channels**: Workflow steps declare named input and output channels typed as `inputFile`, `outputFile`, `inputPath`, or `outputPath`. Channels map step outputs to downstream step inputs by name, enabling modular, reusable workflow definitions.
- **Configuration Hierarchy**: Config settings resolve in order: default config → user config file (~/.biobb/biobb.conf) → environment variables (`BIOBB_*`). Environment variables take highest precedence, useful for containerized or HPC job submissions.

## Pitfalls

- **Specifying an output directory that does not exist**: If `output_path` points to a non-existent directory, the tool raises `FileNotFoundError` or silently writes nowhere depending on the module. Always create the parent directory with `mkdir -p` before executing the CLI.
- **Type mismatches in workflow channel connections**: Connecting an `outputFile` channel to an `inputPath` channel (or vice versa) causes a type-validation error at workflow parse time, even before any subprocess runs. Verify that source and sink channel types match in the workflow YAML.
- **Forgetting to set `--config` for non-default binary paths**: When GROMACS or AMBER binaries are installed in a non-standard location, biobb_common defaults will not find them. Without passing `--config` pointing to a YAML file with `binary_path` entries, steps fail with `ExecutableNotFoundError`.
- **Overwriting input files by default**: Many biobb_common tools write output files to the same base name as input if `output_file` is not specified, overwriting source PDB/MOL2 files irreversibly. Always explicitly set a distinct output file path.
- **Running workflows without checking the `.biobb/` cache**: Reference structures cached in the local `.biobb/` directory persist between runs. If upstream data has changed, stale cached files cause silent inconsistencies. Delete `.biobb/` or set `BIOBB_CACHE_DISABLED=1` when re-running with updated references.

## Examples

### Launch a workflow defined in a YAML file
**Args:** `launch --workflow_file workflow_gromacs.yaml`
**Explanation:** The `launch` subcommand reads a workflow YAML definition and executes each step sequentially, connecting inputs and outputs through named channels declared in the definition file.

### List all available building blocks and their required parameters
**Args:** `list --all`
**Explanation:** The `--all` flag prints every building block class available across all installed BioBB modules, including each step's required and optional input/output channels.

### Execute a single building block step directly with explicit config
**Args:** `run --block gromacs.StructureMinimization --input_structure input.pdb --config gromacs.conf --output_minimized output.gro`
**Explanation:** The `run` command executes one building block class in isolation, bypassing the workflow manager, which is useful for debugging individual steps before full workflow execution.

### Set binary paths via environment variable for a containerized run
**Args:** `launch --workflow_file md_setup.yaml --envvar "BINARY_GMX=/opt/gromacs/bin/gmx"`
**Explanation:** Passing binary overrides via `--envvar` avoids editing config files inside containers; the environment variable takes precedence over the config file for that run only.

### Generate a skeleton workflow YAML with placeholder channels
**Args:** `scaffold --step gromacs.EnergyMinimization --step gromacs.NvtEquilibration --output workflow_skeleton.yaml`
**Explanation:** The `scaffold` command writes a minimal workflow YAML with declared channels and placeholder paths for the specified steps, serving as a starting template that you then populate with real paths.