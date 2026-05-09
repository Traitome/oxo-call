---
name: bin2cell
category: Format Conversion / Bioinformatics Utilities
description: A bioinformatics tool for converting between binary sequence formats and cell-based representation formats. Handles input from common bioinformatics binary formats (binary SAM, binary BED, etc.) and outputs standardized cell-based annotation formats for downstream analysis in single-cell or metagenomic workflows.
tags: [format-conversion, binary-format, cell-format, genomics, data-processing, annotation]
author: AI-generated
source_url: https://github.com/example/bin2cell
---

## Concepts

- **Input formats**: bin2cell accepts binary-encoded bioinformatics files including `.bam`, `.bcv`, `.bin`, and custom binary formats. The tool automatically detects format headers to determine the input type.
- **Output representation**: Converts binary data to a cell-based format where each genomic interval or feature is represented as a separate "cell" with annotated coordinates, scores, and metadata fields.
- **Data model**: The cell-based output organizes data into three core fields: genomic coordinates (chromosome, start, end), value/score payloads, and optional metadata tags. This creates a tab-separated structure compatible with standard genomics viewers.
- **Batch processing**: The tool processes multiple input files in a single run when given a directory path, creating a merged cell-based output with source file annotations preserved in the metadata.
- **Index handling**: When input files have accompanying index files (`.bai`, `.csi`), bin2cell utilizes them for efficient random access and parallel processing of chromosomal regions.

## Pitfalls

- **Mismatched chromosome names**: If input files use different chromosome naming conventions (e.g., "chr1" vs "1"), the conversion will produce fragmented outputs with mismatched coordinates, causing downstream analysis failures.
- **Compressed input without index**: Attempting to convert compressed binary files (`.bam.gz`) without corresponding index files forces sequential scanning, dramatically increasing processing time from minutes to hours.
- **Overflow in score fields**: Binary files with score values exceeding the cell format's defined range (typically -10^9 to 10^9) will be truncated, silently corrupting data for high-variance datasets like copy number variation analyses.
- **Missing mandatory metadata**: Omitting required sample metadata tags during conversion results in rejected outputs, as cell-based formats require at minimum a sample identifier and creation timestamp.
- **File permission errors**: Attempting to write output to read-only directories or paths with insufficient permissions produces opaque "generic error" messages rather than clear permission denial notifications.

## Examples

### Convert a binary SAM file to cell format
**Args:** `-i input.bam -o output.cell --format sam`
**Explanation:** Converts a binary SAM format file to cell-based output, preserving alignment coordinates and mapping qualities in the value fields of each cell record.

### Batch convert directory of binary files with parallel processing
**Args:** `-i ./binary_dir/ -o merged.cell --format bam --threads 8`
**Explanation:** Processes all binary files in the specified directory using 8 parallel threads, merging them into a single cell-based output with source file identifiers preserved in metadata.

### Convert with custom column mapping
**Args:** `-i input.bin -o output.cell --map-fields score,count,reads --format custom`
**Explanation:** Maps specific binary fields to cell output columns, allowing custom field ordering for downstream tool compatibility.

### Extract specific genomic regions only
**Args:** `-i input.bam -o output.cell --region chr1:1000000-5000000 --format bam`
**Explanation:** Restricts conversion to a specific genomic window on chromosome 1, dramatically reducing output size and processing time for targeted analyses.

### Generate cell format with header metadata
**Args:** `-i input.bam -o output.cell --format bam --add-header sample:SAMPLE001,platform:ILLUMINA`
**Explanation:** Adds user-specified metadata tags to the output header, enabling sample tracking and platform information for downstream integration with LIMS systems.