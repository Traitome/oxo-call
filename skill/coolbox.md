---
name: coolbox
category: bioinformatics/visualization
description: A command-line utility for managing and visualizing genomic contact matrices, particularly Hi-C data in .cool format. Provides tools for data inspection, format conversion, normalization, and generating publication-ready contact matrix plots.
tags:
  - genomics
  - hic
  - contact-matrix
  - visualization
  - cooler
  - epigenomics
author: AI-generated
source_url: https://github.com/example/coolbox
---

## Concepts

- **Input Formats**: coolbox accepts `.cool` files (binary Hi-C contact matrix format) and `.mcool` (multi-resolution cooler files). It can also accept raw tab-separated text matrices via stdin for quick visualization tasks.
- **Output Formats**: Generates static visualizations as PNG, PDF, or SVG. Also exports processed matrices as `.cool` files or tab-delimited text for downstream analysis.
- **Multi-Resolution Support**: When working with `.mcool` files, coolbox automatically handles multiple resolution levels. Use `--resolution` to specify which level to load (e.g., 1000, 5000, 10000 bp).
- **Normalization**: Supports several normalization methods including KR (Knight-Ruiz), ICE (Iterative Correction and Eigenvector decomposition), and VC (Vector Balancing). Specify via `--normalize` flag.

## Pitfalls

- **Mismatched Chromosome Names**: Providing chromosome names that don't exist in the cooler file (e.g., using "chr1" when the file uses "1") will cause the tool to fail silently with no output. Always verify chromosome naming convention first using `coolbox inspect`.
- **Memory Overflow with Large Matrices**: Loading entire high-resolution matrices (>1 million bins) into memory without preprocessing can crash the session or cause extreme slowdowns. Always downsample with `--resolution` or use `--region` to restrict to specific genomic regions.
- **Incorrect Resolution Specification**: Selecting a resolution value not present in the multi-resolution file results in an error. Use `coolbox inspect` to list available resolutions before specifying them.
- **Normalization Conflicts**: Applying multiple normalization methods simultaneously (e.g., both `--normalize KR` and `--normalize ICE`) will cause the tool to use only the first one specified, potentially leading to unintended results.

## Examples

### Inspect the contents and metadata of a cooler file

**Args:** `inspect path/to/filename.cool`

**Explanation:** Displays chromosome names, available resolutions, cell counts, and normalization metadata embedded in the file.

### Generate a basic contact matrix plot for a specific genomic region

**Args:** `plot -o output.png --region chr1:10000000-20000000 path/to/filename.cool`

**Explanation:** Creates a PNG image showing the Hi-C contact matrix for the specified region on chromosome 1 between 10mb and 20mb.

### Export a normalized contact matrix to a text file

**Args:** `export -o matrix.txt --normalize KR --region chr2:5000000-15000000 path/to/filename.cool`

**Explanation:** Exports Knight-Ruiz normalized contact frequencies as tab-separated text for the specified region on chromosome 2.

### Create a publication-ready PDF with custom colormap

**Args:** `plot -o publication.pdf --region chr3:0-50000000 --cmap RdYlBu_r --resolution 10000 path/to/filename.mcool`

**Explanation:** Generates a PDF with the Red-Yellow-Blue reversed colormap at 10kb resolution for chromosome 3's first 50mb.

### Compare two regions by generating a differential heatmap

**Args:** `compare -o differential.png --region-a chr1:0-10000000 --region-b chr1:10000001-20000000 --method ratio path/to/sample.cool`

**Explanation:** Creates a ratio-based differential heatmap comparing two adjacent regions on chromosome 1.

### List all available resolutions in a multi-resolution file

**Args:** `inspect --resolutions path/to/filename.mcool`

**Explanation:** Lists every resolution level present in the multi-resolution cooler file, enabling correct downstream resolution selection.

### Generate multiple plots for different chromosomes in batch mode

**Args:** `batch -o results/ --resolution 5000 --pattern "chr{1,2,3,4,5}" path/to/filename.cool`

**Explanation:** Generates 5kb resolution plots for chromosomes 1 through 5, saving each to the results directory with chromosome-appropriate filenames.