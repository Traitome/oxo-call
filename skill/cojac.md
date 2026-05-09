---
name: cojac
category: bioinformatics/quality-control
description: A tool for comparing and analyzing biological sequence data, typically used to evaluate the accuracy of sequencing or alignment results by comparing expected versus observed outputs.
tags: [comparison, validation, sequencing, quality-control, bioinformatics]
author: AI-generated
source_url: https://github.com/username/cojac
---

## Concepts

- **Data Model**: Operates on paired input files containing sequence data (typically in FASTA, FASTQ, or VCF format) representing expected/reference sequences and observed/actual sequences from sequencing runs or alignments.
- **I/O Format**: Accepts multiple input file types including FASTA (.fa/.fasta), FASTQ (.fq/.fastq), and could include alignment-specific formats like BAM/SAM. Outputs reports in text, JSON, or CSV format summarizing discrepancies.
- **Key Behavior**: Performs base-by-base or position-by-position comparison between reference and query sequences, computing metrics such as identity percentage, mismatch counts, insertion/deletion rates, and quality scores.
- **Scoring Logic**: Uses configurable matching criteria including minimum quality thresholds, ignore ambiguous base calls (N), and specific handling of indels versus substitutions.

## Pitfalls

- **Mismatch in file formats**: Using incompatible input file formats (e.g., BAM for one input and FASTQ for another) will cause parsing errors and fail to produce meaningful comparisons.
- **Ignoring quality score thresholds**: Not setting appropriate quality score cutoffs (Phred scores) can cause true sequencing errors to be counted as real variants, inflating false positive mismatch rates.
- **Misaligned sequence headers**: Input sequences with non-matching or modified headers between reference and query files will fail to pair correctly, resulting in zero alignments or complete mismatch reports.

## Examples

### Compare two FASTQ files for basic identity verification
**Args:** input1=sample1.fq input2=sample2.fq output=comparison_report.txt
**Explanation:** This runs a straightforward base-level comparison between two FASTQ files and writes the identity metrics to a text report.

### Compare sequences with high-quality threshold filtering
**Args:** input1=run1.fq input2=expected.fq min-qual=30 output=high_quality_results.txt
**Explanation:** This filters out low-confidence base calls (Phred