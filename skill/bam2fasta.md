---
name: bam2fasta
category: Format Conversion
description: A BAM-to-FASTA sequence extractor that converts binary alignment map files to text-based FASTA format, supporting mapped/unmapped read filtering and genomic region extraction.
tags:
  - bam
  - fasta
  - conversion
  - sequence-extraction
  - genomics
  - alignment
author: AI-generated
source_url: https://github.com/najoshi/bam2fasta
---

## Concepts

- **BAM is binary; FASTA is text-based**: bam2fasta reads the binary BAM format and extracts the SEQ field from each alignment record, converting it to the standard `>readname\nsequence` FASTA format. The quality (QUAL) field is not included in standard FASTA output and must be requested separately if needed.
- **Filtering by alignment status**: By default, bam2fasta extracts all reads regardless of mapping status. Use flags like `-u` to include only unmapped reads or `-m` to extract only mapped reads. This distinction is critical when you need consensus sequences from only the callable regions of a genome.
- **Genomic region extraction requires an index**: When specifying genomic coordinates with the `-r` flag, bam2fasta relies on the BAM index file (.bai) to perform efficient lookup. Without an index, region extraction will fail or produce incomplete results.
- **Paired-end read handling**: For paired-end data, bam2fasta can extract both reads (`-p` flag) or only properly paired reads (`-f 2` flag). Extracting unpaired reads from a paired-end dataset will double your output file size and may introduce artifacts from duplicate reads.

## Pitfalls

- **Assuming unaligned BAM files work for region extraction**: Attempting to use the `-r` flag on an unsorted or unindexed BAM file produces empty output without a clear error message. Always run `samtools sort` and `samtools index` on your BAM file before region-specific extraction.
- **Not specifying output compression**: By default, bam2fasta outputs to stdout. Redirecting to a file with `>` may create a large uncompressed FASTA file that is slow to process downstream. Use `-z` for gzip compression or `-c` for bzip2 compression if the output file will be used in pipelines.
- **Extracting both read pairs without deduplication**: With paired-end data, using `-p` outputs both reads independently, which means the same genomic region appears twice. If you need a single consensus, this duplication causes downstream issues like inflated coverage estimates.
- **Ignoring the reverse-complement flag**: Sequences from the reverse strand in BAM files have their SEQ field already reverse-complemented, but the original orientation is not preserved in FASTA output. If strand orientation matters for your analysis, check the FLAG field manually before assuming all output sequences are in the same orientation.
- **Processing without checking BAM integrity**: Running bam2fasta on a corrupted BAM file produces truncated output or hangs silently without reporting the corruption. Validate the BAM file with `samtools quickcheck` before conversion to avoid wasting compute time.

## Examples

### Convert an entire BAM file to FASTA with gzip compression

**Args:** `-i input.bam -z -o output.fa.gz`
**Explanation:** The `-i` flag specifies the input BAM file, `-z` enables gzip compression of the output, and `-o` redirects the compressed FASTA to a file rather than stdout.

### Extract only mapped reads from a sorted BAM file

**Args:** `-i aligned_sorted.bam -m -o mapped_reads.fa`
**Explanation:** The `-m` flag filters the input to include only reads with the mapped flag set in the alignment, producing a FASTA containing sequences from successfully aligned reads only.

### Extract unmapped reads from a BAM file containing both aligned and unaligned data

**Args:** `-i mixed_data.bam -u -o unmapped_reads.fa`
**Explanation:** The `-u` flag extracts reads marked as unmapped (typically due to failed alignment or poor quality), which is useful for de novo assembly of leftover reads.

### Extract reads from a specific genomic region using an indexed BAM file

**Args:** `-i sample_sorted.bam -r chr1:1000000-2000000 -o region1Mb.fa`
**Explanation:** The `-r` flag specifies genomic coordinates (chromosome, start, end) and requires an indexed BAM file. The output contains only the sequences of reads overlapping this 1 Mb region of chromosome 1.

### Extract both read pairs from a paired-end BAM file

**Args:** `-i paired_sample.bam -p -o paired_reads.fa`
**Explanation:** The `-p` flag ensures both forward and reverse reads from each proper pair are included in the output. Each read appears as a separate FASTA entry, which doubles the total entry count compared to single-end data.