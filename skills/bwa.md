---
name: bwa
category: alignment
description: Burrows-Wheeler Aligner for short reads against a reference genome
tags: [alignment, mapping, short-read, ngs, reference, illumina]
author: oxo-call built-in
source_url: "http://bio-bwa.sourceforge.net/bwa.shtml"
---

## Concepts

- bwa requires the reference genome to be indexed first with 'bwa index ref.fa' — this creates .amb/.ann/.bwt/.pac/.sa files.
- bwa mem is the primary algorithm for Illumina reads ≥70 bp; bwa aln/samse/sampe is for shorter reads.
- bwa mem outputs SAM to stdout — always pipe to 'samtools view -b' or redirect to a .sam file.
- For paired-end reads, pass both FASTQ files as two positional arguments after the index.
- Use -t N to specify the number of threads; -R '@RG\tID:sample\tSM:sample' to add a read group (required by GATK).
- The reference index base name is used, not the full .fa filename (e.g., 'ref' not 'ref.fa.amb').

## Pitfalls

- Running bwa mem without first indexing the reference will fail with 'fail to open index'.
- bwa mem output is SAM text to stdout — pipe to samtools view -b -o output.bam or add > output.sam.
- For GATK downstream analysis, always add a read group with -R '@RG\tID:id\tSM:sample\tLB:lib\tPL:ILLUMINA'.
- The reference argument is the index prefix (same as ref.fa if you ran 'bwa index ref.fa').
- bwa does not support gzipped references directly — decompress first.
- Memory usage scales with genome size; for human genome (~3 GB), expect ~6 GB RAM.

## Examples

### index a reference genome FASTA file
**Args:** `index reference.fa`
**Explanation:** creates .amb, .ann, .bwt, .pac, .sa index files alongside reference.fa

### align paired-end reads to a reference genome using 8 threads
**Args:** `mem -t 8 reference.fa R1.fastq.gz R2.fastq.gz`
**Explanation:** outputs SAM to stdout; pipe to samtools: bwa mem -t 8 ref.fa R1.fq.gz R2.fq.gz | samtools view -b -o out.bam

### align single-end reads and save as BAM with read group for GATK
**Args:** `mem -t 4 -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA' reference.fa reads.fastq.gz`
**Explanation:** read group (-R) is required by GATK; output is SAM to stdout, pipe to samtools

### align long reads (PacBio/Oxford Nanopore) to reference
**Args:** `mem -x ont2d reference.fa reads.fastq`
**Explanation:** -x ont2d preset for Oxford Nanopore; -x pacbio for PacBio; outputs SAM to stdout

### align paired-end reads and sort the output directly to a BAM file
**Args:** `mem -t 8 reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -@ 4 -o sorted.bam`
**Explanation:** pipe bwa mem output directly to samtools sort to avoid intermediate SAM file
