---
name: bwise
category: Quality Filtering
description: A bioinformatics tool for FASTQ quality score filtering, analysis, and statistics. Operates on Phred+33 or Phred+64 encoded quality strings to filter reads, generate quality reports, and convert between quality score formats.
tags:
  - fastq
  - quality-scores
  - sequencing
  - illumina
  - filtering
  - bioinformatics
author: AI-generated
source_url: https://github.com/agordon/fastx_toolkit
---

## Concepts

- **Quality Score Encoding**: bwise handles both Phred+33 (Illumina 1.8+) and Phred+64 (Illumina 1.3-1.7) encoding schemes. The tool automatically detects the encoding based on the ASCII range of quality characters in the input FASTQ file.
- **Input/Output Formats**: Accepts standard FASTQ files (`.fq` or `.fastq` extensions) and supports output to stdout for piping into other tools. Works with both single-end and paired-end read data.
- **Filtering Logic**: Reads are retained or discarded based on minimum quality threshold comparisons at each base position or across the entire read length. The tool evaluates quality scores as integer values (Phred scores).
- **Quality Statistics**: Generates summary statistics including mean, minimum, maximum, and per-position quality distributions for the filtered or unfiltered dataset.

## Pitfalls

- **Mismatched Quality Encoding**: Specifying the wrong quality score encoding (--phred64 vs --phred33) produces corrupt output or silent data loss because characters are interpreted using the wrong ASCII offset.
- **Threshold Too Strict**: Setting an excessively high minimum quality threshold (e.g., -q 30 for data with typicalQuality ~25) filters out nearly all reads, resulting in insufficient data for downstream analysis.
- **Missing Input File Handling**: Forgetting to provide an input file or using incorrect file redirection causes bwise to read from stdin indefinitely, hanging the terminal without error messages in some implementations.

## Examples

### Filter reads with minimum quality below 20
**Args:** `-q 20 input.fq`
**Explanation:** Retains only reads where every base has a Phred quality score of 20 or higher, removing low-confidence calls from the dataset.

### Use phred64 encoding for older Illumina data
**Args:** `--phred64 -q 15 old_data.fastq`
**Explanation:** Processes quality scores as Phred+64 encoded (ASCII 64-126 range) and applies a minimum quality threshold of 15, which was standard for older Illumina platforms.

### Output only high-quality reads to a new file
**Args:** `-q 25 -i reads.fq -o clean_reads.fq`
**Explanation:** Filters the input FASTQ file to keep only reads with all bases scoring Phred 25 or higher, saving the result to a new file for downstream analysis.

### Print quality statistics without filtering
**Args:** `-z -i input.fq`
**Explanation:** Reports statistics about the input file's quality score distribution (minimum, maximum, mean per position) without performing any filtering.

### Trim reads using a sliding window quality cutoff
**Args:** `-t 20 -w 4 input.fq`
**Explanation:** Trims reads from the 3' end using a sliding window of size 4 with minimum quality threshold 20, removing low-quality trailing bases while preserving higher-quality upstream sequence.