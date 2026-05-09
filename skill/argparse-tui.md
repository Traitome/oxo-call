---
name: argparse-tui
category: Bioinformatics Tools / Utilities
description: An interactive TUI (Text User Interface) wrapper for Python argparse-based scripts that provides interactive prompting, auto-completion, and validation of command-line arguments through a terminal-based user interface.
tags:
  - argparse
  - tui
  - command-line
  - interface
  - python
  - interactive
  - prompting
  - bioinformatics
  - utilities
author: AI-generated
source_url: https://github.com/argparse-tui/argparse-tui
---

## Concepts

- argparse-tui wraps existing Python scripts that use argparse, providing an interactive menu-driven interface where users can navigate through argument options using arrow keys, select from choicelists, and receive real-time validation feedback.
- The tool reads the argparse configuration from the target script's argument parser definition (typically via `argparse.ArgumentParser()`) and automatically generates corresponding TUI widgets including text inputs, checkboxes, radio buttons, and dropdown selects.
- Input/Output formats include reading Python scripts (.py files) containing argparse definitions as input, and outputting validated argument strings that can be passed directly to the original script or executed via subprocess call.
- Key behaviors include automatic type conversion (e.g., converting string inputs to integers, floats, or paths), enforce choices from predefined options, support for required/optional arguments, and preserve help text from the original argparse setup.

## Pitfalls

- **Specifying an incorrect input script path**: If the target Python script lacks proper argparse initialization or uses a different argument parsing library (like click or docopt), argparse-tui will fail to detect arguments and produce an empty interface with no selectable options.
- **Forgetting to export parsed arguments**: When using argparse-tui programmatically (as a library), failing to capture the return value from the TUI session results in arguments being discarded, requiring users to re-enter all parameters.
- **Mismatching argument types**: Manually overriding argparse types in the TUI configuration (e.g., setting a numeric field as free-text) bypasses built-in validation, leading to runtime errors when the generated argument string is passed to the underlying script.
- **Ignoring required argument constraints**: The TUI allows navigation between fields, but submitting without filling required arguments produces an error only at the final step, potentially wasting time if the user intended to skip optional flags.

## Examples

### Generate an interactive interface for a bioinformatics alignment script
**Args:** `/path/to/bwa_align.py`
**Explanation:** This loads the argparse configuration from the bwa_align.py script and launches an interactive TUI where users can fill in arguments like input FASTQ files, reference genome, and alignment parameters without memorizing flag syntax.

### Run with predefined config values loaded
**Args:** `--config alignment_config.yaml /path/to/bwa_align.py`
**Explanation:** This populates the TUI fields with values from a YAML configuration file, allowing users to review and modify preset values rather than entering them manually each time.

### Output argument string to stdout instead of executing
**Args:** `--dry-run --output-format string /path/to/bwa_align.py`
**Explanation:** Instead of running the wrapped script, argparse-tui prints the generated argument string (e.g., `-i reads.fq -ref genome.fa -o output.sam`) to stdout for verification before manual execution.

### Enable vim-style keybindings for navigation
**Args:** `--keymap vim /path/to/vcf_filter.py`
**Explanation:** This configures the TUI to use vim-style keybindings (h/j/k/l for navigation, `i` for insert mode) instead of default Emacs-style bindings, appealing to users familiar with vim editor.

### Save session history for later reuse
**Args:** `--history-file session.history /path/to/gatk_haplotype.py`
**Explanation:** This records all arguments entered during the TUI session to a history file, enabling recall of previous configurations via up/down arrow keys in subsequent runs.