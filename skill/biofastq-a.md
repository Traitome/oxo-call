---
name: biofastq-a
category: sequence_analysis
description: A FASTQ file manipulation tool for quality filtering, length filtering, statistics, conversion, and sequence extraction operations on FASTQ formatted sequence data.
tags:
  - fastq
  - quality-control
  - sequencing
  - ngs
  - bioinformatics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/biofastq-a
---

## Concepts

- biofastq-a operates on FASTQ files (`.fq` or `.fastq` extensions), which store sequencing reads as four lines per record: header, sequence, separator (`+`), and quality scores encoded in ASCII.
- The tool supports both single-end and paired-end FASTQ inputs; for paired files, operations apply to both files unless explicitly filtered using file index flags.
- Quality scores are accepted in either Phred+33 (Sanger) or Phred+64 (Illumina 1.3-1.7) encoding; the tool auto-detects encoding but can be forced via the `--encoding` flag.
- Output writing is streaming by default; use `--outfile` with a specified path to write to a file, otherwise output goes to stdout.
- The tool indexes no files by default; operations requiring random access (like sequence retrieval by index) need `--build-index` pre-processing using the companion binary `biofastq-a-build`.

## Pitfalls

- Specifying the wrong quality encoding (Phred+33 vs Phred+64) produces incorrect filtering results because score thresholds are evaluated against wrong numeric ranges; consequence: sequences that should pass may be discarded or vice versa.
- Forgetting to specify `--pair-by-name` when processing paired-end FASTQ files causes the tool to treat read pairs as independent single reads, leading to misalignment between R1 and R2 files after filtering.
- Using `--min-len` or `--max-len` without considering that lengths are counted in nucleotides, not base-pairs; for paired-end data the combined length is not summed automatically.
- Not providing the `--output` flag when piping to another tool causes filtered reads to print to stdout which may be captured incorrectly by subsequent processes expecting structured output.
- Attempting to retrieve sequences by index on non-indexed FASTQ files fails with an error; must run `biofastq-a-build` first to create the required index.

## Examples

### Filter reads by minimum quality score

**Args:** `--min-q 30 input.fq --out output.fq`
**Explanation:** Keeps only reads where every base has a Phred quality score of 30 or higher, suitable for high-stringency downstream analysis.

### Filter reads by minimum sequence length

**Args:** `--min-len 50 input.fq`
**Explanation:** Retains only reads with at least 50 nucleotides, useful for removing trimmed or too-short sequences before alignment.

### Convert FASTQ to FASTA format

**Args:** `--to-fasta input.fq`
**Explanation:** Converts the input FASTQ to FASTA format by dropping the quality line and replacing the separator with a `>` header, outputting to stdout.

### Calculate per-base quality statistics

**Args:** `--stats input.fq`
**Explanation:** Outputs a table showing mean, min, max, and standard deviation of quality scores at each position across all reads.

### Extract specific reads by index

**Args:** `--index 100 --index 250 --index 500 input.fq --build-index`
**Explanation:** Retrieves the 100th, 250th, and 500th reads from the file after building an index for random access; the `--build-index` flag ensures an index exists for efficient lookup.

### Trim reads from the 5 prime end

**Args:** `--trim-5p 10 input.fq --out trimmed.fq`
**Explanation:** Removes the first 10 nucleotides from the 5 prime end of each read, commonly used to remove adapter contamination or low-quality bases at the start.

### Filter paired-end files maintaining read pairing

**Args:** `R1.fq R2.fq --min-q 25 --pair-by-name --out R1_filtered.fq --out2 R2_filtered.fq`
**Explanation:** Filters both paired-end files simultaneously, keeping only read pairs where both R1 and R2 reads meet the minimum quality threshold, maintaining proper pairing.