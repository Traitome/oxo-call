---
name: beav
category: Sequence Analysis / Read Extraction
description: A bioinformatics tool for extracting specific sequences or regions from alignment files (BAM/CRAM). Often used for targeted read retrieval, read filtering by various attributes, and creating subset files for downstream analysis.
tags: [bioinformatics, sequence-extraction, bam, cram, read-filtering, samtools-companion, variant-calling, genomics]
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **Input Format**: Primary input is BAM or CRAM alignment files (and matching BAI index). Also accepts SAM format for small inputs. The tool requires a coordinate-sorted and indexed input file for efficient region-based extraction.
- **Region Specification**: Uses 1-based inclusive coordinates in the format `chr:start-end`. Multiple regions can be specified in a single command. Without explicit region flags, the tool behaves as a passthrough (header preservation).
- **Output Behavior**: By default, outputs to stdout in SAM format. Use `-b` for BAM output or `-C` for CRAM output. The output format determines the compression and binary encoding.
- **Filtering Capabilities**: Supports read-level filtering by flags (e.g., properly paired reads only), mapping quality thresholds (`-q`), and read name lists (`-R`). These filters are applied before region extraction when combined.
- **Companion Binary**: The companion tool `beav-build` creates index files (BAI/CSI for BAM, CRI for CRAM) required for efficient random access. Always index your alignment files before using `beav` for large datasets.

## Pitfalls

- **Forgetting to Index**: Attempting region-based extraction on an unindexed BAM/CRAM file will fail or require scanning the entire file (extremely slow for large files). Always run `beav-build` on coordinate-sorted alignment files first.
- **Wrong Coordinate System**: Using 0-based coordinates (common in programming) instead of 1-based genomic coordinates will extract the wrong base or fail silently. Verify coordinates from BED files and databases use 0-based while `beav` uses 1-based.
- **Output Format Mismatch**: Writing BAM output to a file without the `.bam` extension or stdout when piped to another tool expecting BAM can cause downstream tools to fail. Always specify `-b` explicitly when BAM output is needed.
- **Memory Issues with Large Regions**: Extracting very large regions or entire chromosomes into memory can cause OOM errors on systems with limited RAM. Process large files in chunks or use streaming mode with reduced buffer sizes.
- **Compressed Output to Terminal**: Attempting to write compressed BAM/CRAM directly to stdout without redirection will produce garbled output. Use `-o` for file output or ensure proper redirection when working with compressed formats.

## Examples

### Extract reads from a specific genomic region
**Args:** `region.bam -r chr1:1000000-2000000 -b -o region_extract.bam`
**Explanation:** Extracts all reads overlapping positions 1,000,000 to 2,000,000 on chromosome 1 and writes them to a compressed BAM file.

### Get reads supporting a specific variant call
**Args:** `sample.bam -r chr5:123456-123456 -f 0x2 -b -o variant_reads.bam`
**Explanation:** Extracts reads that properly pair and overlap position 123,456 on chromosome 5, useful for reviewing read-level evidence for a variant call.

### Filter reads by mapping quality
**Args:** `reads.bam -q 30 -b -o highmq_reads.bam`
**Explanation:** Retains only reads with mapping quality (MAPQ) of 30 or higher, removing ambiguous or poorly mapped reads before downstream analysis.

### Extract reads by a list of read names
**Args:** `input.bam -R readnames.txt -o captured_reads.sam`
**Explanation:** Extracts specifically named reads from a file containing a read name list (one per line), useful for extracting specific alignments from a larger dataset.

### View header information only
**Args:** `dataset.bam -H`
**Explanation:** Prints only the header section of the BAM file without any alignment records, useful for checking sample names, reference sequences, and file metadata.

### Convert SAM to indexed BAM
**Args:** `input.sam -b -o output.bam && beav-build output.bam`
**Explanation:** Converts a SAM file to compressed BAM and creates the required index file in a single pipeline operation, making it ready for random access.

### Extract paired-end reads only
**Args:** `experiment.bam -f 0x1 -F 0x4 -b -o paired_reads.bam`
**Explanation:** Filters to keep only reads that are properly paired (flag 0x1) and are not unmapped (flag 0x4), standard for variant calling pipelines.