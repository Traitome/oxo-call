---
name: artic-tools
category: Bioinformatics / Viral Genomics / Nanopore Sequencing
description: Suite of command-line tools for processing ARTIC network nanopore sequencing data, including demultiplexing, collation, reference building, and variant summary generation for viral genomes such as SARS-CoV-2.
tags: ["nanopore", "artic", "viral-sequencing", "demultiplex", "fastq", "variant-calling", "sars-cov-2", "consensus"]
author: AI-generated
source_url: https://github.com/artic-network/artic-tools
---

## Concepts

- **ARTIC Pipeline Integration**: artic-tools is designed to work alongside the ARTIC nCoV-2019 bioinformatics pipeline; it handles sample demultiplexing before variant calling and collates results after Medaka consensus generation. Input FASTQ files typically come directly from Nanopore sequencing runs.
- **Barcode-Based Demultiplexing**: The `demux` subcommand assigns reads to samples using `--barcode-directories` containing individual barcode folders or a unified sample sheet with `--sample-sheet` mapping barcode names to sample IDs. Barcodes must be in format `barcodeXX` matching the sequencing summary.
- **Multi-Format I/O**: Tools accept FASTQ/FASTA for sequences, CSV/TSV for sample sheets, and produce VCF/CSV outputs for variants. The `--output-format` flag controls whether results are written as VCF or tabular formats.
- **Schema Versions for Primer Schemes**: Each viral protocol uses specific primer scheme versions (e.g., `V3`, `V4.1` for SARS-CoV-2). The `build` subcommand requires `--scheme-version` to generate the correct reference with proper primer coordinates; mismatched versions produce incorrect variant calls.

## Pitfalls

- **Specifying Wrong Barcode Directory Structure**: Providing `--barcode-directories` pointing to a parent folder rather than individual barcode subfolders causes demux to fail silently or assign all reads to a single sample. Always ensure each barcode folder (e.g., `barcode01/`, `barcode02/`) is a separate directory argument.
- **Omitting Required Sample Sheet Columns**: A sample sheet for demux must contain at least `sample_id` and `barcode` columns; missing columns cause the tool to abort with a cryptic error. Using tabs instead of commas in CSV format also breaks parsing.
- **Ignoring Output Directory Overwrites**: Running demux with `--output-dir` pointing to an existing directory overwrites files without prompting for confirmation, potentially losing previous demultiplexing results; always use a fresh directory or backup existing data.
- **Using Incorrect Scheme Version for Your Virus**: The `build` subcommand with SARS-CoV-2 scheme `V3` on samples run with `V4.1` primers produces false indels at primer binding sites. Verify your protocol version from the sequencing facility before building references.

## Examples

### Demultiplex FASTQ files using individual barcode directories
**Args:** `demux --barcode-directories barcode01 barcode02 barcode03 barcode04 --output-dir demux_out --sample-sheet samples.csv`
**Explanation:** This runs demux with four separate barcode folders as input; the sample sheet maps barcodes to sample IDs and all demultiplexed FASTQ files are written to the output directory.

### Collate variant calls across multiple sample directories
**Args:** `collate --directory CollateDir --output collated.csv --output-format csv`
**Explanation:** This aggregates variant calls from individual sample directories (each containing VCF/JSON from Medaka) into a single CSV file, making downstream comparative analysis easier.

### Build a reference with specific SARS-CoV-2 scheme version
**Args:** `build --scheme-directory /path/to/schemes --scheme-name SARS-CoV-2 --scheme-version V4.1 --output-dir ref_build`
**Explanation:** This generates a primer-aligned reference using the V4.1 scheme, required for accurate variant calling when your sequencing run used that protocol version.

### Summarise variant calls in tabular format
**Args:** `summarise --vcfs sample1.vcf sample2.vcf sample3.vcf --output summary.tsv`
**Explanation:** This converts per-sample VCF files into a unified TSV table with genomic positions, alleles, and quality scores for easier inspection and reporting.

### Demultiplex with explicit FASTQ input and specified output prefix
**Args:** `demux --fastq-dir runs/fastq_pass --barcode-directories barcodes/ --output-dir demux_results --prefix myrun`
**Explanation:** This explicitly reads all FASTQ files from a nanopore run directory, demultiplexes using barcodes in the provided directory, and prefixes output files with 'myrun' for traceability.