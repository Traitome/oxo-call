---
name: borf
category: sequence_analysis
description: A command-line tool for processing and filtering biological sequence data with support for FASTQ, FASTA, and SAM/BAM formats. Designed for rapid quality control, trimming, and conversion of next-generation sequencing reads.
tags: [sequence-processing, quality-control, fastq, fasta, ngs, read-filtering]
author: AI-generated
source_url: https://github.com/borf/borf
---

## Concepts

- **Input formats:** borf accepts FASTQ, FASTA, and SAM/BAM files as input. Use `-i/--input` to specify the input file and `-f/--format` to explicitly declare the format (fastq, fasta, sam, bam). When format is ambiguous, borf automatically detects based on file extension.
- **Output handling:** By default, borf writes to stdout. Use `-o/--output` to redirect to a file. The output format is inferred from the extension unless overridden with `--out-format`. Multiple output files can be generated simultaneously using the `--split` flag.
- **Quality filtering:** The tool implements sliding-window quality trimming via `--min-qual` (minimum quality threshold) and `--window-size`. Reads below the quality threshold are discarded unless `--discard-low` is set to false. Use `--min-length` to filter reads shorter than a specified length.
- **Threading:** borf supports multi-threading via `-t/--threads` for parallel processing of large files. The default is 1 thread. Using more threads significantly reduces processing time for large datasets but increases memory usage proportionally.

## Pitfalls

- **Memory exhaustion with large files:** Processing uncompressed BAM/SAM files without specifying `--chunk-size` can consume excessive memory. For a 100GB BAM file, ensure at least 8GB available RAM or use streaming mode with `--stream` to avoid crashes.
- **Format mismatches cause silent failures:** If the declared input format (`-f fastq`) does not match the actual file format, borf may produce empty output without warning. Always verify the format matches the file content, not just the extension.
- **Overwriting outputs without confirmation:** The `--overwrite` flag will silently overwrite existing output files. Users accidentally running the same command twice will lose the original data. Use `--check-exists` to prompt before overwriting.
- **Inconsistent quality score encodings:** borf assumes Phred+33 quality scoring (Sanger standard). If input files use Phred+64 (Illumina 1.3-1.7) or raw integer scores, results will be incorrect. Specify `--quality-offset` explicitly when working with older Illumina data.

## Examples

### Filter FASTQ reads by minimum quality threshold
**Args:** `-i reads.fastq -f fastq --min-qual 20 --min-length 50`
**Explanation:** Reads with average quality below 20 or final length below 50 bases are discarded from the output, preserving only high-quality sequences.

### Convert FASTQ to FASTA format
**Args:** `-i input.fastq -f fastq --to-fasta -o output.fasta`
**Explanation:** Converts the read sequences from FASTQ (with quality scores) to FASTA (sequence-only) format for compatibility with downstream tools that require FASTA input.

### Trim adapters using a known sequence
**Args:** `-i reads.fastq --trim-adapter AGATCGGAAGAGC --trim-end left --min-length 30`
**Explanation:** Removes the adapter sequence AGATCGGAAGAGC from the left end of reads, then discards any reads shorter than 30 bases after trimming.

### Process a BAM file and output compressed FASTQ
**Args:** `-i alignments.bam -f bam --extract-qual -o filtered.fastq.gz`
**Explanation:** Extracts read sequences and quality scores from a BAM file, outputting them as a gzipped FASTQ file for downstream analysis or re-alignment.

### Parallel processing with 8 threads
**Args:** `-i large_dataset.fastq -t 8 --min-qual 25 -o highqual.fastq`
**Explanation:** Uses 8 parallel threads to filter a large FASTQ file, keeping only reads with minimum quality 25, significantly speeding up processing on multi-core systems.

### Stream process when memory is limited
**Args:** `-i huge.bam -f bam --stream --min-length 40 -o filtered.sam`
**Explanation:** Enables streaming mode to process the BAM file in chunks rather than loading it entirely into memory, preventing out-of-memory errors on systems with limited RAM.

### Split output into multiple files by read identifier prefix
**Args:** `-i mixed.fastq --split-by-prefix --output-dir ./split_out`
**Explanation:** Automatically splits the input into multiple output files based on read name prefixes (e.g., @SIMPLEX_, @COMPLEX_), creating separate files for different read groups.