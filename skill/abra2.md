---
name: abra2
category: Sequence Assembly/Alignment
description: A bioinformatics tool for short-read assembly and alignment operations. Processes nucleotide sequences from various input formats to generate assembled contigs, alignments, or related output files. Supports standard FASTQ/FASTA input and produces assembly metrics or consensus sequences.
tags: [assembly, alignment, genomics, short-reads, contig-formation]
author: AI-generated
source_url: https://github.com/abra2/abra2
---

## Concepts

- **Input Format Handling**: abra2 accepts both FASTQ and FASTA formatted sequence files as primary input. Multi-sample inputs can be provided via comma-separated lists or directory globbing patterns. Files may be compressed (gzip/zip) or uncompressed.
- **Assembly Algorithm**: The tool implements overlap-layout-consensus (OLC) or de Bruijn graph-based assembly strategies depending on specified parameters. Read k-mer size selection critically impacts assembly continuity — smaller k-mers increase sensitivity but produce more complex graphs.
- **Output Types**: Produces assembled contigs in FASTA format, alignment summaries in SAM/BAM format, and quality metrics in JSON/CSV. Assembly statistics include N50, coverage depth, and total contig count.
- **Parameter Modes**: Supports three operation modes — assembly (default), alignment, and validation. Each mode has mode-specific required arguments and complementary optional flags.

## Pitfalls

- **Incompatible Read Lengths**: Mixing reads of substantially different lengths (e.g., 150bp and 300bp) without adjusting k-mer size causes fragmented or failed assemblies. Consequence: low N50 values and excessive chimeric contigs.
- **Memory Exhaustion with Large Datasets**: Large FASTQ files (exceeding available RAM) without streaming flags cause out-of-memory crashes. Always use `--streaming` or `--chunk-size` for genomes >100MB raw data.
- **Duplicate Read Filtering**: Overlooking duplicate reads inflates coverage estimates and creates assembly artifacts. Failing to enable `--remove-duplicates` leads to false coverage metrics and potentially misassembled repeat regions.
- **Invalid Reference Alignment**: Using `--align` mode without a valid reference sequence file produces empty or misleading output. The reference must be in FASTA format with searchable contig names.

## Examples

### Assemble raw short reads into contigs
**Args:** `--mode assembly --input reads.fq --output assembly.fasta --kmer-size 21`
**Explanation:** This runs the default assembly mode using 21-mers for the de Bruijn graph, outputting assembled contigs to the specified FASTA file.

### Align reads to a reference genome
**Args:** `--mode align --reads sample1.fq,sample2.fq --reference ref.fasta --output alignments.sam`
**Explanation:** Aligns multiple sample FASTQ files against the reference, generating SAM format alignments for downstream variant calling.

### Generate assembly statistics without full assembly
**Args:** `--mode validate --input reads.fq --output metrics.json --kmer-size 25 --verbose`
**Explanation:** Runs validation mode to compute coverage, N50, and other metrics using 25-mers, writing detailed statistics to JSON for quality assessment.

### Streaming mode for large files
**Args:** `--mode assembly --input /path/to/large_dir/*.fq.gz --output large_assembly.fasta --streaming --threads 16`
**Explanation:** Enables streaming to process gzipped FASTQ files sequentially without loading entire dataset into memory, using 16 parallel threads.

### Remove duplicate reads before assembly
**Args:** `--mode assembly --input dedup_test.fq --output clean_assembly.fasta --remove-duplicates --min-coverage 3`
**Explanation:** Filters duplicate reads prior to assembly, ensuring coverage estimates reflect true unique read depth and reducing repeat-induced misassemblies.