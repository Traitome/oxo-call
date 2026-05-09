---
name: cats-rb
category: bioinformatics/sequence-utilities
description: A Ruby tool for concatenating FASTQ, FASTA, and related sequence files with proper handling of fileheaders, quality scores, and read identifiers during merge operations.
tags: bioinformatics, sequence-files, fastq, fasta, concatenation, ruby, read-processing
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cats-rb
---

## Concepts

- **File format support**: cats-rb handles FASTQ, FASTA, and interleaved sequence formats as input and output, preserving quality score lines (ASCII-encoded per base) and read identifiers throughout concatenation.
- **Read identifier management**: When concatenating multiple FASTQ files, cats-rb can optionally rename or renumber read identifiers to maintain uniqueness across the merged dataset, preventing duplicate ID errors in downstream processing pipelines.
- **Header and separator handling**: The tool includes options to preserve or strip file headers (like @sq lines in SAM-compatible formats) and can insert custom separators between concatenated files, which is useful for creating merged datasets from multiple sequencing runs.
- **Paired-end and interleaved processing**: cats-rb supports concatenation of paired-end files by synchronizing read order between forward (.R1) and reverse (.R2) files, and can convert between paired and interleaved FASTQ representations.

## Pitfalls

- **Mismatched read order between pairs**: If input FASTQ files have reads in different orders, concatenating them without synchronization will produce misaligned forward/reverse pairs, causing alignment failures or incorrect variant calls in downstream analysis.
- **Quality score offset inconsistencies**: Mixing files with different quality score encodings (Phred+33 vs Phred+64) without conversion will corrupt quality data, leading to unreliable variant calling or filtering errors in tools like GATK or FreeBayes.
- **Duplicate read identifiers after concatenation**: Failing to renumber reads when concatenating multiple files that share read IDs will cause intermediate pipeline tools to crash or produce incorrect results, as many aligners require unique read names.
- **File format mismatch**: Attempting to concatenate FASTQ with FASTA files without format conversion will produce malformed output since FASTQ includes quality score lines that have no equivalent in FASTA format.

## Examples

### Concatenate multiple FASTQ files into one
**Args:** `-o merged.fastq input1.fastq input2.fastq input3.fastq`
**Explanation:** The `-o` flag specifies the output filename and accepts multiple input files in order, merging them sequentially into a single FASTQ file.

### Merge FASTQ files with automatic read ID renumbering
**Args:** `--renumber -o merged.fastq run1.fastq run2.fastq`
**Explanation:** The `--renumber` option automatically assigns new sequential identifiers to prevent duplicate read names that would confuse downstream alignment tools.

### Convert paired-end FASTQ to interleaved format
**Args:** `--interleave -o interleaved.fastq reads_R1.fastq reads_R2.fastq`
**Explanation:** This merges corresponding forward and reverse reads into a single interleaved FASTQ file where each read pair appears on consecutive lines.

### Concatenate FASTA files without quality handling
**Args:** `-o combined.fasta sequences1.fasta sequences2.fasta`
**Explanation:** For FASTA input, the tool simply concatenates the sequence records without considering quality scores, appropriate for reference or annotation merges.

### Preserve file headers during merge
**Args:** `--keep-headers -o output.fastq lanes/*.fastq`
**Explanation:** The `--keep-headers` flag preserves any embedded headers or metadata comments from source files, useful when header information must be retained for downstream processing.