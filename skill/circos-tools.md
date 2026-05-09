---
name: circos-tools
category: Bioinformatics Data Visualization Utilities
description: A suite of command-line utilities for generating, managing, and manipulating data and configuration files used by Circos, a Perl-based tool for visualizing genomic data in circular layouts. Provides utilities for karyotype data creation, data format conversion, and configuration file management.
tags: [genomics, visualization, karyotype, configuration, data-format, genomic-data]
author: AI-generated
source_url: https://github.com/circos/circos-tools
---

## Concepts

- Circos-tools operates on karyotype and ideogram data files (tab-delimited format with chromosome ID, label, start, end, and optional color/staining fields) and generates or modifies configuration blocks used by the Circos Perl engine.
- The suite uses a modular plugin architecture where commands like `build`, `import`, and `manager` perform distinct operations on data; plugins are specified via `-plugins` and configuration via `-conf` YAML files.
- Data I/O supports multiple genomic formats including BED, GTF, and general feature formats; output is typically written to stdout or specified output files with `.conf` or `.txt` extensions depending on the operation.
- The `circos-tools` command accepts a `-dir` parameter to set the working directory for data and configuration files, enabling batch processing of genomic datasets organized in directory structures.
- Tool behavior is configurable through environment variables (`COLORS`, `LINES`) and command-line flags that control formatting, filtering by genomic coordinates, and output verbosity.

## Pitfalls

- Specifying an incorrect karyotype file path with `-file` causes the tool to fail silently or produce empty output, leading to blank circular plots when the resulting configuration is fed to Circos.
- Using `-plugins` without ensuring the corresponding plugin module is installed in the Perl library path results in "plugin not found" errors; the `cpanm` or `cpan` installation step is required separately from the tool binary.
- Feeding non-chromosomal karyotype data (e.g., with scaffold names instead of `chr` prefixes) to `build` commands produces ideograms that fail validation in Circos, causing runtime errors during plot generation.
- Omitting the required `-dir` parameter when working with relative file paths causes file-not-found exceptions, even when input files exist in the expected subdirectory structure.
- Conflicting `-color` specifications in both input data and configuration files produce unexpected color mappings, as later specifications override earlier ones without warning during rendering.

## Examples

### Generate a karyotype file from a BED file
**Args:** `build -file input.bed -out karyotype.txt -type karyotype`
**Explanation:** Reads genomic intervals from a BED file and converts them to Circos-compatible karyotype format with chromosome definitions, enabling visualization of specified regions as ideograms.

### Import genomic features with color coding
**Args:** `import -file genes.gtf -format gtf -color hs1 -out features.conf`
**Explanation:** Converts a GTF annotation file into a Circos link/highlight configuration file using the built-in `hs1` color palette, producing colored genomic features ready for inclusion in a Circos configuration block.

### Create configuration with directory scoping
**Args:** `build -dir /data/genomeproject -out circos.conf -plugins ideogram -label-type text`
**Explanation:** Sets the working directory explicitly and outputs a complete configuration section with textual chromosome labels, ensuring all relative file references resolve correctly during Circos execution.

### Sync data between two karyotype files
**Args:** `sync -file reference.txt -mask regions.bed -out synchronized.txt`
**Explanation:** Removes genomic regions specified in the mask BED file from a reference karyotype, producing a filtered data file that excludes those coordinates from the circular plot layout.

### Generate multiple ideogram blocks with sequential output
**Args:** `build -file scaffolds.tsv -out ideogram.conf -type ideogram -from 1 -to 500 -title HumanScaffolds`
**Explanation:** Processes a subset of entries from a tab-separated scaffold file, limiting output to the first 500 entries and adding a configuration block title, useful for generating preview configurations before full dataset processing.