---
name: bammap2
category: BAM/SAM File Manipulation
description: A command-line tool for BAM file manipulation, including sorting, indexing, filtering, merging, and generating alignment statistics for next-generation sequencing data.
tags: [bam, sam, alignment, ngs, sequencing, bioinformatics, sorting, indexing, filtering]
author: AI-Generated
source_url: https://github.com/bammap2/bammap2
---

## Concepts

- **BAM File Format**: BAM is the binary, compressed version of SAM (Sequence Alignment/Map). bammap2 operates directly on BAM files, and when indexing is required, it generates corresponding .bai index files automatically. Always ensure input BAM files are sorted by genomic position before indexing, or specify sorting options explicitly.
- **Alignment Flags**: BAM stores alignment metadata as bitwise flags (e.g., 0x0002 for paired reads, 0x0004 for unmapped reads). bammap2 uses standard SAM flag conventions for filtering—understanding flag values (decimal or hexadecimal) is essential for selecting reads by pair status, mapping quality, or strand orientation.
- **Reference Indexing**: Genomic regions in bammap2 are specified as `chromosome:start-end` using 1-based inclusive coordinates. The tool internally converts these to 0-based half-open intervals for compatibility with the underlying htslib engine. Specifying regions outside the reference contig sizes will result in empty output.
- **Streaming and Compression**: bammap2 processes files in streaming mode when possible, reducing memory footprint for large BAM files. Output is always written in BAM format (uncompressed or with configurable compression level), and gzip/wrap output can be enabled for compatibility with downstream tools like IGV or GATK.

## Pitfalls

- **Unsorted Input for Indexing**: Attempting to create a BAM index (.bai) on an unsorted BAM file will silently succeed but produce a corrupt index. Downstream tools (e.g., GATK, IGV) will fail or produce incorrect region queries. Always sort BAM files using `sort` subcommand before indexing.
- **Conflicting Flag Filters**: Using multiple flag filter conditions that are mutually exclusive (e.g., `--include-flags 0x0004 --exclude-flags 0x0004` to include and exclude unmapped reads simultaneously) results in zero reads being selected. The tool returns empty output without an explicit error message.
- **Coordinate System Mismatch**: Specifying genomic coordinates in 0-based format when the tool expects 1-based input (or vice versa) leads to off-by-one errors in region extraction. bammap2 expects 1-based start positions, but genomic arrays in scripts may use 0-based counting, causing subtle coordinate shifts.
- **Memory Limits for Large Files**: Processing unchunked BAM files larger than available RAM without the `--chunk-size` option causes excessive disk swapping. On systems with limited memory, use streaming mode (`--streaming`) to limit memory usage at the cost of processing speed.
- **Overwriting Input Files**: Redirecting output to the same filename as the input (e.g., `bammap2 filter input.bam > input.bam`) truncates the input file before reading, resulting in data loss. Always use temporary intermediate files or the `--out` option to specify a different output path.

## Examples

### Sort a BAM file by genomic position

**Args:** `sort --output sorted_output.bam --threads 4 input.bam`
**Explanation:** Sorts the input BAM file by reference name and position using 4 parallel threads, writing the result to `sorted_output.bam`. Sorted BAM files are required for indexing and many downstream analyses.

### Filter reads by mapping quality threshold

**Args:** `filter --min-mapq 30 --output high_qual.bam input.bam`
**Explanation:** Retains only reads with mapping quality (MAPQ) >= 30, removing low-quality alignments that may introduce false positive variants in variant calling workflows.

### Index a sorted BAM file for random access

**Args:** `index sorted_output.bam`
**Explanation:** Creates a `.bai` index file alongside the sorted BAM file, enabling efficient region queries by tools like samtools, GATK, and IGV. The BAM file must be sorted before indexing.

### Extract reads in a specific genomic region

**Args:** `view --region chr1:1000000-2000000 input.bam`
**Explanation:** Extracts all reads overlapping the specified genomic interval on chromosome 1 from position 1,000,000 to 2,000,000 (1-based coordinates), outputting them in BAM format.

### Remove unmapped reads and secondary alignments

**Args:** `filter --exclude-flags 0x0004,0x0100 --output primary_mapped.bam input.bam`
**Explanation:** Excludes reads that are unmapped (flag 0x0004) or are secondary alignments (flag 0x0100), producing a filtered BAM containing only primary, mapped reads suitable for variant calling.

### Generate alignment statistics summary

**Args:** `stats --output stats.txt input.bam`
**Explanation:** Computes summary statistics including total reads, mapped/unmapped counts, coverage depth, and insert size distribution, writing the report to `stats.txt` for quality control assessment.

### Merge multiple BAM files into one

**Args:** `merge --output merged.bam sample1.bam sample2.bam sample3.bam`
**Explanation:** Combines multiple BAM files into a single output file, automatically sorting by genomic position. Input files should ideally be from the same reference to avoid coordinate inconsistencies.

### Filter paired reads with abnormal insert size

**Args:** `filter --include-flags 0x0002 --min-insert 50 --max-insert 600 --output normal_inserts.bam input.bam`
**Explanation:** Retains only properly paired reads (flag 0x0002) with insert sizes between 50 and 600 base pairs, removing outliers that may indicate chimeric reads or structural variants for standard variant calling.

### Count reads per chromosome

**Args:** `idxstats sorted_indexed.bam`
**Explanation:** Reports the reference name, reference length, number of mapped reads, and number of unmapped reads for each contig in the sorted and indexed BAM file, useful for assessing sequencing uniformity.

### Compress output BAM with maximum compression

**Args:** `filter --compression-level 9 --output compressed.bam input.bam`
**Explanation:** Applies maximum gzip compression (level 9) to the output BAM file, reducing file size significantly at the cost of slower processing and higher CPU usage for long-term storage or file transfer.