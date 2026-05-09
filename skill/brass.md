---
name: brass
category: sequence_alignment
description: Brass is a bioinformatics tool for advanced sequence read processing and assembly-based alignment, typically used in the BWA family of tools for handling complex genomic data with support for multiple input formats and filtering options.
tags:
- sequence_processing
- read_filtering
- assembly
- genomics
- alignment
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- Brass processes FASTQ/FASTA sequence reads as primary input, accepting both single-end and paired-end read formats for flexible genomic analysis workflows.
- The tool operates on a data model that treats input reads as individual records with associated quality scores, enabling sophisticated filtering based on mapping quality thresholds.
- Output is generated in standard SAM/BAM format or as filtered read sets, allowing seamless integration with downstream bioinformatics pipelines.
- Brass supports multiple indexing modes for efficient read lookup, including hash-based and suffix array approaches depending on the reference size.

## Pitfalls

- Specifying an incorrect reference genome or using a reference that doesn't match your read samples will result in alignment artifacts and invalid downstream analyses.
- Omitting proper quality score thresholds can allow low-confidence reads to pass through, potentially introducing false positive variants in variant calling workflows.
- Forgetting to specify read orientation flags when processing paired-end data leads to incorrect pair handling and potentially dropped read pairs.
- Using mismatched or version-incompatible index files causes silent failures where reads align incorrectly without obvious error messages.
- Neglecting to adjust thread count for large datasets results in unnecessarily long processing times, as Brass is computationally intensive on whole-genome data.

## Examples

### Filter reads by mapping quality threshold
**Args:** -q 20 input.fq -ref reference.fa
**Explanation:** This filters input FASTQ reads against the reference, retaining only those with mapping quality scores of 20 or higher for high-confidence downstream analysis.

### Process paired-end reads with proper orientation
**Args:** -1 left.fq -2 right.fq -ref genome.fa -f 0x2
**Explanation:** This processes paired-end reads with the 0x2 flag to ensure proper read pairing, preventing misalignment of read pairs in the dataset.

### Convert filtered output to BAM format
**Args:** input.fq -ref ref.fa -o output.bam -b
**Explanation:** This processes input reads and outputs the resulting alignments directly in BAM format for compatibility with standard genomic tools.

### Specify multiple threads for parallel processing
**Args:** -t 8 input.fq -ref reference.fa -o filtered.sam
**Explanation:** This uses 8 CPU threads to accelerate processing of large genomic datasets, significantly reducing runtime on multi-core systems.

### Enable verbose logging for debugging
**Args:** -v input.fq -ref reference.fa -o output.sam
**Explanation:** This enables verbose output mode to provide detailed status messages during processing, useful for troubleshooting pipeline issues.