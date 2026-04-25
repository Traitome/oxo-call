---
name: bamtools
category: utilities
description: Command-line toolkit for reading, writing, and manipulating BAM format alignment files
tags: [bam, alignment, statistics, filtering, merging, splitting, ngs, utility, convert]
author: oxo-call built-in
source_url: "https://github.com/pezmaster31/bamtools"
---

## Concepts

- bamtools provides 12 subcommands: stats, count, coverage, filter, merge, split, sort, index, convert, random, resolve, revert.
- All subcommands use `-in` for input BAM and `-out` for output. Region filtering: `-region chr:start-end`.
- `bamtools stats`: comprehensive alignment statistics (total reads, mapped, unmapped, duplicates, paired, insert sizes).
- `bamtools filter`: filter alignments by flags, region, mapping quality, or JSON script. Flag filters use string names (`isDuplicate`, `isMapped`, `isPaired`, `isProperPair`, `isMateMapped`) not numeric values like samtools.
- `bamtools filter` JSON script (`-script filter.json`): allows complex multi-rule filtering with AND/OR logic and comparison operators on tags and properties.
- `bamtools merge`: merge multiple BAM files with `-in` repeated for each file.
- `bamtools split`: split BAM by reference (`-reference`), read group (`-tag RG`), or mapping quality. Outputs one BAM per unique value.
- `bamtools convert`: convert between BAM, SAM, BED, FASTQ, FASTA, JSON, and YAML formats using `-format`.
- `bamtools coverage`: per-position coverage depth across the genome.
- `bamtools resolve`: resolves paired-end reads by marking the IsProperPair flag.
- `bamtools revert`: removes duplicate marks and restores original base qualities.
- `bamtools sort`: sorts BAM file by various criteria; `-sortby` accepts `coordinate` or `readname`.

## Pitfalls

- bamtools ARGS must start with a subcommand (convert, count, coverage, filter, header, index, merge, random, resolve, revert, sort, split, stats) — never with flags like -in, -out, -region. The subcommand ALWAYS comes first.
- bamtools and samtools have overlapping functionality — samtools is more commonly used and actively maintained. Prefer samtools for most operations.
- bamtools filter flags use string names (`isDuplicate`, `isMapped`) not numeric values like samtools (`-F 4`). This is a key difference.
- The `-region` format is `chromosome:start-end`. Note: bamtools uses 0-based half-open coordinates for region queries.
- bamtools index creates a .bai index; compatible with samtools index output.
- For large files, samtools is typically faster than bamtools. Use bamtools primarily for its JSON filter script and split functionality.
- bamtools subcommands do not have detailed `--help` output — the tool prints minimal usage errors when arguments are missing.
- bamtools filter `-isMapped true` uses lowercase `true`/`false`, not numeric 0/1.

## Examples

### get alignment statistics from a BAM file
**Args:** `stats -in input.bam > alignment_stats.txt`
**Explanation:** stats subcommand; -in input.bam input file; provides total reads, mapped, unmapped, duplicates, paired statistics; redirect output to save

### count aligned reads in a BAM file
**Args:** `count -in input.bam`
**Explanation:** count subcommand; -in input.bam input file; outputs total read count to stdout; quick way to check BAM file size

### filter BAM to keep only mapped, properly paired reads
**Args:** `filter -in input.bam -out filtered.bam -isMapped true -isProperPair true`
**Explanation:** filter subcommand; -in input.bam input file; -out filtered.bam output file; -isMapped true keeps only mapped reads; -isProperPair true keeps properly paired reads; string boolean values, not numeric

### filter BAM by mapping quality threshold
**Args:** `filter -in input.bam -out filtered.bam -mapQuality 30`
**Explanation:** filter subcommand; -in input.bam input file; -out filtered.bam output file; -mapQuality 30 keeps reads with mapping quality >= 30; equivalent to samtools view -q 30

### filter BAM using a JSON script with complex rules
**Args:** `filter -in input.bam -out filtered.bam -script filter_rules.json`
**Explanation:** filter subcommand; -in input.bam input file; -out filtered.bam output file; -script filter_rules.json applies complex multi-rule filtering from a JSON file; supports AND/OR logic, tag comparisons, and multiple conditions in one pass

### merge multiple BAM files
**Args:** `merge -in sample1.bam -in sample2.bam -in sample3.bam -out merged.bam`
**Explanation:** merge subcommand; multiple -in flags for each input BAM; -out merged.bam output file; all inputs should be sorted the same way

### split BAM by reference (chromosome)
**Args:** `split -in input.bam -reference`
**Explanation:** split subcommand; -in input.bam input file; -reference creates one BAM per chromosome/reference; output files named like input.REF.bam

### split BAM by read group tag
**Args:** `split -in input.bam -tag RG`
**Explanation:** split subcommand; -in input.bam input file; -tag RG splits by read group; each read group gets its own output BAM; useful for demultiplexing

### convert BAM to SAM format
**Args:** `convert -in input.bam -format sam -out output.sam`
**Explanation:** convert subcommand; -in input.bam input file; -format sam converts to human-readable SAM; -out output.sam output file; also supports fastq, fasta, bed, json, yaml

### convert BAM to FASTQ for re-alignment
**Args:** `convert -in input.bam -format fastq -out reads.fastq`
**Explanation:** convert subcommand; -in input.bam input file; -format fastq extracts original read sequences; -out reads.fastq output file; useful for re-alignment with different parameters

### get coverage statistics per position
**Args:** `coverage -in input.bam > coverage.txt`
**Explanation:** coverage subcommand; -in input.bam input file; outputs per-position depth of coverage; tab-separated format with chromosome, position, and depth

### print the BAM header
**Args:** `header -in input.bam`
**Explanation:** header subcommand; -in input.bam input file; prints the SAM header (including @SQ, @RG, @PG lines) to stdout; useful for checking reference names and read groups

### sort a BAM file by coordinate
**Args:** `sort -in input.bam -out sorted.bam -sortby coordinate`
**Explanation:** sort subcommand; -in input.bam input file; -out sorted.bam output file; -sortby coordinate sorts by genomic position (default); also accepts readname for name-sorted output

### resolve paired-end reads
**Args:** `resolve -in input.bam -out resolved.bam`
**Explanation:** resolve subcommand; -in input.bam input file; -out resolved.bam output file; marks the IsProperPair flag for paired-end reads; ensures proper pair information is correct

### create index for a BAM file
**Args:** `index -in input.bam`
**Explanation:** index subcommand; -in input.bam input file; creates input.bam.bai index file; required for region-based queries; compatible with samtools
