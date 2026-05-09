---
name: bacprune
category: sequence_filtering
description: A sequence pruning tool for removing redundant, low-quality, or chimeric contigs from BAC assemblies to produce cleaner, more reliable assembly datasets.
tags:
  - bac
  - sequence-filtering
  - assembly-cleanup
  - genomics
  - contig-pruning
author: AI-generated
source_url: https://github.com/bacsuite/bacprune
---

## Concepts

- **BAC Sequence Data Model**: bacprune operates on FASTA or Multi-FASTA input files containing BAC assembly contigs. Each sequence is evaluated individually against quality thresholds for minimum length, base composition, and redundancy metrics.

- **Redundancy Detection via Alignment**: The tool performs all-versus-all pairwise alignment to identify overlapping or nearly identical contigs. Sequences exceeding the identity threshold (default 95%) with a longer partner are flagged for removal, preserving the longest representative.

- **Quality Scoring Metrics**: Contigs are scored using Phred-scaled quality scores, GC content deviation from expected range (default 30-70%), and presence of ambiguous base calls (N characters). Sequences falling below configurable thresholds are automatically pruned from the output dataset.

- **Output Modes**: bacprune produces two output files: a cleaned FASTA containing retained sequences and a pruning report (TSV) documenting which sequences were removed and the specific reason for each removal (redundancy, low-quality, or chimeric).

## Pitfalls

- **Ignoring the Minimum Length Threshold**: Setting `--min-length` too low can allow truncated or incomplete contigs to persist in the output, degrading downstream analysis quality. Conversely, setting it too high may remove legitimate short but high-quality sequences.

- **Assuming Input Quality Without Validation**: Processing contaminated or mixed-species input without the `--species-check` flag can result in cross-contamination artifacts being retained, as the tool cannot distinguish contaminant sequences from target species without explicit reference.

- **Overlooking Strand Orientation**: By default, bacprune treats reverse-complemented sequences as separate entries. If your pipeline produces both orientations of the same sequence, they may be incorrectly flagged as redundant pairs rather than being canonicalized first.

- **Insufficient Memory for Large Datasets**: The all-versus-all alignment step scales quadratically with input size. For assemblies with thousands of contigs, failing to allocate sufficient RAM (minimum 8GB recommended) will cause the tool to crash or swap excessively.

## Examples

### Remove redundant contigs from a BAC assembly FASTA file
**Args:** `input-assembly.fasta --output cleaned.fasta`
**Explanation:** Basic pruning removes contigs that share >95% sequence identity with a longer contig, preserving only the longest variant.

### Filter out short and low-quality contigs with custom thresholds
**Args:** `input.fasta --min-length 500 --min-quality 20 --gc-range 25-75 --output high-quality.fasta`
**Explanation:** This combination of thresholds removes contigs shorter than 500bp, with Phred quality below 20, or with GC content outside the 25-75% range.

### Generate both cleaned sequences and detailed pruning report
**Args:** `assembly.fasta --output cleaned.fasta --report pruning-report.tsv`
**Explanation:** The TSV report file documents every removed sequence with its removal reason code, enabling full auditability of the filtering process.

### Process multiple input files in batch mode
**Args:** `*.fasta --batch --output-dir pruned_output/`
**Explanation:** Batch mode processes all matching FASTA files independently, writing cleaned outputs and reports to the specified directory with preserved file naming.

### Enable species cross-contamination detection with reference genome
**Args:** `sample.fasta --species-check --reference RefGenome.fasta --output clean.fasta`
**Explanation:** The species-check flag aligns input contigs against the reference genome, flagging and removing sequences that do not match the expected species, preventing cross-contamination.