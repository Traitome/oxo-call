---
name: bigmap
category: sequence-alignment
description: A fast and memory-efficient tool for mapping long reads from Oxford Nanopore and PacBio sequencers to reference genomes, producing SAM/BAM output for downstream bioinformatics analysis.
tags:
- long-read-mapping
- nanopore
- pacbio
- sequence-alignment
- bioinformatics
- variant-calling
author: AI-generated
source_url: https://github.com/bigmap-project/bigmap
---

## Concepts

- **Index-First Architecture**: bigmap requires a pre-built reference index using the companion tool `bigmap-build` before mapping can occur. The index file (`.bmi`) stores hash-based reference metadata that accelerates alignment search and must be generated separately from the mapping step.

- **Input Format Handling**: bigmap accepts raw long-read sequences in FASTQ format (with base质量和 read name) or FASTA format (sequences only). The tool automatically detects file compression (gzip/ plain) and supports streaming input from stdin using conventional Unix pipes.

- **SAM/BAM Output Production**: Mapped reads are output in SAM (text) or BAM (binary) format depending on the `-b` flag. SAM records contain CIGAR strings describing insertions, deletions, and soft-clipping events that are characteristic of long-read sequencing, enabling assessment of mapping quality per read.

- **Threading and Performance Tuning**: The `-t` flag controls the number of CPU threads allocated to the mapping process. Default behavior uses a single thread; increasing this value proportionally reduces wall-clock time for large read sets, but memory usage scales with thread count.

- **Sensitivity Modes**: bigmap offers three preset sensitivity levels (`-w fast`, `-w sensitive`, `-w highly-sensitive`) that control seed length and spacing during the hash-based hit detection phase. Longer seeds in "fast" mode increase speed but miss divergent reads, while "highly-sensitive" catches more variants at the cost of slower runtime.

## Pitfalls

- **Omitting Index Creation**: Attempting to map reads against a raw reference FASTA file without first running `bigmap-build` produces a fatal error message "Index file not found". This wastes time as the mapping job aborts before processing any reads.

- **Mismatched Read Technology Parameters**: Using Nanopore reads with PacBio-optimized sensitivity settings (or vice versa) leads to artificially low mapping rates because the default k-mer sizes do not align well with the distinct error profiles of each platform. Always verify that `-w` preset matches your sequencing technology.

- **Output File Overwrite Without Confirmation**: Specifying an existing SAM/BAM file path with `-o` silently overwrites the prior file without warning. Downstream pipelines that rely on incremental output accumulation will silently lose historical data.

- **Insufficient Memory for Thread Count**: Each thread allocates approximately 200 MB of working memory for hash tables and alignment buffers. Setting `-t 8` on a machine with less than 2 GB available RAM triggers out-of-memory kills that produce incomplete output files.

- **Ignoring Mapping Quality Filters**: The `-q` threshold flag filters reads by mapping quality score, but the default setting (`-q 0`) retains all reads including those mapping ambiguously. Downstream variant callers may produce false positives if low-quality alignments are not excluded.

## Examples

### Build a reference index for human genome GRCh38
**Args:** `build GRCh38.fa -o GRCh38.bmi`
**Explanation:** This companion command creates a binary index file from the reference FASTA, which bigmap reads during the mapping phase to locate candidate alignment seeds efficiently.

### Map Nanopore FASTQ reads to a reference with sensitive mode
**Args:** `-i GRCh38.bmi -q reads.fastq -w sensitive -t 4 -o mappings.sam`
**Explanation:** The `-w sensitive` flag enables a balanced seed configuration suitable for Nanopore data, while `-t 4` distributes work across four CPU threads for faster processing.

### Map PacBio reads with high specificity and output BAM format
**Args:** `-i GRCh38.bmi -q pacbio_ccs.fastq.gz -w highly-sensitive -b -o mappings.bam`
**Explanation:** The `-b` flag writes binary BAM output which is smaller and faster to process in downstream tools like HTSlib, while `-w highly-sensitive` maximizes recall for high-identity PacBio CCS reads.

### Map reads and filter low-quality alignments in a single step
**Args:** `-i GRCh38.bmi -q nanopore_reads.fastq -w sensitive -q 20 -o high_quality.sam`
**Explanation:** The second `-q 20` argument sets the minimum mapping quality threshold, discarding reads with more than approximately 1% expected errors and producing a cleaner alignment set for variant calling.

### Stream compressed reads directly without intermediate decompression
**Args:** `-i GRCh38.bmi -q