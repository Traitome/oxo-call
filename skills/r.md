---
name: r
category: statistical-computing
description: R language interpreter and Rscript command; statistical computing, data analysis, and bioinformatics scripting
tags: [r, rscript, statistics, bioconductor, ggplot2, tidyverse, cran, scripting, data-analysis]
author: oxo-call built-in
source_url: "https://www.r-project.org/"
---

## Concepts
- Invoke R interactively with `R`; run scripts non-interactively with `Rscript script.R` or `R --vanilla -f script.R`.
- The user library is in `~/R/<platform-version>/` (e.g. `~/R/x86_64-pc-linux-gnu-library/4.3/`); inspect with `.libPaths()` inside R.
- System library is typically `/usr/lib/R/library` or `/usr/local/lib/R/library`; user packages shadow system packages.
- The R home directory (where base packages live) is returned by `R.home()`; usually `/usr/lib/R` or `/opt/R/<version>`.
- Site-wide configuration: `R_HOME/etc/Renviron` and `R_HOME/etc/Rprofile.site`; user overrides: `~/.Renviron` and `~/.Rprofile`.
- `~/.Renviron` sets environment variables for R sessions (e.g. `R_LIBS_USER`, `JAVA_HOME`). A minimal example: `R_LIBS_USER=~/R/libs`.
- `~/.Rprofile` is an R script that runs at startup; use for options, package loads, and custom functions.
- Bioconductor packages are installed with `BiocManager::install()`; CRAN packages with `install.packages()`.
- `renv` manages project-level package snapshots in `renv/library/`; `renv.lock` tracks exact versions for reproducibility.
- `pak` is a modern CRAN/GitHub package installer that resolves dependencies faster and in parallel.
- In HPC/conda environments, R is commonly installed under `~/mambaforge/envs/<env>/lib/R/`; use `conda install -c conda-forge r-base`.
- The `R CMD BATCH` command runs a script and writes output to a `.Rout` file; useful for cluster job logs.

## Pitfalls
- DANGER: `install.packages()` without specifying `lib` installs to the first writable path in `.libPaths()`; on shared systems this may be the system library.
- Running R scripts that call `setwd()` can break relative paths if the working directory assumption is wrong; prefer `here::here()` for path resolution.
- `Rscript -e` evaluates an expression; pass multi-line code with quoted semicolons or use a temporary script file instead of complex escaping.
- Output buffering: use `message()` for stderr (unbuffered) and `cat()` or `print()` for stdout; avoid `print()` inside loops for large outputs.
- Memory: R loads data into RAM; use `data.table` or `arrow` for large datasets; check available memory with `gc()` and `pryr::mem_used()`.
- `R --vanilla` suppresses loading `~/.Rprofile` and `~/.Renviron`; use for reproducible, headless execution.
- Package conflicts: use `package::function()` syntax to call a specific package's function when multiple packages export the same name.
- Bioconductor version must match the installed R version; mixing versions causes cryptic errors during `BiocManager::install()`.

## Examples

### run an R script non-interactively
**Args:** `Rscript analysis.R`
**Explanation:** executes analysis.R in a non-interactive R session; stdout/stderr go to the terminal

### run an R script with command-line arguments
**Args:** `Rscript analysis.R --input data.csv --output results.csv`
**Explanation:** args after the script name are available via commandArgs(trailingOnly=TRUE) inside the script

### execute a one-liner R expression
**Args:** `Rscript -e "cat(paste(1:10, collapse=','), '\n')"`
**Explanation:** -e evaluates the expression directly without a script file; useful for quick computations and CI checks

### install a CRAN package into the user library
**Args:** `Rscript -e "install.packages('ggplot2', repos='https://cloud.r-project.org', lib=Sys.getenv('R_LIBS_USER'))"`
**Explanation:** specifying repos avoids interactive mirror selection; lib ensures installation to the user library path

### install Bioconductor packages
**Args:** `Rscript -e "BiocManager::install(c('DESeq2','edgeR'))"`
**Explanation:** BiocManager installs packages from Bioconductor; version must match installed R; use BiocManager::version() to check

### check installed package version
**Args:** `Rscript -e "packageVersion('DESeq2')"`
**Explanation:** prints the installed version of the named package; useful for reproducibility checks

### list user-installed packages and their versions
**Args:** `Rscript -e "ip <- installed.packages(lib.loc=.libPaths()[1]); cat(paste(ip[,'Package'],ip[,'Version'],sep='='), sep='\n')"`
**Explanation:** .libPaths()[1] is the user library; prints pkg=version lines for all user-installed packages

### run R script suppressing startup messages
**Args:** `Rscript --vanilla --quiet analysis.R`
**Explanation:** --vanilla skips .Rprofile and .Renviron; --quiet suppresses the R startup banner; ideal for HPC batch jobs

### show R library paths
**Args:** `Rscript -e ".libPaths()"`
**Explanation:** prints the ordered list of library directories R searches for packages; first writable path is the user library

### render an Rmarkdown document to HTML
**Args:** `Rscript -e "rmarkdown::render('report.Rmd', output_format='html_document')"`
**Explanation:** rmarkdown::render() knits the .Rmd file; output_format controls the output type; output written to the same directory

### profile memory usage of an R script
**Args:** `R --vanilla -f analysis.R --args input.csv`
**Explanation:** -f reads a script file; --args passes positional arguments; --vanilla for clean reproducible execution

### run R CMD BATCH for HPC cluster jobs
**Args:** `R CMD BATCH --vanilla analysis.R analysis.Rout`
**Explanation:** runs the script and captures all output to analysis.Rout; --vanilla ensures clean environment; standard pattern for LSF/SLURM jobs
