---
name: cagee
category: Genomics
description: A command-line tool for extracting and analyzing conserved genomic elements from sequence data. Computes element coverage, generates statistics, and supports batch processing of multiple input files.
tags:
- genomics
- sequence-analysis
- elements
- coverage
- extract
author: AI-generated
source_url: https://github.com/cagee-tool/cagee
---

## Concepts

- **Input formats**: cagee accepts FASTA, FASTQ, and SAM/BAM formats for query sequences. Use `--query` to specify the input file. Reference databases must be pre-built with the companion tool `cagee-build`.
- **Output modes**: By default, cagee prints results to stdout in tab-separated format. Use `--output` to write to a file, or `--json` for JSON output. The `--summary` flag generates a statistical summary report.
- **Index system**: Before running queries, reference sequences must be indexed using `cagee-build`. This creates a .cge binary index file that accelerates subsequent searches. Rebuild indices when reference sequences change.
- **Filtering thresholds**: Use `--min-identity` (default 0.90) and `--min-length` (default 50) to filter results. These flags accept values between 0 and 1 for identity, and positive integers for length.

## Pitfalls

- **Forgetting to build an index**: Running `cagee` without a pre-built index using `cagee-build` will fail with an "index not found" error. Always run `cagee-build` first on your reference sequences before querying.
- **Mismatched file formats**: Providing a FASTQ query when the tool expects FASTA (or vice versa) causes parsing errors. Ensure your `--query` input matches the expected format or use `--auto-detect` for automatic format recognition.
- **Insufficient memory for large indexes**: Loading large .cge index files into memory can cause out-of-memory errors on systems with limited RAM. Use the `--mmap` flag to memory-map the index instead of loading it entirely.
- **Overly restrictive thresholds**: Setting `--min-identity` too high (e.g., 0.99) when querying short sequences may return zero results, as sequence errors reduce effective identity. Validate thresholds on a subset of data first.

## Examples

### Extract elements from a FASTA query file
**Args:** `--query sequences.fasta --ref my_index.cge`
**Explanation:** This runs a standard extraction using query sequences from a FASTA file against a pre-built reference index.

### Output results to a specific file
**Args:** `--query input.fasta --ref index.cge --output results.tsv`
**Explanation:** Redirects the tabular output to results.tsv instead of printing to stdout, useful for downstream processing.

### Generate a JSON summary report
**Args:** `--query reads.fq --ref ref.cge --json --summary`
**Explanation:** Produces machine-readable JSON output with an added summary section containing coverage statistics.

### Filter results by minimum identity threshold
**Args:** `--query query.fa --ref index.cge --min-identity 0.95`
**Explanation:** Only returns matches with at least 95% sequence identity, useful for high-confidence element calls.

### Use memory-mapped indexing for large references
**Args:** `--query data.fasta --ref large_ref.cge --mmap`
**Explanation:** Memory-maps the index file rather than loading it完全 into RAM, preventing out-of-memory errors on large datasets.

### Adjust minimum length filter
**Args:** `--query input.fa --ref idx.cge --min-length 100`
**Explanation:** Filters out matches shorter than 100 bases, keeping only longer element calls in the output.

### Process multiple query files in batch mode
**Args:** --query batch_list.txt --ref index.cge --batch`
**Explanation:** Reads multiple query files listed in batch_list.txt and processes them sequentially, writing separate output files for each.