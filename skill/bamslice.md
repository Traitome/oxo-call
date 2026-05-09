---
name: bamslice
category: BAM Manipulation / Genomic Interval Extraction
description: Extract a subset of reads from a BAM file that overlap with specified genomic regions. Supports both indexed and streaming modes, with options for read filtering, quality trimming, and output compression.
tags: [bam, sam, extraction, interval, genomics, reads, region-query]
author: AI-generated
source_url: https://github.com/DecodeOffline/bamslice
---

## Concepts

- **Index requirement**: When using region queries (`-r`), bamslice requires a corresponding `.bai` index file (generated via `samtools index`). Without an index, only streaming extraction by read name is supported.
- **Region format**: Genomic intervals are specified as `chr:start-end` (1-based, inclusive). For split queries, use comma-separated entries like `chr1:1000-2000,chr2:5000-6000`.
- **Streaming vs indexed mode**: Streaming mode (`-i` without `-r`) scans the entire BAM sequentially and writes matching reads directly, which is slower but works without an index. Indexed mode uses `samtools` tabix-style random access for fast region retrieval.
- **Quality control flags**: The `-m` flag sets a minimum mapping quality threshold; reads with `MAPQ` below this value are excluded from output. The `-q` flag enables base quality trimming before writing.
- **Output formats**: By default, output is written as SAM (uncompressed). Use `-c` for BAM output or `-g` for CRAM output. Explicit `-O` enables uncompressed BAM for intermediate processing.

## Pitfalls

- **Missing BAI index causes failure**: If you specify `-r` for a region query but the BAI file does not exist alongside the BAM, bamslice terminates with an error instead of falling back to streaming mode.
- **Wrong chromosome names**: If your BAM uses `chr1` but you query `1` (or vice versa), zero reads are returned silently. Always verify that reference names match between your query and the alignment file header.
- **Integer overflow with large regions**: Specifying a region with a start coordinate of `0` or end coordinate exceeding `INT_MAX` (2,147,483,647) can cause silent overflow, resulting in truncated or empty output.
- **Conflicting compression flags**: Using both `-c` (BAM output) and `-g` (CRAM output) simultaneously produces undefined behavior; only the last flag on the command line takes effect.
- **Missing output directory**: If the `-o` flag specifies a path with a directory that does not exist, bamslice prints a warning but continues writing to the current directory, potentially overwriting existing files.

## Examples

### Extract all reads overlapping the BRCA1 gene region
**Args:** `-r chr17:43044295-43125345 -i reference.bam -o brca1_reads.bam`
**Explanation:** This queries the indexed BAM using random access to retrieve all reads overlapping the specified genomic interval on chromosome 17 where BRCA1 is located.

### Stream all reads by read name without an index
**Args:** `-n my_read_name -i reference.bam -o matched_read.sam`
**Explanation:** Using the `-n` flag enables streaming mode that scans the BAM sequentially to find all records with the specified read name, requiring no index file.

### Filter reads by minimum mapping quality of 20
**Args:** `-r chr3:100000-500000 -m 20 -i reference.bam -o highq_reads.bam`
**Explanation:** This extracts reads in the region while discarding any reads with a mapping quality below 20, ensuring only reliably aligned reads are included in the output.

### Query multiple discontiguous genomic regions
**Args:** `-r chr1:1000000-1050000,chr1:2000000-2050000,chr2:5000000-5100000 -i reference.bam -o multi_region_reads.bam`
**Explanation:** Comma-separated region queries allow retrieving reads from multiple separate genomic intervals in a single command without requiring a BED file.

### Trim base qualities before writing output
**Args:** `-r chrX:150000000-155000000 -i reference.bam -o trimmed_reads.bam -q 20`
**Explanation:** The `-q` flag activates quality trimming, removing low-quality bases from the right end of each read, which is useful for downstream variant calling to reduce false positives.

### Export reads as uncompressed BAM for compatibility
**Args:** `-r chr4:100000-200000 -i reference.bam -o unpacked_reads.bam -O`
**Explanation:** The `-O` flag produces uncompressed BAM output, which is faster to write and often better suited for intermediate pipeline steps that will reprocess the data.

### Retrieve unmapped reads only using name-based streaming
**Args:** `-n "" -i reference.bam -o unmapped_reads.sam`
**Explanation:** Streaming mode with an empty name query pattern combined with appropriate filtering can extract specific read categories depending on your BAM's tag conventions.