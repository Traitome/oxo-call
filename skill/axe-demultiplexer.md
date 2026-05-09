---
name: axe-demultiplexer
category: Sequence Analysis
description: Demultiplexes multiplexed FASTQ files by assigning reads to individual samples based on their index/barcode sequences. Reads are sorted into separate output files according to matching barcode sequences, enabling downstream sample-specific analysis.
tags: [demultiplexing, barcoding, fastq, illumina, preprocessing]
author: AI-generated
source_url: https://github.com/koheron/axe
---

## Concepts

- **Index sequencing**: axe-demultiplexer reads index sequences from index FASTQ files (typically files with "_I1_", "_I2_" or "index" in the name) and matches them against a barcode file to assign each read to a sample.
- **Mismatch tolerance**: The tool allows a configurable number of mismatches when matching barcodes to indexes, which is essential for handling sequencing errors in index reads.
- **Reverse complement handling**: Many Illumina workflows use the reverse complement of the index sequence; axe-demultiplexer can optionally check both the forward index and its reverse complement.
- **Output organization**: Demultiplexed reads are written to separate FASTQ files named according to the sample IDs in the barcode file, preserving the original quality scores.

## Pitfalls

- **Mismatched barcode file format**: Using an incorrectly formatted barcode file (wrong delimiter, wrong column order) will cause all reads to be classified as "unknown" rather than assigned to samples.
- **Insufficient mismatch tolerance**: Setting `--max-mismatch` too low may cause valid reads with sequencing errors in their index to be discarded as unmatched, reducing yield.
- **Index file confusion**: Providing the wrong index file (e.g., using a read file instead of the index file) will result in complete demultiplexing failure or all reads going to unknown samples.
- **Paired-end orientation mismatch**: For paired-end data, failing to specify both read files in the correct order can lead to mismatched read pairs in the output.

## Examples

### Demultiplex a single-end multiplexed FASTQ file
**Args:** `--barcode-file barcodes.txt --output-dir demux_out input_R1.fastq`
**Explanation:** Assigns each read in the input FASTQ to a sample based on matching the index sequence (read from the file header or separate index file) against the barcode file.

### Demultiplex paired-end data with separate index files
**Args:** `--barcode-file barcodes.txt --index1 index_I1.fastq --output-dir demux_out input_R1.fastq input_R2.fastq`
**Explanation:** Uses the separate index FASTQ file to extract index sequences for demultiplexing while preserving both read pairs.

### Allow up to 1 mismatch when matching barcodes
**Args:** `--barcode-file barcodes.txt --max-mismatch 1 --output-dir demux_out input_R1.fastq`
**Explanation:** Enables matching of index sequences with one sequencing error, increasing read recovery at the cost of potential cross-contamination.

### Check reverse complement of index sequences
**Args:** `--barcode-file barcodes.txt --rev-comp --output-dir demux_out input_R1.fastq`
**Explanation:** Also checks the reverse complement of each index sequence when matching, which is required for some Illumina library preparations.

### Specify output prefix for unknown reads
**Args:** `--barcode-file barcodes.txt --unknown-prefix unknown --output-dir demux_out input_R1.fastq`
**Explanation:** Writes reads that do not match any barcode (or match multiple barcodes) to files prefixed with "unknown" instead of discarding them.