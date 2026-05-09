---
name: bambamc
category: bioinformatics/sequence-alignment
description: BAM/CRAM file compression and manipulation tool from the BamBam toolkit, used for converting, sorting, and indexing aligned sequence data
tags: [bam, cram, sam, compression, alignment, sequencing]
author: AI-generated
source_url: https://github.com/gt1/bambam
---

## Concepts

- `bambamc` is a BAM/CRAM compression utility that converts SAM (Sequence Alignment Map) files to their binary compressed formats (BAM or CRAM), reducing storage requirements significantly while maintaining full alignment information
- The tool accepts input from stdin or file arguments, supporting streaming operations that allow processing of large alignment files without loading entire datasets into memory; this is critical for whole-genome-scale analyses
- Output format is determined by the file extension or explicit flags: `.bam` produces BAM format, `.cram` produces CRAM format; CRAM offers better compression ratios but requires a reference genome
- Input files must be sorted (either by coordinate or read name) for optimal processing; unsorted inputs may trigger errors or produce non-standard output that downstream tools cannot handle correctly
- The tool integrates with the broader BamBam ecosystem, producing output compatible with other utilities like `bambamc-build` for indexing and `bambasky` for visualization

## Pitfalls

- Using CRAM output without specifying a valid reference genome (`-T` flag) causes immediate failure with a reference-dependent error, leaving no output file and wasting processing time on large datasets
- Attempting to compress files with missing or malformed headers leads to partial output or silent data corruption; always verify input file integrity with `samtools view -H` before compression
- Processing unsorted BAM files results in non-standard output that breaks coordinate-sorted tools like `bcftools` and `GATK`, creating downstream analysis failures that are difficult to diagnose
- Outputting to the same file path as the input causes data loss if the input is a special file or if the tool encounters an error mid-processing; always use temporary output locations and atomic renaming
- Failing to specify explicit output format when using stdout results in BAM format by default, which may be unexpected when CRAM was intended; confirm format via `-O` flag for clarity

## Examples

### Convert a SAM file to compressed BAM format
**Args:** `-o output.bam input.sam`
**Explanation:** Converts the human-readable SAM alignment file to binary BAM format, reducing file size by approximately 90% while preserving all alignment data and metadata

### Convert BAM to CRAM with a reference genome
**Args:** `-T /path/to/reference.fa -O cram input.bam -o output.cram`
**Explanation:** Transforms BAM to CRAM format using the specified FASTA reference, achieving superior compression especially for high-coverage datasets where CRAM can reduce storage by 40-60% over BAM

### Stream convert SAM to BAM with explicit output format
**Args:** `-O b