---
name: clipandmerge
category: preprocessing
description: A tool for adapter clipping and overlapping paired-end read merging for Illumina sequencing data, commonly used in amplicon-based workflows like 16S rRNA gene sequencing.
tags: [preprocessing, adapters, read-merging, paired-end, illumina]
author: AI-generated
source_url: https://github.com/fo出资xx/clipandmerge
---

## Concepts

- **Input format**: clipandmerge expects paired-end FASTQ files (typically with `_R1_001.fastq.gz` and `_R2_001.fastq.gz` naming conventions) and requires that forward and reverse reads are in identical sort order. Files may be gzip-compressed.
- **Adapter clipping behavior**: The tool detects and removes Illumina sequencing adapters (Nextera, Truseq, or custom sequences) from read ends using overlap-based detection, not explicit adapter sequences. It only clips if an overlap between the read and adapter is found.
- **Read merging logic**: Forward and reverse reads are merged based on minimum overlap (default 10 bp) and maximum mismatch thresholds. Merged reads are written to the output; unmerged reads are retained as orphan pairs or discarded based on the `--max_ungapped` setting.
- **Output streams**: clipandmerge writes merged reads to STDOUT in FASTQ format, while statistics are printed to STDERR. Users must redirect appropriately to capture merged data separately from logs.
- **Quality control flags**: The `--min_merge_qual` flag sets a minimum base quality threshold for merged regions, and `--min_len` discards reads shorter than the specified length post-processing.

## Pitfalls

- **Mismatched read ordering**: If forward and reverse FASTQ files are not identically ordered (e.g., after independent quality filtering), merging will fail silently or produce incorrect read pairs, leading to corrupted downstream analysis. Always process both files together through the same pipeline.
- **Ignoring the `--max_ungapped` setting**: Setting `--max_ungapped` to 0 discards all unmerged reads without warning, which may cause significant data loss if overlap-based merging is insufficient for your library prep. Default preserves unmerged pairs for manual inspection.
- **Redirecting output incorrectly**: Redirecting both STDOUT and STDERR to the same file mixes FASTQ data with log statistics, rendering the FASTQ file unreadable by downstream tools. Always use `> merged.fastq 2> stats.log` or similar separate redirections.
- **Insufficient overlap for short amplicons**: For 16S amplicons where forward and reverse primers amplify the same region, reads may overlap entirely, causing the tool to misclassify them as adapter-contaminated. Adjusting `--min_merge_qual` and `--min_overlap` may be necessary.
- **Compressed output expectations**: By default, clipandmerge outputs uncompressed FASTQ. Downstream tools expecting `.gz` input will fail. Use external piping to `gzip` if compressed output is required.

## Examples

### Clipping and merging a single paired-end dataset with default parameters
**Args:** `-1 sample_R1.fastq.gz -2 sample_R2.fastq.gz`
**Explanation:** This runs clipandmerge with all default settings on paired FASTQ files, clipping adapters and merging overlapping reads based on built-in overlap detection and default thresholds.

### Merging reads with stricter overlap and quality thresholds for high-accuracy applications
**Args:** `-1 run1_R1.fastq.gz -2 run1_R2.fastq.gz --min_merge_qual 30 --min_overlap 20`
**Explanation:** Setting `--min_merge_qual 30` requires merged bases to have Phred quality of at least 30, and `--min_overlap 20` increases the minimum required overlap to 20 bp, reducing false merges.

### Saving merged output separately from logging statistics
**Args:** `-1 dataset_R1.fastq.gz -2 dataset_R2.fastq.gz > merged_reads.fastq 2> clipping_stats.log`
**Explanation:** Redirecting STDOUT to a FASTQ file and STDERR to a log file prevents mixing of merged read data with statistics, making the output suitable for downstream tools like Mothur or QIIME2.

### Discarding short unmerged reads to reduce noise in amplicon sequencing
**Args:** `-1 amplicon_R1.fastq.gz -2 amplicon_R2.fastq.gz --min_len 200 --max_ungapped 8`
**Explanation:** `--min_len 200` discards any merged or unmerged reads shorter than 200 bp, which removes primer dimers and short fragments common in 16S amplicon sequencing.

### Processing multiple files in a batch using GNU parallel for large projects
**Args:** `-1 sample1_R1.fastq.gz -2 sample1_R2.fastq.gz | gzip > sample1_merged.fastq.gz`
**Explanation:** Piping output directly through `gzip` compresses the merged FASTQ output on the fly, which is storage-efficient for large datasets and compatible with most downstream bioinformatics pipelines.

### Removing unmerged reads entirely to focus only on high-confidence merged pairs
**Args:** `-1 dataset_R1.fastq.gz -2 dataset_R2.fastq.gz --max_ungapped 0 --min_merge_qual 25`
**Explanation:** Setting `--max_ungapped 0` discards all reads that fail to merge, and `--min_merge_qual 25` ensures merged bases meet a minimum quality threshold, producing a high-confidence but smaller dataset.