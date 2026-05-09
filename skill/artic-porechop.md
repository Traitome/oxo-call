---
name: artic-porechop
category: read-quality-control
description: A wrapper around pychopper for trimming Oxford Nanopore sequencing adapters and barcodes from FASTQ reads. It performs detection and removal of native barcodes, sequencing adapters, and hairpin adapters, and can discard or reclassify reads that fail to assemble correctly. Works on single or paired FASTQ inputs and outputs cleaned reads ready for downstream basecalling or demultiplexing.
tags: [nanopore, adapter-trimming, read-qc, barcoding, pychopper]
author: AI-Generated
source_url: https://github.com/artic-network/artic-pychopper
---

## Concepts

- artic-porechop detects and removes three classes of adapters from Oxford Nanopore reads: sequencing adapters (e.g., SQK-RAK001, SQK-LSK001), hairpin adapters (used in RPS flowcells), and native barcodes (NB01–NB48). It scans both read ends and discards the adapter portion, ensuring downstream tools receive clean sequences.
- Input is a single FASTQ file or two files for paired-end reads; output is one or two trimmed FASTQ files written to the specified directory. The `--input` flag names the input file (or first read of a pair), and `--output-dir` controls where the trimmed file is written.
- The `--kit-name` or `--sequencing-kit` flag selects a preconfigured adapter scheme; omitting it forces the detector to infer adapters from the data, which is slower and less accurate, especially for mixed-kit runs or barcode-heavy multiplexed samples.
- Reads that fail to have adapters detected on both ends are classified as "unassigned" and are typically dropped with `--discard-unassigned`. Keeping unassigned reads is useful only when the run mixes barcoded and non-barcoded samples on the same flowcell.
- Quality trimming and length filtering happen during the two-pass pipeline: the first pass counts all adapter detections to set a dynamic threshold, and the second pass applies trimming. The `--min-score` and `--min-length` flags override the default detector stringency and minimum read length.

## Pitfalls

- Not specifying `--kit-name` causes the detector to fall back to blind inference, which misses low-frequency adapters in multiplexed runs and results in residual adapters being written to the output file. Downstream consensus callers then interpret adapter sequence as genomic signal, producing false variants.
- Using `--discard-unassigned` on a non-barcoded, single-sample run discards all reads because no barcode adapters are present to classify them as "assigned," effectively emptying the output file. Always check whether your run used barcoding before enabling this flag.
- Running artic-porechop on basecalled reads that were already trimmed by another tool (e.g., dorado's built-in trimming) causes double-trimming: the tool removes the barcode stub that was intentionally retained, corrupting read ends and breaking downstream consensus assembly.
- Specifying `--min-length` lower than 100 bp retains very short fragments and primer-dimer products, inflatingating the read set size and causing Medaka or Megalodon to generate spurious low-coverage variants. Set this to at least 500 bp for viral genomes to match the ARTIC primer scheme.
- Using `--force` on a directory that already contains trimmed reads from a previous run causes silent overwriting with no warning. If the previous run used different parameters, the newly trimmed reads will have inconsistent quality and length profiles.

## Examples

### Basic trimming with default settings on a single FASTQ file
**Args:** `--input sample.fastq --output-dir trimmed/`
**Explanation:** Runs porechop with the default adapter inference mode, trimming all detected adapters and writing the output to `trimmed/`. This is appropriate for quick quality checks but may miss barcodes in multiplexed runs.

### Trimming with an explicit sequencing kit for an SQK-LSK109 flowcell
**Args:** `--input sample.fastq --output-dir trimmed/ --kit-name SQK-LSK109`
**Explanation:** Instructs porechop to use the preconfigured SQK-LSK109 adapter scheme for targeted detection, which is more sensitive and accurate than blind inference for standard ligation-based kits.

### Adapter detection only (no trimming) on a barcoded sample
**Args:** `--input sample.fastq --detect-only`
**Explanation:** Scans the file for adapters and barcode tags and prints a table of detections without writing any trimmed output. Useful for validating your kit configuration before committing to a full trim run.

### Trimming with read length filtering to remove short fragments
**Args:** `--input sample.fastq --output-dir trimmed/ --min-length 500`
**Explanation:** Discards any reads shorter than 500 bp during the second pass, removing primer dimers and very short fragments before the file is written. Recommended for viral ARTIC runs where primer amplicons are ~400 bp.

### Trimming with a custom minimum detection score for noisy data
**Args:** `--input sample.fastq --output-dir trimmed/ --min-score 80`
**Explanation:** Raises the detection threshold to 80 (default is lower), reducing false-positive adapter calls in data with high error rates or secondary structure. Use this when you observe over-trimming in poor-quality runs.

### Trimming with forced overwrite of a previous output directory
**Args:** `--input sample.fastq --output-dir trimmed/ --force`
**Explanation:** Clears any existing output files in `trimmed/` before starting, ensuring a clean run without mixing results from different parameter sets. Always verify the directory contents before using this flag.