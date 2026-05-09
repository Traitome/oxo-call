---
name: amplify
category: Spatial Transcriptomics
description: A bioinformatics tool for processing spatial transcriptomics sequencing data, including read error correction, adapter trimming, and UMI-based read deduplication for ST-seq pipelines.
tags:
  - spatial-transcriptomics
  - ST-seq
  - umi
  - deduplication
  - error-correction
  - rna-seq
  - transcriptomics
author: AI-generated
source_url: https://github.com/空间-transcriptomics/amplify
---

## Concepts

- **Read Structure Parsing**: Amplify expects input FASTQ reads with a specific structure containing positional barcode (spatial coordinates), UMI sequence, and transcript read. The tool parses these segments based on user-specified lengths using the `--read-structure` argument.
- **UMI-based Deduplication**: The tool collapses PCR duplicates by comparing UMI sequences and genomic positions, keeping only unique molecule combinations with the highest quality score.
- **Error Correction in UMIs**: Amplify performs nearest-neighbor error correction on UMI sequences, allowing configurable edit distance thresholds (default: 1) to recover true unique molecules with sequencing errors.
- **Output Format**: The tool outputs corrected and collapsed reads in FASTQ format, plus a summary statistics file containing read counts, deduplication rates, and error correction events.

## Pitfalls

- **Mismatch in read structure specification**: Specifying incorrect read structure lengths (e.g., wrong barcode length) will cause the tool to extract wrong sequence segments, leading to complete data loss and meaningless output files.
- **Assuming UMI correction is disabled by default**: The error correction feature is enabled with a threshold of 1 edit distance, which may over-correct in low-complexity UMI sets and artificially reduce molecule diversity estimates.
- **Processing raw FASTQ without quality filtering**: Running amplify on unfiltered data will propagate low-quality reads, causing inflated duplicate counts and misleading statistics in downstream analysis.
- **Ignoring output statistics**: Failing to review the summary statistics file means missing critical QC metrics like duplicate rate and error correction count, which indicate sample quality issues.

## Examples

### Trim adapters from ST-seq reads

**Args:** `--input sample.fastq.gz --output trimmed.fq --adapter-sequence AGATCGGAAGAGCACACGTCTGAACTCCAGTCA`
**Explanation:** The adapter sequence flag instructs amplify to locate and remove sequencing adapters from read ends before further processing.

### Collapse PCR duplicates using UMI and position

**Args:** `--input raw.fq --output collapsed.fq --method umi-position`
**Explanation:** The umi-position method directs the tool to identify duplicates by matching both UMI sequences and genomic alignment positions, keeping one representative read per unique molecule.

### Set custom UMI length

**Args:** `--input reads.fq --output processed.fq --umi-length 12`
**Explanation:** Specifying a 12-base UMI length tells amplify to extract the first 12 nucleotides after the barcode as the unique molecular identifier for deduplication.

### Adjust error correction stringency

**Args:** `--input reads.fq --output corrected.fq --error-correction --edit-distance 2`
**Explanation:** Increasing the edit distance to 2 allows more aggressive UMI error correction, grouping sequences within 2 edits as likely the same molecule.

### Disable error correction entirely

**Args:** `--input reads.fq --output unique.fq --no-error-correction`
**Explanation:** Disabling error correction preserves original UMI sequences without clustering, suitable for datasets with high-quality UMIs where over-correction is a concern.

### Generate detailed statistics report

**Args:** `--input sample.fq --output processed.fq --stats-file stats.json`
**Explanation:** Writing statistics to a JSON file provides machine-readable QC metrics for integration into automated pipelines and downstream reporting.