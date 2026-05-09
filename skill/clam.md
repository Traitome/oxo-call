---
name: clam
category: Bioinformatics - Reference-guided assembly
description: CLAM (Compact Linear Assembler) is a reference-guided viral genome assembler that builds consensus sequences from short reads using coverage information. It takes a reference genome and aligned read data (SAM/BAM or FASTQ), performing coverage-based consensus calling to produce assembled contigs.
tags:
  - assembly
  - viral-genomes
  - reference-guided
  - consensus-calling
  - nextflow
author: AI-generated
source_url: https://github.com/malEWU/clam
---

## Concepts

- CLAM performs reference-guided assembly by aligning reads to a reference genome and calling consensus based on coverage thresholds. Input requires a reference FASTA file and either SAM/BAM alignment files or unaligned FASTQ reads with a reference.
- The tool uses a minimum coverage threshold (default: 3) to determine valid base calls; positions with coverage below this threshold are masked as 'N' or skipped in output.
- CLAM supports both SAM/BAM alignment inputs (pre-aligned reads) and FASTQ inputs (with --ref-is-sam option for inline alignment), giving flexibility for different workflow stages.
- Output is written as assembled contigs in FASTA format, preserving the reference sequence identifier with a suffix indicating the sample or iteration.

## Pitfalls

- Using mismatched read and reference formats (e.g., passing FASTQ reads without specifying --ref-is-sam when the reference contains inline SAM alignments) produces empty or incorrect contigs without clear error messages.
- Setting a coverage threshold too high for low-depth data causes the entire assembly to fail or produce fragmented contigs with excessive N characters, losing meaningful sequence.
- Specifying an incorrect or contaminated reference genome leads to assemblies that reflect the wrong organism, with no built-in validation of reference appropriateness.
- CLAM expects coordinate-sorted SAM/BAM files; using name-sorted alignments results in incorrect coverage calculation and malformed output.

## Examples

### Assemble reads aligned to a reference using SAM input
**Args:** --ref reference.fa --sam aligned.sam --out assembly.fa
**Explanation:** This runs CLAM with a reference FASTA and pre-aligned SAM file, outputting consensus contigs to the specified FASTA file.

### Assemble unaligned FASTQ reads against a reference
**Args:** --ref reference.fa --fastq reads.fq --out assembly.fa
**Explanation:** This performs inline alignment of FASTQ reads to the reference before consensus calling, useful when reads are not pre-aligned.

### Adjust minimum coverage threshold for low-depth data
**Args:** --ref reference.fa --sam aligned.sam --out assembly.fa --min-cov 2
**Explanation:** Lowering the coverage threshold to 2 allows consensus calling at positions with 2x coverage, useful for low-depth viral samples.

### Specify a sample name for output contig headers
**Args:** --ref reference.fa --sam sample.sam --out sample_assembly.fa --sample-name SampleA
**Explanation:** Adding a sample name modifies the output contig header to include the sample identifier for downstream traceability.

### Use multiple threads to accelerate alignment processing
**Args:** --ref reference.fa --fastq reads.fq --out assembly.fa --threads 4
**Explanation:** Enabling multi-threaded processing with 4 threads speeds up the inline alignment step for large FASTQ inputs.