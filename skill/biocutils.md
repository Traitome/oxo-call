---
name: biocutils
category: Bioinformatics Utilities
description: A collection of command-line utilities for common bioinformatics tasks including format conversion, sequence manipulation, quality control, and file statistics.
tags: [bioinformatics, sequence, format-conversion, quality-control, utils]
author: AI-generated
source_url: https://github.com/biocutils/biocutils
---

## Concepts

- **Module-based architecture**: biocutils consists of multiple companion binaries (biocutils-seq, biocutils-convert, biocutils-stats, etc.) for different operations. Each subcommand handles a specific task.
- **Standard I/O streams**: Tools accept input from stdin when no input file is specified, and write to stdout by default. This enables piping between biocutils modules and other bioinformatics tools.
- **Supported formats**: Common formats include FASTA, FASTQ, SAM, BAM, VCF, BED, and CSV. Format auto-detection is based on file extension or content sniffing when possible.
- **Streaming processing**: Most modules process data line-by-line or record-by-record, enabling handling of large files without loading entire datasets into memory.

## Pitfalls

- **Forcing wrong format**: Specifying an incorrect format with `--format` when the input is actually in a different format will produce silently corrupted output or parsing errors.
- **Missing input causes stdin hang**: Running a biocutils module without input and without redirecting stdin will cause the tool to hang waiting for input from the terminal.
- **Incompatible output format**: Attempting to convert between formats that have incompatible data structures (e.g., converting a FASTQ with quality scores to FASTA without handling quality data) will either error or truncate data.
- **Large file memory issues**: Some subcommands like `biocutils-stats --all` may load entire files into memory for global statistics, causing memory exhaustion on very large datasets.

## Examples

### Convert FASTQ to FASTA format

**Args:** convert --input example.fastq --output example.fasta --from fastq --to fasta

**Explanation:** Converts a FASTQ file containing sequences with quality scores to FASTA format, stripping quality data as it is not supported in FASTA.

### Get basic sequence statistics

**Args:** stats --input sequences.fasta

**Explanation:** Displays statistics including total sequence count, total base count, average length, N50, and GC content for a FASTA file.

### Reverse complement a DNA sequence

**Args:** seq --input sequence.fasta --reverse-complement

**Explanation:** Takes DNA sequences from a FASTA file and outputs their reverse complements, preserving the original strand in the header.

### Filter sequences by minimum length

**Args:** filter --input reads.fastq --min-length 50 --format fastq

**Explanation:** Filters a FASTQ file to keep only sequences with at least 50 bases, preserving quality scores for retained sequences.

### Count k-mers in sequences

**Args:** kmer --input sequence.fasta -k 3 --count

**Explanation:** Counts all 3-mers (triucleotides) across all sequences in a FASTA file and outputs frequencies sorted by count.

### Subsample sequences to a target number

**Args:** sample --input reads.fastq --number 10000 --format fastq

**Explanation:** Randomly selects 10,000 sequences from a FASTQ file, useful for creating test datasets or reducing computational load.

### Extract specific fields from a TSV/CSV

**Args:** select --input data.tsv --columns 1,3,5 --delimiter $'\t'

**Explanation:** Extracts columns 1, 3, and 5 from a tab-delimited file, useful for pulling specific annotations from bioinformatics tables.

### Validate format integrity

**Args:** validate --input sequences.fasta

**Explanation:** Checks FASTA file for common issues like malformed headers, invalid characters, or inconsistent sequence lengths without modifying the input.

### Calculate coverage from BED file

**Args:** coverage --bed regions.bed --genome genome.sizes

**Explanation:** Calculates genomic coverage statistics from a BED file against a genome size file, reporting total and per-chromosome coverage.

### Merge multiple FASTA files

**Args:** merge --inputs file1.fasta file2.fasta --output combined.fasta

**Explanation:** Combines sequences from multiple FASTA files into a single output file, useful for consolidating related sequence collections.