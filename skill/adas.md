---
name: adas
category: Sequence Analysis
description: Advanced DNA Analysis System - a bioinformatics tool for DNA sequence quality control, filtering, trimming, and summary statistics generation.
tags: [dna, sequence, quality-control, trimming, bioinformatics]
author: AI-generated
source_url: https://example.com/adas
---

## Concepts

- **Input formats**: adas accepts FASTA (.fa, .fasta) and FASTQ (.fq, .fastq) sequence files, optionally gzip-compressed (.gz). The tool reads sequences from stdin when no input file is specified.
- **Quality score handling**: PHRED quality scores are parsed automatically; ASCII-offset (33 for modern Illumina, 64 for legacy) detection is performed automatically unless manually specified via `--offset`.
- **Output modes**: adas can output filtered sequences (`--pass`), removed sequences (`--remove`), or both with prefixes (`--out-pass`, `--out-remove`). Statistics are always printed to stderr unless `--stats` redirects to a file.
- **Filtering criteria**: Supports minimum/maximum length (`--min-len`, `--max-len`), minimum quality threshold (`--min-qual`), N content percentage (`--max-n`), and exact sequence ID filtering via pattern files.

## Pitfalls

- **Missing quality score offset**: Not specifying `--offset` when using older FASTQ files (pre-Illumina 1.8) will cause all quality filtering to fail silently because quality scores will be misinterpreted as extremely low. Always verify the offset with `--detect-offset` first.
- **Input file permissions**: Running adas on read-only input files will fail with a generic error; ensure the input file is readable and, more importantly, that output directories are writable before running.
- **Confusing read-pairs**: When processing paired-end FASTQ files, using only one file instead of both will result in half the data being silently dropped. Always specify both files with `--R1` and `--R2` for paired data.
- **Out-of-memory with large files**: Loading entire compressed files into memory without using `--chunk-size` can exhaust RAM on whole-genome datasets. Process large files in chunks (e.g., `--chunk-size 100000`) to avoid crashes.

## Examples

### Filter sequences by minimum quality score
**Args:** `--min-qual 20 --pass trimmed.fq`
**Explanation:** Removes any base in a read with quality below 20 and outputs remaining reads to trimmed.fq, keeping high-confidence sequences for downstream analysis.

### Trim adapters from paired-end reads
**Args:** `--R1 R1.fq.gz --R2 R2.fq.gz --adapter AGATCGGAAGAGC --trim-front 5 --trim-tail 2 --out-pass trimmed`
**Explanation:** Trims 5 bases from the front and 2 from the tail of both read files using the specified adapter sequence, outputting results to trimmed_R1.fq.gz and trimmed_R2.fq.gz.

### Detect quality score offset automatically
**Args:** `--detect-offset sample.fq.gz`
**Explanation:** Scans the first 1000 reads to determine whether quality scores use ASCII offset 33 or 64, printing the detected offset to stderr for verification.

### Generate statistics without filtering
**Args:** sample.fq.gz --stats stats.txt`
**Explanation:** Reads the input FASTQ file and generates summary statistics (total reads, average length, quality distribution) without performing any filtering, outputting stats to stats.txt.

### Filter reads by maximum N content
**Args:** `--max-n 0.05 --pass clean.fq --remove N_rich.fq`
**Explanation:** Keeps reads where N content is at most 5% in clean.fq and moves reads exceeding this threshold to N_rich.fq, allowing separation of poor-quality sequences for manual inspection.

### Process large files in memory-safe chunks
**Args:** huge.fq.gz --chunk-size 50000 --pass processed.fq`
**Explanation:** Processes the input in batches of 50,000 reads at a time to prevent memory exhaustion, outputting only passing sequences to processed.fq.

### Filter by minimum and maximum read length
**Args:** `--min-len 50 --max-len 150 --pass length_filtered.fq`
**Explanation:** Removes reads shorter than 50 bases or longer than 150 bases, retaining only reads within the ideal size range for downstream applications like variant calling.

### Use exact sequence ID list for selective filtering
**Args:** `--ids id_list.txt --action keep --pass selected.fq`
**Explanation:** Keeps only sequences whose Exact Sequence IDs appear in the provided text file, useful for extracting specific targets from a larger dataset.