---
name: ccne
category: Bioinformatics
description: A command-line tool for analysis of genomic data, supporting operations such as sequence filtering, transformation, and statistical analysis.
tags: [genomics, sequence-analysis, bioinformatics, command-line]
author: AI-generated
source_url: https://github.com/example/ccne
---

## Concepts

- **Input Format**: Accepts standard FASTA, FASTQ, or plain text files containing genomic sequences, with optional compressed (.gz) input support.
- **Output Modes**: Produces filtered or transformed sequences in FASTA/FASTQ format, optionally generating summary statistics in TSV or JSON format.
- **Key Functionality**: Performs sequence-level operations including quality filtering, length-based selection, and basic transformations like reverse complementation.
- **Filtering Criteria**: Supports multiple filter flags that can be combined using AND/OR logic for complex filtering workflows.
- **Stream Processing**: Can process files streaming-style without loading entire datasets into memory, enabling handling of large files.

## Pitfalls

- **Omitting Output Flag**: Running without specifying `--output` or `-o` will write to stdout, which may cause data loss if the terminal buffer is limited or redirected unexpectedly.
- **Incompatible Filter Combinations**: Using mutually exclusive filter flags (e.g., conflicting length ranges) willresult in empty output with no error message.
- **Case Sensitivity**: Input sequence identifiers are case-sensitive; mismatched headers will cause tools expecting exact matches to fail silently.
- **Memory Limits with Large Files**: Processing extremely large files without streaming mode can exhaust available RAM, causing the process to terminate unexpectedly.
- **Missing Required Arguments**: Forgetting to specify required input files will produce a generic error message without clarifying which argument is missing.

## Examples

### Filter sequences shorter than 50 bases
**Args:** `--min-length 50 --input sequences.fasta`
**Explanation:** Retains only sequences with length greater than or equal to 50 bases, removing shorter sequences from the output.

### Output filtered sequences to a new file
**Args:** `--input reads.fq.gz --output filtered.fq.gz`
**Explanation:** Writes the processed or filtered sequences to a specified output file instead of streaming to stdout.

### Reverse complement all sequences in FASTA
**Args:** `--reverse-complement --input input.fasta`
**Explanation:** Transforms each input sequence to its reverse complement while preserving the original sequence order in the output.

### Filter sequences with minimum average quality score
**Args:** `--min-quality 30 --input reads.fastq`
**Explanation:** Removes sequencing reads where the average Phred quality score falls below the specified threshold.

### Generate summary statistics in JSON format
**Args:** `--stats --output report.json --input sequences.fasta`
**Explanation:** Produces a machine-readable JSON report containing sequence counts, length distributions, and base composition metrics.

### Combine multiple filters with AND logic
**Args:** `--min-length 100 --max-length 500 --min-gc 40 --input genome.fasta`
**Explanation:** Applies sequential filtering to retain only sequences matching all specified criteria simultaneously.

### Process gzip-compressed input directly
**Args:** `--input reads.fastq.gz --output results.txt`
**Explanation:** Reads compressed FASTQ files without explicit decompression, reducing disk usage during processing.

### Filter by sequence header pattern
**Args:** `--filter-header "chr[0-9]+" --input alignment.fasta`
**Explanation:** Retains only sequences with headers matching the provided regular expression pattern.

### Convert FASTQ to FASTA format
**Args:** `--convert fasta --input reads.fastq`
**Explanation:** Transforms input sequences from FASTQ to FASTA format, stripping quality information from output.