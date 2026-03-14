---
name: bowtie2
category: alignment
description: Fast and sensitive short read aligner for gapped alignment to reference genomes
tags: [alignment, mapping, short-read, ngs, illumina, bowtie2]
author: oxo-call built-in
source_url: "https://bowtie-bio.sourceforge.net/bowtie2/manual.shtml"
---

## Concepts

- bowtie2 requires a pre-built index from 'bowtie2-build ref.fa index_prefix'; the index is a set of .bt2 files.
- Use -x index_prefix (without the .bt2 extension), -U for single-end or -1/-2 for paired-end reads.
- bowtie2 outputs SAM to stdout by default — pipe to samtools or use -S output.sam.
- The --very-sensitive preset improves sensitivity at the cost of speed; --fast and --very-fast are faster but less sensitive.
- Use -p N for multi-threading; --no-unal suppresses unmapped reads in output.
- For bisulfite sequencing (BS-seq), use bismark instead of bowtie2 directly.

## Pitfalls

- bowtie2 outputs SAM to stdout — always pipe to 'samtools view -b -o output.bam' or use -S for SAM output.
- The -x argument takes the index prefix, not the .fa file or any .bt2 file.
- For paired-end reads, -1 and -2 must be used (not -U); using -U with paired files treats them as single-end.
- The --very-sensitive-local mode allows soft-clipping which changes the CIGAR string — verify downstream tools support this.
- bowtie2 alignment rate in the log is to stderr; always check it after alignment.

## Examples

### build a bowtie2 index from a reference FASTA
**Args:** `-build reference.fa reference_index`
**Explanation:** use bowtie2-build (not bowtie2) to build index; creates reference_index.*.bt2 files

### align paired-end reads to a reference genome using 8 threads
**Args:** `-x reference_index -1 R1.fastq.gz -2 R2.fastq.gz -p 8 | samtools view -b -o aligned.bam`
**Explanation:** -x is the index prefix; output is SAM piped to samtools view -b for BAM output

### align single-end reads with sensitive settings
**Args:** `-x reference_index -U reads.fastq.gz --very-sensitive -p 8 | samtools sort -o sorted.bam`
**Explanation:** --very-sensitive increases accuracy; output SAM piped directly to samtools sort

### align paired-end reads and save the alignment statistics
**Args:** `-x reference_index -1 R1.fq.gz -2 R2.fq.gz -p 8 --no-unal -S aligned.sam 2> align_stats.txt`
**Explanation:** --no-unal suppresses unmapped reads; 2> redirects alignment stats to a file
