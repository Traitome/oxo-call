---
name: bamtools
category: alignment-manipulation
description: A command-line utility for manipulating and analyzing BAM/SAM alignment files, providing operations such as merge, sort, filter, split, convert, index, and statistics extraction.
tags: [bam, sam, alignment, conversion, manipulation, sequencing, samtools]
author: AI-generated
source_url: https://github.com/petkov/bamtools
---

## Concepts

- **BAM/SAM I/O formats**: bamtools reads and writes BAM (binary) and SAM (text) formats transparently; specify the format explicitly using `-in` and `-out` flags to avoid format ambiguity errors.
- **Command-based architecture**: bamtools uses a subcommand structure where the first argument determines the operation (e.g., `bamtools merge`, `bamtools sort`, `bamtools filter`), and each subcommand has its own unique flags.
- **Filtering expression language**: The filter subcommand uses a simple text-based expression syntax (e.g., `mapQual >= 30`, `flag.pcrDuplicate == 0`, `sequence =~ /ATCG/`) that evaluates against read metadata fields.
- **Read Group handling**: bamtools preserves and can manipulate read group tags (`@RG`) during merge and sort operations, which is critical for downstream GATK pipelines.

## Pitfalls

- **Missing sorted index for merge**: When merging BAM files, the input files must be coordinate-sorted; attempting to merge unsorted or queryname-sorted files produces corrupted output without an error message.
- **Integer overflow in memory**: Processing very large BAM files with `bamtools merge` or `bamtools sort` loads entire files into memory; files larger than available RAM will cause the program to crash silently or produce truncated output.
- **Overwriting input files by accident**: The `-out` flag when omitted defaults to overwriting the input file in-place; this causes permanent data loss if the same filename is specified for both input and output.
- **Incorrect flag filter syntax**: Using bitwise operators (e.g., `flag & 1024`) in filter expressions fails silently; the correct syntax uses field modifiers like `flag.pcrDuplicate == 1` or specific flag names.

## Examples

### Convert a BAM file to SAM format
**Args:** convert -in /data/alignments.bam -out /data/alignments.sam
**Explanation:** This reads the binary BAM input and writes the human-readable SAM format to the specified output file while preserving all alignment and read metadata.

### Sort a BAM file by genomic position
**Args:** sort -in /data/alignments.bam -out /data/alignments.sorted.bam -position
**Explanation:** This sorts alignments by chromosome reference and leftmost coordinate position, which is required for many downstream tools like GATK and for generating a valid index.

### Filter reads with mapping quality >= 30
**Args:** filter -in /data/alignments.bam -out /data/highqual.bam -mapQual ">= 30"
**Explanation:** This retains only reads with a minimum mapping quality of 30, reducing false positive alignments in variant calling or expression quantification analyses.

### Merge multiple BAM files into one
**Args:** merge -in A.bam -in B.bam -in C.bam -out combined.bam
**Explanation:** This combines three separate BAM files into a single output file, preserving all reads and read group information from each input file.

### Extract basic statistics from a BAM file
**Args:** stats -in /data/alignments.bam
**Explanation:** This displays summary statistics including total read count, mapped/unmapped read counts, coverage breadth, and average read depth without modifying any files.

### Extract headers from a BAM file
**Args:** header -in /data/alignments.bam -text
**Explanation:** This outputs the SAM header section in text format, showing all reference sequences (@SQ lines) and read group (@RG) definitions present in the file.

### Split a BAM file by read group
**Args:** split -in /data/alignments.bam -prefix split_reads_ -by RG
**Explanation:** This creates separate BAM files for each read group, naming them with the specified prefix and appending the read group identifier, useful for multiplexed sample demultiplexing.