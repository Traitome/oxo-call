---
name: ccfind
category: Sequencing Analysis
description: A bioinformatics tool for finding consensus sequences and clusters from read datasets. Operates on FASTA/FASTQ inputs to identify sequences meeting similarity thresholds, supporting single-file and paired read modes.
tags:
  - consensus
  - clustering
  - sequence-analysis
  - ngs
  - similarity-search
author: AI-Generated
source_https://github.com/ccfind/ccfind
---

## Concepts

- **Sequence input formats**: ccfind accepts FASTA and FASTQ files as primary input. FASTQ files preserve quality scores for threshold filtering, while FASTA files are treated as equal-quality sequences.
- **Clustering model**: Sequences are grouped by pairwise similarity using edit distance. A `--threshold` value between 0.0 and 1.0 specifies the minimum fraction of matching positions for cluster membership.
- **Output modes**: The tool produces a consensus sequence per cluster using majority-vote base calling from contributing reads. Output writes to `--outdir` when specified, otherwise streams to stdout.
- **Paired-end support**: When two FASTQ files are provided, ccfind clusters read pairs jointly. A `--insert-size` flag sets the expected inner distance for mate-pair validation.

## Pitfalls

- **Specifying a threshold of 1.0**: A `--threshold` of 1.0 requires perfect matches only, which effectively disables clustering for most real read datasets containing sequencing errors. Results will list singletons as individual clusters.
- **Omitting output directory for large runs**: Writing consensus sequences to stdout without `--outdir` causes interleaved output that is difficult to parse downstream. Always specify a directory for datasets exceeding 10,000 reads.
- **Using unpaired mode on mate-pair data**: Passing `--mate1` without `--mate2` processes only the first read file, silently discarding the second file entirely. The `--paired` flag must be provided when mate-pair clustering is intended.
- **Ignoring minimum cluster size**: By default, clusters with fewer than `--min-size` reads are discarded. Setting `--min-size` too high can eliminate valid low-abundance clusters, particularly in amplicon sequencing or single-cell applications.

## Examples

### Find consensus sequences with default settings

**Args:** `input reads.fq`

**Explanation:** Runs ccfind on a single FASTQ file using default threshold (0.9) and minimum cluster size (2), outputting consensus sequences for each discovered cluster to stdout.

---

### Adjust similarity threshold for loose clustering

**Args:** `--threshold 0.7 input reads.fq`

**Explanation:** Sets the similarity threshold to 0.7, grouping sequences sharing at least 70% positional identity. Useful for highly variable regions where perfect conservation is unrealistic.

---

### Save results to a named output directory

**Args:** `--outdir ./ccfind_results input reads.fq`

**Explanation:** Creates the specified output directory and writes consensus FASTA files, one per cluster, with cluster membership statistics written to a summary TSV file.

---

### Cluster paired-end reads with insert size validation

**Args:** `--paired --mate1 R1.fq --mate2 R2.fq --insert-size 250 --outdir paired_results`

**Explanation:** Enables paired-end mode, validating insert sizes up to 250 bp and clustering read pairs jointly. Consensus sequences include bases from both ends when possible.

---

### Filter clusters by minimum read count

**Args:** `--min-size 10 input reads.fq --outdir robust_clusters`

**Explanation:** Discards any cluster containing fewer than 10 reads, retaining only well-supported consensus sequences. Appropriate for removing rare PCR artifacts or low-coverage variants.