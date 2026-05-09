---
name: biopet-seattleseqkit
category: sequence_analysis
description: A bioinformatics toolkit for sequence manipulation, quality control, and filtering operations on genomic reads. Provides utilities for trimming, filtering, and analyzing FASTQ/FASTA sequence data.
tags:
  - sequence-processing
  - fastq
  - fasta
  - quality-control
  - trimming
  - bioinformatics
author: AI-generated
source_url: https://biopet.readthedocs.io
---

## Concepts

- **Input Formats**: The toolkit processes standard sequencing formats including FASTQ (single-end and paired-end), FASTA, and gzipped versions (.fastq.gz, .fasta.gz). File compression is automatically detected based on file extension.
- **Output Handling**: By default, processed reads are written to stdout in the same format as input unless explicitly redirected. Multiple output streams can be generated using the --output-prefix flag for batched results.
- **Quality Scoring**: All quality score operations use Phred+33 encoding by default (Sanger format). Use --illumina-quals for older Illumina 1.3-1.7 quality scores (Phred+64). The tool automatically detects format from input file headers when possible.
- **Paired-End Processing**: For paired-end data, mate pairs are tracked using read IDs. The --pe-mode flag controls how mismatched pairs are handled: 'strict' discards both reads if either fails filters, 'loose' keeps the passing read.

## Pitfalls

- **Mismatched read IDs in paired-end mode**: If read IDs between forward and reverse files don't match exactly (including any trailing comments), the mate pairing will be lost, leading to orphans in the output. Always validate read ID consistency before processing with --validate-pe.
- **Quality score encoding mismatches**: Using --illumina-quals with Sanger-encoded input (or vice versa) produces corrupted quality scores and downstream analysis failures. Verify encoding using --detect-quals on a subset before full processing.
- **Output file overwrites without confirmation**: The toolkit silently overwrites existing output files. For batch processing, use --output-prefix which appends a sample identifier rather than a specific filename to prevent accidental data loss.
- **Insufficient disk space for large files**: The toolkit may create temporary files during processing that require additional space equal to the input file size. Monitor available disk space with --check-space before running on whole-genome datasets.

## Examples

### Trim adaptors from FASTQ reads using automatic detection
**Args:** --input sample.fastq.gz --trim-adaptors --auto-detect
**Explanation:** Enables automatic adaptor sequence detection from the input file, trimming identified adaptors without requiring explicit adaptor sequences to be specified.

### Convert FASTQ to FASTA format
**Args:** --input reads.fastq --output-format fasta
**Explanation:** Converts a FASTQ file to FASTA format, stripping quality scores and generating standard FASTA headers with sequence data only.

### Filter reads by minimum quality threshold
**Args:** --input sample.fastq.gz --min-quality 30 --min-length 50
**Explanation:** Retains only reads with average Phred quality score of 30 or higher and a minimum length of 50 nucleotides, discarding low-quality and short sequences.

### Process paired-end reads in strict mode
**Args:** --pe input_R1.fastq.gz input_R2.fastq.gz --pe-mode strict --min-quality 25
**Explanation:** Processes paired-end files requiring both reads to pass the quality threshold; if either read fails, both are discarded to maintain synchronization.

### Generate quality control statistics without modifying input
**Args:** --input sample.fastq.gz --stats-only --output-prefix qc_report
**Explanation:** Runs quality analysis on the input file and generates a summary report without producing filtered output, useful for assessing data quality before processing.

### Trim leading/trailing N nucleotides and low-quality bases
**Args:** --input reads.fastq.gz --trim-left 5 --trim-right 10 --quality-cutoff 20
**Explanation:** Removes 5 nucleotides from the start and 10 from the end of each read, then performs quality-based trimming where any base below Phred 20 is removed from the trailing end of reads.

### Extract a subset of reads by name list
**Args:** --input sample.fastq.gz --keep-names gene_list.txt --output-prefix subset
**Explanation:** Filters the input to retain only reads with IDs present in the provided gene list file, creating output files with the subset prefix for downstream analysis.