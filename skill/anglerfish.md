---
name: anglerfish
category: Long-read sequencing / Nanopore analysis
description: Anglerfish is a bioinformatics tool for processing Oxford Nanopore long-read sequencing data, providing basecalling, quality control, and read classification functionality. The companion binary anglerfish-build constructs index files from reference sequences for alignment-free read classification.
tags:
- nanopore
- long-reads
- basecalling
- sequencing
- fastq
- pod5
- read-classification
author: AI-generated
source_url: https://github.com/nanoporetech/anglerfish
---

## Concepts

- Anglerfish processes raw Nanopore signal data (POD5 format) into basecalled sequences (FASTQ), supporting both GPU-accelerated and CPU-only workflows depending on installation variant.
- The tool maintains read-level metadata including channel, start time, and sampling frequency, which are critical for downstream analyses like methylation detection and variant calling.
- anglerfish-build creates k-mer indices from reference sequences (FASTA) for fast classification; these indices use variable k-mer sizes optimized for the input read lengths.
- Output formats include raw FASTQ reads, summary statistics (JSON), and optional tag files indicating classification results against indexed references.
- The tool implements a streaming architecture allowing processing of multi-gigabyte POD5 files without loading entire datasets into memory.

## Pitfalls

- Using mismatched k-mer sizes between anglerfish-build and anglerfish classification causes zero reads to match, because indices are built with specific k values that must match during lookup.
- Processing corrupted POD5 files without error checking silently drops reads; always verify input integrity with pod5tools validate before running anglerfish.
- Running anglerfish on insufficient GPU memory leads to out-of-memory crashes during batching; reduce --batch-size instead of skipping GPU flags entirely for large datasets.
- Mixing POD5 files from different flowcells (e.g., R9.4 vs R10.4) without specifying --flowcell-type reduces basecalling accuracy because the tool applies flowcell-specific neural network models.
- Overwriting existing index files with anglerfish-build without --force produces no warning and replaces prior indices, potentially losing reference datasets.

## Examples

### Build an index from a bacterial reference genome for read classification
**Args:** build --reference genome.fa --output prefix --kmer-size 15
**Explanation:** This constructs a k-mer index using 15-mers from the provided FASTA file, enabling anglerfish to classify reads matching against this reference in downstream classification runs.

### Basecall a POD5 file with GPU acceleration
**Args:** call --input reads.pod5 --output called.fastq --device cuda:0
**Explanation:** This processes the raw signal data using the first CUDA GPU device, outputting basecalled FASTQ sequences to the specified file.

### Filter basecalled reads by minimum q-score threshold
**Args:** call --input reads.pod5 --output passed.fastq --min-qscore 12 --device cpu
**Explanation:** This runs basecalling on CPU and filters output to retain only reads with average quality scores of 12 or higher, discarding lower-quality reads.

### Classify reads against multiple reference indices
**Args:** classify --input called.fastq --index-dir /indices --output classifications.tsv --threads 8
**Explanation:** This classifies each read by matching against all k-mer indices in the specified directory, outputting a tab-separated results file with read IDs and matching references.

### Generate a summary report of basecalling statistics
**Args:** summary --fastq called.fastq --output report.json
**Explanation:** This computes read length distribution, q-score histograms, and throughput metrics, writing them in JSON format for downstream reporting or integration.

### Convert existing FASTQ to POD5-compatible format
**Args:** convert --input legacy.fastq --output converted.pod5 --sampling-rate 4000
**Explanation:** This transforms legacy FASTQ files into POD5 format with the specified sampling rate, allowing integration with anglerfish workflows requiring native signal files.

### Process multiple POD5 files in a directory batch
**Args:** batch --input-dir ./pod5_files --output-dir ./basecalled --pattern "*.pod5" --device cuda
**Explanation:** This iterates over all POD5 files matching the glob pattern, basecalling each with GPU acceleration and writing outputs to the specified directory.

### Append classification results to existing FASTQ headers
**Args:** tag --input reads.fastq --index-dir /indices --output tagged.fastq
**Explanation:** This appends classification metadata as inline tags within FASTQ sequence headers, preserving read data in a single file for simplified downstream processing.

### Adjust basecalling model sensitivity for high-accuracy reads
**Args:** call --input reads.pod5 --output called.fastq --model high_accuracy --device cpu
**Explanation:** This applies the high-accuracy basecalling model which prioritizes precision over throughput, suitable for applications requiring maximum per-base correctness.

### Run basecalling with automatic batch size optimization
**Args:** call --input large.pod5 --output called.fastq --device cuda:0 --auto-batch
**Explanation:** This enables automatic batch size determination based on available GPU memory, preventing out-of-memory errors while maximizing throughput across hardware configurations.