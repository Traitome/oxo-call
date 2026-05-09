---
name: bcbreport
category: bioinformatics/utility
description: A tool for reporting statistics and information about BCB (BWT-Compressed Binary) files in the BBTools suite. Displays compression metrics, read counts, position mappings, and summary data for compressed genomic data files.
tags:
  - bcb
  - compression-stats
  - bwt-based
  - report
  - bioinformatics
  - sequencing
author: AI-generated
source_url: https://github.com/bcbio/bcbiTools
---

## Concepts

- **BCB File Format**: bcbreport operates on `.bcb` files created by BBTools compression utilities (e.g., `bcbio-abpool`, `bcbio-fastq`). These files contain BWT-compressed sequences with built-in FM-index capabilities for fast substring lookup.
- **Input Requirement**: The primary argument is always a path to an existing BCB file. The tool does not accept raw FASTQ/FASTA files directly—they must be pre-compressed using companion tools like `bcbio-build`.
- **Output Modes**: bcbreport supports structured output modes including plain text summary (default), JSON (`--json`), and XML (`--xml`) for integration into pipelines.
- **Statistics Reported**: Depending on flags, output includes total read count, unique sequence count, average sequence length, compression ratio, occurrence counts per position, and file metadata (creation date, tool version).

## Pitfalls

- **Specifying Uncompressed Input**: Passing a FASTQ or FASTA file directly to bcbreport causes a format-parse error because the tool expects BCB-compressed binary format. Always compress raw files first using `bcbio-build` before reporting.
- **Misinterpreting Compression Ratio**: The reported compression ratio is calculated as `original_size / compressed_size`. Users sometimes invert this, expecting compressed/original, leading to confusion about actual space savings.
- **Missing File Extension**: Providing a file path without the `.bcb` extension may cause silent failure if the tool searches for specific extensions. Explicit full paths (e.g., `sample1.bcb`) are recommended.
- **Insufficient Permissions**: Reading a BCB file without read permissions produces a generic I/O error. Verify file access with `ls -la` before running.

## Examples

### Report basic summary statistics for a BCB file
**Args:** `sample1.bcb`
**Explanation:** Displays default summary including read count, unique sequence count, average length, and basic file metadata without additional formatting flags.

### Output statistics in JSON format for pipeline integration
**Args:** `sample1.bcb --json`
**Explanation:** Returns machine-readable JSON output containing all statistics fields, enabling automated parsing by downstream scripts.

### Include detailed occurrence counts per position
**Args:** `sample1.bcb --positions`
**Explanation:** Augments output with per-position occurrence counts, useful for analyzing sequence distribution biases in compressed data.

### Limit output to specific read count metrics only
**Args:** `sample1.bcb --reads-only`
**Explanation:** Suppresses all other output and prints only the total read count, useful for scripting simple read enumeration.

### Display help information and available options
**Args:** `--help`
**Explanation:** Shows full command-line usage, including all available flags and their descriptions, when unsure about specific options.

### Output in XML format for compatibility with older analysis pipelines
**Args:** `sample1.bcb --xml`
**Explanation:** Returns structured XML output containing all statistics with proper tags, suitable for integration into legacy systems that require XML parsing.

### Set verbosity to include debug-level information
**Args:** `sample1.bcb --verbose`
**Explanation:** Enables debug logging showing internal processing steps and intermediate values, helpful when troubleshooting unusual file behavior or suspected corruption.