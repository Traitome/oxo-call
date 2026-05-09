---
name: cansam
category: bioinformatics/sequence-analysis
description: A command-line tool for viewing, filtering, and converting SAM/BAM alignment files. Provides fast random access to alignments, supports various output formats, and enables efficient manipulation of short read data.
tags:
  - sam
  - bam
  - alignment
  - sequence-analysis
  - read-mapping
author: AI-generated
source_url: https://github.com/genome/cansam
---

## Concepts

- **Data Model**: Cansam operates on SAM (Sequence Alignment/Map) and BAM (binary SAM) alignment files, treating each read alignment as a record with mandatory fields (QNAME, FLAG, RNAME, POS, MAPQ, CIGAR, SEQ, QUAL) plus optional tags for NM, MD, XS, and other annotations.
- **I/O Formats**: Supports streaming from stdin and writing to stdout in SAM, BAM, or CRAM formats depending on file extensions; automatic format detection based on magic bytes for BAM/CRAM, text detection for SAM.
- **Indexing**: Works with companion index files (`.bai`, `.csi`) for random access by genomic coordinate; without an index, operations require full file scans which are significantly slower for large datasets.
- **Filtering**: Accepts field-based filter expressions (e.g., `flag & 0x4 == 0` to keep mapped reads) applied before output, reducing memory footprint and improving performance for targeted analyses.

## Pitfalls

- **Forgetting to sort alignments**: Many downstream tools require coordinate-sorted BAM files; feeding unsorted output from cansam to tools expecting sorted data produces incorrect results without error messages in subtle ways.
- **Mismatched index files**: Using an index generated for a different BAM file leads to invalid random access lookups that may silently return no hits or cause crashes; always regenerate indices after any reordering operation.
- **Ignoring the header**: The SAM header (`@HD`, `@SQ`, `@PG`) contains critical metadata about read group IDs, sample names, and reference sequences; omitting header preservation (`-h`) breaks tools that depend on @RG tags for demultiplexing.
- **Compression format confusion**:Writing BAM to stdout in a pipe without explicitly specifying `-b` can produce gzip-compressed SAM that downstream tools cannot parse, leading to cryptic format errors.

## Examples

### View alignments in a specific genomic region
**Args:** -r chr1:1000000-2000000 input.bam
**Explanation:** Extracts all alignments overlapping the specified 1 Mb window on chromosome 1 using the index for fast random access rather than scanning the entire file.

### Convert BAM to SAM format
**Args:** -S input.bam
**Explanation:** Converts binary BAM to human-readable SAM format, useful for debugging or when working with text-processing pipelines that expect SAM.

### Filter unmapped reads and keep only primary alignments
**Args:** -f "flag & 0x4 == 0 && flag & 0x100 == 0" input.bam
**Explanation:** Removes both unmapped reads and secondary/supplementary alignments using bitwise FLAG filtering, leaving only primary mapped reads for variant calling.

### Output only read names and mapping qualities
**Args:** -c "{qname}\t{mapq}\n" input.bam
**Explanation:** Prints a custom tab-separated list of read names and MAPQ values using a format string, efficient for quick quality assessment without parsing full records.

### Sort alignments by read name
**Args:** -n input.bam -o sorted_by_name.bam
**Explanation:** Produces a read-name sorted BAM file required for certain tools like Picard's MarkDuplicates that expect query-name ordering, different from coordinate sorting.

### Preserve and include SAM header in output
**Args:** -h input.bam -o output.bam
**Explanation:** Copies the complete SAM header (@HD, @SQ, @RG, @PG lines) to the output file, essential when chaining multiple cansam operations or when downstream tools need reference sequence dictionary.

### Count reads per chromosome
**Args:** -c "{rname}\n" input.bam | sort | uniq -c
**Explanation:** Extracts only the reference sequence name field and pipes to Unix utilities for fast per-chromosome read counting, leveraging cansam as a filter before standard Unix processing.