---
name: bmfilter
category: k-mer Analysis / Filtering
description: A Bloom filter-based sequence filtering tool for fast membership testing against large k-mer sets. Reads sequences from FASTQ/FASTA files and reports which sequences contain k-mers present (or absent) in a pre-built Bloom filter. Optimized for metagenomic classification, contaminant screening, and read filtering tasks where speed and memory efficiency are critical.
tags: [bloom-filter, k-mer, filtering, sequence-analysis, fastq, fasta, metagenomics, contaminant-screening]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bmfilter
---

## Concepts

- bmfilter uses probabilistic Bloom filter data structures to test whether k-mers from input sequences match any k-mers stored in the filter. A match indicates that sequence contains at least one k-mer present in the filter's source dataset (e.g., contaminant database, host genome, or known taxa).
- Input format: Sequences are provided in FASTQ or FASTA format via stdin or file arguments. The Bloom filter itself is loaded from a `.bf` file created by `bmfilter-build`. Both plain text and gzipped (.gz) input files are supported.
- Output behavior: By default, bmfilter outputs sequences that contain at least one matching k-mer. The `-v` flag inverts this behavior, outputting sequences with no matching k-mers instead. Exit codes indicate number of matching sequences found.

## Pitfalls

- **Mismatched k-mer size**: If the Bloom filter was built with `-k 31` but filtering is done with `-k 21`, results are invalid since k-mer sizes must match exactly between filter construction and usage. Always verify k-mer size consistency.
- **Empty filter file**: Loading a filter file that was not properly constructed or is empty (zero entries) will cause all sequences to be considered non-matches, leading to complete data loss without error warnings.
- **Confusing -v behavior**: Using `-v` outputs non-matching sequences, which is useful for subtracting host reads. However, forgetting this flag when performing contaminant screening results in retaining the contaminants instead of removing them.

## Examples

### Filter reads against a contaminant Bloom filter
**Args:** `-f contaminants.bf input.fastq.gz`
**Explanation:** Loads the Bloom filter from `contaminants.bf` and reads from `input.fastq.gz`, outputting sequences that contain matching k-mers (reads likely containing contaminants).

### Output sequences with NO matches (host read extraction)
**Args:** `-f host.bf -v reads.fq.gz -o host_reads.fq.gz`
**Explanation:** Inverts match logic with `-v` so that non-matching sequences are output, effectively extracting host reads by removing all sequences with k-mers in the host filter.

### Set explicit k-mer size override
**Args:** `-f database.bf -k 25 input.fq -o output.fq`
**Explanation:** Explicitly sets k-mer size to 25 regardless of filter metadata, useful when filter file lacks embedded k-mer size or when testing different k-mer lengths.

### Suppress output and count matches only
**Args:** `-f target.bf -c reads.fq`
**Explanation:** Runs without writing output sequences; the `-c` flag reports only the count of matching sequences to stdout, useful for quick abundance estimation.

### Use paired-end mode for split output
**Args:** `-f filter.bf -p R1.fq.gz R2.fq.gz -a both`
**Explanation:** In paired-end mode (`-p`), requires both reads in a pair to contain matching k-mers before outputting either read, controlled by `-a both` (alternatives: `any`, `R1`).

### Set minimum match threshold (2 unique k-mers)
**Args:** `-f db.bf -m 2 sequences.fq -o filtered.fq`
**Explanation:** Requires at least 2 unique matching k-mers before reporting a sequence as matching, reducing false positives from spurious single k-mer matches.