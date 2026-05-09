---
name: anospp-analysis
category: Bioinformatics Sequence Analysis
description: A bioinformatics tool for analyzing nanopore sequencing data with support for basecalling, alignment, and variant calling workflows. Processes raw signal data into actionable results with configurable quality thresholds.
tags:
  - nanopore
  - sequencing
  - signal-analysis
  - basecalling
  - variant-calling
  - long-reads
author: AI-Generated
source_url: https://github.com/anospp/anospp-analysis
---

## Concepts

- **Signal-based input format**: anospp-analysis accepts raw nanopore signal files (e.g., `.fast5`) as primary input, extracting electrical current measurements and mapping them to k-mer events during processing. The tool maintains signal-to-basecall traceability throughout the analysis pipeline.
- **Output formats and downstream compatibility**: Results are produced in standard bioinformatics formats including FASTQ (for basecalled reads), SAM/BAM (for alignments), and VCF (for variants). These outputs are compatible with standard visualization tools like IGV and downstream analysis packages.
- **Quality-aware processing thresholds**: The tool applies user-configurable Q-score thresholds to filter reads, with a default minimum of Q7 for passing reads. Filtering occurs at multiple stages: after basecalling, after alignment, and after variant calling, allowing granular control over the sensitivity-specificity trade-off.
- **Scalable multi-sample batch processing**: anospp-analysis supports directory-based batch processing where multiple samples can be analyzed in a single invocation by specifying an input directory. Sample separation is determined by directory structure, and results are written to sample-specific output subdirectories.

## Pitfalls

- **Insufficient RAM allocation causing crash**: Specifying `--threads` without proportionally increasing `--memory` leads to out-of-memory errors when processing large `.fast5` files. The tool requires approximately 2GB RAM per thread for optimal performance, and underallocation causes silent data loss when swap is exhausted.
- **Ignoring basecall model mismatch warnings**: When the input reads were generated with a different basecalling model than specified via `--model`, the tool produces low-quality FASTQ output without halting. This results in inflated indel error rates in downstream VCF files, causing false positive variant calls.
- **Output directory not empty causing file collision**: Running analysis on an existing output directory with `--output` does not prompt for confirmation and silently overwrites existing VCF and BAM files. Results from previous runs become unrecoverable, corrupting longitudinal study analyses.
- **Misconfiguring barcode demultiplexing**: Specifying `--barcode` with a single sample causes the tool to skip all unclassified reads instead of assigning them to the primary sample. Low-complexity libraries lose 10-30% of reads that were not recognized by the barcode caller.

## Examples

### Analyze a single FAST5 file with default settings
**Args:** `input/sample_001.fast5 --output results/`
**Explanation:** Processes the raw nanopore signal file using default basecalling model and quality thresholds, writing all results to the specified output directory with automatic subdirectory creation.

### Run batch analysis on multiple samples with 16 threads
**Args:** `input/ --batch --threads 16 --memory 32g --output results/`
**Explanation:** Processes all `.fast5` files in the input directory across 16 parallel threads with 32GB RAM, producing sample-separated result directories with consistent naming.

### Generate filtered FASTQ with Q-score threshold of 10
**Args:** `--input reads.fast5 --min-qscore 10 --output-fmt fastq --output filtered_reads.fq`
**Explanation:** Extracts basecalled reads from signal data, applies stringent Q-score filtering to retain only high-confidence bases, and outputs a single FASTQ file for downstream mapping.

### Perform alignment and variant calling with reference genome
**Args:** `--input sample.fast5 --ref GRCh38.fasta --align --variants --output variants.vcf`
**Explanation:** Aligns basecalled reads to the specified reference genome using integrated minimap2 backend and calls variants, producing a standard VCF file ready for annotation tools.

### Export signal-level data for custom basecalling analysis
**Args:** `--input sample.fast5 --export-signals events.tsv --event-columns time,current,kmer`
**Explanation:** Exports extracted event detection data as a tab-separated file with user-specified columns, enabling custom k-mer probability modeling outside the built-in basecaller.