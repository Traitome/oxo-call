---
name: artex
category: Text Extraction / Document Processing
description: A command-line tool for extracting and processing text content from bioinformatics document archives, supporting common archive formats (ZIP, TAR, GZ) and plain text files. Enables batch extraction, metadata parsing, and filtering of scientific articles, sequences, and related publications.
tags: [text-extraction, document-processing, bioinformatics-articles, archive, batch-processing]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/artex
---

## Concepts

- **Input Formats**: artex processes individual text files (`.txt`, `.fa`, `.fasta`) and compressed archives (`.zip`, `.tar`, `.tar.gz`, `.tgz`). The tool automatically detects format based on file extension and magic bytes.
- **Output Modes**: Extracted content can be written to stdout (default), saved to a specified directory with `--output-dir`, or piped into downstream tools. Metadata is emitted as JSON when `--metadata` flag is used.
- **Batch Processing**: Multiple input paths (files or directories) can be provided as positional arguments. The tool recursively scans directories unless `--no-recursive` is specified, matching files against `--pattern` glob expressions.
- **Filtering Options**: Content filtering uses regex patterns via `--include` and `--exclude` flags. Filters apply to filenames when operating on archives, and to file content when extracting text.
- **Metadata Extraction**: When processing bioinformatics articles, artex extracts DOI, authors, journal, and publication year. This requires the `--parse-metadata` flag and valid document structure.

## Pitfalls

- **Specifying Non-Existent Output Directory**: Using `--output-dir` with a path that does not exist causes immediate failure with a "Directory not found" error. The tool does not auto-create directories—you must create them beforehand or use `-` to extract to current working directory.
- **Confusing Filter Scope**: The `--include` and `--exclude` flags operate on filenames for archive contents but on file content for extracted text. Applying content filters to archive operations yields unexpected empty results because filenames rarely match biological regex patterns.
- **Memory Issues with Large Archives**: Processing multi-GB TAR or ZIP archives without the `--stream` flag loads the entire archive into memory. For large bioinformatics datasets, always enable streaming with `--stream` to avoid OOM crashes.
- **Incorrect Pattern Glob Syntax**: Using regex syntax (like `\d+`) in `--pattern` fails silently because it expects glob patterns (like `*.txt` or `seq*.fa`). Use `--include-regex` for regex-based filename matching.
- **Overwriting Existing Files**: By default, artex extracts with overwrite behavior controlled by `--overwrite` (default: prompt). In scripts, always explicitly set `--overwrite true` or `--overwrite false` to avoid interactive prompts that stall automated pipelines.

## Examples

### Extract a single text file from a ZIP archive
**Args:** `input.zip --extract-to stdout --filter "sequences.txt"`
**Explanation:** Extracts only the file named "sequences.txt" from the archive and prints its content to stdout.

### Extract all FASTA files from a TAR.GZ archive to a directory
**Args:** `data.tar.gz --output-dir ./extracted_fasta --pattern "*.fa"`
**Explanation:** Decompresses the archive and extracts only files matching the FASTA extension into the specified output directory.

### List without extracting (Dry-run)
**Args:** `archive.zip --list-only`
**Explanation:** Lists all files contained in the archive without extracting them, useful for inspecting archive contents first.

### Batch extract multiple archives in a directory
**Args:** `./archives/ --output-dir ./all_extracted --no-recursive`
**Explanation:** Processes all archives in the specified directory (non-recursively) and extracts their contents to a unified output folder.

### Extract and filter sequences by a regex pattern
**Args:** `sequences.gz --output-dir ./filtered --include-regex "^>TP53|^>BRCA"`
**Explanation:** Extracts all sequence files but keeps only those with header lines starting with TP53 or BRCA genes.

### Parse metadata from article archives
**Args:** `papers.zip --parse-metadata --output-dir ./metadata`
**Explanation:** Extracts bibliographic metadata (DOI, authors, year) from supported article formats and writes JSON output.

### Stream-process a very large archive without loading into memory
**Args:** `huge_data.tar.gz --stream --output-dir ./streamed_output`
**Explanation:** Uses streaming mode to extract a large archive incrementally, preventing memory exhaustion on systems with limited RAM.

### Extract specific file types using exclude filter
**Args:** `mixed.zip --output-dir ./text_only --exclude "*.log"`
**Explanation:** Extracts all files except those ending in `.log`, useful for filtering out unwanted log files from mixed archives.