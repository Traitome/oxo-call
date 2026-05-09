---
name: barcodeforge
category: Bioinformatics/Sequence Analysis
description: A tool for generating, designing, and analyzing molecular barcodes (UMIs and custom barcodes) for next-generation sequencing applications. Supports multiple barcode formats, quality control checks, and batch generation with configurable properties.
tags:
  - barcodes
  - UMI
  - sequencing
  - genomics
  - sequence-generation
  - molecular-barcodes
author: AI-generated
source_url: https://github.com/parseq-treeval/barcodeforge
---

## Concepts

- BarcodeForge supports generating random barcodes of specified length (e.g., 8-12 nucleotides) with configurable properties including GC content bounds, homopolymer repeat avoidance, and base composition constraints.
- Input formats include plain text (one sequence per line), FASTA, and CSV with metadata; output can be written to FASTA, CSV, or tab-delimited formats for downstream pipeline integration.
- The tool provides quality control subcommands to check for homopolymer runs, GC skew, self-complementarity, and sequence similarity against a user-provided reference database using exact matching.
- Barcodes can be generated with built-in sequence uniqueness guarantees — the tool removes duplicates from output sets and can enforce minimum Hamming distance thresholds between all generated sequences.
- Companion binaries include `barcodeforge-build` for creating reference databases from existing barcode sets, and `barcodeforge-validate` for checking FASTA/Qual files against design specifications.

## Pitfalls

- Specifying a barcode length outside the recommended range (typically 4-20 nucleotides) can produce barcodes that are too short for unique identification or too long for efficient sequencing library amplification.
- Failing to set a sufficiently high minimum Hamming distance (recommended: ≥3) results in barcode sequences that may be confused by sequencing error, compromising downstream demultiplexing accuracy.
- Using homopolymer-enforcing flags without checking for barcode overlap can generate sequences with consecutive identical bases that perform poorly in PCR and sequencing workflows.
- Outputting to an existing file without using overwrite flags causes the tool to fail silently or produce mixed files — always use `-f` or `--force` when overwriting is intended.
- Not validating generated barcodes against a reference database can lead to collisions with known adapter sequences or other project barcodes in multiplexed runs.

## Examples

### Generate 100 random barcodes of length 8 nucleotides

**Args:** `gen -n 100 -l 8 -o barcodes.fasta`
**Explanation:** This generates 100 random 8-nucleotide barcodes and writes them in FASTA format to `barcodes.fasta`, suitable for downstream UMI assignment in NGS pipelines.

### Generate barcodes with GC content between 40% and 60%

**Args:** `gen -n 500 -l 12 --min-gc 0.4 --max-gc 0.6 -o balanced_barcodes.csv`
**Explanation:** This creates 500 12-nucleotide barcodes with GC content constrained to the 40-60% range, helping ensure uniform PCR amplification across the barcode pool.

### Analyze existing barcodes for homopolymer runs

**Args:** `analyze -i barcodes.fasta --check-homopolymer --max-run 2`
**Explanation:** This reads the input barcode file and reports any sequences containing homopolymer runs longer than 2 consecutive identical bases, flagging problematic sequences for removal.

### Validate barcodes against a reference database

**Args:** `validate -i design.fasta -r existing_umis.fasta --distance 1`
**Explanation:** This checks if any sequences in `design.fasta` match within 1 Hamming distance of sequences in the reference, ensuring no unintended overlap with existing UMIs.

### Generate and enforce minimum Hamming distance between all barcodes

**Args:** `gen -n 1000 -l 10 --min-hdist 3 -o unique_barcodes.fasta`
**Explanation:** This generates 1000 10-nucleotide barcodes while enforcing that every pair differs by at least 3 positions, reducing demultiplexing errors due to sequencing noise.

### Build a reference database from existing barcode set

**Args:** `barcodeforge-build -i known_umis.fasta -o umi_db --format fasta`
**Explanation:** The companion binary `barcodeforge-build` creates an indexed reference database from an input FASTA file for fast validation queries in subsequent runs.

### Convert barcode output from FASTA to CSV format

**Args:** `convert -i barcodes.fasta -o barcodes.csv --csv-header`
**Explanation:** This converts a FASTA-formatted barcode file to CSV with a header row, useful for importing into spreadsheet software or integrating with sample sheet generators.