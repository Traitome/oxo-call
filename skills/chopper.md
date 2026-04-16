---
name: chopper
category: qc
description: Quality filtering and trimming of Oxford Nanopore reads based on quality score and length
tags: [nanopore, long-read, qc, filtering, trimming, quality-control, ont, rust, nanofilt]
author: oxo-call built-in
source_url: "https://github.com/wdecoster/chopper"
---

## Concepts

- chopper filters and trims Oxford Nanopore reads by quality score and length; it replaces NanoFilt.
- chopper reads from stdin and writes to stdout — pipe from gunzip/cat and pipe to gzip.
- Use -q to set minimum mean quality (Phred score); -l for minimum read length; --maxlength for maximum.
- Use --headcrop N to trim N bases from the start; --tailcrop N to trim N bases from the end.
- Use --threads N for parallel processing.
- Standard usage: gunzip -c reads.fastq.gz | chopper -q 10 -l 1000 | gzip > filtered.fastq.gz
- chopper is Rust-based and much faster than NanoFilt for large ONT datasets.
- --trim-approach offers four strategies: fixed-crop, trim-by-quality, best-read-segment, split-by-low-quality.
- --cutoff sets the quality threshold for trim-by-quality and best-read-segment approaches.
- --mingc and --maxgc filter reads by GC content percentage.
- --inverse outputs reads that FAIL filters instead of those that pass.
- -i/--input allows reading from a file instead of stdin (optional).

## Pitfalls

- chopper reads from stdin — must pipe input, not pass a file argument.
- gzipped input requires gunzip -c or zcat piped to chopper; gzipped output requires piping to gzip.
- --headcrop and --tailcrop values are in bases, not quality scores.
- Quality threshold (-q) applies to mean read quality, not per-base quality.
- Minimum length (-l 1000) is essential for most long-read analyses — short reads from DNA fragmentation add noise.
- chopper does NOT replace Porechop for adapter trimming — run Porechop before chopper for best results.
- --trim-approach requires --cutoff for trim-by-quality and best-read-segment strategies.
- --headcrop and --tailcrop only work with fixed-crop trim approach; other approaches ignore these flags.
- --maxqual filters out reads with quality ABOVE the threshold; use with caution.
- Default threads is 4; increase for large datasets but watch memory usage.

## Examples

### filter ONT reads by minimum quality Q10 and minimum length 1000 bp
**Args:** `-q 10 -l 1000 --threads 8`
**Explanation:** pipe: gunzip -c reads.fastq.gz | chopper -q 10 -l 1000 --threads 8 | gzip > filtered.fastq.gz

### filter high-quality ONT reads for variant calling (Q15, min 500 bp)
**Args:** `-q 15 -l 500 --threads 8`
**Explanation:** Q15 ≈ 97% accuracy; -l 500 minimum length; pipe input from gunzip and output to gzip

### filter reads and remove low-quality ends
**Args:** `-q 10 -l 1000 --headcrop 30 --tailcrop 30 --threads 8`
**Explanation:** --headcrop 30 removes first 30 bases (often lower quality); --tailcrop 30 removes last 30 bases

### filter reads with maximum length cutoff for specific applications
**Args:** `-q 8 -l 200 --maxlength 50000 --threads 4`
**Explanation:** --maxlength removes very long reads that may be chimeric; -l 200 for short-fragment applications

### trim low-quality bases from read ends using trim-by-quality
**Args:** `--trim-approach trim-by-quality --cutoff 10 -q 10 -l 1000 --threads 8`
**Explanation:** --trim-approach trim-by-quality removes low-quality bases from ends until reaching Q10; --cutoff is required for this approach

### extract highest-quality read segment using best-read-segment
**Args:** `--trim-approach best-read-segment --cutoff 12 -q 10 -l 1000 --threads 8`
**Explanation:** best-read-segment extracts the highest-quality portion of each read; useful for reads with variable quality across length

### filter reads by GC content for AT-rich genomes
**Args:** `-q 10 -l 1000 --mingc 30 --maxgc 60 --threads 8`
**Explanation:** --mingc 30 --maxgc 60 keeps reads with 30-60% GC content; useful for filtering extreme GC outliers

### output reads that fail quality filters (inverse mode)
**Args:** `-q 10 -l 1000 --inverse --threads 8`
**Explanation:** --inverse outputs reads that FAIL the filters instead of passing; useful for examining low-quality data

### filter contaminants against a reference FASTA
**Args:** `-q 10 -l 1000 -c contaminants.fasta --threads 8`
**Explanation:** -c filters out reads matching sequences in contaminants.fasta; useful for removing host or adapter contamination

### split reads by low-quality segments
**Args:** `--trim-approach split-by-low-quality --cutoff 8 -q 10 -l 500 --threads 8`
**Explanation:** split-by-low-quality breaks reads at low-quality regions and outputs high-quality parts; useful for rescue of chimeric reads
