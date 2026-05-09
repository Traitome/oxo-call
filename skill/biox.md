---
name: biox
category: Sequence Analysis
description: Fast k-mer counting and biological sequence analysis tool for FASTA/FASTQ files with subcommands for counting, finding, filtering, and statistics.
tags:
  - k-mer
  - sequence-analysis
  - genomics
  - bioinformatics
  - fasta
  - fastq
author: AI-generated
source_url: https://github.com/biox-tools/biox
---

## Concepts

- biox operates as a command-line tool with subcommands (count, find, filter, stats) for different sequence analysis tasks, accepting FASTA, FASTQ, or raw sequence input via stdin or files.
- The k-mer counting functionality uses a probabilistic data structure (Bloom filter or similar) for memory-efficient counting of short sequences across potentially large genomic datasets.
- Output formats include plain text for sequences, JSON for statistics, and binary formats for compact storage of k-mer counts, with verbose flags available for debugging.

## Pitfalls

- Using an incorrect k-mer size (-k value) that is either too small (produces overwhelming unique k-mers) or too large (no matches found) will generate useless or empty output for downstream analysis.
- Forgetting to specify input format when working with non-standard file extensions causes biox to default to FASTA parsing, potentially misinterpreting FASTQ quality lines as sequences.
- Running on very large files without memory limits (-m flag) can cause system memory exhaustion, especially when using counting modes that load entire datasets into memory.

## Examples

### Count 21-mers in a FASTA file
**Args:** count -k 21 input.fasta
**Explanation:** Counts all unique 21-base k-mers in the provided FASTA file, outputting unique k-mer frequencies for downstream bioinformatics analysis.

### Find exact matches of a motif in sequences
**Args:** find -p AGCTAGCT sample.fasta
**Args:** find -p "AGCT?AGCT" sample.fasta
**Explanation:** Searches for exact DNA motif matches (or with wildcard '?' character) within sequences, outputting positions and sequences containing the pattern.

### Filter sequences by minimum length
**Args:** filter -m 100 -M 1000 sequences.fasta
**Explanation:** Filters input sequences to keep only those with length between 100 and 1000 base pairs, useful for preparing datasets for assembly or mapping.

### Get k-mer statistics in JSON format
**Args:** stats -j input.fasta
**Explanation:** Outputs comprehensive statistics (total bases, sequence count, N50, GC content, k-mer distribution) in JSON format for integration with pipelines.

### Process FASTQ with quality filtering
**Args:** filter -q 30 input.fastq -o output.fasta
**Explanation:** Reads FASTQ input, filters sequences where all bases have quality scores above 30, and outputs to FASTA format for conserved sequence analysis.

### Count k-mers with multiple threads
**Args:** count -k 25 -t 8 large_genome.fasta
**Args:** count -k 25 -t 8 large_genome.fasta
**Explanation:** Uses 8 CPU threads to accelerate k-mer counting on large genome files, significantly reducing processing time on multi-core systems.

### Output unique k-mers to file
**Args:** count -k 20 -u input.fasta -o unique_kmers.txt
**Explanation:** Counts 20-mers and outputs only unique k-mers (without counts) to a text file, useful for set operations or creating custom databases.

### Find reverse complement matches
**Args:** find -p ATGC -r target_sequences.fasta
**Explanation:** Searches for both forward and reverse complement matches of the specified pattern, essential for detecting motifs on both DNA strands.