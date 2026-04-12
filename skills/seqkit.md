---
name: seqkit
category: sequence-utilities
description: Cross-platform ultrafast toolkit for FASTA/Q file manipulation
tags: [fasta, fastq, sequence, qc, statistics, manipulation, filtering]
author: oxo-call built-in
source_url: "https://bioinf.shenwei.me/seqkit/"
---

## Concepts

- seqkit works on both FASTA and FASTQ files and auto-detects gzip/bzip2/xz compression.
- Use -j N to set the number of threads; use -o output.fa to write to a file.
- seqkit stats gives basic statistics; seqkit fx2tab converts to tabular format for downstream processing.
- seqkit grep uses regex by default; use -s to search in sequences, -n for names; -f for file of patterns.
- seqkit seq -m MIN -M MAX filters by sequence length; -g removes gaps; -r reverse complements.
- seqkit common finds sequences shared across multiple files by ID; seqkit rmdup removes duplicate sequences.

## Pitfalls

- seqkit grep -p matches the full sequence name — use -r (regex) or -n to limit to the ID portion.
- seqkit split/split2 creates many files; use -O to specify output directory.
- seqkit seq -n outputs only names (useful for getting a list of IDs), not the sequences themselves.
- seqkit sample for random subsampling requires -s SEED for reproducibility.
- seqkit translate expects DNA/RNA input; use -f 1 to specify forward strand reading frame.

## Examples

### get basic statistics of a FASTQ file (read count, total bases, average length)
**Args:** `stats -a reads.fastq.gz`
**Explanation:** -a shows all statistics including N50, Q20/Q30 percentages

### filter reads shorter than 100 bp and write to a new file
**Args:** `seq -m 100 -j 4 -o filtered.fastq.gz input.fastq.gz`
**Explanation:** -m 100 keeps reads ≥100 bp; -j 4 uses 4 threads; auto-compresses based on output extension

### get the reverse complement of all sequences in a FASTA file
**Args:** `seq -r -p -j 4 input.fa -o revcomp.fa`
**Explanation:** -r reverses; -p complements (so -r -p gives reverse complement)

### extract sequences by name from a list file
**Args:** `grep -f id_list.txt input.fa -o subset.fa`
**Explanation:** -f reads patterns from a file (one per line); outputs matching sequences

### randomly sample 10000 reads from a large FASTQ file
**Args:** `sample -n 10000 -s 42 -j 4 -o sample.fastq.gz input.fastq.gz`
**Explanation:** -n specifies count; -s sets random seed for reproducibility

### convert FASTQ to FASTA
**Args:** `fq2fa -j 4 input.fastq.gz -o output.fa.gz`
**Explanation:** fq2fa subcommand handles the conversion; preserves compression if output ends in .gz

### split FASTA file into chunks of 1000 sequences each
**Args:** `split2 -s 1000 -j 4 -O split_output input.fa`
**Explanation:** -s sets sequences per file; -O specifies output directory
