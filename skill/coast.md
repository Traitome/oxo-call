---
name: coast
category: Sequencing Data Analysis
description: Coast is a bioinformatics command-line tool for processing, quality control, and compression of sequencing data files. It provides utilities for converting between common sequencing formats, filtering reads by quality thresholds, and generating summary statistics for FASTQ or BAM inputs.
tags:
  - sequencing
  - QC
  - fastq
  - bam
  - compression
  - bioinformatics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/coast
---

## Concepts

- **Input/Output formats**: Coast reads from FASTQ, gzip-compressed FASTQ (.fastq.gz), and BAM files. Output formats are inferred from file extensions unless explicitly overridden with `--out-format`. Specifying the correct format is critical for downstream compatibility.
- **Quality filtering**: Reads can be trimmed and filtered using per-base quality thresholds. Coast uses a sliding window approach by default, making it more robust than simple末端 trimming for removing low-quality tails.
- **Read data model**: Each read in Coast's internal model carries the following fields: read ID, sequence, quality string, optional CIGAR string (BAM mode), and an assigned filter flag. Reads marked with a non-zero filter flag are excluded from output unless `--keep-filtered` is specified.
- **Compression behavior**: When input is FASTQ.gz, Coast preserves the gzip wrapper by default and does not decompress fully into memory. For very large files, streaming mode (`--stream`) is recommended to avoid memory exhaustion.

## Pitfalls

- **Mismatched input and output format flags**: Specifying `--out-format bam` on a FASTQ input will fail at the CIGAR construction step because FASTQ reads lack alignment information. Attempting this will raise a "CIGAR missing for BAM output" error and terminate without producing output.
- **Overlapping quality thresholds for paired-end data**: Setting `--min-quality 30` on both reads individually does not guarantee that both reads in a pair pass the filter when a `--pair-min-mean-quality` is not also set. This leads to asymmetric filtering and unbalanced output files, which downstream tools like BWA-MEM will reject.
- **Default memory limits on large files**: Coast's default in-memory buffer is sized for files up to approximately 4 GB. Processing a 10 GB gzip FASTQ without `--stream` will cause an out-of-memory error. Always use `--stream` for files exceeding 4 GB.
- **Forgetting `--reverse-complement` for BAM input**: BAM files store reads in the forward orientation relative to the reference. If your downstream pipeline expects the original read orientation (e.g., for variant calling from RNA-seq data), omitting `--reverse-complement` will flip the sequence and quality strings incorrectly, corrupting downstream results.

## Examples

### Compute quality summary statistics for a gzip-compressed FASTQ file
**Args:** `stats --input sample.fastq.gz`
**Explanation:** The `stats` subcommand reads the gzip-compressed FASTQ file and outputs per-base quality distributions, read length histograms, and overall pass/fail QC flags to stdout without modifying the input.

### Trim low-quality bases from a FASTQ file using a sliding window approach
**Args:** `trim --input raw_reads.fastq.gz --output clean_reads.fastq.gz --window-size 4 --min-quality 20`
**Explanation:** The `--window-size 4` argument applies a 4-base sliding window and drops any window whose mean quality falls below Phred score 20, writing the trimmed reads to the specified output file.

### Filter paired-end reads where both reads must pass a mean quality threshold
**Args:** `filter-pairs --read1 R1.fastq.gz --read2 R2.fastq.gz --out-r1 filtered_R1.fastq.gz --out-r2 filtered_R2.fastq.gz --pair-min-mean-quality 25`
**Explanation:** Setting `--pair-min-mean-quality 25` ensures that both reads in a pair must individually exceed a mean Phred score of 25, producing matched output files suitable for paired-end alignment pipelines.

### Convert a FASTQ file to BAM format for use with genome viewers
**Args:** `convert --input reads.fastq.gz --out-format bam --reference ref.fasta --output reads.bam`
**Explanation:** The `--reference` flag is required when converting FASTQ to BAM because Coast builds a temporary reference index to encode MD tags, and omitting it will cause the conversion to fail.

### Stream-process a very large FASTQ file to avoid memory exhaustion
**Args:** `stats --input huge_dataset.fastq.gz --stream --output summary.tsv`
**Explanation:** The `--stream` flag forces sequential processing, writing summary statistics to the specified file incrementally and keeping memory usage constant regardless of input file size.

### Extract reads matching a specific barcode tag from a BAM file
**Args:** `filter-bam --input aligned.bam --output barcode_subset.bam --tag BC --tag-value AGCTTGCA`
**Explanation:** The `--tag BC` and `--tag-value` arguments filter the BAM to only those reads with the exact barcode sequence AGCTTGCA in the BC auxiliary tag, which is useful for demultiplexing single-cell or multiplexed sequencing runs.