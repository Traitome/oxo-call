---
name: argparse2tool
category: Script Conversion / Code Generation
description: Converts Python scripts using the argparse module into standalone command-line executable wrappers, enabling Python-based CLI tools to be invoked like native shell commands.
tags: [argparse, python, cli, wrapper, conversion, code-generation]
author: AI-generated
source_url: https://github.com/bioconda-conda-forge/argparse2tool-feedstock
---

## Concepts

- argparse2tool reads Python source files containing `argparse.ArgumentParser()` instances and automatically generates shell scripts or executable wrappers that expose the parser's arguments as native CLI flags.
- The tool analyzes the `add_argument()` calls within the Python script, extracting positional arguments, optional flags (`-x`/`--xoo`), argument types, default values, and help strings to construct equivalent CLI interfaces.
- Generated wrappers invoke the original Python interpreter and script, passing CLI arguments in the correct order and format expected by the `ArgumentParser`, preserving type validation and error handling behavior.
- Output formats include self-contained bash wrapper scripts (`.sh`), executable files with shebang lines, and optionally manifest files listing dependencies and metadata for packaging purposes.
- The tool supports Python 2 and Python 3 argparse modules, handling differences in `argparse` API between versions (e.g., `action='store_const'` syntax variations).

## Pitfalls

- If the Python script uses `argparse.FileType()` for file argument handling, the generated wrapper may not preserve file mode or encoding assumptions, causing silent failures or corrupted output when binary or non-UTF8 files are passed.
- Scripts with nested subparsers (`add_subparsers()`) are only partially converted; only the top-level parser arguments are exposed, and subcommand routing logic is lost in simple wrapper generation modes.
- Running `argparse2tool` on scripts that import `argparse` conditionally (behind `try/except ImportError`) produces empty or broken wrappers because the tool cannot statically resolve the argument definitions.
- Generated wrappers hardcode the absolute path to the Python interpreter at generation time, breaking portability when the script is moved to a system with a different Python installation path (e.g., `/usr/bin/python3` vs. `/opt/conda/bin/python3`).
- Argument choices constraints (`choices=['a', 'b']`) are converted but not validated at wrapper invocation time; users receive Python-side error messages that reference the original script's internal naming rather than the wrapper context.

## Examples

### Convert a single Python script with all default options

**Args:** `script.py`
**Explanation:** Converts `script.py` containing an `ArgumentParser` instance into an executable wrapper using default settings, outputting a bash script in the current directory with the same base name.

### Generate a wrapper with a custom output filename

**Args:** `script.py --output my_tool`
**Explanation:** Converts the argparse script and writes the generated wrapper to a file named `my_tool` instead of the default `script` filename.

### Include argparse dependencies in generated manifest

**Args:** `script.py --package-name bioseq --version 1.0.0`
**Explanation:** Generates both the wrapper script and a manifest YAML recording the package name, version, and inferred dependencies for later packaging or conda build workflows.

### Override the Python interpreter path in the generated wrapper

**Args:** `script.py --python-bin /opt/custom/bin/python3`
**Explanation:** Embeds a non-default Python interpreter path (`/opt/custom/bin/python3`) in the generated wrapper's shebang line, useful for environments with multiple Python installations.

### Create a wrapper that exposes environment variables to the Python script

**Args:** `script.py --export-env VAR1 VAR2`
**Explanation:** Configures the generated wrapper to accept environment variable names (`VAR1`, `VAR2`) that will be passed as arguments to the underlying Python script, enabling configuration without CLI flag modification.