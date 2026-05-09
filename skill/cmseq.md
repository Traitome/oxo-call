---
name: cmseq
category: bioinformatics/sequence-analysis
description: A bioinformatics tool for analyzing circular sequences (circRNA, plasmids, mitochondrial DNA) from mapped sequencing reads. Provides coverage calculation, sequence extraction, breakpoint detection, and polymorphism analysis on circular genomes.
tags: circularRNA, coverage, breakpoint, sequence-extraction, circRNA, genomics, BAM, variant-analysis
author: AI-generated
source_url: https://github.com/mother-gene/cmseq
---

## Concepts

- **Input Data Model**: cmseq operates on SAM/BAM files containing aligned reads. Reads must be coordinate-sorted and indexed with valid chromosome names matching the reference. For circular sequence analysis, the alignment must include junction-spanning reads that cross the assumed junction point (position 0).

- **Circular Topology Handling**: When calculating coverage on circular sequences, cmseq accounts for wrap-around by treating the sequence as circular—reads mapping near the end can contribute coverage at the beginning. The `--pos` or `--junction` flag specifies the assumed break point location (default: 0).

- **Subcommand Structure**: cmseq provides distinct subcommands including `coverage` (per-base coverage metrics), `extract` (pull sequences from specific regions), `polymake` (polymorphism analysis), and `merge` (combine overlapping reads). Each subcommand has specific required arguments and output formats.

- **Output Formats**: Output is typically plain text (TSV/CSV) for metrics, and FASTQ/FASTA for sequence extraction. Coverage output includes columns for position, coverage depth, and optionally base-level breakdowns.

## Pitfalls

- **Unsorted or Unindexed BAM Files**: Running cmseq on unsorted BAM files produces incorrect coverage calculations because read ordering is assumed to be coordinate-sorted. Unindexed BAM files cause failures when cmseq attempts random access. Always sort with `samtools sort` and index with `samtools index` before analysis.

- **Incorrect Junction Position**: Specifying the wrong `--junction` coordinate causes coverage to wrap at the wrong point, leading to artificially inflated or deflated coverage at sequence ends. This is especially problematic when the true circular junction location is unknown—always verify and specify the expected break point.

- **Mismatched Reference Names**: If BAM file chromosome names don't match the reference sequence name provided to cmseq, no reads will be found for analysis. Reference FASTA headers must exactly match the RNAME field in the BAM file (e.g., "chr1" vs "1" mismatch).

- **Memory Limits with Large Files**: Processing whole-genome BAM files without restricting to specific regions can exceed available RAM, especially for the `extract` subcommand. Use BAI-indexed random access or specify genomic intervals to limit memory usage.

- **Ignoring Multimapped Reads**: By default, cmseq includes all mapped reads in coverage calculations. Reads mapped to multiple locations (.mapq = 0) can artificially inflate coverage. Use `--mapq` threshold flags when appropriate to filter ambiguous mappings.

## Examples

### Calculate per-base coverage on a circular RNA

**Args:** `coverage --bam alignments.bam --fasta reference.fa --output coverage.tsv --junction 150`

**Explanation:** This calculates coverage across the circular reference sequence, accounting for wrap-around at position 150 as the junction point, and writes per-base depth to a TSV file.

### Extract sequences spanning the circular junction

**Args:** `extract --bam alignments.bam --fasta reference.fa --region 100-200 --junction 150 --output extracted.fa`

**Explanation:** Extracts reads mapping to the specified region, handling wrap-around for positions crossing the junction at position 150, and outputs sequences to FASTA format.

### Run polymorphism analysis on circularized sequences

**Args:** `polymake --bam alignments.bam --fasta reference.fa --min-count 3 --output variants.tsv`

**Explanation:** Identifies polymorphic positions with at least 3 supporting reads, accounting for the circular topology of the reference sequence.

### Calculate coverage with minimum mapping quality filter

**Args:** `coverage --bam alignments.bam --fasta reference.fa --mapq 30 --output filtered_cov.tsv`

**Explanation:** Only includes reads with mapping quality >= 30 in the coverage calculation, excluding ambiguously mapped reads from the results.

### Extract read sequences from a specific interval with junction handling

**Args:** `extract --bam alignments.bam --fasta reference.fa --region 1200-100 --junction 150 --output cross_junction.fa`

**Explanation:** Extracts reads from a region crossing the junction (from position 1200 to position 100), properly handling the circular wrap-around for sequences spanning the break point.

### Calculate coverage with JSON output for downstream parsing

**Args:** `coverage --bam alignments.bam --fasta reference.fa --format json --output cov.json`

**Explanation:** Outputs coverage data in JSON format instead of default TSV, making it easier to parse and integrate into automated bioinformatics pipelines.