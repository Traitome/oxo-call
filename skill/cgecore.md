---
name: cgecore
category: Bioinformatics Core Utilities
description: A core bioinformatics utility from the Center for Genomic Epidemiology (CGE) suite, providing essential functions for DNA sequence analysis, database management, and integration with CGE tools such as mlst, amrgeneab, serotypefinder, virulencefinder, and plasmidfinder.
tags:
  - genomics
  - dna-analysis
  - sequence-typing
  - antimicrobial-resistance
  - bioinformatics
author: AI-generated
source_url: https://github.com/genomicepidemiology/cgecore
---

## Concepts

- **Input Format**: cgecore accepts FASTA (.fasta/.fa) and FASTQ (.fastq/.fq) sequence files as primary input, as well as raw reads in compressed (.gz) format. Sequence files must use standard nucleotide letter codes (A, T, G, C, N) and may contain multiple sequences in a single file.
- **Output Modes**: Results are printed to stdout by default in tab-delimited or JSON format depending on flags. Use `-o` or `--outfile` to write results directly to a file. The `-json` flag enables JSON output for programmatic parsing.
- **Database Integration**: cgecore relies on externally maintained database files in specific directories. Database paths can be specified via `-db` or `--database` flags; default paths are typically `/opt/cge/databases/` in Docker installations. Updates to database files are required for accurate results.
- **Threading and Performance**: Multi-threaded execution is supported via `-threads` or `-t` flags. The tool automatically detects available CPU cores; explicitly setting threads can improve throughput for batch analysis of multiple sequence files.

## Pitfalls

- **Stale Database Files**: Using outdated database files leads to missing detections for recently discovered genes or alleles. consequence: False-negative results for antimicrobial resistance genes or incorrect sequence typing results.
- **Invalid Sequence Encoding**: Input files with degenerate nucleotide codes beyond standard IUPAC symbols (e.g., lowercase letters, special characters) cause parsing failures. consequence: Tool crashes mid-analysis or produces no output without clear error messages.
- **Memory Overflow with Large Files**: Processing very large FASTA files or many concurrent files without adjusting memory limits can cause out-of-memory errors. consequence: Process termination and partial or missing results.
- **Incorrect File Permissions**: Running cgecore on read-only input directories or with inadequate write permissions for output files. consequence: Tool reports permission errors and fails to generate output files.
- **Mismatched Database Schema**: Using database files from a different cgecore version causes parsing errors or incorrect associations. consequence: Silent failures or scrambled results that appear valid but are inaccurate.

## Examples

### Perform basic sequence analysis on a FASTA file
**Args:** `-i input_sequence.fasta -json`
**Explanation:** This runs cgecore on the provided FASTA file with JSON output enabled for easy parsing by downstream scripts.

### Analyze sequences using a custom database directory
**Args:** `-i input.fasta -db /custom/path/to/databases -o results.txt`
**Explanation:** This specifies a custom database directory path and writes output to a file, useful when databases are stored in non-standard locations.

### Run multi-threaded analysis on multiple sequence files
**Args:** `-i batch/*.fasta -threads 4 -o batch_results.tsv`
**Explanation:** This processes all FASTA files in the batch directory using 4 threads for parallel execution, improving throughput for large datasets.

### Enable verbose output for debugging analysis issues
**Args:** `-i sequences.fasta -v -o debug_output.txt`
**Explanation:** This enables verbose logging to help diagnose configuration problems or unexpected behavior during analysis.

### Output results in tab-delimited format for spreadsheet import
**Args:** `-i sample.fasta -tab -o table_output.tsv`
**Explanation:** This outputs results in tab-delimited format, convenient for manual review or import into Excel/Google Sheets.