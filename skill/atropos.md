---
name: atropos
category: Sequencing Read Processing
description: Atropos is an Illumina sequencing read trimmer that removes adapters and low-quality bases using probabilistic alignment. It supports paired-end and single-end reads, multi-threaded execution, and gzipped I/O with automatic format detection.
tags: [trimming, quality-control, adapters, fastq, paired-end]
author: AI-generated
source_url: https://github.com/jdidion/atropos
---

## Concepts

- Atropos consumes FASTQ files (gzipped or plain) on stdin or via `-e`/`--input` and produces trimmed FASTQ to stdout or via `-o`/`--output`. Input format (FASTQ, FASTA, or BAM) is auto-detected from file magic bytes, so explicit `-f`/`--input-format` is only needed for non-standard encodings.
- Trim modes are controlled by distinct flags: `-a`/`--adapter` trims from read 3′ ends, `-A` trims the second read in a pair, `-b`/`--anywhere` trims at any position, `-g`/`--global` applies trimming symmetrically, and `-u`/`--cut` removes a fixed number of bases from either end. Multiple adapter sequences can be supplied by repeating these flags.
- Quality trimming is governed by `-q`/`--quality-cutoff` (Phred score threshold, applied from 3′ end by default) and `-q2`/`--quality-cutoff2` for the second read. Reads shorter than `--min-length` after trimming are discarded unless `--max-n` is exceeded.
- Paired-end trimming uses `-pe1`/`--pair-input` and `-pe2`/`--pair-output` for the second read file, with `--paired-reads-mode` controlling behavior: `2`, `t` (trim both reads to same length), or `o` (require both reads to survive). Misaligned read pairs can be rescued using `--overlap` or `--aligner下次`.
- The `-w`/`--width` and `--indel-tolerance` flags control aligner sensitivity. Higher `-w` values (default 10) allow longer insert sizes to be inferred, while `--indel-tolerance` (default 0.04) permits insertions/deletions during adapter alignment.

## Pitfalls

- Specifying only `-a` without `-A` for paired-end data leaves read 2 untrimmed, because Atropos treats each read independently by default. The `-A` flag must be set explicitly for mate trimming, otherwise residual adapter sequences corrupt downstream mapping.
- Using `--max-n` without `--min-length` silently discards reads with excessive `N` bases but no length enforcement, so short `N`-heavy fragments pass through and corrupt assembly or variant calling. Always pair `--max-n` with a suitable `--min-length` threshold.
- Setting `-w` too low (e.g., below 5) collapses reads with long insert sizes, causing valid fragments to be misclassified as adapter-contaminated and inflating the proportion of trimmed-out reads in the output. Calibrate `-w` against the expected library insert size distribution.
- Inconsistent pairing modes (`--paired-reads-mode`) between input and output files causes record-order mismatches and crashes when writing interleaved output. Verify the mode is consistent or use separate output files with mode `2`.
- Forgetting that output compression is independent of input detection causes silent issues: output FASTQ files are not automatically gzip-compressed, so downstream tools expecting `.fq.gz` may fail. Always specify `-o output.fq.gz` or pipe through `gzip` explicitly.

## Examples

### Trim a single-end FASTQ file with a known adapter sequence
**Args:** `-e input.fq.gz -o trimmed.fq.gz -a AGATCGGAAGAGCACACGTCTGAACTCCAGTCAC`
**Explanation:** Atropos reads the gzipped input, aligns the given 3′ adapter against each read, and removes the adapter plus downstream bases, writing the clean result to the gzipped output file.

### Trim paired-end reads with separate adapter sequences for each read
**Args:** `-e R1.fq.gz -o R1.trim.fq.gz -pe2 R2.fq.gz -o2 R2.trim.fq.gz -a AGATCGGAAGAGC -A CTGTCTCTTATACACATCT -q 20`
**Explanation:** Both read files are trimmed independently for their respective adapters, and any bases with Phred scores below 20 are removed from the 3′ end of each read before output.

### Trim fixed 5 bases from the 5′ end and remove reads shorter than 50 bp after trimming
**Args:** `-e input.fq.gz -o trimmed.fq.gz -u 5 -u 0 --min-length 50`
**Explanation:** Atropos cuts the first 5 bases from the 5′ end (using `-u 5`) and the first 0 bases from the 3′ end, discarding any reads that fall below 50 bp in final length.

### Trim with multi-threaded execution using 8 CPU cores
**Args:** `-e input.fq.gz -o trimmed.fq.gz -a AAAAACCCCCTTTTT -T 8 --insert-match-order first`
**Explanation:** Atropos spawns 8 worker threads to parallelize alignment, and prioritizes insert-match detection over other trim operations via `--insert-match-order`, significantly accelerating large file processing.

### Apply quality trimming and discard reads with more than 2 N bases
**Args:** `-e input.fq.gz -o trimmed.fq.gz -q 30 --max-n 2 --min-length 36`
**Explanation:** Bases below Phred 30 are removed from each read's 3′ end, and any read containing more than 2 ambiguous bases or shorter than 36 bp is discarded entirely from the output stream.