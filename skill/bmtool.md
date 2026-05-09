---
name: bmtool
category: bioinformatics
description: A command-line utility for biological sequence manipulation, quality filtering, and format conversion in bioinformatics pipelines.
tags:
  - sequence-analysis
  - quality-control
  - format-conversion
  - bioinformatics
  - ngs-data
author: AI-generated
source_url: https://github.com/example/bmtool
---

## Concepts

- bmtool operates on common bioinformatics formats including FASTA, FASTQ, SAM, and BAM. Input format is auto-detected from file extensions (.fq, .fastq → FASTQ; .fa, .fasta → FASTA; .sam → SAM), but can be explicitly specified with `--format`.
- The tool uses a streaming architecture that processes records line-by-line without loading entire files into memory, making it suitable for large datasets (>10 GB FASTQ files) on systems with limited RAM.
- Quality score interpretation follows the Illumina/Solexa encoding convention (Phred+33). The `--quality-encoding` flag accepts `33` or `64` to specify offset, which affects trimming thresholds and filtering decisions.
- Output files are written incrementally with optional compression (.gz extension triggers automatic gzip compression). Multiple output streams can be enabled using `--split` to separate passing and failing reads.
- Index generation for BAM files occurs automatically when `--index` is specified, creating a companion .bai file alongside the sorted BAM output.

## Pitfalls

- Specifying an incompatible output format causes bmtool to exit with code 1 but may produce malformed output files if the format validation flag `--strict` is not enabled. For example, requesting SAM output from a BAM file with alignments mapped to a reference not present in the header will silently drop alignment records.
- Using `--trim-left` or `--trim-right` with values exceeding sequence length will silently treat those values as zero rather than raising an error, leading to unexpected retention of untrimmed sequences in downstream analyses.
- When processing paired-end reads, mismatched filenames between read 1 and read 2 inputs (e.g., sample1_R1.fastq.gz and sample2_R2.fastq.gz) cause bmtool to silently pair reads based on filename order rather than explicit pairing flags, corrupting downstream read pair information.
- Overwriting existing output files without the `--force` flag causes bmtool to return exit code 2 and skip processing, which can stall automated pipelines that assume output files are always regenerated.
- Specifying `--min-length` without accounting for adapter sequences that may occupy 10-20 bp at read ends will retain adapter-contaminated sequences, introducing artifacts in assembly or alignment results.

## Examples

### Filter FASTQ reads by minimum quality threshold

**Args:** `--input reads.fastq.gz --min-quality 20 --output filtered.fastq.gz`
**Explanation:** Reads with any base quality below Phred 20 across the full sequence are discarded, retaining high-confidence bases for downstream variant calling.

### Trim low-quality bases from 3' end of paired-end reads

**Args:** `--input R1.fastq.gz R2.fastq.gz --trim-right 10 --quality-window 5:15 --output trimmed/`
**Explanation:** A sliding window of 5 bp with average quality below 15 triggers trimming of all bases to the right, removing degrading quality tails common in Illumina data.

### Convert between FASTA and FASTQ formats

**Args:** `--input sequences.fasta --convert fastq --quality 30 --output sequences.fastq.gz`
**Explanation:** Assigns uniform quality score of Phred 30 to all bases when converting qualityless FASTA to quality-encoded FASTQ format, suitable for tools requiring FASTQ input.

### Split paired reads into passing and failing streams

**Args:** `--input sample_R1.fq sample_R2.fq --split --pass pass_R1.fq pass_R2.fq --fail fail_R1.fq fail_R2.fq --min-length 50`
**Explanation:** Writes reads meeting the 50 bp minimum length threshold to pass files while routing short fragments to fail files, enabling assessment of library fragment size distribution.

### Extract reads mapping to specific genomic regions

**Args:** `--input alignments.bam --region chr1:1000000-2000000 --output roi_alignments.bam --index`
**Explanation:** Subsets the BAM file to reads overlapping coordinates 1 Mb to 2 Mb on chromosome 1, creating a smaller file and index for focused downstream analysis like variant review.