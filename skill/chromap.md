---
name: chromap
category: epigenomics
description: A tool for analyzing chromatin interaction sequences from HiChIP, ChiA-PET, PLAC-seq and other chromatin conformation capture assays. Maps paired-end reads to a reference genome and extracts chromatin interaction pairs.
tags: chromatin, epigenomics, hi-chip, chromatin-interactions, epitranscriptomics, loop-calling
author: AI-generated
source_url: https://github.com/maiziezhouhlab/chromap
---

## Concepts

- **Paired-end read pairing**: Chromap processes paired-end reads where each read pair represents a potential chromatin interaction. The tool aligns both ends independently and pairs reads based on alignment positions to identify interaction endpoints.
- **Index-based mapping**: Chromap requires a pre-built index of the reference genome created using the `chromap-build` companion binary. The index accelerates alignment and enables rapid read mapping.
- **Multiple input formats**: Chromap accepts input in FASTQ format (direct mapping) or SAM/BAM format (already-mapped reads). It supports both single-end and paired-end sequencing data.
- **Interaction output formats**: Chromap outputs interaction data in multiple formats including pairs (read-pair interactions), fragments (restriction fragment-level), and loops (peaks of interaction enrichment).
- **Barcode handling**: For multiplexed data, Chromap supports cell-level barcodes (e.g., 10x Genomics-style) for single-cell chromatin interaction analysis.

## Pitfalls

- **Forgetting to build an index**: Running chromap without a pre-built index will fail. Always generate the index first using `chromap-build` with your reference genome FASTA file.
- **Mismatched read pairing mode**: Using single-end mode when data is paired-end results in undercounted interactions. Explicitly specify `-2` for paired-end FASTQ files.
- **Inconsistent reference genome**: Using different genome builds for indexing and mapping leads to misaligned interactions. Ensure the same FASTA is used throughout your pipeline.
- **Missing SAM header information**: When providing pre-mapped SAM/BAM files, ensure the header contains read group information; otherwise barcode assignment may fail.
- **Output format confusion**: The default output is pairs format, which may be too granular for downstream peak-calling. Use `--output-fmt pairs` or `--output-fmt fragments` based on your analysis needs.

## Examples

### Mapping paired-end FASTQ reads to a reference genome
**Args:** -i genome.idx -1 read1.fq.gz -2 read2.fq.gz -o interactions.pairs
**Explanation:** Maps paired-end FASTQ files using a pre-built index and outputs interaction pairs in standard pairs format for downstream analysis.

### Building an index from a reference genome FASTA
**Args:** chromap-build -i reference.fasta -o genome.idx
**Explanation:** Creates a genomic index from the reference FASTA file for use in subsequent chromap mapping runs.

### Processing pre-mapped SAM files to extract interactions
**Args:** -i genome.idx --sam inputs.sam -o interactions.pairs
**Explanation:** Takes already-aligned SAM reads and extracts interaction pairs based on alignment coordinates without re-mapping.

### Using 10x Genomics barcode whitelists for single-cell data
**Args:** -i genome.idx -1 read1.fq.gz -2 read2.fq.gz -x barcode.txt -o interactions.pairs
**Explanation:** Enables barcode-aware mapping that preserves single-cell identity for multiplexed chromatin interaction data.

### Outputting fragment-level interactions instead of read pairs
**Args:** -i genome.idx -1 read1.fq.gz -2 read2.fq.gz -o fragments.tsv --output-fmt fragments
**Explanation:** Outputs fragment-level interactions aggregated by restriction enzyme fragment, suitable for peak-calling workflows.

### Specifying multiple threads for parallel processing
**Args:** -i genome.idx -1 read1.fq.gz -2 read2.fq.gz -o interactions.pairs -t 16
**Explanation:** Allocates 16 threads to parallelize read mapping, significantly speeding up processing of large datasets.

### Using annotation file to filter interactions by genomic features
**Args:** -i genome.idx -1 read1.fq.gz -2 read2.fq.gz -o interactions.pairs -a genes.gtf
**Explanation:** Includes genomic annotation to annotate interactions with nearby gene features, adding context to interaction data.