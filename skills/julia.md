---
name: julia
category: programming
description: Julia programming language interpreter; high-performance scientific computing, bioinformatics, data analysis, and statistical modelling
tags: [julia, scientific-computing, biojulia, pkg, depot, statistics, high-performance, pluto]
author: oxo-call built-in
source_url: "https://docs.julialang.org/"
---

## Concepts
- Julia is invoked with `julia`; scripts have a `.jl` extension; run interactively with `julia` or a script with `julia script.jl`.
- The **Julia depot** (`~/.julia/`) stores packages, registries, compiled artifacts, logs, and preferences.
- `JULIA_DEPOT_PATH` env var overrides the depot search path (colon-separated list of directories); default is `~/.julia:$JULIA_DEPOT_PATH`.
- Installed packages live in `~/.julia/packages/`; compiled system images (sysimages) in `~/.julia/compiled/`.
- **Project environment**: a `Project.toml` and `Manifest.toml` pair pins all dependencies for reproducibility; activate with `julia --project` or `julia --project=@.`.
- Activate the current directory's project inside the REPL: `using Pkg; Pkg.activate(".")`.
- The default global environment is `~/.julia/environments/v<version>/`; avoid installing too many packages globally.
- `JULIA_NUM_THREADS` env var controls the number of threads; set to `auto` to use all CPU cores: `JULIA_NUM_THREADS=auto julia script.jl`.
- `JULIA_DEPOT_PATH` can point to a shared depot on HPC clusters (read-only); user depot comes first.
- BioJulia ecosystem: `BioSequences`, `FASTX`, `GenomicFeatures`, `BioAlignments` — install via `Pkg.add("BioSequences")`.
- Pluto notebooks: reactive notebook environment installed with `Pkg.add("Pluto")` and launched with `Pluto.run()`.
- `--startup-file=no` skips `~/.julia/config/startup.jl`; use for reproducible, clean execution in pipelines.

## Pitfalls
- First run of a script is slow due to JIT compilation ("Time To First Plot" problem); precompile packages or build a custom sysimage to mitigate.
- `Pkg.rm("Package")` removes a package from the current environment; `Pkg.gc()` then cleans unused artifacts from the depot.
- Mixing global and project environments causes version conflicts; always activate a project env before installing packages for a project.
- `JULIA_NUM_THREADS` must be set *before* starting Julia; `Threads.nthreads()` shows the active count.
- On HPC clusters with read-only system Julia, set `JULIA_DEPOT_PATH` to a writable location in your home/scratch before installing packages.
- Package precompilation happens automatically on first `using`; if it fails, run `Pkg.precompile()` and check error logs in `~/.julia/logs/`.
- The `Manifest.toml` is platform-specific; do NOT commit it when sharing libraries across OS types.

## Examples

### run a Julia script
**Args:** `script.jl`
**Explanation:** executes script.jl in the default environment; prints output to stdout; exit code reflects script completion status

### run a script in a specific project environment
**Args:** `--project=. script.jl`
**Explanation:** --project=. activates the Project.toml in the current directory; ensures correct package versions are used

### run a script with multiple threads
**Args:** `--threads auto script.jl`
**Explanation:** --threads auto uses all available CPU cores; equivalent to setting JULIA_NUM_THREADS=auto before invoking Julia

### install BioSequences package from the Julia REPL (batch mode)
**Args:** `-e 'using Pkg; Pkg.add("BioSequences")'`
**Explanation:** -e evaluates the expression; installs BioSequences into the currently active environment; prefer --project for reproducibility

### show installed packages in the current environment
**Args:** `-e 'using Pkg; Pkg.status()'`
**Explanation:** lists all installed packages with their versions in the active environment; useful for debugging dependency issues

### add BioJulia packages BioSequences, FASTX, and GenomicFeatures
**Args:** `-e 'using Pkg; Pkg.add(["BioSequences","FASTX","GenomicFeatures"])'`
**Explanation:** installs multiple BioJulia packages in one call; packages are resolved together for compatible versions

### run script without loading startup.jl (for CI/pipelines)
**Args:** `--startup-file=no --project=. script.jl`
**Explanation:** --startup-file=no skips ~/.julia/config/startup.jl; produces reproducible headless execution in pipeline contexts

### check Julia version and depot paths
**Args:** `-e 'println(VERSION); println(DEPOT_PATH)'`
**Explanation:** prints Julia version and the ordered list of depot directories Julia searches for packages and compiled artifacts

### compile a script ahead of time to reduce startup latency
**Args:** `--compile=all -O2 script.jl`
**Explanation:** --compile=all forces ahead-of-time compilation; -O2 enables level-2 optimisations; reduces first-run latency for recurring pipeline jobs

### run a Pluto notebook server on a specific port
**Args:** `-e 'import Pluto; Pluto.run(port=1234)'`
**Explanation:** starts the Pluto reactive notebook server on port 1234; useful when SSH tunnelling from an HPC login node to a local browser
