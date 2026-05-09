---
name: babappalign
category: sequence_alignment
description: A fast short-read sequence alignment tool for mapping sequencing reads to a reference genome. Supports end-to-end and clipped alignment modes, generates SAM format output, and offers configurable seeding and scoring parameters for optimal sensitivity and speed.
tags:
  - alignment
  - short-reads
  - genomics
  - read-mapping
  - sam
author: AI-generated
source_url: https://github.com/bioinfo-tools/babappalign
---

## Concepts

- **Input Formats**: babappalign accepts FASTQ files (single-end or paired-end) as input and requires a pre-built index of the reference genome created with the companion `babappalign-build` binary. Reads can be provided via stdin or file arguments.
- **Output Format**: Alignment results are emitted in SAM (Sequence Alignment/Map) format by default, with optional BAM output when piped through external tools. Each alignment record contains the query name, flag, reference position, CIGAR string, mapping quality, and optional tags.
- **Indexing Model**: The reference genome must be indexed using `babappalign-build` before alignment. The index consists of multiple files with `.bin` extensions stored alongside the reference FASTA. Indexing is a one-time cost that enables O(1) lookup of genomic positions during alignment.
- **Alignment Algorithms**: babappalign supports three modes: exact-match seeding (fast but less sensitive), local alignment (allows soft clipping and indels), and global alignment (forces full-length alignment). Mode selection affects runtime and alignment accuracy for divergent reads.

## Pitfalls

- **Using an outdated index**: If the reference genome is modified (e.g., sequences added or changed) but the index is not rebuilt, alignments will be incorrect or fail silently. Always rebuild the index with `babappalign-build` after any change to the reference FASTA.
- **Specifying the wrong read orientation for paired-end data**: Setting `--mate-reverse` incorrectly or omitting it when reads are reverse-oriented will cause incorrect pairing and may lead to spurious alignments or total failure to find valid mates.
- **Excessive seed length causing missed alignments**: Setting `--seed-length` too high (e.g., > 20 for 75bp reads) makes the aligner less sensitive to reads with mismatches or mutations, potentially missing valid alignments for genetically variant strains.
- **Conflicting thread settings**: Using both `-t` and `--thread` simultaneously with different values can cause undefined behavior; the tool may silently use only one setting, leading to unexpected performance.

## Examples

### Align single-end FASTQ reads to an indexed reference
**Args:** -t 8 ref.genome.fasta reads.fq > output.sam
**Explanation:** This aligns single-end reads from reads.fq to the reference using 8 threads, outputting results in SAM format to stdout for redirection.

### Align paired-end FASTQ files with a specific read group
**Args:** -t 16 -r "@RG\tID:sample1\tSM:sample1\tPL:ILLUMINA" ref.fa left.fq right.fq > paired.sam
**Explanation:** This performs paired-end alignment with 16 threads and adds the read group metadata to the SAM header, identifying the sample for downstream GATK processing.

### Perform local alignment allowing soft clipping
**Args:** --mode local -t 4 ref.fa reads.fq > local_align.sam
**Explanation:** This uses local alignment mode which permits soft clipping at read ends, enabling detection of partial alignments and variants near read termini.

### Align with reduced seeding for higher sensitivity
**Args:** --seed-length 15 --max-seed-diff 2 -t 8 ref.fa reads.fq > sensitive.sam
**Explanation:** This reduces the seed length to 15bp and allows 2 mismatches in the seed, increasing sensitivity for reads with higher divergence at the cost of slower runtime.

### Output alignment statistics to a file instead of SAM
**Args:** --sam-no-output -f stats.txt -t 8 ref.fa reads.fq
**Explanation:** This runs alignment without producing SAM output but writes alignment statistics (hit rates, coverage) to stats.txt, useful for quality assessment before full alignment.

### Build an index for a large reference genome with bucket size optimization
**Args:** -a bwtsw --bucket-size 1000000000 ref.fa
**Explanation:** This invokes the companion `babappalign-build` binary to create an index using the bwtsw algorithm with a large bucket size for mammalian-scale genomes, improving index lookup speed.