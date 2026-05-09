---
name: barseqcount
category: sequence_analysis
description: A bioinformatics tool for counting barcoded sequencing reads, commonly used in CRISPR fitness screens, essentiality screens, and other high-throughput experiments with unique molecular barcodes. It maps sequencing reads to known barcodes and generates count tables for downstream analysis.
tags: [barcode-counting, crispr-screens, essentiality-sequencing, high-throughput, read-counting]
author: AI-generated
source_url: https://github.com/broadinstitute/barseqcount
---

## Concepts

- **Barcoded read counting**: BarSeqCount takes FASTQ/SAM/BAM input files and a mapping file that associates known barcodes (e.g., sgRNA guides, UMIs) with gene identifiers. It extracts the barcode sequence from each read and tallies occurrences per gene.

- **Input file formats**: The tool accepts single-end or paired-end FASTQ files as primary input. It requires a mapping file in tab-separated format with at least two columns: barcode sequence and corresponding gene name/identifier.

- **Output format**: BarSeqCount generates a count table (tab-separated text file) with two columns — gene identifier and raw read count. Optional columns include normalized counts, read counts per barcode, and mapping statistics.

- **Barcode extraction strategies**: Users can specify the barcode position using read position (e.g., `--read1-sep`), offset coordinates (e.g., `--start`, `--length`), or use aligner output for extraction. This flexibility supports inline barcodes, UMI schemes, and adapter-ligated barcodes.

- **Quality filtering**: The tool supports base quality score filtering (via Phred scores) and can discard reads with ambiguous barcodes or low-quality barcode calls, improving count accuracy in noisy datasets.

## Pitfalls

- **Using an incorrectly formatted mapping file**: If the mapping file lacks a proper header or uses an unsupported delimiter (e.g., spaces instead of tabs), BarSeqCount will fail to associate barcodes with genes, producing all-zero counts. Always verify the file uses tab separation and matches the expected column order.

- **Specifying the wrong barcode position**: Incorrect `--start`, `--length`, or separators will cause the tool to extract the wrong sequence region, leading to massive undercounting or zero counts. Confirm the barcode location in your sequencing data using a viewer like IGV or a quick FASTQ inspection.

- **Ignoring read orientation**: When reads are reverse-complemented relative to the reference, failing to use the `--rev-comp` flag will cause barcode mismatches. Always check whether your barcodes appear in the forward or reverse strand in the input data.

- **Mismatching read type (single vs paired-end)**: Specifying single-end mode for paired-end data (or vice versa) leads to incorrect read pairing and inflated or zero counts. Use `--paired` only when your data is actually paired-end.

- **Not accounting for barcode redundancy**: Multiple barcodes can map to the same gene in CRISPR libraries. Failing to use aggregate mode or handle duplicates will skew downstream enrichment analysis. Ensure the mapping file is properly configured for summation.

## Examples

### Basic barcode counting from a single-end FASTQ file

**Args:** `--map-file guides.txt --input sample.fq.gz --output counts.txt`
**Explanation:** This runs standard barcode counting where the tool reads the mapping file to associate barcodes with gene names and outputs total counts per gene.

### Counting with paired-end reads where barcode is on read 2

**Args:** `--paired --read1-sep file1_R1.fastq.gz --read2-sep file1_R2.fastq.gz --map-file guides.txt --output counts.txt`
**Explanation:** In paired-end mode, BarSeqCount uses read 2 to extract the barcode while read 1 provides the sequence context for quality filtering.

### Extracting a barcode at a specific position (reads with inline barcodes)

**Args:** `--input sample.fq.gz --map-file guides.txt --start 1 --length 20 --output counts.txt`
**Explanation:** This specifies that the barcode begins at position 1 of the read and spans 20 bases, commonly used when barcodes are inline at the start of sequencing adapters.

### Filtering reads by minimum quality score

**Args:** `--input sample.fq.gz --map-file guides.txt --min-quality 20 --output counts.txt`
**Explanation:** Reads with barcode regions having Phred quality scores below 20 are discarded, reducing false positives from sequencing errors.

### Using reverse complement mode for opposite-strand barcodes

**Args:** `--input sample.fq.gz --map-file guides.txt --rev-comp --output counts.txt`
**Explanation:** When barcodes are encoded on the reverse strand in the sequencing data, this flag reverse-complements the extracted sequence before matching.

### Counting with UMI deduplication enabled

**Args:** `--input sample.fq.gz --map-file guides.txt --umi --output counts.txt`
**Explanation:** This collapses reads with the same barcode and UMI to a single count, correcting for PCR amplification bias in molecular counting assays.

### Producing additional output with per-barcode counts

**Args:** `--input sample.fq.gz --map-file guides.txt --output counts.txt --per-barcode-output barcode_counts.txt`
**Explanation:** This generates two output files — the standard gene-level count table and a detailed file showing counts for each individual barcode.

### Limiting analysis to a whitelist of known barcodes

**Args:** `--input sample.fq.gz --map-file guides.txt --whitelist barcodes_whitelist.txt --output counts.txt`
**Explanation:** Only barcodes present in the whitelist are counted; all others are discarded, reducing noise from adapter dimers or unknown sequences.

---