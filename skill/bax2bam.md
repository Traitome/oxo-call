---
name: bax2bam
category: Format Conversion
description: Converts PacBio BAX format files to standard BAM format, enabling interoperability with standard bioinformatics pipelines. Handles raw SMRT sequencing data from PacBio instruments.
tags: [pacbio, smrt, bax, bam, format-conversion, sequencing, long-reads]
author: AI-generated
source_url: https://github.com/PacificBiosciences/bax2bam
---

## Concepts

- **BAX Format**: BAX (Biomolecular Analysis by DNA eXtract) is PacBio's legacy raw data format containing base-calls, quality scores, and pulse metrics in HDF5-based `.bax.h5` files. The tool reads these native PacBio files and converts them to the standard BAM format used by most bioinformatics tools.

- **Input Requirements**: bax2bam accepts `.bax.h5` files (or directories containing them) as primary input, along with associatedbas.h5 files containing base-call information. The tool requires both the data file and associated metadata for accurate conversion.

- **Output Formats**: The tool produces standard BAM or CRAM output files with proper read groups, RG tags, and sequencing metadata. Output can be configured for compatibility with downstream tools like GATK, SAMtools, or BcfTools.

- **Quality Filtering**: Built-in quality score filtering allows removal of low-confidence base-calls based on QV (Quality Value) thresholds. This is critical for variant calling accuracy as PacBio raw reads have higher error rates that can be filtered out.

- **Read Types**: The tool can separate different read types (e.g., circular consensus sequences vs. subreads) into distinct outputs, which is essential for different downstream analyses.

## Pitfalls

- **Incompatible Input Files**: Attempting to convert modern PacBio formats (e.g., CCS-generated BAM from Sequel II data) using bax2bam will fail since bax2bam is designed only for legacy RS instrument data. Always verify source instrument before using this tool.

- **Missing Metadata Files**: Running bax2bam without both the `.bax.h5` data file and corresponding metadata file results in incomplete conversion or silent failures. Ensure both files are present and properly paired.

- **Quality Filter Misconfiguration**: Setting qv-trim or quality thresholds incorrectly can either remove valid reads (reducing coverage) or retain low-quality data (introducing false variants). The default minimum QV of 25 is conservative for most applications.

- **Output File Overwrites**: Running bax2bam without specifying a unique output filename will overwrite existing files without warning. Always use explicit output paths to prevent data loss.

- **Memory Constraints**: Processing large BAX files without sufficient RAM can cause crashes or excessive slowdown. For datasets >10GB, ensure system has adequate memory or process in chunks.

## Examples

### Convert a single BAX file to BAM format

**Args:** `input.bax.h5 -o output.bam`
**Explanation:** Converts the PacBio BAX file to standard BAM format using default settings, suitable for most downstream analyses.

### Convert with explicit output filename

**Args:** `sample.bax.h5 -o sample_converted.bam`
**Explanation:** Specifies the output filename explicitly to avoid accidental overwrites and maintain organized file naming conventions.

### Apply quality filtering during conversion

**Args:** `input.h5 -o filtered.bam --minQuality 30`
**Explanation:** Removes reads with quality scores below 30 during conversion, improving downstream variant call accuracy by filtering out low-confidence base-calls.

### Convert to CRAM format for reduced file size

**Args:** `input.bax.h5 -o output.cram -t cram`
**Explanation:** Outputs to CRAM format instead of BAM, reducing storage requirements by approximately 40% while maintaining compatibility with most tools.

### Process multiple input files in a directory

**Args:** `/path/to/bax_files/ -o merged_output.bam`
**Explanation:** Processes all BAX files in the specified directory and merges them into a single output BAM file, useful for combining multiple runs of the same sample.