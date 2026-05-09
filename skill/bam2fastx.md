---
name: bam2fastx
category: Bioinformatics / Sequence Conversion
description: Converts BAM or CRAM aligned reads to FASTQ or FASTA format
tags: [bam, cram, fastq, fasta, conversion, genomics, sequencing]
author: AI-generated
source_url: https://github.com/fulcrumgenomics/bam2fastx
---

## Concepts

- **Input formats**: Accepts BAM and CRAM files as input, including both single-end and paired-end read alignments. The input file must be coordinate-sorted for proper paired-end extraction.
- **Output formats**: Generates FASTQ (with quality scores) or FASTA (quality scores stripped) output files. For paired-end data, produces two separate files (R1 and R2) or interleaved paired format.
- **Quality score handling**: Preserves PHRED quality scores from the BAM/CRAM file when converting to FASTQ format, maintaining data fidelity for downstream analyses like variant calling.
- **Read filtering**: Can filter reads based on flags (e.g., exclude unmapped, secondary alignments, or PCR duplicates) using standard SAMtools-style flag filtering.
- **Output naming conventions**: By default, appends `_1` and `_2` suffixes to paired-end output filenames, or uses `.fastq`/`.fasta` extensions based on the specified format.

## Pitfalls

- **Using an unsorted BAM file for paired-end conversion**: Paired-end reads will not be properly paired in the output if the input BAM is not coordinate-sorted, leading to misaligned R1/R2 relationships in downstream analyses.
- **Confusing FASTQ and FASTA output**: Switching to FASTA output loses all quality score information, which breaks tools requiring quality data (e.g., variant callers, quality filtering pipelines).
- **Not excluding duplicate reads**: Including PCR duplicates in the conversion can inflate coverage and bias downstream quantitation or variant analysis results.
- **Specifying incorrect read orientation**: Misunderstanding whether the data is paired-end or single-end and using the wrong flags produces malformed or empty output files.
- **Ignoring read group information**: When converting samples with multiple read groups, failing to handle them separately can mix barcode information in the FASTQ output.

## Examples

### Convert a BAM file to FASTQ (single-end reads)
**Args:** `--bam input.bam --fastq output.fastq`
**Explanation:** Extracts all single-end aligned reads from the BAM file into a single FASTQ file, preserving read sequences and PHRED quality scores.

### Convert a BAM file to paired-end FASTQ files
**Args:** `--bam input.bam --fastq1 output_R1.fastq --fastq2 output_R2.fastq`
**Explanation:** Splits paired-end reads from the BAM into separate R1 and R2 FASTQ files, maintaining read pairing information correctly.

### Convert BAM to FASTA format (quality scores stripped)
**Args:** `--bam input.bam --fasta output.fasta`
**Explanation:** Extracts read sequences from the BAM file into FASTA format without quality scores, useful for sequence identity comparisons or de novo assembly.

### Convert CRAM file to FASTQ with duplicate filtering
**Args:** `--bam input.cram --fastq output.fastq --filterflag 1024`
**Explanation:** Converts CRAM to FASTQ while excluding PCR duplicate reads (flag 1024), reducing redundancy in the output for downstream analyses.

### Convert BAM to interleaved FASTQ (paired-end)
**Args:** `--bam input.bam --interleave output.fastq`
**Explanation:** Writes paired-end reads as interleaved FASTQ (R1 followed immediately by R2), which is required by certain aligners like BWA-MEM.

---

**Note:** bam2fastx is part of the Fulcrum Genomics tool suite. For the most up-to-date options and flags, consult the tool's GitHub repository documentation.