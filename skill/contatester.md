---
name: contatester
category: Assembly Analysis
description: A bioinformatics tool for testing, validating, and analyzing assembly contigs. Evaluates contig quality metrics, performs reference-based comparisons, and generates QC reports for de novo assembly outputs.
tags:
  - assembly
  - contigs
  - quality-control
  - validation
  - genomics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/contatester
---

## Concepts

- **Input Formats**: Accepts FASTA and FASTQ files containing assembled contig sequences. Multi-sequence inputs are processed sequentially, with each contig evaluated independently for length, GC content, and complexity metrics.
- **Output Modes**: Produces tabular text output by default (TSV), with optional JSON reporting for programmatic integration. The tool writes summary statistics to stdout and detailed per-contig metrics to the specified output file.
- **Quality Metrics**: Computes N50, L50, contig length distribution, ambiguous base counts (N characters), and repeat content estimation. Reference-based mode adds alignment coverage and identity calculations.
- **Companion Binary**: `contatester-build` creates a compressed index from reference sequences for rapid lookup during comparison tasks. Index files use `.cti` extension and retain sequence metadata.

## Pitfalls

- **Missing Index for Reference Mode**: Running `contatester` with a reference database without first building an index using `contatester-build` results in repeated disk seeks and dramatically slower performance. Always pre-build indexes for reference sets larger than 100 sequences.
- **Non-standard Line Endings**: FASTA files with Windows-style CRLF line endings may cause sequence parsing errors, resulting in truncated contigs and misleading length statistics. Convert input files to Unix line endings before processing.
- **Memory Limits with Large Sets**: Processing thousands of contigs without specifying `--memory-limit` causes excessive RAM consumption. The tool defaults to 2GB; adjust with the flag for large assemblies (>500 MB total sequence).
- **Confusing Output File Permissions**: The `-o` flag overwrites existing output files without prompting. Specifying an output path that you lack write permission to results in a silent failure and no error message displayed.

## Examples

### Calculate basic assembly statistics from a FASTA file
**Args:** `input.fasta --stats`
**Explanation:** Computes N50, L50, total base count, and contig count without requiring a reference database. Outputs summary metrics to stdout.

### Generate detailed per-contig quality report in JSON format
**Args:** `input.fasta --detailed --format json --output contigs_report.json`
**Explanation:** Produces a JSON file containing individual contig metrics including GC content, ambiguous bases, and estimated complexity scores for downstream analysis.

### Analyze only contigs longer than 500 bp
**Args:** `input.fasta --stats --min-length 500`
**Explanation:** Filters out shorter contigs before calculating assembly statistics, useful for focusing on high-confidence sequence regions.

### Compare contigs against an indexed reference database
**Args:** `input.fasta --reference refs.cti --compare`
** explanation:** Aligns each contig against the pre-built index and reports best matches, coverage depth, and sequence identity. Requires prior index creation with contatester-build.

### Build an index from multiple reference FASTA files
**Args:** `ref1.fasta ref2.fasta ref3.fasta --index-out references.cti`
** explanation:** Creates a unified `.cti` index file from multiple input files for efficient reference-based comparisons in subsequent contatester runs.

### Run with verbose logging for troubleshooting
**Args:** `input.fasta --stats --verbose --log-file contatester.log`
**Explanation:** Enables detailed progress messages written to the log file, useful for diagnosing parsing issues or unexpected filtering behavior.

### Limit memory usage for large assemblies
**Args:** `input.fasta --stats --memory-limit 8G`
**Explanation:** Allocates 8 GB of RAM for internal buffers, preventing out-of-memory crashes when processing assemblies exceeding the default 2 GB limit.

### Output summary to custom file with tabular format
**Args:** `input.fasta --stats --format tsv --output summary.tsv`
**Explanation:** Writes a tab-separated values file with labeled columns for easy import into spreadsheets or custom scripts for further analysis.

### Process gzip-compressed input directly
**Args:** `input.fasta.gz --stats`
**Explanation:** Automatically detects and decompresses gzip-compressed input files in memory, eliminating the need for manual decompression steps.

### Skip contigs with excessive N characters
**Args:** `input.fasta --stats --max-ambiguous 0.05`
**Explanation:** Excludes contigs where the proportion of ambiguous bases (N) exceeds 5% when computing summary statistics, filtering low-quality sequences.