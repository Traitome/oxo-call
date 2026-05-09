---
name: cogtriangles
category: Comparative Genomics
description: A tool for computing triangle-based similarity measures between genomic regions or sequences, commonly used in comparative genomics and evolutionary analysis to quantify shared structural or sequence features.
tags: [comparative-genomics, similarity-scoring, genomic-regions, phylogenetic]
author: AI-generated
source_url: https://github.com/cogtriangles/cogtriangles
---

## Concepts

- cogtriangles computes pairwise or multiple similarity scores using a triangle inequality model, where three genomic elements form a comparison unit and the scoring reflects shared ancestry, structural homology, or sequence conservation.
- Input files typically accept BED, GFF3, or custom coordinate formats for genomic intervals, while sequence-based comparisons accept FASTA or multi-FASTA inputs; output is written as TSV or CSV with configurable precision.
- The tool supports a scoring matrix via the `--matrix` flag, where integer values define match/mismatch penalties analogous to BLAST scoring, and the `--min-score` threshold filters results below a specified similarity cutoff.
- Triangle-based comparisons leverage the property that for three elements A, B, and C, the similarity score S(A,B) + S(B,C) >= S(A,C) when regions are collinear and structurally conserved, enabling detection of rearrangements.
- Batch processing is enabled with `--batch-dir`, which recursively scans a directory for input files and writes per-file outputs named with the input basename plus a configurable suffix.

## Pitfalls

- Using coordinate systems that are reference-assembly-specific without specifying `--assembly` causes coordinate mismatches when comparing regions across different genome builds, leading to silent false negatives.
- Setting `--min-score` too low produces an excessive number of low-confidence triangle hits that inflate runtime and memory usage, while setting it too high may discard biologically meaningful weak similarities in divergent genomes.
- Passing unsorted BED files without first sorting by chromosome and position violates the internal indexing assumption, causing I/O errors or truncated output when the tool attempts binary-search lookups.
- Omitting `--reciprocal` when computing orthology-suggestive comparisons means non-reciprocal best hits are included, which confuses downstream ortholog assignment and introduces false positive synteny calls.
- Confusing `--gap-open` and `--gap-extend` units (which are in score units, not base pairs) with actual gap lengths causes incorrect penalty calculations and skewed similarity scores for indel-rich regions.

## Examples

### Computing pairwise similarity between two genomic intervals
**Args:** `--ref-file regions_setA.bed --query-file regions_setB.bed --matrix scoring.mat --output simA_vs_B.tsv`
**Explanation:** This compares each interval in `regions_setA.bed` against all intervals in `regions_setB.bed` using the scoring matrix, producing pairwise similarity scores in TSV format.

### Filtering results by a minimum similarity threshold
**Args:** `--input all_hits.tsv --min-score 42 --output high_conf_hits.tsv`
**Explanation:** This post-filters an existing hit file to retain only triangle comparisons with a similarity score of 42 or higher, removing low-confidence alignments from consideration.

### Batch processing all BED files in a directory
**Args:** `--batch-dir ./intervals/ --matrix scoring.mat --suffix _cog.tsv --output-dir ./results/`
**Explanation:** This processes every BED file found in `./intervals/` individually, writing output files named with the `_cog.tsv` suffix into `./results/` for high-throughput comparative analysis.

### Running reciprocal best-hit orthology detection
**Args:** `--ref-file species1_intervals.bed --query-file species2_intervals.bed --reciprocal --min-score 50 --output ortholog_pairs.tsv`
**Explanation:** This enforces reciprocal filtering so that a triangle hit is only reported if region A in species1 and region B in species2 are each other's top-scoring match above the threshold.

### Specifying a custom scoring matrix with gap penalties
**Args:** `--ref-file genes.fasta --query-file homologs.fasta --matrix nucleotide.matrix --gap-open -3 --gap-extend -1 --output gene_sims.tsv`
**Explanation:** This runs sequence-based triangle scoring using a nucleotide substitution matrix with opening and extension gap penalties applied during alignment-style comparisons.