---
name: conda
category: package-management
description: Open-source package and environment management system for Python and data science
tags: [conda, environment, python, package, data-science, bioconda, anaconda, mamba, micromamba]
author: oxo-call built-in
source_url: "https://docs.conda.io/projects/conda/en/stable/commands/"
---

## Concepts

- conda manages both Python packages AND non-Python dependencies (C libraries, compilers, bioinformatics tools). It uses environments to isolate dependency sets: 'conda create', 'conda activate', 'conda deactivate'.
- Environments are isolated from each other. Always activate an environment before installing into it: 'conda activate myenv'. Without activation, packages go into base — avoid polluting base.
- Channel priority determines where packages are found. Common channels: defaults, conda-forge (community), bioconda (bioinformatics). Specify with -c: 'conda install -c conda-forge -c bioconda samtools'.
- conda env create -f environment.yml creates an environment from a YAML spec file. Export current environment with 'conda env export > environment.yml'. Use --no-builds for cross-platform compatibility.
- mamba is a faster drop-in replacement for conda using a C++ solver. If available, replace 'conda install' with 'mamba install' for dramatically faster dependency solving.
- conda remove uninstalls packages from the active environment. Use 'conda env remove -n envname' to delete an entire environment. Both operations are reversible only if you have an environment.yml backup.
- --from-history exports only explicitly installed packages (not dependencies) for better cross-platform compatibility.
- conda rename renames existing environments without recreating them.
- conda doctor/check displays health reports for environment diagnostics.
- conda clean removes unused packages and caches to free disk space.
- conda compare compares packages between different environments.

## Pitfalls

- 'conda env remove -n myenv' permanently deletes the entire environment and all its packages. Export the environment first: 'conda env export > myenv.yml'.
- 'conda remove --all' in an activated environment removes ALL packages including conda itself from that environment, making it unusable.
- Mixing pip and conda installs in the same environment can break dependency tracking. Always install with conda first; use pip only for packages unavailable in conda channels.
- Without -y, conda prompts for confirmation before installing. In scripts, add -y to avoid interactive prompts.
- conda update conda should be run in the base environment, not in a project environment. Activate base first: 'conda activate base && conda update conda'.
- conda activate myenv only works after 'conda init' has been run for your shell. In scripts, use 'source activate myenv' or 'conda run -n myenv command' instead.
- --no-builds is essential for cross-platform sharing; build strings are platform-specific.
- --from-history only includes explicitly installed packages, making exports much more portable.
- conda env update -f environment.yml updates existing environment; different from conda update.
- Channel order matters: conda searches channels in order, first match wins.

## Examples

### create a new Python environment with a specific version
**Args:** `create -n myproject python=3.11`
**Explanation:** conda create subcommand; -n myproject environment name; python=3.11 pins Python version; activate with 'conda activate myproject'

### install packages into the active environment from conda-forge
**Args:** `install -c conda-forge numpy pandas matplotlib`
**Explanation:** conda install subcommand; -c conda-forge channel; numpy pandas matplotlib package names; add -y to skip confirmation

### create an environment from a YAML specification file
**Args:** `env create -f environment.yml`
**Explanation:** conda env create subcommand; -f environment.yml YAML file; reads packages and channels from file; creates environment with name specified in YAML

### export current environment to a YAML file for sharing
**Args:** `env export --no-builds -f environment.yml`
**Explanation:** conda env export subcommand; --no-builds omits build strings for cross-platform compatibility; -f environment.yml output file

### list all installed packages in the active environment
**Args:** `list`
**Explanation:** conda list subcommand; shows all packages with versions and build strings in the currently active environment

### update all packages in the active environment
**Args:** `update --all -y`
**Explanation:** conda update subcommand; --all updates every package; -y skips confirmation; run after activating target environment

### remove a package from the active environment
**Args:** `remove old-package -y`
**Explanation:** conda remove subcommand; old-package package name to remove; -y skips confirmation; removes package and orphaned dependencies

### list all environments
**Args:** `env list`
**Explanation:** conda env list subcommand; shows all conda environments with paths; active environment marked with *

### search for available versions of a package
**Args:** `search -c conda-forge bioconductor-deseq2`
**Explanation:** conda search subcommand; -c conda-forge channel; bioconductor-deseq2 package name; shows all available versions

### run a command in a specific environment without activating
**Args:** `run -n myenv python script.py`
**Explanation:** conda run subcommand; -n myenv environment name; python script.py command to run; runs in named env without conda activate; useful in scripts

### export environment with only explicitly installed packages
**Args:** `env export --from-history -f environment.yml`
**Explanation:** conda env export subcommand; --from-history exports only explicitly installed packages; -f environment.yml output file; more portable across platforms

### rename an existing environment
**Args:** `rename old_env_name new_env_name`
**Explanation:** conda rename subcommand; old_env_name current name; new_env_name new name; changes environment name without recreating; faster than export/create/remove workflow

### clean unused packages and caches
**Args:** `clean --all -y`
**Explanation:** conda clean subcommand; --all removes unused packages, tarballs, and caches; -y skips confirmation; frees significant disk space

### check environment health
**Args:** `doctor`
**Explanation:** conda doctor subcommand; displays health report for current environment; checks for missing files and dependency issues

### compare packages between environments
**Args:** `compare myenv1 myenv2`
**Explanation:** conda compare subcommand; myenv1 myenv2 environment names to compare; shows differences in packages; useful for debugging version issues

### update environment from YAML file
**Args:** `env update -f environment.yml --prune`
**Explanation:** conda env update subcommand; -f environment.yml YAML file; --prune removes packages not in YAML; modifies existing environment to match YAML

### clone an existing environment
**Args:** `create -n new_env --clone existing_env`
**Explanation:** conda create subcommand; -n new_env new environment name; --clone existing_env existing environment to copy; creates exact copy for safe testing
