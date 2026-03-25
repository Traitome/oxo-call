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
- Use -t N to specify the number of threads; -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA' to add a read group (required by GATK).
- The -R read group string must preserve all field values exactly as specified — do not simplify or shorten sample names, IDs, or library tags.

## Pitfalls

- Running bwa mem without first indexing the reference will fail with 'fail to open index'.
- bwa mem output is SAM text to stdout — pipe to samtools view -b -o output.bam or add > output.sam.
- For GATK downstream analysis, always add a read group with -R '@RG\tID:sample1\tSM:sample1\tLB:lib1\tPL:ILLUMINA'. The exact sample/library names in the RG must match the task description — never simplify 'sample1' to 'sample' or 'lib1' to 'lib'.
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
**Explanation:** read group (-R) is required by GATK; exact RG field values (ID, SM, LB) must match the sample; output is SAM to stdout

### align long reads (PacBio/Oxford Nanopore) to reference
**Args:** `mem -x ont2d reference.fa reads.fastq`
**Explanation:** -x ont2d preset for Oxford Nanopore; -x pacbio for PacBio; outputs SAM to stdout

### align paired-end reads and sort the output directly to a BAM file
**Args:** `mem -t 8 reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -@ 4 -o sorted.bam`
**Explanation:** pipe bwa mem output directly to samtools sort to avoid intermediate SAM file

### align paired-end reads with complete read group for GATK HaplotypeCaller
**Args:** `mem -t 8 -R '@RG\tID:run1\tSM:patient1\tLB:lib1\tPL:ILLUMINA\tPU:unit1' reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o sample1.bam`
**Explanation:** -R adds a full read group with ID, SM, LB, PL, PU fields; preserve all exact values including sample and library IDs

### align paired-end reads and report only mapped reads
**Args:** `mem -t 8 reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -F 4 -o mapped.bam`
**Explanation:** -F 4 in samtools view excludes unmapped reads (flag 4); useful to reduce file size in the output BAM

### align with specific gap extension and mismatch penalties
**Args:** `mem -t 4 -B 4 -O 6 -E 1 reference.fa reads.fastq.gz > aligned.sam`
**Explanation:** -B 4 mismatch penalty; -O 6 gap open penalty; -E 1 gap extension penalty; tuned for specific read types

### align paired-end reads in a pipeline saving both BAM and stats
**Args:** `mem -t 8 -R '@RG\tID:sample2\tSM:sample2\tLB:lib2\tPL:ILLUMINA' reference.fa R1.fastq.gz R2.fastq.gz | samtools sort -@ 4 -o sample2_sorted.bam && samtools index sample2_sorted.bam`
**Explanation:** full pipeline: align with read group → sort → index; preserving exact sample2 and lib2 identifiers in the RG

### align with soft-clipping allowed for structural variant discovery
**Args:** `mem -t 8 -Y reference.fa R1.fastq.gz R2.fastq.gz | samtools view -b -o sv_aligned.bam`
**Explanation:** -Y enables soft-clipping of supplementary alignments; recommended for SV callers like LUMPY or Manta
