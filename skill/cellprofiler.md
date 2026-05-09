---
name: cellprofiler
category: Image Analysis
description: Open-source software for quantitative analysis of biological images, designed for cell imaging segmentation, feature extraction, and high-content screening.
tags: image-analysis, cell-imaging, bioimaging, phenotyping, feature-extraction
author: AI-generated
source_url: https://cellprofiler.org/
---

## Concepts

- CellProfiler operates through **pipeline files** (`.cppipe` format) that define a sequence of analysis modules (e.g., input, processing, measurement, and export steps). Each pipeline specifies how images are loaded, processed, and quantified.
- The CLI runs pipelines in **headless mode** using the `-c` flag (core) combined with `-p` for the pipeline path. Images are processed from the input directory and measurements written to the output directory automatically.
- Output formats include **CSV** (per-image measurements), **SQLite** (database存储), and **HDF5** (hierarchical data). Users can configure export modules in the pipeline to select specific output formats.
- Image sets are matched between input images and metadata using **rules** or **csv Table** files. Proper metadata matching ensures correct image pairing (e.g., illumination vs. nuclei channels).

## Pitfalls

- **Omitting the `-c` flag** causes CellProfiler to attempt GUI launch, which fails on headless servers without display. Always include `-c` for CLI execution.
- **Pipeline path errors** (incorrect or missing `.cppipe` file) result in immediate termination. The pipeline file must exist and be valid.
- **Input/output directory confusion**: Without `-o`, results default to the input directory, overwriting previous runs. Always specify an explicit output directory.
- **Metadata mismatches** between the image files and the CSV metadata table cause missing image pairs or incorrect processing order, leading to incomplete or wrong measurements.

## Examples

### Run a pipeline on a directory of images
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/input/images -o /path/to/output`
**Explanation:** The `-c` flag enables headless CLI mode, `-p` specifies the pipeline file, `-i` defines where input images are read from, and `-o` sets the output directory for results.

### Run a pipeline with explicit image file list
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/images --file-list /path/to/filelist.csv -o /path/to/output`
**Explanation:** The `--file-list` flag references a CSV file that explicitly lists which image files to process, allowing precise control over the image set.

### Run pipeline and limit to specific modules
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/images -o /path/to/output --do-not-reset`
**Explanation:** The `--do-not-reset` flag allows continuing a previous run without resetting module states, useful for resuming long analyses.

### Run pipeline with specific logging verbosity
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/images -o /path/to/output --verbosity 2`
**Explanation:** The `--verbosity` flag controls log output detail (0=minimal, 5=maximum), helping diagnose pipeline issues during batch processing.

### Run pipeline with temporary workspace cleanup disabled
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/images -o /path/to/output --use-dirty`
**Explanation:** The `--use-dirty` flag prevents cleanup of intermediate files, allowing inspection of processing stages without regenerating them on reruns.

### Run pipeline using existing workspace to resume analysis
**Args:** `-c -p /path/to/pipeline.cppipe -i /path/to/images -o /path/to/output -w /path/to/workspace`
**Explanation:** The `-w` flag specifies an existing workspace file to resume previous analysis, skipping already completed image sets.