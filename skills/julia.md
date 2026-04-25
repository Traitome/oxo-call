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
- `--optimize={0|1|2|3}` sets optimization level; default is 2; higher levels improve performance but increase compilation time.
- `--compile={yes|no|all|min}` controls compilation; `all` forces ahead-of-time compilation; `min` reduces compilation for faster startup.
- `--heap-size-hint=<size>` forces garbage collection when memory exceeds the specified value; useful for memory-constrained environments.
- `--procs {N|auto}` launches N additional worker processes for distributed computing; `auto` uses all CPU threads.
- `--sysimage <file>` starts Julia with a custom system image for faster package loading.

## Pitfalls
- First run of a script is slow due to JIT compilation ("Time To First Plot" problem); precompile packages or build a custom sysimage to mitigate.
- `Pkg.rm("Package")` removes a package from the current environment; `Pkg.gc()` then cleans unused artifacts from the depot.
- Mixing global and project environments causes version conflicts; always activate a project env before installing packages for a project.
- `JULIA_NUM_THREADS` must be set *before* starting Julia; `Threads.nthreads()` shows the active count.
- On HPC clusters with read-only system Julia, set `JULIA_DEPOT_PATH` to a writable location in your home/scratch before installing packages.
- Package precompilation happens automatically on first `using`; if it fails, run `Pkg.precompile()` and check error logs in `~/.julia/logs/`.
- The `Manifest.toml` is platform-specific; do NOT commit it when sharing libraries across OS types.
- `--optimize=3` can significantly increase compilation time; use only for production runs, not development.
- `--compile=all` increases startup time but reduces runtime latency; trade-off depends on script execution frequency.
- `--heap-size-hint` accepts units like 4G, 512M, or percentage like 80%; helps prevent OOM on memory-limited systems.
- Distributed computing with `--procs` requires `@everywhere` to load packages on all workers; common source of errors.
- Custom sysimages require PackageCompiler.jl; building takes time but dramatically reduces package load times.

## Examples

### run a Julia script
**Args:** `script.jl`
**Explanation:** julia command; script.jl script file positional argument

### run a script in a specific project environment
**Args:** `--project=. script.jl`
**Explanation:** julia command; --project=. activates Project.toml in current directory; script.jl script file

### run a script with multiple threads
**Args:** `--threads auto script.jl`
**Explanation:** julia command; --threads auto uses all CPU cores; script.jl script file

### install BioSequences package from the Julia REPL (batch mode)
**Args:** `-e 'using Pkg; Pkg.add("BioSequences")'`
**Explanation:** julia command; -e evaluates expression; 'using Pkg; Pkg.add("BioSequences")' installs BioSequences package

### show installed packages in the current environment
**Args:** `-e 'using Pkg; Pkg.status()'`
**Explanation:** julia command; -e evaluates expression; 'using Pkg; Pkg.status()' shows installed packages

### add BioJulia packages BioSequences, FASTX, and GenomicFeatures
**Args:** `-e 'using Pkg; Pkg.add(["BioSequences","FASTX","GenomicFeatures"])'`
**Explanation:** julia command; -e evaluates expression; 'using Pkg; Pkg.add(["BioSequences","FASTX","GenomicFeatures"])' installs multiple packages

### run script without loading startup.jl (for CI/pipelines)
**Args:** `--startup-file=no --project=. script.jl`
**Explanation:** julia command; --startup-file=no skips startup.jl; --project=. activates project; script.jl script file

### check Julia version and depot paths
**Args:** `-e 'println(VERSION); println(DEPOT_PATH)'`
**Explanation:** julia command; -e evaluates expression; 'println(VERSION); println(DEPOT_PATH)' prints version and depot paths

### compile a script ahead of time to reduce startup latency
**Args:** `--compile=all -O2 script.jl`
**Explanation:** julia command; --compile=all forces AOT compilation; -O2 level-2 optimizations; script.jl script file

### run a Pluto notebook server on a specific port
**Args:** `-e 'import Pluto; Pluto.run(port=1234)'`
**Explanation:** julia command; -e evaluates expression; 'import Pluto; Pluto.run(port=1234)' starts notebook server on port 1234

### run Julia with memory limit hint
**Args:** `--heap-size-hint=8G script.jl`
**Explanation:** julia command; --heap-size-hint=8G memory limit hint; script.jl script file

### run Julia with distributed processes
**Args:** `--procs auto --project=. script.jl`
**Explanation:** julia command; --procs auto launches worker processes; --project=. activates project; script.jl script file

### run Julia with maximum optimization
**Args:** `--optimize=3 --project=. script.jl`
**Explanation:** julia command; --optimize=3 highest optimization level; --project=. activates project; script.jl script file

### run Julia with custom system image
**Args:** `--sysimage=myimage.so --project=. script.jl`
**Explanation:** julia command; --sysimage=myimage.so custom system image; --project=. activates project; script.jl script file

### run Julia in quiet mode for pipelines
**Args:** `--quiet --startup-file=no --project=. script.jl`
**Explanation:** julia command; --quiet suppresses banner; --startup-file=no skips startup.jl; --project=. activates project; script.jl script file

### precompile all packages in current environment
**Args:** `-e 'using Pkg; Pkg.precompile()'`
**Explanation:** julia command; -e evaluates expression; 'using Pkg; Pkg.precompile()' precompiles packages

### instantiate environment from Manifest.toml
**Args:** `--project=. -e 'using Pkg; Pkg.instantiate()'`
**Explanation:** julia command; --project=. activates project; -e evaluates expression; 'using Pkg; Pkg.instantiate()' installs from Manifest.toml
