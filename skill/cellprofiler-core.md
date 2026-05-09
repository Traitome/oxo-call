---
name: cellprofiler-core
category: Bioinformatics - Image Analysis
description: CellProfiler is free, open-source software for quantitatively analyzing biological images. It enables automated image analysis pipelines that identify and measure cells, tissues, and subcellular compartments across thousands of images.
tags:
  - cellprofiler
  - image-analysis
  - bioimaging
  - cell-segmentation
  - phenotype-analysis
author: AI-generated
source_url: https://cellprofiler.org
---

## Concepts

- **Pipeline-based workflow**: CellProfiler uses `.cppipe` pipeline files to define a sequential series of image analysis modules (e.g., illumination correction, object identification, measurement) that process images reproducibly across batches.

- **Object identification pipeline**: Images are processed through three stages — **IdentifyPrimaryObjects** (detect nuclei/cells), **IdentifySecondaryObjects** (delineate cell boundaries using stains or borders), and **IdentifyTertiaryObjects** (distinguish touching objects) — producing object sets and per-object measurements.

- **Image sets and grouping**: CellProfiler processes images in **ImageSets** defined by metadata; grouping (`GroupBy` metadata) enables batch processing of image groups (e.g., wells, fields of view) together, preserving relationships between channels in multi-channel experiments.

- **Measurement output formats**: Measurements are exported to CSV or SQLite databases via **ExportToCsv** or **ExportToDatabase** modules, with one row per object (or per image if aggregate) and columns for each feature (e.g., Area, Intensity_Mean, Location_Center_X).

## Pitfalls

- **Mismatched Image and Pipeline versions**: Loading a pipeline created in CellProfiler 3.x with CellProfiler 4.x may fail silently or produce different results because module settings (e.g., thresholding methods, default parameters) changed between versions. Always verify pipeline compatibility.

- **Incorrect metadata matching in batch processing**: If metadata rules (e.g., `file_division`) do not match the actual filename patterns, CellProfiler produces an "No images found" error or processes the wrong files, leading to missing or incorrect measurements in output files.

- **Memory exhaustion with large image sets**: Loading high-resolution images (e.g., 2048×2048 or larger) without enabling "Rescale intensity" or reducing image size first causes memory errors, crashing the analysis mid-batch and losing partial results.

- **Missing required modules in pipeline**: A pipeline may lack essential preprocessing (e.g., **NamesAndTypes** not assigning image channels correctly, **IdentifyPrimaryObjects** using an unthresholded image), resulting in zero objects detected and empty measurement output, requiring pipeline redesign.

## Examples

### Run a pipeline on a directory of images
**Args:** `-p /path/to/pipeline.cppipe -i /path/to/image_directory -o /path/to/output_directory`
**Explanation:** This executes the defined pipeline on all images in the specified input directory, saving results (objects, measurements) to the output directory.

### Run a pipeline with a specific .cppipe file
**Args:** `--pipeline-file=/analysis/whole_cell_pipeline.cppipe`
**Explanation:** Explicitly specifying the pipeline file ensures the correct analysis workflow loads, avoiding reliance on default or previously opened pipelines.

### Process images grouped by well metadata
**Args:** `-p /path/to/pipeline.cppipe -g "Well" --do-not-resume`
**Explanation:** Grouping by the "Well" metadata tag processes all images from each well together, maintaining proper associations for multi-channel plates and enabling correct per-well analysis.

### Export measurements to CSV
**Args:** `-p /path/to/pipeline.cppipe --output-file=/results/measurements.csv`
**Explanation:** Directing output to a CSV file creates a tabulated result file containing per-object measurements (e.g., area, intensity), enabling downstream statistical analysis in R or Python.

### Run pipeline in headless mode (no GUI)
**Args:** `-p /path/to/pipeline.cppipe -i /path/to/input -o /path/to/output -f`
**Explanation:** The `-f` flag forces headless execution without the GUI, suitable for cluster or server environments where interactive display is unavailable.

### Run a specific image subset using file list
**Args:** `-p /path/to/pipeline.cppipe -i /path/to/image_dir --file-list=/path/to/image_subset.txt`
**Explanation:** A file list restricts processing to only the specified image files, useful for testing pipelines on a small image subset before full batch processing.

### Configure measurement database output
**Args:** `-p /path/to/pipeline.cppipe --output-directory=/sql_results --export-format=sqlite`
**Explanation:** Setting SQLite output stores all measurements in a relational database, enabling efficient querying of complex measurement relationships across many images and objects.

### Set maximum workers for parallel processing
**Args:** `-p /path/to/pipeline.cppipe -i /path/to/images -o /path/to/output --local-workers=8`
**Explanation:** Configuring 8 local workers enables parallel image processing, significantly reducing runtime on multi-core servers when analyzing large image sets.