---
name: pixi
category: package-management
description: Modern, extremely fast conda-compatible package manager and environment tool; installs packages from conda-forge and bioconda into isolated per-project or global environments
tags: [conda, environment, package, bioconda, conda-forge, pixi, rattler, fast]
author: oxo-call built-in
source_url: "https://pixi.sh/latest/"
---

## Concepts
- Pixi uses the rattler solver, making it significantly faster than classic conda at resolving and installing packages.
- Projects are defined by a `pixi.toml` manifest in the project root; running `pixi init` creates one automatically.
- Default channel is `conda-forge`; add `bioconda` globally with `pixi config set default-channels '["conda-forge", "bioconda"]'`.
- `pixi global install <pkg>` installs a package into a dedicated isolated environment and exposes its binaries on PATH — equivalent to `pipx` but for conda packages.
- `pixi add <pkg>` adds a dependency to the current project's `pixi.toml` and updates the lockfile (`pixi.lock`).
- `pixi run <cmd>` executes a command inside the project environment without requiring an explicit activation step.
- `pixi shell` drops into an interactive shell with the project environment activated; exit with `exit`.
- Environments are stored under `.pixi/envs/` in the project directory, keeping them fully project-local.
- Multiple environments (e.g. `default`, `dev`, `test`) can be defined in one `pixi.toml` under `[feature.<name>]` sections.
- The `pixi.lock` lockfile pins exact package versions and hashes for fully reproducible installs across machines.

## Pitfalls
- DANGER: `pixi global remove <pkg>` permanently removes the global environment for that package.
- Running `pixi add` without a `pixi.toml` in the working tree will fail; initialise the project first with `pixi init`.
- Mixing pip (`pip install`) inside a pixi environment can break the lockfile; use `pixi add --pypi <pkg>` instead for PyPI-only packages.
- `pixi update` regenerates `pixi.lock` and may change transitive dependency versions; review the diff before committing.
- The channel search order matters: list higher-priority channels first in `default-channels`; conda-forge should precede bioconda.
- `pixi shell` and `pixi run` only work inside a directory that contains (or is nested under) a `pixi.toml`.
- Global installs (`pixi global install`) are per-user and stored under `~/.pixi/envs/<pkg>/`; they are not visible to project environments.
- On systems where conda/mamba is already on PATH, `pixi` manages its own independent package cache at `~/.pixi/cache` — the two caches are not shared.

## Examples

### initialise a new pixi project in the current directory
**Args:** `init`
**Explanation:** creates `pixi.toml` and `pixi.lock` in the current directory; sets up the default conda-forge channel

### configure pixi to use bioconda in addition to conda-forge globally
**Args:** `config set default-channels '["conda-forge", "bioconda"]'`
**Explanation:** updates the global pixi config so every new project automatically searches both channels

### install samtools globally so it is available system-wide
**Args:** `global install samtools`
**Explanation:** creates an isolated global environment for samtools and exposes the `samtools` binary on PATH

### add a package dependency to the current project
**Args:** `add bwa`
**Explanation:** resolves bwa from the configured channels, adds it to pixi.toml, and updates pixi.lock

### add a package from a specific channel to the current project
**Args:** `add -c bioconda star`
**Explanation:** -c bioconda overrides the search to bioconda; useful when a package is not on conda-forge

### run a command inside the project environment without activating it
**Args:** `run samtools sort -@ 4 -o sorted.bam input.bam`
**Explanation:** executes samtools inside the pixi-managed environment transparently; no manual activate needed

### open an interactive shell with the project environment activated
**Args:** `shell`
**Explanation:** spawns a sub-shell with all project packages on PATH; type `exit` to return to the outer shell

### install multiple bioinformatics tools globally in one command
**Args:** `global install fastp multiqc fastqc`
**Explanation:** each tool gets its own isolated global environment; all binaries are linked onto PATH

### update all packages in the current project to their latest compatible versions
**Args:** `update`
**Explanation:** re-solves all dependencies and regenerates pixi.lock; review changes before committing the lockfile

### list all packages installed in the current project environment
**Args:** `list`
**Explanation:** shows every installed package with its version, build string, and source channel

### remove a package dependency from the current project
**Args:** `remove star`
**Explanation:** removes star from pixi.toml and updates pixi.lock; the package is uninstalled from the project env

### install all dependencies from an existing pixi.lock for a reproducible environment
**Args:** `install`
**Explanation:** reads pixi.lock and installs exact pinned versions; use after cloning a repo to reproduce the environment
