---
name: appspam
category: Sequence Filtering / Quality Control
description: A bioinformatics tool for filtering low-complexity and repetitive sequences from FASTA/FASTQ files, often used as a preprocessing step before sequence alignment or assembly.
tags: [sequence-filtering, low-complexity, preprocessing, quality-control, fasta, fastq]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/appspam
---

## Concepts

- **Input/Output Formats**: appspam accepts FASTA and FASTQ input (single or paired-end) and produces filtered output in the same format, with optional companion index files for downstream tools.
- **Low-Complexity Filtering**: The tool identifies and masks or removes sequences with low compositional complexity using entropy-based or dust-like algorithms, configurable via the `--method` flag.
- **Companion Binaries**: appspam typically ships with companion tools like `appspam-build` for building custom repeat databases and `appspam-merge` for combining filtered outputs.
- **Streaming Support**: appspam supports streaming input via stdin, making it pipeline-friendly when combined with tools like bowtie2 or bwa in Unix pipes.

## Pitfalls

- **Forgetting to Preserve Reverse Complements**: When filtering paired-end reads, failing to use `--filter-both` retains reads whose forward mate passes but reverse complement fails, causing downstream alignment errors.
- **Threshold Too Aggressive**: Setting `--entropy-threshold` too high (e.g., >0.7) removes functionally important low-complexity regions like repeat sequences or simple sequence repeats, reducing alignment sensitivity.
- **Mismatched Format Modes**: Using text output flags (`--out-fmt fasta`) with binary index generation (`--write-index`) creates incompatible index files that downstream tools cannot read.
- **Ignoring Quality Score Recalibration**: appspam modifies sequences but does not automatically recalculate per-base quality scores, requiring manual adjustment if quality-dependent tools are used downstream.

## Examples

### Filter low-complexity sequences from a single FASTA file

**Args:** `--input reads.fasta --output filtered.fasta --method dust --threshold 0.2`
**Explanation:** Uses dust-like algorithm with entropy threshold 0.2 to filter low-complexity sequences, outputting only high-complexity reads.

### Filter paired-end FASTQ reads and preserve both mates

**Args:** `--input R1.fq --input R2.fq --out-r1 R1_filtered.fq --out-r2 R2_filtered.fq --filter-both --method entropy --threshold 0.3`
**Explanation:** Removes paired-end reads only when both R1 and R2 fail the entropy filter, ensuring consistent read pairs for alignment.

### Build a custom repeat database for filtering

**Args:** `--database custom_repeats.fasta --output custom_db.asdb --build-index`
**Explanation:** Compiles a custom repeat database from a FASTA file into an index for use in subsequent filtering runs.

### Stream filtered output directly to an aligner

**Args:** `--input - --output - --method dust --threshold 0.25 | bwa mem - ref.fa -`
**Explanation:** Reads from stdin and writes to stdout, enabling direct piping of filtered reads to bwa without intermediate files.

### Generate masked sequences with N replacements

**Args:** `--input genes.fasta --output genes_masked.fasta --mask-char N --method dust --threshold 0.15`
**Explanation:** Replaces low-complexity regions with N characters instead of removing sequences, preserving sequence length for coordinate-based downstream analysis.