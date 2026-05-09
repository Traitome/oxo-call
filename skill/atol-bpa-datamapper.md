---
name: atol-bpa-datamapper
category: Bioinformatics Data Transformation
description: A bioinformatics tool for mapping and transforming sequencing data between different formats and coordinate systems. Handles base-pair alignment data, enables format conversion between SAM/BAM/VCF/BCF, and supports genomic coordinate transformations for downstream analysis pipelines.
tags:
- bioinformatics
- data-mapping
- sequence-analysis
- format-conversion
- genomics
- sam-bam
- vcf
author: AI-generated
source_url: https://github.com/example/atol-bpa-datamapper
---

## Concepts

- The tool operates on alignment data in SAM (text) or BAM (binary) format, accepting input from standard input or file arguments, and outputs mapped results to standard output or specified output files.
- It supports multiple input/output format flags including `--input-format` (sam/bam/vcf/bcf), `--output-format` (sam/bam/vcf/bcf), and `--output` for directing results to specific files rather than stdout.
- Coordinate system handling includes reference sequence naming conventions, 0-based versus 1-based indexing conversion, and proper chromosome naming preservation across format transformations.
- The data model treats paired-end reads as fragment records with proper mate pair tracking, preserving read flags through transformations and maintaining read group information when specified.

## Pitfalls

- Using mismatched input and output format flags (e.g., specifying `--input-format bam` with a `.sam` file) causes silent failures orcorrupted output that may not be detected until downstream tools attempt to parse the results.
- Forgetting to specify the reference genome when converting from formats that store sequences inline (like SAM) to binary formats (like BAM) results in missing MD tags and reduced alignment fidelity in downstream variant calling.
- Neglecting to handle read group IDs during multi-sample processing leads to sample confusion in merged outputs, making it impossible to correctly attribute variants to their source samples later in the pipeline.
- Omitting the `--preserve-header` flag when processing files without proper headers causes loss of metadata including @RG, @PG, and @CO lines that are essential for reproducible bioinformatics workflows.

## Examples

### Convert a SAM file to BAM format
**Args:** `--input-format sam --output-format bam --output aligned.bam`
**Explanation:** This converts a text-based SAM alignment file to binary BAM format, reducing file size and enabling efficient random access for downstream processing.

### Extract alignments for a specific chromosome
**Args:** `--input-format sam --region chr1:1000000-2000000 --output-format sam`
**Explanation:** This subsets the input alignment file to only include reads mapping to the specified chromosomal region, useful for targeted analysis workflows.

### Merge multiple alignment files with read group tagging
**Args:** `--input-format bam --read-group SM:Sample1 --read-group ID:lane1 --output-format bam`
**Explanation:** This combines alignment data while embedding sample and lane metadata into the read group tags, enabling proper sample tracking in downstream variant analysis.

### Convert VCF to BCF for compressed storage
**Args:** `--input-format vcf --output-format bcf --output variants.bcf`
**Explanation:** This converts text-based variant calling format to binary compressed BCF format, significantly reducing storage requirements for large-scale variant datasets.

### Transform alignment coordinates from 1-based to 0-based indexing
**Args:** `--input-format sam --coordinate-system 0-based --output-format sam`
**Explanation:** This converts coordinate representations in alignment files, which is required when passing data to tools expecting different indexing conventions like BED files.