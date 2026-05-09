---
name: clipkit
category: alignment-processing
description: Alignment trimming tool that removes phylogenetically uninformative columns from multiple sequence alignments using smart gap-based, length-based, similarity-based, or combined algorithms.
tags: [alignment, trimming, masking, phylogenetics, fasta, gap, conservation]
author: AI-Generated
source_url: https://github.com/joheo/clipkit
---

## Concepts

- **Smart-gap mode is the default algorithm**: ClipKit's default smart-gap algorithm dynamically determines which columns to remove based on the gap distribution across the alignment rather than applying a fixed threshold to every column. It iteratively removes the most gapped column until all remaining columns meet the minimum presence criterion.
- **Input must be a pre-aligned FASTA file**: ClipKit requires sequences that are already aligned to identical length with gaps represented by '-' characters. Each sequence must be the same length because the tool processes the alignment column-by-column to identify which positions to retain or remove.
- **Gap threshold controls stringency via the -k flag**: The -k parameter specifies the minimum fraction of non-gap sequences required to retain a column (e.g., -k 0.5 requires at least 50% of sequences to have data). Lower values retain more columns, higher values produce stricter trimming.
- **Multiple trimming modes via -m flag**: The -m flag selects the algorithm—default (smart-gap), length (removes sequences with too many gaps), similarity (removes low-conservation columns), allsitesgap (removes columns that are entirely gaps), and more. Each mode serves different downstream analytical goals.
- **Output preserves coordinate tracking with -c flag**: Adding -c includes coordinate annotations in the output, reporting which positions from the original alignment were retained. This is essential when mapping trimmed results back to the source alignment for downstream phylogenetic or comparative analyses.

## Pitfalls

- **Feeding unaligned sequences**: ClipKit expects all input sequences to be aligned to the same length with consistent gap characters. Unaligned FASTA input causes the tool to calculate incorrect column statistics, producing meaningless or corrupt output without necessarily throwing an error.
- **Setting gap threshold too low or too high**: Using -k 0 removes every column containing even one gap, which often eliminates useful data in real alignments that naturally have some taxon-specific insertions. Conversely, -k 1 keeps only columns with zero gaps, which rarely produces any output for biological data. Moderate thresholds like 0.3 to 0.7 are usually most appropriate.
- **Confusing length mode with column trimming**: The -m length mode removes entire sequences (rows) rather than columns, based on the fraction of gaps per sequence. Users expecting column-wise trimming may misread the output and lose most of their sequences instead of just the gappy columns.
- **Ignoring the output file flag**: The default behavior prints results to stdout, which can be accidentally truncated or mixed with error messages when redirecting. Using -o to specify an explicit output file ensures clean, reproducible results.
- **Using clipkit on heavily incomplete alignments without inspection**: The tool cannot compensate for fundamentally poor alignments. For alignments with >80% missing data, clipkit may trim away nearly everything, leaving insufficient data for meaningful phylogenetic inference. Always visually inspect the alignment quality before and after trimming.

## Examples

### Basic trimming with default smart-gap mode
**Args:** `-i input_alignment.fasta`
**Explanation:** Runs the default smart-gap algorithm, which iteratively removes the most gapped column until all remaining columns meet the minimum presence threshold, providing clean output without additional flags.

### Trimming with a specific gap threshold
**Args:** `-i alignment.fasta -k 0.5`
**Explanation:** Sets the minimum presence to 50%, meaning each retained column must have non-gap data in at least half of the sequences, which is a balanced setting for most phylogenetic analyses.

### Saving output to a file with coordinate annotations
**Args:** `-i alignment.fasta -o trimmed_output.fasta -c`
**Explanation:** The -o flag writes the cleaned alignment to a named file while -c adds coordinate annotations so you can map retained positions back to the original alignment.

### Using similarity-based trimming mode
**Args:** `-i protein_alignment.fasta -m similarity`
**Explanation:** The similarity mode removes columns with low amino acid or nucleotide conservation, targeting hypervariable regions that may introduce homoplasy in phylogenetic reconstruction.

### Using all-sites-gap mode for maximum pruning
**Args:** `-i alignment.fasta -m allsitesgap`
**Explanation:** Removes only columns that are entirely gaps across all sequences, which is the gentlest trimming option useful for identifying sequences missing from specific taxa without aggressively pruning the alignment.

### Trimming with minimum presence of 90%
**Args:** `-i alignment.fasta -k 0.9`
**Explanation:** Requires 90% of sequences to have non-gap data for each retained column, producing an ultra-conservative trimmed alignment that preserves only the most complete and phylogenetically reliable regions.