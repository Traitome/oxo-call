---
name: bamcmp
category: sequencing/alignment Quality Control
description: Compares two BAM files to identify overlapping reads between them, useful for improving duplicate marking algorithms and assessing library complexity in NGS pipelines.
tags: bam, overlap-detection, duplicate-marking, paired-end, quality-control, samtools
author: AI-generated
source_url: https://github.com/ncbi/bamcmp
---

## Concepts

- bamcmp takes two BAM files as input (typically a "primary" and a "secondary" file from the same library) and identifies reads that share the same genomic coordinates, distinguishing between PCR duplicates and genuine overlapping fragments from library preparation.
- The tool adds custom tags to the BAM output (such as overlap status flags) indicating whether each read is unique, a duplicate, or overlapping with a read in the other file, enabling downstream tools to make informed decisions about duplicate handling.
- Input BAM files must be coordinate-sorted and indexed (BAI file present), with matching read groups and consistent headers for proper overlap detection.
- For paired-end data, bamcmp considers both read orientation and insert size when determining overlaps, whereas for single-end data it evaluates read position only.
- The tool can operate in different modes: reporting overlap counts, marking overlaps in the BAM output, or generating a simple text summary of overlap statistics.

## Pitfalls

- **Using unsorted or improperly indexed BAM files** will cause bamcmp to fail or produce incorrect results; always ensure both input files are coordinate-sorted and have corresponding .bai index files.
- **Confusing the order of input BAM files** (primary vs. secondary) can lead to unexpected overlap tagging if the tool treats one file as the "reference" for comparison; verify the correct file order in your pipeline documentation.
- **Ignoring read group consistency** between the two BAM files may result in failed comparisons or missing overlaps; ensure both files share the same read group IDs and sample names in their headers.
- **Setting overlap tolerance too loosely** (e.g., allowing large position windows) creates false positives where unrelated reads are incorrectly marked as overlapping, inflating apparent library complexity.
- **Not accounting for sequencing technology differences** when comparing files from different platforms (e.g., HiSeq vs. NovaSeq) can produce misleading overlap statistics due to varying base quality distributions and alignment behaviors.

## Examples

### Compare two BAM files from the same library to identify overlapping reads

**Args:** -1 sample1.bam -2 sample2.bam -o overlap_output.bam
**Explanation:** This runs bamcmp in standard mode comparing two BAM files and outputs a marked BAM file containing overlap information for each read.

### Generate a summary report of overlapping reads without modifying the BAM

**Args:** -1 sample1.bam -2 sample2.bam -s overlap_summary.txt
**Explanation:** This produces a text summary file with statistics about reads that overlap between the two files, useful for quality assessment without altering alignment data.

### Set a specific nucleotide position tolerance for overlap detection

**Args:** -1 sample1.bam -2 sample2.bam -t 5 -o output_marked.bam
**Explanation:** This allows reads within 5 nucleotides of each other to be considered overlapping rather than requiring exact coordinate matches, accounting for small alignment variations.

### Process paired-end data with strict insert size matching

**Args:** -1 pe_sample1.bam -2 pe_sample2.bam -p -o pe_overlap.bam
**Explanation:** The -p flag enables paired-end mode, where bamcmp considers both read pairs and insert size when determining overlaps, ensuring accurate duplicate detection for paired-end libraries.

### Use a custom overlap tag name in the output

**Args:** -1 sample1.bam -2 sample2.bam --tag-name XC -o custom_tagged.bam
**Explanation:** This writes overlap information to a custom tag (XC) instead of the default tag name, useful when integrating with downstream pipelines that expect specific tag conventions.

### Filter overlaps by minimum mapping quality

**Args:** -1 sample1.bam -2 sample2.bam -q 30 -o highq_overlap.bam
**Explanation:** This only considers reads with a mapping quality (MAPQ) of 30 or higher when detecting overlaps, filtering out low-confidence alignments from the comparison.