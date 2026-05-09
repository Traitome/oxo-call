---
name: boms
category: bioinformatics/sequences
description: A bioinformatics tool for analyzing and processing biological sequence data, providing utilities for sequence manipulation, filtering, and statistical analysis of molecular sequences.
tags:
  - sequences
  - genomics
  - filtering
  - analysis
author: AI-generated
source_url: https://github.com/example/boms
---

## Concepts

- **Input Formats**: boms accepts standard sequence file formats including FASTA (.fa, .fasta), FASTQ (.fq, .fastq), and plain text with one sequence per line. The tool automatically detects format based on file extension.
- **Output Modes**: Results can be written to stdout for piping, to specified output files, or in batch mode to generate multiple output files. The tool supports TSV, CSV, and JSON output formats.
- **Sequence Processing Model**: boms processes sequences as string objects with metadata, supporting filtering by sequence length, GC content, and pattern matching. Each sequence retains header information for traceability.
- **Filtering Logic**: Filters are applied sequentially using AND logic by default. Use the `--any` flag to switch to OR logic for multi-criteria filtering.

## Pitfalls

- **Empty Output Files**: Running boms without specifying `--out` or using `--out -` without redirecting will print results to stdout. If no sequences pass filters, the output will be empty with no warning message.
- **Case Sensitivity**: Pattern matching with `--pattern` is case-sensitive by default. Sequences containing the pattern in different case will not match, leading to missed sequences in results.
- **Memory Constraints**: Loading large sequence files (>1GB) entirely into memory can cause performance degradation. Use `--chunk-size` to limit memory usage at the cost of multiple file passes.
- **Invalid Sequence Characters**: Sequences containing characters outside standard IUPAC nucleotide codes will trigger errors unless `--allow-ambiguous` is specified, causing the pipeline to fail mid-run.

## Examples

### Filter sequences by minimum length

**Args:** `--min-length 100 input.fa`
**Explanation:** Retains only sequences with 100 or more nucleotides, useful for removing short reads or incomplete sequences from analysis.

### Extract sequences with high GC content

**Args:** `--gc-min 60 input.fq --out high_gc.fasta`
**Explanation:** Selects sequences with GC content >= 60%, helpful for identifying GC-rich regions that may indicate stable secondary structures.

### Search for a specific nucleotide pattern

**Args:** `--pattern "GATK" input.fa --out motif_hits.txt`
**Explanation:** Finds sequences containing the "GATK" motif in uppercase, useful for locating potential binding sites or conserved regions.

### Convert FASTQ to FASTA format

**Args:** `--convert fasta input.fastq --out sequences.fasta`
**Explanation:** Converts input from FASTQ to FASTA format, stripping quality scores and outputting only sequence name and bases.

### Process multiple sequence files in batch

**Args:** `*.fa --out-dir results/ --format csv`
**Explanation:** Processes all FASTA files matching the glob pattern, writing CSV-formatted results to the specified output directory.

### Filter sequences by maximum length

**Args:** `--max-length 500 input.fasta --out short_sequences.fasta`
**Explanation:** Keeps only sequences with 500 or fewer nucleotides, useful for analyzing small RNA fragments or short read datasets.

### Use OR logic for multiple filter criteria

**Args:** `--any --min-length 200 --gc-min 65 input.fa`
**Explanation:** Selects sequences that meet EITHER the minimum length OR the minimum GC content threshold, expanding the result set compared to default AND logic.

### Suppress header output for pipeline compatibility

**Args:** `--no-header input.fa --out results.txt`
**Explanation:** Outputs sequence data without column headers, making the output suitable for direct piping into downstream bioinformatics tools.

### Process sequences with ambiguous characters

**Args:** `--allow-ambiguous input.fa --out clean_output.fasta`
**Explanation:** Allows processing of sequences with non-standard IUPAC nucleotide codes (like N, R, Y) without triggering errors.

### Set custom chunk size for large files

**Args:** `--chunk-size 50000 large_dataset.fa --out filtered.fa`
**Explanation:** Processes the file in 50,000 sequence chunks to manage memory usage when working with large datasets, ensuring stable performance.