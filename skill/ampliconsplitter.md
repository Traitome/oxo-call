---
name: ampliconsplitter
category: Bioinformatics - Sequence Processing
description: Splits amplicon sequencing data (typically paired-end Illumina reads) into individual amplicon files based on index sequences, barcodes, or sample mapping files. Commonly used for marker gene studies, multi-plexed amplicon panels, and dual-indexed library preparation separation.
tags: [amplicon, demultiplexing, fastq, illumina, sequence-splitting, barcoding]
author: AI-generated
source_url: https://github.com/ncbi/ampliconsplitter
---

## Concepts

- **Input Format**: Accepts paired-end FASTQ files (R1/R2) and a sample mapping/index file (TSV or CSV format) that maps sample IDs to index sequences (dual-indexed or single-index barcodes). The mapping file must contain columns for sample ID, forward index (i7), and reverse index (i5) when using dual-indexed library prep.
- **Output Generation**: Produces separate FASTQ file pairs for each amplicon/sample, with reads sorted by their assigned amplicon. Unassigned reads (due to no match or ambiguous assignment) are written to a designated "unknown" or "unassigned" output file.
- **Assignment Logic**: Uses exact matching by default (reads matching index sequences exactly are assigned to corresponding amplicons). Supports configurable maximum barcode mismatch thresholds (--mismatches) to handle low-quality index reads or rare sequencing errors.
- **File Naming Convention**: Output files follow a consistent naming pattern: `{sample_id}_R1.fastq.gz` and `{sample_id}_R2.fastq.gz`. The output directory must exist or be created; the tool does not auto-create directories unless --out-dir is specified.

## Pitfalls

- **Using an Index File with Wrong Read Architecture**: If the mapping file specifies single-index barcodes but the library was prepared with dual indexes (or vice versa), most or all reads will be unassigned, resulting in empty output files. Always verify the library prep method matches the index file format.
- **Setting Maximum Mismatches Too High**: Allowing >1-2 mismatches per index can cause reads to be incorrectly assigned to the wrong amplicon due to sequencing errors in index reads, introducing cross-contamination between samples. For high-quality sequencing runs, keep mismatches at 0-1.
- **Forgetting to Create the Output Directory**: If the specified output directory does not exist, the tool may fail silently or write to an unexpected location. Always create the output directory beforehand or use the --create-dir flag if available.
- **Mismatched Read Ordering in Input Files**: Providing R1 and R2 files that are not properly interleaved or synced (e.g., different read counts or misaligned read names) will produce corrupted output files with mismatched read pairs, compromising downstream analysis.

## Examples

### Split paired-end amplicon data using a sample mapping file
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping sample_mapping.tsv --out-dir split_amplicons
**Explanation:** This splits the paired-end FASTQ files into separate amplicon files based on the index-to-sample mapping defined in the TSV file, writing results to the specified output directory.

### Split data allowing 1 mismatch in index sequences
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping sample_mapping.tsv --out-dir split_amplicons --mismatches 1
**Explanation:** This assigns reads to amplicons even if the index sequence differs by up to 1 base, useful for handling low-quality index reads without excluding too many reads.

### Split data and keep unassigned reads in a separate file
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping sample_mapping.tsv --out-dir split_amplicons --write-unassigned
**Explanation:** This ensures reads that cannot be assigned to any amplicon (due to missing or ambiguous indices) are written to an "unassigned" FASTQ file for manual inspection.

### Split using dual-index barcodes from a CSV mapping file
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping dual_index_samples.csv --out-dir split_amplicons --index-format csv
**Explanation:** This uses a CSV file containing dual-index (i7 and i5) barcode mappings to assign paired-end reads to their correct amplicons using dual-barcode library prep.

### Split with compressed output files
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping sample_mapping.tsv --out-dir split_amplicons --compress
**Explanation:** This writes output FASTQ files in compressed gzip format (.fastq.gz) to reduce disk space usage, which is the default for most modern sequencing workflows.

### Split data with verbose logging for debugging
**Args:** --r1 Sample_R1.fastq.gz --r2 Sample_R2.fastq.gz --mapping sample_mapping.tsv --out-dir split_amplicons --verbose
**Explanation:** This enables detailed logging of the splitting process, including read counts per amplicon and any issues encountered, useful for troubleshooting or QC reporting.