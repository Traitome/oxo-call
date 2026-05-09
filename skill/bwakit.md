---
name: bwakit
category: alignment
description: A collection of tools for next-generation sequencing alignment, variant calling, and HLA typing based on the Burrows-Wheeler Aligner. Includes bwa aligners, post-processing scripts for streaming, and specialized workflows for HLA typing and RNA-seq analysis.
tags: [ngs, alignment, variant-calling, hla-typing, dna-seq, rna-seq, short-reads]
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- **Three alignment algorithms**: BWA-backtrack (`bwa aln`) for reads ≤100bp, BWA-SW for medium reads up to 10kb, and BWA-MEM for reads 70bp-1Mbp with split read support. Choose the algorithm matching your read length.
- **Input formats**: Accepts FASTQ (single/paired-end), FASTA, and SAM (for subsequent alignment) as input. Output can be SAM, BAM (with `-b`), or CRAM (with `-C`). Reads can be streamed via stdin for large files.
- **Index construction**: The FM-index is built with `bwa index` using the BWA-SW algorithm by default, producing files with `.bwt`, `.pac`, `.ann`, `.amb`, `.sa` extensions. The index must match the reference exactly.
- **HLA typing workflow**: `run-HLA` performs specific HLA allele typing from FASTQ files using BWA-MEM with curated HLA allele libraries, outputting typed HLA alleles and coverage statistics.
- **BAM post-processing**: Stream alignment output directly to `samtools sort` and `samtools index` in a pipeline to avoid intermediate SAM files, which significantly reduces disk I/O for large datasets.

## Pitfalls

- **Mismatched algorithm for read length**: Using BWA-backtrack (`bwa aln`) for long reads (>100bp) produces many unmapped reads because the algorithm was designed for short reads. This leads to lost data and incomplete variant detection.
- **Missing read group information**: Not adding `@RG` tags via `-R` causes problems for downstream tools that expect read group metadata (e.g., GATK requires this for duplicate marking and base quality recalibration).
- **Unsorted output for variant calling**: Passing unsorted SAM/BAM directly to variant callers leads to coordinate-sorted requirements violations, causing errors or incorrect variant calls. Always sort before variant calling.
- **Forgetting to index after sorting**: Variant callers and visualizers require `.bai` index files. Forgetting to run `samtools index` on sorted BAM files breaks downstream analysis tools.
- **Insufficient memory for large references**: The default index parameters may cause memory allocation failures for large genomes (>4 billion bases). Always specify `-a bwtsw` for large genomes to use the BWT-SW algorithm.

## Examples

### Build a FM-index for a reference genome
**Args:** `index -a bwtsw reference.fa`
**Explanation:** The `-a bwtsw` flag explicitly selects the BWT-SW algorithm for index construction, which is required for reference genomes larger than 2 billion bases to avoid memory allocation failures.

### Align single-end short reads using BWA-MEM
**Args:** `mem reference.fa reads.fq > output.sam`
**Explanation:** BWA-MEM is the recommended algorithm for modern short-read data (≥70bp) as it handles longer reads, supports split alignments, and produces better results than older algorithms for most use cases.

### Align paired-end reads with read group information
**Args:** `mem -R '@RG\tID:sample1\tSM:sample1\tPL:ILLUMINA' reference.fa read1.fq read2.fq > output.sam`
**Explanation:** The `-R` flag adds read group metadata required by downstream tools like GATK for proper sample identification, duplicate marking, and base quality recalibration.

### Convert alignment to sorted BAM directly in a pipeline
**Args:** `mem reference.fa reads.fq | samtools sort -o sorted.bam -`
**Explanation:** Streaming output directly to `samtools sort` avoids writing intermediate SAM files, significantly reducing disk usage and I/O time for large sequencing datasets.

### Index a sorted BAM file for downstream tools
**Args:** `index sorted.bam`
**Explanation:** Creating the `.bai` index file is required for all tools that access reads by genomic coordinate, including IGV, samtools view, and GATK variant callers. Without it, tools will fail or refuse to process the file.

### Run HLA typing from FASTQ files
**Args:** `run-HLA -t 8 reference.fa sample1_R1.fq.gz sample1_R2.fq.gz`
**Explanation:** The `run-HLA` script uses BWA-MEM with a curated HLA allele database to type HLA-A, -B, -C, -DRB1, and -DQB1 alleles, outputting typed alleles and coverage per allele for clinical or research HLA studies.

### Align RNA-seq reads with junction detection
**Args:** `run-RNA reference.fa read1.fq read2.fq > rna_align.sam`
**Explanation:** The `run-RNA` wrapper runs BWA-MEM with specific parameters for spliced alignment, enabling detection of transcript junctions and handling of introns in a reference-based RNA-seq alignment workflow.