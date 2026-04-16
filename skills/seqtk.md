---
name: seqtk
category: sequence-utilities
description: Fast and lightweight toolkit for processing FASTA and FASTQ files
tags: [fastq, fasta, sequence, subsample, format-conversion, trimming, utility]
author: oxo-call built-in
source_url: "https://github.com/lh3/seqtk"
---

## Concepts

- seqtk is a versatile toolkit for FASTA/FASTQ processing; subcommands: seq, subseq, sample, trimfq, mergefa, fqchk, size, comp, mergepe, split.
- seqtk seq converts between FASTA and FASTQ formats and performs basic sequence manipulations.
- seqtk sample subsamples reads with a fixed seed for reproducibility; use the SAME seed for paired-end files.
- seqtk trimfq trims reads by quality score; seqtk subseq extracts sequences by name or coordinate.
- seqtk always outputs to stdout — pipe or redirect to save output.
- seqtk seq -a converts FASTQ to FASTA; seqtk seq -q converts FASTA to FASTQ with quality 40.
- Use seqtk seq -l 60 to wrap FASTA sequences at 60 characters per line (NCBI format).
- seqtk size reports the number of sequences and total bases; useful for quick file statistics.
- seqtk comp computes nucleotide composition (A, C, G, T, N counts) for QC.
- seqtk mergepe interleaves two paired-end FASTQ files into one interleaved file.
- seqtk split divides one FASTQ/FASTA file into multiple smaller files.
- seqtk fqchk provides FASTQ quality statistics including base quality distribution.

## Pitfalls

- seqtk ARGS must start with a subcommand (seq, size, comp, sample, subseq, fqchk, mergepe, split, trimfq, hety, gc, mutfa, mergefa, famask, dropse, rename, randbase, cutN, gap, listhet, hpc, telo) — never with flags or input files first. The subcommand ALWAYS comes first.
- seqtk sample paired-end reads MUST use the same seed for both files to maintain read pairing.
- seqtk outputs to stdout — always redirect to a file or pipe to another tool.
- seqtk sample N when N < 1 treats it as a fraction; when N > 1 treats it as absolute count.
- seqtk does not support gzipped output directly — pipe to gzip: seqtk seq input.fq | gzip > output.fq.gz.
- seqtk subseq for region extraction requires a sorted BED file and indexed FASTA.
- seqtk trimfq quality trimming uses Phred-encoded scores — check for phred33 vs phred64.
- seqtk seq -Q INT sets quality shift; default is 33 for Phred+33 (Sanger), use 64 for Illumina 1.5+.
- seqtk trimfq -q FLOAT uses error rate, not Phred score directly (0.05 ≈ Q13, 0.01 ≈ Q20).
- seqtk split creates files in current directory by default; use -p prefix to control output naming.

## Examples

### subsample 1 million read pairs from paired-end FASTQ files
**Args:** `sample -s 42 R1.fastq.gz 1000000 | gzip > sub_R1.fastq.gz`
**Explanation:** -s 42 random seed; same seed must be used for R2: seqtk sample -s 42 R2.fastq.gz 1000000 | gzip > sub_R2.fastq.gz

### convert FASTQ to FASTA format
**Args:** `seq -a reads.fastq.gz > reads.fasta`
**Explanation:** -a flag outputs FASTA format; works on both compressed and uncompressed FASTQ

### reverse complement all sequences in a FASTA file
**Args:** `seq -r sequences.fasta > revcomp.fasta`
**Explanation:** -r flag reverse complements all sequences

### extract specific sequences by name from a FASTQ file
**Args:** `subseq reads.fastq.gz read_names.txt > extracted_reads.fastq`
**Explanation:** read_names.txt contains one read name per line; extracts matching reads

### quality trim reads below Phred 20 from both ends
**Args:** `trimfq -q 0.05 reads.fastq.gz | gzip > trimmed.fastq.gz`
**Explanation:** -q sets the trimming error rate (0.05 ≈ Phred 13); alternatively use -l and -r for fixed trimming

### subsample 10% of reads with reproducible seed
**Args:** `sample -s 100 reads.fastq.gz 0.1 > subsampled_10pct.fastq`
**Explanation:** -s 100 random seed; 0.1 is 10% fraction; for PE use same seed on both files

### get file statistics (sequence count, total bases)
**Args:** `size reads.fastq.gz`
**Explanation:** outputs number of sequences and total base count; fast way to check file size without full parsing

### compute nucleotide composition
**Args:** `comp reads.fastq.gz > composition.txt`
**Explanation:** outputs A, C, G, T, N counts and percentages per sequence; useful for GC content analysis

### interleave two paired-end FASTQ files
**Args:** `mergepe R1.fastq.gz R2.fastq.gz | gzip > interleaved.fastq.gz`
**Explanation:** merges R1 and R2 into interleaved format (R1, R2, R1, R2...); useful for tools requiring interleaved input

### split FASTQ into multiple files
**Args:** `split -n 4 reads.fastq.gz`
**Explanation:** splits into 4 output files; use -p prefix to set output file prefix (default: stdin.split.0001.fa)

### check FASTQ quality statistics
**Args:** `fqchk reads.fastq.gz > qc_stats.txt`
**Explanation:** outputs base quality distribution and other QC metrics; useful for assessing data quality before analysis

### mask low-quality bases
**Args:** `seq -q 20 reads.fastq.gz | gzip > masked.fastq.gz`
**Explanation:** -q 20 masks bases with quality < 20; masked bases become 'N'; useful for downstream variant calling

### drop sequences containing ambiguous bases
**Args:** `seq -N reads.fastq.gz | gzip > clean.fastq.gz`
**Explanation:** -N removes sequences with any N bases; useful for filtering out low-quality reads
