---
name: baredsc
category: Bioinformatics / Single-Cell RNA-Seq Analysis
description: A tool for processing and analyzing single-cell RNA-seq data with barcoded cellular identifiers. Performs read alignment, barcode extraction, and digital expression matrix construction from BAR-seq or similar barcode-based single-cell protocols.
tags:
  - rnaseq
  - single-cell
  - barcode
  - expression-matrix
  - count-matrix
author: AI-generated
source_url: https://github.com/baredsc/baredsc
---

## Concepts

- **Input Format**: baredsc accepts raw FASTQ files (single-end or paired-end) containing reads with embedded cellular barcodes (cellular UMIs) in the read sequence or in a separate index file. The tool expects barcodes to be formatted as `BC:barcode_sequence` in the read header or as a separate fastq file with barcode sequences.
- **Output Matrix**: The tool produces a digital expression matrix (DGE) in TSV or MTX format, where rows represent genes/features and columns represent cells. The DGE includes raw UMI counts per gene per cell, along with quality metrics per cell (reads assigned, valid barcodes, etc.).
- **Barcode Whitelist**: For known barcode protocols (e.g., 10x Genomics-like), a whitelist file can be provided to filter valid cellular barcodes and discard invalid or empty barcodes, significantly improving cell calling accuracy.
- **Annotation**: baredsc requires a reference annotation (GTF/GFF) to map reads to genes. The tool uses featureCounts-like semantics for assigning reads to genes based on overlapping genomic features.

## Pitfalls

- **Missing GTF/GFF Annotation**: Running without a reference annotation file produces a sparse count matrix with genomic coordinates instead of gene symbols, making downstream analysis impossible. Always provide a proper GTF file via the annotation flag.
- **Mismatched Barcode Length**: If the whitelist barcode length doesn't match the actual barcode embedded in the reads, all valid barcodes will be filtered out, producing an empty expression matrix. Verify barcode length in your data matches the whitelist.
- **Not Specifying Protocol Type**: Different single-cell protocols (10x, inDrop, CEL-seq) use different barcode structures and UMI patterns. Failing to specify the protocol causes barcode parsing failures and produces zero counts for all cells.
- **Paired-End Misconfiguration**: Using single-end mode for paired-end data (or vice versa) results in incorrect read pairing and severely reduced mapping rates. Double-check the --layout flag matches your actual data.
- **Insufficient Memory for Large DGE**: Constructing expression matrices from large datasets (>100k cells) requires significant RAM. Out-of-memory errors during matrix construction are common; use chunked processing or increase available memory.

## Examples

### Align reads to reference and construct DGE from single-end FASTQ
**Args:** --layout single --read1 reads.fastq.gz --index genome_index --gtf annotation.gtf --outdir output/
**Explanation:** This runs baredsc in single-end mode, aligning reads from reads.fastq.gz to the pre-built genome index, counting overlaps with genes from the GTF annotation, and outputting the DGE matrix to the specified directory.

### Process paired-end data with explicit barcode and UMI files
**Args:** --layout paired --read1 R1.fastq.gz --read2 R2.fastq.gz --barcodefile barcodes.fastq.gz --index genome_index --gtf annotation.gtf --outdir output/
**Explanation:** Use this for paired-end protocols where the cellular barcode and UMI are in a separate fastq file. The tool pairs R1 (genomic read) with R2 (barcode/UMI) to construct the expression matrix.

### Apply barcode whitelist filtering for 10x-like data
**Args:** --layout single --read1 reads.fastq.gz --whitelist 10x_whitelist.txt --index genome_index --gtf annotation.gtf --outdir output/
**Explanation:** This filters reads to only those containing valid cellular barcodes from the whitelist file, dramatically improving cell calling accuracy for 10x Genomics-style protocols.

### Specify custom protocol with non-standard barcode position
**Args:** --layout single --read1 reads.fastq.gz --protocol indrop --barcode_start 0 --barcode_length 12 --umi_start 12 --umi_length 8 --index genome_index --gtf annotation.gtf --outdir output/
**Explanation:** Use this when analyzing inDrop or similar custom protocols where the barcode position in the read differs from standard protocols. Positions are zero-indexed.

### Output in MTX format for downstream tools like scanpy
**Args:** --layout single --read1 reads.fastq.gz --index genome_index --gtf annotation.gtf --outdir output/ --fmt mtx
**Explanation:** This outputs the digital expression matrix in Market Matrix (.mtx) format rather than the default TSV, which is directly compatible with scanpy and other Python-based single-cell analysis frameworks.

### Limit minimum reads per cell for quality filtering
**Args:** --layout single --read1 reads.fastq.gz --index genome_index --gtf annotation.gtf --outdir output/ --min_reads 100
**Explanation:** This filters out cells with fewer than 100 reads in the output DGE, removing low-quality droplets from the final expression matrix before downstream analysis.

### Enable multi-threaded processing for large datasets
**Args:** --layout single --read1 reads.fastq.gz --index genome_index --gtf annotation.gtf --outdir output/ --threads 8
**Explanation:** This uses 8 threads for parallel alignment and counting, significantly speeding up processing for large single-cell datasets with millions of reads.