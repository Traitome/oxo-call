---
name: blockbuster
category: Alignment Analysis / Genomics
description: Identifies and extracts blocks of similarity in pairwise sequence alignments based on configurable gap and score thresholds. Operates on alignment chain/net files to report contiguous regions separated by large insertions, deletions, or inversions.
tags:
  - alignment
  - similarity-blocks
  - chain-format
  - synteny
  - genomics
  - structural-variants
  - chain-reader
author: AI-generated
source_url: https://hgdownload.soe.ucsc.edu/downloads.html
---

## Concepts

- Blockbuster reads **pairwise alignment chain files** (or net files) from standard input and identifies contiguous segments of similarity separated by gaps exceeding a user-defined threshold. The chain format stores alignment records as `track` lines followed by `chain` directives with score, strand, and coordinates.
- The **gap threshold** (`-g`) is the primary parameter controlling block definition; gaps larger than the threshold terminate the current block and begin a new one. Setting this value inappropriately merges distinct homologous regions or fragments genuine syntenic blocks.
- Output formats include **short** (feature count per region), **bed** ( Browser Extended Display with block coordinates and scores), and **all** (full per-base representation). The `-do` flag selects the output mode and determines downstream compatibility with genome browsers or custom parsers.
- Block scores (`-score`) represent the total alignment column count weighted by match/mismatch penalties; blocks scoring below `-minScore` are discarded before emission. Filtering by both score and length (`-minBlockSize`) reduces noise from spurious micro-alignments.
- The tool requires **coordinate-sorted alignments** and a reference/query chromosome naming convention consistent between input files. Chromosome names are matched literally; case mismatches cause silent data loss.

## Pitfalls

- **Setting `-g` too large** collapses multiple genuine syntenic blocks into single regions, obscuring evolutionary rearrangements such as inversions or translocations. Conversely, a `-g` value too small splits authentic continuous alignments at small indels.
- **Forgetting to sort alignments** by genomic coordinate before input results in blocks being emitted out of order or silently dropped. The tool does not validate ordering and produces no warning for unsorted input.
- **Using `-do all` on large genomes** without restricting chromosome range with `-chr` generates enormously verbose output, consuming excessive disk space and processing time. Prefer `short` or `bed` for exploratory analysis.
- **Confusing block score with alignment coverage**: a block's score reflects column count minus gap penalties, not percentage identity. Two blocks of equal length may have different scores due to mismatch density, causing unexpected filtering at `-minScore`.
- **Omitting strand information** when querying minus-strand alignments causes coordinate confusion in BED output, as BED format stores 0-based start coordinates but blockbuster reports end coordinates inconsistently across output modes.

## Examples

### Detect blocks in a chain file using default gap threshold

**Args:** `myAlignments.chain -g 100000`
**Explanation:** Reads a sorted chain alignment file and identifies blocks separated by gaps larger than 100 kb, emitting all matching blocks with default short output format.

### Extract high-confidence blocks with minimum score filter in BED format

**Args:** `alignments.chain -g 200000 -minScore 5000 -do bed`
**Explanation:** Filters blocks to only those with alignment scores ≥ 5000 and outputs results in BED format suitable for visualization in UCSC Genome Browser or similar tools.

### Restrict analysis to a specific chromosome to reduce output

**Args:** `alignments.chain -g 50000 -chr chr22 -do short`
**Explanation:** Limits block detection to chromosome chr22 only, reducing runtime and output size when working with whole-genome alignment files.

### Identify small syntenic blocks below typical gap thresholds

**Args:** `alignments.chain -g 10000 -minBlockSize 1000 -minScore 500 -do bed`
**Explanation:** Detects small but substantial blocks with minimum length of 1000 bases and score of 500, capturing short conserved regions often missed by default parameters.

### Process compressed alignment file via stream redirection

**Args:** `gunzip -c alignments.gz -g 250000 -do short`
**Explanation:** Decompresses a gzip-compressed alignment file on the fly and feeds it to blockbuster, avoiding disk space usage for intermediate uncompressed files.