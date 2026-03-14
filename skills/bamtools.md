---
name: bamtools
category: utilities
description: Command-line toolkit for reading, writing, and manipulating BAM format alignment files
tags: [bam, alignment, statistics, filtering, merging, ngs, utility]
author: oxo-call built-in
source_url: "https://github.com/pezmaster31/bamtools"
---

## Concepts

- bamtools provides commands: stats, count, merge, split, filter, sort, index, random, convert.
- bamtools stats: comprehensive alignment statistics per BAM file.
- bamtools filter: filter alignments by flags, region, mapping quality, or custom scripts.
- bamtools merge: merge multiple BAM files into one; bamtools split: split BAM by tag or reference.
- bamtools convert: convert between BAM, SAM, BED, FASTQ, FASTA, JSON, and YAML formats.
- All subcommands use -in for input, -out for output; -region chr:start-end for region filtering.
- bamtools has a JSON-based filter language for complex filtering rules.

## Pitfalls

- bamtools and samtools have overlapping functionality — samtools is more commonly used and actively maintained.
- bamtools filter flags use string names ('isDuplicate', 'isMapped') not numeric values like samtools.
- The -region format is chromosome:start-end (0-based in BAM coordinates).
- bamtools index creates a .bai index; compatible with samtools index output.
- For large files, samtools is typically faster than bamtools.

## Examples

### get alignment statistics from a BAM file
**Args:** `stats -in input.bam > alignment_stats.txt`
**Explanation:** stats subcommand provides total reads, mapped, unmapped, duplicates, paired statistics

### count aligned reads in a BAM file
**Args:** `count -in input.bam`
**Explanation:** count outputs total read count to stdout

### filter BAM to keep only mapped, properly paired reads
**Args:** `filter -in input.bam -out filtered.bam -isMapped true -isProperPair true`
**Explanation:** -isMapped true keeps only mapped reads; -isProperPair true keeps properly paired reads

### merge multiple BAM files
**Args:** `merge -in sample1.bam -in sample2.bam -in sample3.bam -out merged.bam`
**Explanation:** multiple -in flags for each input BAM; -out for merged output

### convert BAM to FASTQ
**Args:** `convert -in input.bam -format fastq -out reads.fastq`
**Explanation:** -format fastq; also supports -format sam, fasta, bed, json
