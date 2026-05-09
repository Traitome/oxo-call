---
name: AdapterRemovalFixPrefix
category: Quality Control
description: A bioinformatics utility for fixing and normalizing FASTQ read name prefixes after adapter removal. Handles common issues such as malformed headers, missing delimiters, and inconsistent naming schemes across paired-end reads.
tags:
  - ngs
  - preprocessing
  - fastq
  - adapter-trimming
  - quality-control
  - sequencing
author: AI-generated
source_url: https://github.com/ncbi/adapterremovalfixprefix
---

## Concepts

- **FASTQ Prefix Normalization**: The tool standardizes read name prefixes in FASTQ files by ensuring consistent delimiters (@ or >), correct sequence identifiers, and proper mate-pair notation (e.g., /1 and /2 suffixes for paired-end reads).
- **Mate-Pair Synchronization**: When processing paired-end FASTQ files, the tool maintains synchronization between R1 and R2 files by verifying that read IDs match and fixing discrepancies that arise from incomplete adapter removal or corruption.
- **Output Format Handling**: Supports both gzipped (.gz) and uncompressed FASTQ inputs, preserving quality scores and read sequences while only modifying header fields.
- **In-Place vs Redirected Output**: By default outputs to stdout for piping, but can write fixed reads directly to specified output files using --out1 and --out2 flags.

## Pitfalls

- **Mismatched Paired-Read IDs**: Failing to process both R1 and R2 files together results in de-synchronized read pairs where one file has corrected headers while the other retains original (potentially corrupted) headers, breaking downstream alignment.
- **Quality Score Decoding**: Attempting to modify files with unsupported encoding (e.g., custom binary formats) will produce invalid FASTQ files; the tool only supports standard Phred+33 and Phred+64 quality score schemes.
- **Overwriting Input Files**: Using the same file for input and output without creating a backup risks data loss if the process is interrupted; always redirect to new files first and verify before replacing originals.
- **Mixed Adapter Sequences**: Not checking if adapter sequences were fully removed before prefix fixing can result in hybrid headers containing partial adapter remnants merged with read identifiers.

## Examples

### Fix read name prefixes in a single-end FASTQ file

**Args:** --in1 reads.fastq.gz --out1 fixed_reads.fastq.gz
**Explanation:** Reads the gzipped FASTQ file containing single-end reads and outputs a new file with normalized @ prefixes and consistent read name formatting throughout.

### Synchronize paired-end FASTQ file headers

**Args:** --in1 R1.fastq.gz --in2 R2.fastq.gz --out1 R1_fixed.fastq.gz --out2 R2_fixed.fastq.gz
**Explanation:** Processes both paired-end files simultaneously, ensuring that read identifiers are properly synchronized with matching /1 and /2 suffixes between the two files.

### Repair malformed headers containing partial adapter sequences

**Args:** --in1 trimmed_r1.fastq.gz --out1 repaired_r1.fastq.gz --strip-adapter-fragments
**Explanation:** Parses each read header and removes any residual adapter sequence fragments that became concatenated to read names during incomplete adapter removal, restoring clean FASTQ format.

### Convert quality scores from Phred+64 to Phred+33 encoding

**Args:** --in1 input.fastq.gz --out1 normalized.fastq.gz --quality-offset 33
**Explanation:** Fixes quality score encoding that may have become mislabeled during processing, converting scores to standard Phred+33 format where ! represents the lowest quality.

### Process multiple files in batch with pattern matching

**Args:** --pattern "sample_*_R{1,2}.fastq.gz" --outdir ./fixed/
**Explanation:** Uses glob-style pattern matching to identify all paired-end FASTQ files matching the wildcard pattern and processes them together, writing fixed outputs to the specified directory with preserved filename structure.

### Preserve read filtering statistics during prefix fixing

**Args:** --in1 reads.fastq.gz --out1 fixed.fastq.gz --stats-json stats.json
**Explanation:** Generates a JSON report containing counts of reads processed, headers modified, errors encountered, and quality metrics summary while performing the prefix normalization.