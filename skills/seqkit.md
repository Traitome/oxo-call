---
name: seqkit
category: sequence-utilities
description: Cross-platform ultrafast toolkit for FASTA/Q file manipulation
tags: [fasta, fastq, sequence, qc, statistics, manipulation, filtering]
author: oxo-call built-in
source_url: "https://bioinf.shenwei.me/seqkit/"
---

## Concepts

- seqkit works on both FASTA and FASTQ files and auto-detects gzip/bzip2/xz/zstd compression.
- Use -j N to set the number of threads; use -o output.fa to write to a file.
- seqkit stats gives basic statistics; seqkit fx2tab converts to tabular format for downstream processing.
- seqkit grep uses regex by default; use -s to search in sequences, -n for names; -f for file of patterns.
- seqkit seq -m MIN -M MAX filters by sequence length; -g removes gaps; -r reverse complements.
- seqkit common finds sequences shared across multiple files by ID; seqkit rmdup removes duplicate sequences.
- seqkit locate finds subsequences/motifs with mismatch support and outputs BED/GTF format.
- seqkit subseq extracts subsequences by region (1-based coordinates), BED, or GTF files.
- seqkit faidx creates FASTA index for random access and extracts subsequences efficiently.
- seqkit pair matches up paired-end reads from two FASTQ files when order is disrupted.
- seqkit supports circular genomes with --circular flag for locate and subseq operations.

## Pitfalls

- CRITICAL: seqkit ARGS must start with a subcommand (stats, seq, subseq, sliding, faidx, scat, translate, watch, fa2fq, fq2fa, fx2tab, tab2fx, convert, grep, locate, fish, amplicon, common, rmdup, split, split2, sample, shuffle, sort, merge, concat, replace, rename, restart, head, tail, range, concat, mutate) — never with flags like -j, -o, -f. The subcommand ALWAYS comes first.
- seqkit grep -p matches the full sequence name — use -r (regex) or -n to limit to the ID portion.
- seqkit split/split2 creates many files; use -O to specify output directory.
- seqkit seq -n outputs only names (useful for getting a list of IDs), not the sequences themselves.
- seqkit sample for random subsampling requires -s SEED for reproducibility.
- seqkit translate expects DNA/RNA input; use -f 1 to specify forward strand reading frame.
- seqkit locate patterns with commas must be quoted with double quotes: -p '"A{2,}"' to avoid CSV parsing issues.
- seqkit subseq with BED/GTF on plain FASTA outputs random order; compress the FASTA first to preserve order.
- seqkit faidx requires the FASTA file to be indexed first for efficient random access; use -U to update existing index.

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

### locate motifs in sequences with mismatch tolerance
**Args:** `locate -p "ATG" -m 1 -j 4 input.fa -o locations.bed`
**Explanation:** -p specifies pattern; -m 1 allows 1 mismatch; outputs BED format with match positions

### extract subsequences by region
**Args:** `subseq -r 1:100 -j 4 input.fa -o first_100bp.fa`
**Explanation:** -r specifies 1-based region (start:end); extracts first 100 bases from each sequence

### create FASTA index and extract specific sequences
**Args:** `faidx input.fa && seqkit faidx input.fa seq1 seq2 seq3 -o subset.fa`
**Explanation:** first command creates .fai index; second extracts specific sequences by ID efficiently

### match paired-end reads from two files
**Args:** `pair -1 reads_R1.fastq.gz -2 reads_R2.fastq.gz -o matched_R1.fastq.gz -O matched_R2.fastq.gz`
**Explanation:** matches paired reads when files have different ordering; outputs synchronized pairs

### convert FASTA/Q to tabular format with GC content
**Args:** `fx2tab -g -l -j 4 input.fa -o output.tsv`
**Explanation:** -g adds GC content column; -l adds sequence length; useful for downstream analysis in R/pandas

### remove duplicate sequences by sequence content
**Args:** `rmdup -s -j 4 input.fa -o unique.fa`
**Explanation:** -s removes duplicates by sequence (not ID); keeps first occurrence; useful for dereplicating databases
