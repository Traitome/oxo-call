---
name: clever-toolkit
category: sequence-analysis
description: A versatile bioinformatics toolkit for sequence manipulation, format conversion, quality control, and statistical analysis of genomic data. Supports FASTA, FASTQ, SAM, BAM, VCF, and BED formats with streaming capabilities for large datasets.
tags: [fasta, fastq, sam, bam, vcf, bed, sequence-analysis, format-conversion, quality-control, genomics]
author: AI-generated
source_url: https://github.com/clever-toolkit/clever-toolkit
---

## Concepts

- **Multi-subcommand architecture**: clever-toolkit operates as a collection of discrete functions invoked via subcommands (e.g., `clever-toolkit seq`, `clever-toolkit convert`, `clever-toolkit stats`), each designed for specific bioinformatics tasks with consistent input/output handling.
- **Streaming and indexed formats**: The tool processes SAM/BAM files via streams when piped, but reads indexed BAM/VCF files directly by genomic coordinate range using .bai/.tbi indices, enabling efficient random access without full file loading.
- **Standard genomic I/O formats**: All subcommands accept FASTA (`-f`), FASTQ (`-q`), SAM (`-s`), BAM (`-b`), VCF (`-v`), and BED (`-d`) inputs, auto-detecting format from file extensions unless explicitly specified via flags; output defaults to stdout in the same format as input unless redirected.

## Pitfalls

- **Forgetting to index reference sequences**: When converting or aligning against a reference, failing to generate an index (typically `.fai` for FASTA) causes the tool to scan the entire reference on each query, dramatically slowing performance on large genomes like human chromosome 1.
- **Mismatching input format flags**: Specifying `-f fastq` when providing a FASTA file, or omitting format flags entirely when working with non-standard file extensions, leads to silent parsing errors or corrupted output without clear error messages.
- **Ignoring coordinate conventions**: SAM/BAM coordinates are 1-based and inclusive, while BED coordinates are 0-based and half-open; mixing these conventions across subcommands produces off-by-one errors in filtering or annotation tasks.

## Examples

### Convert FASTQ to FASTA format
**Args:** convert -i sample.fastq -o sample.fasta
**Explanation:** This converts a FASTQ file containing quality scores to a plain FASTA file, stripping all quality information in the process.

### Filter reads by mapping quality
**Args:** filter -q 30 input.bam -o filtered.bam
**Explanation:** This removes alignments with a mapping quality (MAPQ) below 30, retaining only high-confidence mappings in the output BAM file.

### Generate sequence length statistics
**Args:** stats -l sequences.fasta
**Explanation:** This computes summary statistics including total bases, N50, total contig count, and mean sequence length for the provided FASTA file.

### Extract reads from a specific genomic region
**Args:** region -r chr1:1000000-2000000 -i alignments.bam -o region_reads.bam
**Explanation:** This extracts all alignments overlapping chromosome 1 from positions 1,000,000 to 2,000,000 using the indexed BAM file for efficient random access.

### Split a multi-sample VCF by sample name
**Args:** splitvcf -s NA12878 -i multisample.vcf -o NA12878.vcf
**Explanation:** This extracts only the genotype calls for sample NA12878 from a multi-sample VCF file, creating a single-sample VCF output.

### Reverse complement a DNA sequence
**Args:** seq -r sequence.fasta -o reversed.fasta
**Explanation:** This generates the reverse complement of each sequence in the input FASTA file, maintaining the original sequence order with all bases reversed and complemented.

### Count reads per chromosome from a BAM file
**Args:** count -t chr -i alignments.bam
**Explanation:** This tallies the number of aligned reads mapping to each chromosome in the input BAM file, outputting a tab-separated table of chromosome counts.