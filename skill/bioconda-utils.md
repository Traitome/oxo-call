---
name: bioconda-utils
category: package-management / build-automation
description: A suite of command-line utilities for building, linting, and managing conda packages in the Bioconda ecosystem. Provides recipe linting, multi-container builds, channel aggregation, dependency locking, and pinnings update workflows.
tags:
  - conda
  - bioconda
  - package-build
  - recipe-linting
  - containerized-builds
  - bioinformatics-infrastructure
  - dependency-management
author: AI-Generated
source_url: https://github.com/bioconda/bioconda-utils
---

## Concepts

- **Conda recipe as primary data model.** A Bioconda package is defined by a `meta.yaml` (conda recipe) that specifies name, version, source URL, build dependencies, run dependencies, build script, and test commands. The entire build, lint, and lock pipeline operates on this recipe YAML structure.
- **Multi-container architecture via `--docker` and `--container-engine`.** The `bioconda-utils build` subcommand spawns build containers using Docker or Singularity, enabling isolated, reproducible builds across Linux distributions. The `--mml` (minimum minor version of Linux) flag controls glibc compatibility, ensuring built packages link correctly against the target image's libc.
- **Build matrix via `build: number` field and environment constraints.** When multiple build entries exist with incrementing `build: number` values, `bioconda-utils lint` treats them as a matrix of configurations. The lint command resolves selectors (`#[linux]` etc.) and conditional dependencies before evaluating recipe consistency.
- **Channel aggregation and artifact staging.** The `bioconda-utils aggregate` command merges multiple per-recipe build outputs into a consolidated conda channel directory, placing `*.tar.bz2` or `*.conda` package artifacts alongside `repodata.json` indexes. This is the final step before publishing to quay.io or a file-based channel.
- **Dependency lock files pin exact sub-packages.** The `bioconda-utils lock` command generates `lock文件` (YAML) that freeze every transitive dependency — including subdependencies like `libgcc-ng` or `numpy` — to specific channel, version, and build string combinations, ensuring reproducible environments across machines.

## Pitfalls

- **Running `bioconda-utils lint` on a dirty Git working tree.** Linting evaluates the diff between the current branch and the target branch (default `master`). Uncommitted local changes can cause false-positive lint failures or hide real issues that would fail CI on the base branch. Always commit or stash local changes before linting.
- **Omitting `--linter` when using custom linter modules.** `bioconda-utils lint` defaults to the Bioconda-maintained `bioconda-recipe-linter` plugin. If a project bundles a custom linter (e.g., internal channel policy checks), omitting `--linter /path/to/custom_linter.py` silently falls back to the default and skips site-specific rules, causing submitted PRs to pass locally but fail CI.
- **Mismatching `--channels` ordering between build and lock steps.** Dependency resolution is channel-order–sensitive. If `bioconda-utils build` uses `--channels conda-forge,bioconda,defaults` but `bioconda-utils lock` uses `--channels bioconda,conda-forge,defaults`, the resolved versions may differ, producing lock files incompatible with the built packages.
- **Forgetting to increment `build: number` on version-only changes.** When only the version string changes (no recipe logic change), conda-build requires an incremented `build` number to trigger a new package build. Skipping this causes conda to skip the package entirely (treat as already satisfied), leaving users on the old version.
- **Using `single_quotes` in `script:` blocks without proper escaping.** The lint rule `double-quotes` requires all strings in the recipe to use double quotes unless `single_quotes` is explicitly listed as allowed. A `script:` block using single quotes will pass the shell but fail the linter, blocking merge.

## Examples

### Lint a conda recipe file for Bioconda compliance
**Args:** `lint --recipe-dir /path/to/recipes --output-format text examples/recipes`
**Explanation:** Runs the default Bioconda linter over all recipes in the given directory, printing human-readable violations in text format so you can review failures before pushing a branch.

### Build a conda package using a Docker container
**Args:** `build --recipe-dir /path/to/recipes --docker-image bioconda/bioconda-utils-build --mml 2.17 pkg_name`
**Explanation:** Spawns a Docker container from the official build image and compiles package `pkg_name` inside it, ensuring the binary is linked against the specified minimum glibc version.

### Update dependency pinnings for the entire Bioconda channel
**Args:** `update-pinnings --pinnings-file pinnings.yaml --cache-dir /path/to/cache`
**Explanation:** Reads the current pinnings constraint file, resolves updated upper/lower bounds for all pinned packages, and writes a new pinnings YAML — used to keep the ecosystem's dependency bounds current without manually auditing every package.

### Aggregate build artifacts into a consolidated conda channel
**Args:** `aggregate --artifacts-dir /path/to/artifacts --output-dir /path/to/channel --merge-channel conda-forge`
**Explanation:** Collects all `.tar.bz2` or `.conda` packages staged under `artifacts-dir`, merges them into `output-dir` alongside a rebuilt `repodata.json`, and optionally overlays packages from `conda-forge` to create a combined channel.

### Generate a dependency lock file for reproducibility
**Args:** `lock --recipe-file meta.yaml --output lock.yaml --channels bioconda,conda-forge,defaults`
**Explanation:** Resolves the full dependency graph (including transitive subdependencies) and writes exact versions, build strings, and channel sources to `lock.yaml`, enabling bit-for-bit reproducible environments across CI runs.

### Lint a specific single recipe with JSON output for CI parsing
**Args:** `lint --recipe meta.yaml --linter bioconda_recipe_linter --output-format json`
**Explanation:** Runs linting on one recipe file and emits machine-readable JSON, making it suitable for automated CI gates that parse violation codes and block merges programmatically.

### Build multiple packages in parallel using Singularity containers
**Args:** `build --recipe-dir recipes/ --container-engine singularity --parallel 4 --bind /tmp/build_cache`
**Explanation:** Uses Singularity instead of Docker, spawns up to 4 concurrent build containers, and mounts a shared cache directory so compiled artifacts are reused across builds to speed up the pipeline.

### Dry-run linting to preview failures without CI environment
**Args:** `lint --recipe meta.yaml --dry-run --env linux_aarch64`
**Explanation:** Simulates the lint pass for the aarch64 Linux environment without executing the full linter plugin, printing expected selector evaluations and dependency constraints so you can debug locally before pushing.
---