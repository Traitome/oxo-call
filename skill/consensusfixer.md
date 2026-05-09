---
name: consensusfixer
category: Variant Calling / Sequence Correction
description: A bioinformatics tool for fixing consensus sequences and alignment artifacts in SAM/BAM files. Used to correct basecalling errors, fix indel offsets, and generateclean consensus sequences from aligned read data for improved variant calling accuracy.
tags: [sam, bam, consensus, variant-calling, sequence-correction, genomics]
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **Input Format**: consensusfixer operates on SAM/BAM alignment files, reading pileup information to identify and correct misaligned or incorrectlycalled bases in the consensus sequence.
- **Output Formats**: The tool can generate corrected consensus sequences in FASTA format, or output fixed SAM/BAM records with updated CIGAR strings and basequalities.
- **Indel Correction**: One of its primary functions is fixing indel offset errors where insertions or deletions are mispositioned relative to the reference, which is critical for accurate indel variant calling.
- **Pileup Processing**: The tool processes read pileups at each genomic position, using read depth and base quality information to determine the correct consensus base.

## Pitfalls

- **Missing Index Files**: Running consensusfixer on an unindexed BAM file will fail; always ensure the corresponding .bai index file exists in the same directory before processing.
- **Low Quality Threshold**: Setting the base quality threshold too low (e.g., below 10) can cause erroneous consensus calls due to sequencing noise, leading to falsepositive variants in downstream analysis.
- **Reference Mismatch**: Using a different reference genome version than what the reads were aligned against will produce corrupted consensus sequences with systematic positionalerrors.
- **Memory Intensive**: Processing whole-genome BAM files without sufficient RAM can cause the tool to crash; for large files, consider processing by chromosome or using chunked input.

## Examples

### Generate consensus from an aligned BAM file

**Args:** -in sample.bam -out consensus.fa

**Explanation:** This reads the aligned BAM file and outputs a FASTA format consensus sequence derived from the pileup of all aligned reads at each position.

### Fix indel offsets in a variant call set

**Args:** -in variants.bam -fix-indels -reference ref.fa -out fixed.vcf

**Explanation:** This corrects positional errors in indels by realigning them to the provided reference, outputting a VCF with properly offset insertion and deletion calls.

### Set minimum base quality for consensus calling

**Args:** -in reads.bam -minq 20 -out highq_consensus.fa

**Explanation:** This applies a quality threshold of 20, ensuring only high-confidence bases contribute to the consensus, reducing sequencing artifact influence.

### Process a specific genomic region

**Args:** -in alignment.bam -region chr1:1000000-2000000 -out region_cons.fa

**Explanation:** This extracts and generates consensus only for the specified chromosomal region, useful for targeting specific loci without processing the entire file.

### Generate consensus with read depth information

**Args:** -in sample.bam -depth -out consensus_withdepth.txt

**Explanation:** This outputs both the consensus sequence and per-position read depth counts, useful for assessing coverage and confidence in downstream analysis.

### Use multiple threads for faster processing

**Args:** -in large_sample.bam -out consensus.fa -t 8

**Explanation:** This enables parallel processing with 8 threads, significantly speeding up consensus generation on large BAM files with high coverage.