---
name: binette
category: Bioinformatics Tools / Quality Control
description: A bioinformatics tool for read quality filtering, trimming, and statistics generation. Supports FASTQ and BAM input formats with configurable quality thresholds and adapter detection.
tags: [quality-control, trimming, filtering, fastq, bam, ngs, sequencing]
author: AI-generated
source_url: https://github.com/example/binette
---

## Concepts

- **Input Formats**: binette accepts FASTQ (single-end and paired-end) and BAM files as input. For paired-end data, use two input files (R1 and R2) or an interleaved FASTQ file.
- **Quality Thresholds**: The tool uses Phred quality scores for filtering. Reads or bases below the specified threshold are either discarded or masked. Default minimum quality is 20 (Q-score 20, corresponding to 99% accuracy).
- **Trimming Modes**: binette supports multiple trimming strategies: quality-based trimming (from ends), adapter trimming, and length-based filtering. Trimming can be applied to both 5' and 3' ends independently.
- **Output Formats**: After processing, binette outputs filtered FASTQ files (or BAM if input was BAM). It also generates a JSON/HTML report containing per-base quality statistics, read length distributions, and filtering summaries.

## Pitfalls

- **Mismatched Paired-End Files**: When processing paired-end data, ensure both files have identical read counts and identical read identifiers. Mismatched files cause binette to fail or produce truncated output files.
- **Invalid Quality Score Encoding**: binette expects Phred+33 (Sanger) quality encoding by default. Using Phred+64 (Illumina 1.5-1.7) or other encodings without specifying `--quality-encoding` produces incorrect filtering results.
- **Insufficient Disk Space**: binette writes output files to the same directory as input unless `--output-dir` is specified. Ensure adequate disk space (approximately 2x the input file size) is available to avoid incomplete output.
- **Conflicting Filter Parameters**: Combining `--min-length` and aggressive quality trimming may remove all reads from sparse datasets. Always inspect summary statistics after the first run.

## Examples

### Filter FASTQ reads by minimum quality score
**Args:** `--input reads.fastq --min-quality 25 --output-dir ./filtered`
**Explanation:** Removes any read with positions having Phred quality scores below 25, writing cleaned reads to the specified output directory.

### Trim adapter sequences from 3' ends
**Args:** `--input reads.fastq --adapter AATGATACGGCGACCACCGAGATCTACAC --trim-mode both --output-dir ./trimmed`
**Explanation:** Detects and removes the specified Illumina adapter sequence from both ends of each read before writing output.

### Process paired-end FASTQ files
**Args:** `--input-f1 reads_R1.fastq --input-f2 reads_R2.fastq --minimum-length 50 --min-quality 20 --paired-only --output-dir ./cleaned`
**Explanation:** Processes both read files together, discarding read pairs where either read is shorter than 50bp or has quality below Q20.

### Generate quality control report without filtering
**Args:** `--input reads.fastq --report-only --output-dir ./qc_report`
**Explanation:** Analyzes the input file and produces detailed quality statistics without modifying or writing filtered reads.

### Specify Illumina 1.8 quality encoding
**Args:** `--input reads.fastq --quality-encoding sanger --min-quality 25 --output-dir ./filtered`
**Explanation:** Explicitly treats input quality scores as Sanger (Phred+33) encoding, which is required for modern Illumina datasets.