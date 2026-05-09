---
name: c4counter
category: sequence-analysis
description: A lightweight bioinformatics tool for counting k-mer frequencies, codon usage, and sequence composition patterns in FASTA/FASTQ files. Outputs counts in compact binary or human-readable text formats.
tags:
  - k-mer
  - codon-counting
  - sequence-analysis
  - genomics
  - read-counting
author: AI-Generated
source_url: https://github.com/example/c4counter
---

## Concepts

- **Input formats**: c4counter accepts raw DNA/RNA sequences as input via STDIN or file arguments using `.fa`, `.fasta`, `.fq`, or `.fastq` extensions. It auto-detects format based on the presence of quality scores in FASTQ headers (`@` vs `>`).
- **K-mer counting model**: The tool treats each input sequence as a sliding window of length k (default k=4). All overlapping k-mers are extracted, canonicalized (reverse-complemented if needed), and tallied in a hash table before output.
- **Output modes**: Counts are written to STDOUT in one of three modes: `simple` (one k-mer per line), `table` (space-delimited matrix), or `binary` (fixed-width binary for space efficiency). Use the `--out-format` flag to select.
- **Canonical k-mer mode**: When `--canonical` is set, each k-mer and its reverse complement are treated as the same entity, with the lexicographically smaller form stored as the key. This prevents double-counting in double-stranded sequences.

## Pitfalls

- **Specifying k-mer length incorrectly**: Setting `--kmer-length` to a value larger than the shortest input read will silently produce zero counts for that length, which is easily mistaken for an empty file or a processing error.
- **Using text output for large datasets**: The `simple` output format grows with the number of distinct k-mers (potentially 4^k entries), which can exhaust disk space on typical file systems when k is large or the genome is repetitive.
- **Ignoring the `--canonical` flag for palindromic sequences**: Palindromic k-mers (e.g., "GGCC" for k=4) are counted only once per strand regardless of canonical mode, but failing to account for this leads to incorrect normalization in downstream analyses.
- **Mixing FASTQ quality score handling**: c4counter ignores quality scores by default and only counts sequence content. Passing FASTQ data without awareness of this will produce nucleotide counts only, not per-base quality statistics.
- **Assuming input is pre-filtered**: The tool does not automatically mask low-complexity regions or adapters. Feeding in contaminated reads inflates k-mer counts for adapter artifacts, skewing biological conclusions.

## Examples

### Count 4-mers in a single FASTQ file
**Args:** `input.fastq -k 4 --out-format table`
**Explanation:** This counts all overlapping 4-mers in the input FASTQ file and prints a tab-delimited table where each row is a k-mer and its count.

### Count canonical 6-mers from multiple FASTA files
**Args:** `ref1.fa ref2.fa -k 6 --canonical --out-format simple`
**Explanation:** Canonical mode merges each k-mer with its reverse complement, and the simple format outputs one entry per line for easy inspection.

### Export binary counts to a specific output file
**Args:** `reads.fq -k 5 --out-format binary -o kmer_counts.bin`
**Explanation:** Binary output encodes counts as fixed-width little-endian integers, which is efficient for downstream tools that re-ingest c4counter's own binary format.

### Count k-mers with a minimum occurrence threshold
**Args:** `genome.fa -k 4 --min-count 10 --out-format table`
**Explanation:** The `--min-count` flag suppresses entries with fewer than the specified occurrences, reducing output noise for large genomes.

### Count all k-mers of lengths 2 through 5 in one command
**Args:** `sample.fa -k 2-5 --out-format table`
**Explanation:** A range specification generates separate count sets for each k, with output blocks delimited by header lines indicating the k-mer length.

### Process compressed input using piped decompression
**Args:** `gunzip -c reads.fastq.gz | c4counter -k 4 --out-format simple`
**Explanation:** c4counter reads from STDIN, so decompressed content can be piped directly without creating an intermediate uncompressed file, saving disk space.

### Count codon usage in a coding sequence file
**Args:** `cds.fa -k 3 --canonical --out-format table`
**Explanation:** Codons are 3-mers in reading frames; canonical mode ensures forward and reverse strands are not double-counted when analyzing coding sequences.

### Suppress header comments in output
**Args:** `seqs.fa -k 4 --no-header --out-format simple`
**Explanation:** The `--no-header` flag removes all comment lines from output, producing clean numeric data suitable for parsing by scripts.

### Use a custom hash table size for memory control
**Args:** `large_genome.fa -k 4 --hash-size 1000000 --out-format binary`
**Explanation:** Specifying `--hash-size` pre-allocates the internal hash table, preventing rehashing overhead and capping memory usage for the job.

### Count and immediately sort by frequency descending
**Args:** `transcripts.fa -k 3 --out-format table | sort -k2 -nr`
**Explanation:** The table output is pipe-friendly; sorting the second numeric column produces a ranked list of most-observed k-mers for quick exploratory analysis.