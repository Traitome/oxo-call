---
name: chum
category: k-mer counting and analysis
description: A fast, disk-based k-mer counting tool for genomic analysis. Chum efficiently counts k-mers from large FASTA/FASTQ datasets using streaming algorithms with low memory footprint, and supports export of k-mer spectra and abundance distributions.
tags: [k-mers, read-processing, genomics, streaming, disk-based]
author: AI-generated
source_ url: https://github.com/yhhshb/chum
---

## Concepts

- Chum processes input reads sequentially from FASTA or FASTQ files, counting exact k-mers using a hash-based disk-backed algorithm that allows counting of k-mers from whole-genome datasets without loading entire datasets into RAM.
- Output formats include JSON k-mer counts, text-based abundance spectra, and binary format for downstream tools; the tool can export both per-kmer counts and aggregate statistics such as coverage histograms.
- K-mer size (k) is configurable but must be consistent throughout a counting session; typical values range from 21 to 31 for read error correction and variant detection workflows, with larger k values reducing noise from sequencing errors.
- The tool supports streaming from standard input, making it compatible with read preprocessing pipelines using Unix pipes; input can be compressed (gzip/bzip2) directly.
- Companion binary `chum-lookup` allows querying a pre-built k-mer count database for abundance values, enabling rapid filtering of reads or identification of high-coverage genomic regions.

## Pitfalls

- Running chum without specifying output format produces text output that can be extremely large for whole-genome datasets, causing disk space exhaustion; always specify binary output (-b flag) or use a compressed format when working with mammalian genomes.
- K-mer size must be specified identically when building a count database and when querying it with chum-lookup; mismatched k values silently produce incorrect zero counts because k-mer hashes are position-dependent.
- Memory usage can exceed the default limit on small-memory systems when counting very large datasets, resulting in OOM termination; always estimate memory requirements (approximately 1 GB per 100M distinct k-mers) and set appropriate limits with `ulimit` or Docker memory flags.
- Specifying an output filename that already exists causes the tool to fail silently or overwrite without warning; explicitly check for existing files or use the `-f` force flag when overwriting is intended.
- Using chum on paired-end reads without first interleaving or concatenating them produces incorrect k-mer counts because read pairs may not be adjacent; always preprocess paired-end data with a tool like `paste` or `fastqCombinePairedEnds.py` before counting.

## Examples

### Count k-mers from a single FASTQ file and save as binary output
**Args:** `k 27 -o genome.k27.fastq.gz -b`
**Explanation:** Specifies k-mer length of 27, directs output to a binary file (fastq.gz is the conventional extension for binary output), and uses binary mode to reduce disk usage and enable downstream lookup operations.

### Count k-mers streaming from stdin with gzip-compressed input
**Args:** `k 25 | chum-lookup -d genome.k25.db -`
**Explanation:** Demonstrates streaming usage where raw k-mers (printed by a hypothetical raw-kmer tool) are piped directly into the lookup companion, enabling real-time analysis of k-mer abundances without intermediate file creation.

### Build a k-mer database from multiple input files
**Args:** `k 31 -o assembly.k31.db -b file1.fq.gz file2.fq.gz file3.fq.gz`
**Explanation:** Processes three input files sequentially to build a single k-mer count database containing counts aggregated across all inputs, which is essential for comprehensive genomic or metagenomic analysis.

### Export k-mer abundance histogram without binary output
**Args:** `k 23 --histogram -o coverage_hist.txt input.fa.gz`
**Explanation:** Uses the histogram mode to output only the distribution of k-mer abundances (how many k-mers occur 1 time, 2 times, etc.) rather than per-kmer counts, which is useful for quality assessment and error profiling without storing full k-mer tables.

### Query k-mer abundance for specific sequences in a pre-built database
**Args:** `chum-lookup -d ecoli.k21.db -q candidate_kmers.txt`
**Explanation:** Uses the companion lookup tool to query a k-mer database (ecoli.k21.db) for abundances of k-mers listed in candidate_kmers.txt, outputting a table of k-mer sequences with their count values for validation or filtering workflows.