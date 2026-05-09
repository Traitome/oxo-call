---
name: bcbio-nextgen-vm
category: NGS Analysis / Variant Calling
description: A VirtualBox-based distribution of bcbio-nextgen for standardized next-generation sequencing analysis, providing a pre-configured virtualized environment for processing FASTQ files through alignment, variant calling, and quality control workflows without native dependency installation.
tags: [ngs, variant-calling, rna-seq, chip-seq, virtualbox, virtualized, qc, alignment, docker-alternative]
author: AI-generated
source_url: https://bcbio-nextgen.readthedocs.io/en/latest/

## Concepts

- The VM distribution wraps bcbio-nextgen in VirtualBox, ensuring a consistent computational environment across different host operating systems without requiring users to install language runtimes, biological libraries, or alignment tools natively.

- Analysis workflows are defined in YAML configuration files containing input sample definitions, reference genome selections, algorithm choices for alignment and variant calling, and output format specifications.

- Resource forecasting (`bcbio-nextgen-vm forecast`) estimates memory, CPU, and storage requirements before execution, preventing runtime failures due to insufficient host resources.

- The tool auto-detects and downloads required reference genomes (hg38, GRCh37, etc.) on first use, but supports custom prepared references for non-model organisms or proprietary assemblies.

- Output artifacts include aligned BAM files, raw and filtered VCF variants, per-sample and aggregated QC reports in JSON/HTML, and optional cohort-level joint-calling outputs.

## Pitfalls

- Attempting to run the VM without hardware virtualization enabled in BIOS causes cryptic "VT-x/AMD-V not available" errors during VirtualBox initialization; verify CPU virtualization extensions are active before troubleshooting the bcbio distribution itself.

- Specifying insufficient RAM for the VM via `--memory` results in OOM-killed processes during alignment or variant calling, corrupting intermediate files and forcing full reruns; always forecast resources first with `bcbio-nextgen-vm forecast`.

- Using mixed encoding (e.g., UTF-16LE) in YAML config files causes PyYAML parsing failures with unhelpful line-number errors; always save project configuration files as UTF-8 plain text.

- Running multiple bcbio-nextgen-vm instances simultaneously from the same working directory overwrites shared CSV tracking files, leading to missing sample records in final reports; use isolated project directories per run.

- Forgetting to register the VM image (`bcbio-nextgen-vm import`) before the first `run` causes "VM not found" errors that appear to be project-level configuration problems; the import step is mandatory on first setup or after VM deletion.

## Examples

### Run a complete germline variant calling analysis from FASTQ inputs

**Args:** run -i input.yaml -n 4

**Explanation:** The `run` command executes the full configured workflow against input samples, `-i` points to the YAML configuration file, and `-n 4` limits parallelism to 4 concurrent sample processes to avoid host resource exhaustion.

### Estimate computational resource requirements before execution

**Args:** forecast -i input.yaml

**Explanation:** The `forecast` subcommand parses the project configuration and reports required RAM, disk space, and CPU core estimates, enabling correct VM sizing and host resource allocation planning.

### Generate a validated template project configuration

**Args:** template -p myproject

**Explanation:** The `template` subcommand creates a starter project directory with a validated `input.yaml` containing documented sections for samples, references, and algorithms, preventing misconfigured first-time runs.

### Display the VM GUI console for interactive debugging

**Args:** display

**Explanation:** The `display` command opens the VirtualBox console window attached to the running VM, useful for inspecting logs or interactively debugging tool failures that don't surface in command-line output.

### Export a specific sample's final VCF and BAM outputs

**Args:** export -p myproject -s Sample1 -o ./results

**Explanation:** The `export` subcommand copies finished output artifacts for named sample to a local directory, avoiding navigation of the VM's internal project structure and enabling direct host filesystem access to downstream tools.

### Start a delayed or rescheduled analysis run

**Args:** run -i input.yaml --delay 3600

**Explanation:** The `--delay` flag postpones the workflow start by the specified seconds (3600 = 1 hour), useful for scheduled execution during off-peak host machine hours or after upstream sequencer processing completes.

### Initialize bcbio-nextgen-vm with an imported appliance file

**Args:** import bcbio-vm-1.0.ova

**Explanation:** The `import` subcommand registers an OVA-format VirtualBox appliance into VirtualBox's global registry, creating a named VM instance required before any `run` or `display` commands can execute.