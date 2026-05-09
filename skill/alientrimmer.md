---
name: alientrimmer
category: Sequence Handling
description: A bioinformatics tool for trimming adapter sequences, low-quality bases, and ambiguous nucleotides from high-throughput sequencing reads. Supports FASTA and FASTQ formats with paired-end and single-end data.
tags: [trimming, sequence-processing, quality-filtering, adapter-removal, fastq]
author: AI-generated
source_url: https://github.com/alientrimmer/alientrimmer
---

## Concepts

- **Input formats**: alientrimmer accepts single-end and paired-end reads in FASTQ (.fq/.fastq) and FASTA (.fa/.fasta) formats. For paired-end data, use two input files specified with separate flags.
- **Quality-based trimming**: The tool implements a sliding-window quality trimming algorithm that removes bases falling below a specified Phred score threshold from the 3' end of reads, improving downstream analysis accuracy.
- **Adapter sequence removal**: Built-in adapter libraries (Illumina, Nextera, TruSeq) or custom adapter sequences can be specified for complete removal using the `--adapters` or `--custom-adapters` flags.
- **Output modes**: Reads can be saved in either FASTQ or FASTA format using `--output-format`. Paired-end trimming produces four output files: forward reads, reverse reads, and their respective unpaired survivors.

## Pitfalls

- **Mismatched paired-end files**: Using different numbers of forward and reverse input files for paired-end mode causes the tool to fail with a file count mismatch error, producing no output.
- **Aggressive trimming thresholds**: Setting `--quality-threshold` too high (e.g., above 30) can eliminate entire reads from low-coverage datasets, reducing effective sequencing depth and potentially introducing bias.
- **Forgetting to specify output directory**: By default, alientrimmer overwrites input files in place if `--output-dir` is not specified, permanently destroying original raw data.
- **Incompatible output format for paired-end data**: Specifying FASTA output format for paired-end input when reads are orphaned in both directions produces incomplete output files missing survival pairs.

## Examples

### Trim adapters from single-end FASTQ reads using default Illumina adapters
**Args:** `--input reads.fq.gz --adapters illumina --quality-threshold 20 --output-dir trimmed/`
**Explanation:** This removes Illumina adapter sequences and low-quality trailing bases (