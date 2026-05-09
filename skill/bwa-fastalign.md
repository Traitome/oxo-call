---
name: bwa-fastalign
category: sequence_alignment
description: Short-read and long-read aligner based on backward search with Burrows-Wheeler Transform (BWT). Performs fast, accurate gapped alignment of sequencing reads to a reference genome.
tags: [alignment, short-read, long-read, bwt, genomics, sam, bioinformatics]
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- **Input formats**: Accepts FASTA and FASTQ files (compressed with gzip or bzip2 supported via pipe). Single-end reads use `-q` flag to read from stdin; paired-end reads require two files via `-0` and `-1` for read 1 and read 2 respectively.
- **Output format**: Generates SAM (Sequence Alignment/Map) format by default. Use `-h` for HEADER, and the output can be streamed to stdout for piping into downstream tools like `samtools`.
- **Index requirement**: Requires a pre-built FM-index of the reference genome created with the companion `bwa index` or equivalent indexing tool. Index files use `.bwt`, `.pac`, `.sa`, `.ann`, `.amb`, and `.rbwt`/`.rpac`/`.rsa` extensions.
- **Algorithm**: Uses BWT-based backward search for seed extension, supporting both short reads (~100bp) and long reads (up to 1Mbp). Employs affine-gap penalty model for accurate indel detection.
- **Paired-end alignment**: Automatically computes insert size distribution when both reads are present. Uses proper pairing flags in SAM output and calculates fragment sizes for mate recovery.

## Pitfalls

- **Missing index causes immediate failure**: Running alignment without pre-built index produces the error "open index file [reference.fa] failed" and aborts. Always run `bwa index reference.fa` first.
- **Mismatched read names between pairs**: If read 1 and read 2 in paired-end mode have different IDs (e.g., "/1" suffix inconsistently applied), the aligner treats them as unpaired, losing proper pairing information in the SAM flags.
- **Excessive seed length causes slow performance**: Setting `-k` to very small values (like 0 or 1) with long reads dramatically increases runtime. Default `-k` of 19 is optimized; reducing below 10 only benefits highly divergent reads.
- **Ignoring read group tags**: Omitting `-R` with read group strings means downstream tools (e.g., GATK) cannot properly distinguish samples, potentially leading to incorrect pooled analysis or duplicate marking errors.
- **Uncompressed reference file causes memory issues**: Passing a reference file that is on-the-fly uncompressed may cause excessive memory usage on systems with limited RAM. Indexing first creates efficient binary representations.

## Examples

### Align single-end reads to a reference genome
**Args:** `aln -t 8 reference.fa reads.fq > alignment.sam`
**Explanation:** Runs single-end alignment with 8 threads, outputting SAM to alignment.sam. The `-t` flag controls parallelism for faster processing on multi-core systems.

### Build FM-index for reference genome
**Args:** `index -a bwtsw reference.fa`
**Explanation:** Creates the required FM-index using the bwtsw algorithm (recommended for reference sequences larger than 2GB). Index must complete before alignment can run.

### Align paired-end reads with proper pairing
**Args:** `mem -t 12 -R "@RG\tID:sample1\tSM:sample1\tPL:ILLUMINA" reference.fa read1.fq read2.fq > paired.sam`
**Explanation:** Aligns paired-end reads using 12 threads and includes read group metadata for downstream GATK pipelines, producing properly flagged paired alignments.

### Align long reads with reduced seed length for divergence
**Args:** `mem -k 15 -a reference.fa long_reads.fq > long_aln.sam`
**Explanation:** Reduces seed length to 15 to improve alignment of long reads with higher divergence from reference, at cost of slightly slower speed.

### Output aligned reads in SAM with header
**Args:** `mem -h reference.fa reads.fq | samtools view -bS - > aligned.bam`
**Explanation:** Generates SAM with headers and pipes directly into samtools to convert to BAM format for efficient storage and downstream processing.

### Suppress secondary alignments for faster processing
**Args:** `mem -F 256 reference.fa reads.fq > primary.sam`
**Explanation:** Filters out secondary alignments at output stage using SAM flag 256, useful for variant calling to reduce file size and processing.

### Map and sort BAM directly with embedded toolchain
**Args:** `mem -t 4 reference.fa reads.fq | samtools sort -@ 4 -o sorted.bam -`
**Explanation:** Streams alignment output directly into samtools sort without intermediate SAM file, reducing disk I/O and speeding up the pipeline on multi-core machines.