---
name: bwa-mem2
category: sequence-alignment
description: A fast and accurate aligner for mapping raw DNA sequence reads to a reference genome using seeded, banded Smith-Waterman dynamic programming. The successor to bwa-mem, delivering 2-3x speedup with identical output.
tags:
  - alignment
  - genomics
  - read-mapping
  - NGS
  - SAM
author: AI-generated
source_url: https://github.com/bwa-mem2/bwa-mem2
---

## Concepts

- **Input format**: Accepts raw FASTA or FASTQ files (plain, gzipped, or BAM format) containing DNA sequence reads. Output is written in SAM (Sequence Alignment/Map) format, which is plain text and can be streamed directly to stdout for downstream piping.
- **Reference indexing**: Requires a FM-index built with `bwa-mem2-index` before alignment. The index consists of multiple binary files stored alongside the reference FASTA. Indexing is a one-time cost per reference build and enables sub-linear time search.
- **Algorithm behavior**: Uses maximal exact matches (MEM) as seeds, then extendsAlignment with a banded Smith-Waterman algorithm. It automatically detects circular versus linear chromosomes and handles split-read alignment. Paired-end reads are processed together to resolve insert sizes and discordant alignments.
- **Scoring and filtering**: Default scoring parameters (match=0, mismatch=-4, gap-open=-6, gap-extension=-1) favor longer alignments with fewer gaps. The `-T` threshold discards alignments scoring below the specified value, useful for filtering low-quality mappings.
- **Threading model**: Uses POSIX threads with the `-t` option, where thread count directly impacts memory usage (approximately 1GB per thread for large genomes). I/O-bound operations scale well with additional threads.

## Pitfalls

- **Missing or corrupted index**: Running alignment without first building the index produces cryptic errors or silent failures with zero alignments. Always verify that `reference.fa.*.bwt.2bit.32` index files exist adjacent to the reference genome before running alignment.
- **Mismatched reference versions**: Reusing an index built for a different reference version or assembly leads to statistically valid but biologically meaningless alignments. Record the MD5 checksum of the reference genome alongside the index for reproducibility.
- **Ignoring read group information**: Omitting `-R` for read group specification causes optical duplicate detection in GATK to fail, as duplicate detection requires read group SM/ID fields. Downstream variant callers may reject or misattribute reads.
- **Insufficient threading for large datasets**: Setting `-t 1` on large FASTQ files causes extremely slow processing. Most modern systems benefit from `-t 8` to `-t 16` without exceeding typical memory constraints.
- **Misinterpreting alignment flags**: SAM format bitwise flags (e.g., 0x2 for properly paired) are frequently misinterpreted. The flag 16 indicates reverse-complemented alignment, not a separate strand concept. Always verify flags using `samtools view -c` with tag expressions rather than manual parsing.

## Examples

### Build FM-index for a reference genome
**Args:** `bwa-mem2-index reference.fa`
**Explanation:** Constructs the FM-index from the FASTA reference, producing binary index files required by the aligner. This is a prerequisite step that must complete successfully before any alignment operations.

### Align single-end reads to a reference
**Args:** `-t 12 -R @SQ SN:GRCh38 LN:3084197674 reference.fa reads_1.fastq.gz`
**Explanation:** Maps single-end reads with 12 threads, passing the read group header to include SM/ID metadata in all output alignments and enabling downstream GATK compatibility.

### Align paired-end reads with minimum alignment score threshold
**Args:** `-t 16 -T 30 -p reference.fa R1.fastq.gz R2.fastq.gz`
**Explanation:** Processes paired reads as an interleaved fragment library (`-p` indicates mates are adjacent in the same file), using 16 threads and discarding alignments scoring below 30 to reduce false-positive mappings.

### Align and save output to a specific file
**Args:** `-t 8 -o 2 aligned.sam reference.fa reads.fastq`
**Explanation:** Writes alignments to `aligned.sam` while suppressing the default verbose logging, allowing direct file capture without shell redirection and preserving the alignment record ordering.

### Align and pipe output to samtools for filtering
**Args:** `-t 8 reference.fa large_R1.fq.gz large_R2.fq.gz | samtools sort -@ 4 -o sorted.bam`
**Explanation:** Streams SAM output directly to samtools for sorting and BAM conversion, avoiding intermediate file creation and reducing disk usage for large datasets.

### Align with custom scoring parameters for RAD-seq data
**Args:** `-t 8 -A 2 -B 5 -O 8 -E 2 reference.fa rad_R1.fq rad_R2.fq`
**Explanation:** Increases mismatch penalty (`-B 5`) and gap open penalty (`-O 8`) to discourage indels common in RAD-seq reduced representation libraries, producing cleaner alignments for downstream population analysis.

### Index alignment output with samtools
**Args:** `bwa-mem2-align reference.fa input.fq | samtools index - sorted.bam`
**Explanation:** Chains alignment output through samtools indexing in a single pipeline, producing a coordinate-sorted BAM with associated index file for efficient random access in IGV or downstream tools like GATK.

### Align and calculate alignment statistics
**Args:** `bwa-mem2-align -t 4 reference.fa reads.fq 2>&1 | grep -E "^\\(|Mapped|Qu"`
**Explanation:** Captures stderr statistics from the verbose output to assess mapping rate and quality distribution without requiring samtools flagstat, useful for quick QC checks during processing.