---
name: compare-reads
category: sequencing-quality
description: A BBMap tool for comparing read datasets and calculating similarity statistics, often used for quality control and contamination screening.
tags:
  - read-comparison
  - quality-control
  - similarity-analysis
  - bbmap
  - sequencing-validation
author: AI-generated
source_url: https://sourceforge.net/projects/bbmap/
---

## Concepts

- **Input Format**: Takes two input datasets—typically a reference set and a query set—or a reference genome with query reads. Supports FASTA, FASTQ, SAM, BAM, and FASTA-with-qualities formats as input, automatically detecting compression (gzip/bzip2) by file extension.

- **Output Statistics**: Generates multiple similarity metrics including aligned reads count, average alignment score, percent identity, coverage depth per position, and kmer-jaccard similarity. Results are printed to stdout as tab-separated values or optionally to specified output files.

- **Alignment Algorithm**: Uses a banded Smith-Waterman alignment with configurable kmer-seed size (default 23 for nucleotides) and mismatch tolerance. The algorithm identifies reciprocal best hits when comparing two read sets, enabling detection of cross-contamination or sample swapping.

- **Key Parameters**: The `k` flag controls kmer size for seeding (higher values increase specificity but reduce sensitivity), `m` sets maximum Hamming distance for alignments, and `t` specifies thread count for parallel processing. The `identity` threshold filters alignments by minimum percent identity.

## Pitfalls

- **Mismatched File Types**: Using the wrong input format specifier (e.g., expecting FASTQ for a FASTA file) causes silent failures or misleading statistics. Always verify file extensions match the actual data format before running comparisons.

- **Insufficient Kmer Size for Short Reads**: Setting kmer size larger than read length results in zero alignments. For reads shorter than 50bp, reduce the `-k` parameter to at least half the read length to allow seeding.

- **Ignoring Paired-End Designations**: When comparing paired-end read files, failing to provide both files in correct order or omitting the `paired` flag causes incorrect pairing logic and inflated duplicate statistics.

- **Thread Over-Allocation**: Specifying more threads (`-t`) than available CPU cores causes memory thrashing and reduces performance. Set threads to available cores minus one for optimal throughput on most systems.

- **Default Sensitivity for Divergent Sequences**: The tool optimizes for highly similar sequences by default. Comparing reads with evolutionary divergence greater than 15% without lowering the mismatch threshold (`-mm`) results in missed alignments and false negatives.

## Examples

### Comparing a query set against a reference genome
**Args:** `ref=ecoli_k12.fasta in=sample_reads.fastq k=25 m=2`
**Explanation:** Aligns sample_reads against the E. coli K12 genome using 25-mers for seeding and allowing up to 2 mismatches per kmer hit.

### Calculating kmer-jaccard similarity between two read sets
**Args:** `in1=control_sample_1.fastq.gz in2=test_sample_1.fastq.gz k=12 jni=t`
**Explanation:** Computes Jaccard similarity between kmer sets from both files using 12-mers, outputting the numeric similarity index.

### Detecting cross-sample contamination with stringent thresholds
**Args:** `ref=known_contaminants.fa in=sample_reads.sam.gz identity=0.95 m=1`
**Explanation:** Screens sample reads against a contaminant database with 95% identity threshold and maximum 1 mismatch to flag likely contamination.

### Parallel processing for large read datasets
**Args:** `in=large_dataset_R1.fastq.gz in2=large_dataset_R2.fastq.gz paired=t t=8`
**Explanation:** Processes paired-end reads across 8 threads, enabling faster alignment of large datasets on multi-core systems.

### Output alignment details to file
**Args:** `ref=target_transcriptome.fa in=rnaseq_reads.fastq out=alignments.sam outm=metrics.txt`
**Explanation:** Writes SAM-format alignments to alignments.sam and summary statistics to metrics.txt for downstream analysis.