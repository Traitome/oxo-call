---
name: mamba
category: package-management
description: Fast, drop-in replacement for conda using libsolv solver; manages Python/R/C environments and packages from conda channels
tags: [conda, environment, package, bioconda, conda-forge, libsolv, solver, mamba]
author: oxo-call built-in
source_url: "https://mamba.readthedocs.io/"
---

## Concepts
- Mamba is a fully compatible drop-in replacement for conda; all `conda` commands work as `mamba` commands.
- Uses the libsolv dependency solver for dramatically faster package resolution than classic conda.
- Default environments directory mirrors conda: `~/mambaforge/envs/` or `~/miniforge3/envs/` depending on install.
- Root conda installation prefix: `~/mambaforge/` (Mambaforge installer) or `~/miniforge3/` (Miniforge installer). Check with `mamba info --base`.
- Channel priority is configured in `~/.condarc`; set `channel_priority: strict` to prefer conda-forge/bioconda packages.
- `mamba activate <env>` is identical to `conda activate <env>`; shell must be initialised first (`mamba init`).
- Micromamba (`micromamba`) is a standalone static binary requiring no base environment; configuration via `~/.mambarc`.
- Package cache is stored at `~/mambaforge/pkgs/` (or equivalent base prefix/pkgs/) — shared across environments.
- Environment YAML files (environment.yml) use the same schema as conda for reproducible builds.
- The base Mambaforge/Miniforge installation activates automatically; child envs are activated on top with `mamba activate`.
- `mamba repoquery` provides advanced package and dependency queries with --recursive flag.
- `--explicit` generates exact package URLs for lockfile-style reproducibility.
- `--prune` removes orphaned packages when updating; --freeze-installed prevents updating existing packages.
- `--override-channels` ignores defaults/.condarc channels; useful for air-gapped or controlled environments.
- `--offline` mode uses cached repodata without network access for reproducible installs.
- `--json` outputs machine-parseable JSON for programmatic use in pipelines.

## Pitfalls
- `mamba env remove -n <env>` permanently deletes the environment and all packages within it.
- `mamba install` without an active non-base environment modifies the base environment; always activate a project env first.
- Mixing pip installs inside a mamba environment can break dependency resolution; install conda packages first, pip packages last.
- `mamba update --all` may downgrade packages to satisfy constraints; review the plan before accepting.
- `mamba clean --all` removes the package cache and all unused tarballs; this frees disk but re-downloads packages on next install.
- Shell integration requires running `mamba init <shell>` once per shell type; without it, `mamba activate` will fail.
- On HPC clusters, module-loaded anaconda may shadow the user mamba install; check `which mamba` before running.
- Large environment locks (conda-lock) must be regenerated after any package change; do not hand-edit lockfiles.
- `--strict-channel-priority` can cause unsolvable environments if higher-priority channels lack required packages.
- `--freeze-installed` prevents updates but may block security patches; use with caution in production.
- `--offline` requires cached repodata; first run must be online to populate the cache.
- `mamba repoquery` recursive queries can be slow for large dependency trees; use judiciously.
- Micromamba and mamba have slightly different CLI behaviors; scripts may need adjustment when switching.

## Examples

### create a new environment named myenv with python 3.11
**Args:** `create -n myenv python=3.11`
**Explanation:** -n sets the environment name; python=3.11 pins the Python version; resolves deps with libsolv

### install samtools and bwa in the biotools environment
**Args:** `install -n biotools -c bioconda -c conda-forge samtools bwa`
**Explanation:** -n targets the named environment; -c adds channels in priority order; packages installed together for consistent resolution

### create environment from YAML specification file
**Args:** `env create -f environment.yml`
**Explanation:** env create reads name, channels, and dependencies from the YAML; produces reproducible environments across machines

### export current environment as YAML
**Args:** `env export -n myenv --no-builds > environment.yml`
**Explanation:** --no-builds omits platform-specific build strings, making the YAML more portable across OS/arch

### activate the rnaseq environment (bash)
**Args:** `activate rnaseq`
**Explanation:** switches the active environment to rnaseq; prepends env bin to PATH; equivalent to conda activate

### list all conda/mamba environments and their locations
**Args:** `env list`
**Explanation:** shows all envs with their full filesystem paths (~/mambaforge/envs/ hierarchy by default)

### search for a package across conda-forge and bioconda channels
**Args:** `search -c conda-forge -c bioconda star`
**Explanation:** searches both channels for packages matching 'star'; shows available versions and build strings

### remove unused packages and tarballs from the package cache
**Args:** `clean -y --all`
**Explanation:** -y skips confirmation; --all removes unused tarballs, packages, index cache, and lock files to free disk space

### update all packages in the ngs environment
**Args:** `update -n ngs --all`
**Explanation:** upgrades all packages in the ngs env to the latest compatible versions; review the plan before accepting

### install a specific package version
**Args:** `install -n myenv -c conda-forge numpy=1.26`
**Explanation:** pins to exactly version 1.26; using -c conda-forge ensures the conda-forge build is preferred

### list packages installed in an environment
**Args:** `list -n myenv`
**Explanation:** shows all installed packages with versions and channel sources in the specified environment

### show mamba configuration and base prefix
**Args:** `info`
**Explanation:** prints active env, base prefix path (e.g. ~/mambaforge/ or ~/miniforge3/), channels, and platform info

### query package dependencies recursively
**Args:** `repoquery depends -c bioconda samtools --recursive`
**Explanation:** repoquery shows dependency tree; --recursive lists all transitive dependencies; useful for understanding package requirements

### create environment with explicit package URLs for reproducibility
**Args:** `create -n locked_env --file explicit_packages.txt`
**Explanation:** --file with explicit URLs ensures exact package versions; generated via conda list --explicit for lockfile-style installs

### install packages without updating existing ones
**Args:** `install -n myenv --freeze-installed numpy pandas`
**Explanation:** --freeze-installed prevents updating already-installed packages; useful for adding tools without disrupting environment

### update environment removing orphaned packages
**Args:** `update -n myenv --prune --all`
**Explanation:** --prune removes packages no longer needed by any dependency; cleans up after major package removals

### create environment with strict channel priority
**Args:** `create -n strict_env -c conda-forge -c bioconda --strict-channel-priority python=3.11 samtools`
**Explanation:** --strict-channel-priority only uses lower-priority channels if higher-priority ones lack the package

### search for package with detailed info
**Args:** `repoquery search -c bioconda "star>=2.7"`
**Explanation:** repoquery search finds packages matching version constraints; supports comparison operators like >=, <, =

### run mamba in offline mode using cached repodata
**Args:** `create -n offline_env --offline -c conda-forge python=3.11`
**Explanation:** --offline uses locally cached package index; requires previous online run to populate cache; for air-gapped systems

### export environment as explicit package list with hashes
**Args:** `list -n myenv --explicit --md5 > explicit_packages.txt`
**Explanation:** --explicit outputs exact package URLs; --md5 includes checksums for verification; most reproducible export format

### install from conda-lock lockfile with micromamba
**Args:** `micromamba create -n locked_env -f conda-lock.yml`
**Explanation:** micromamba can install conda-lock generated lockfiles without conda-lock installed; for reproducible deployments
