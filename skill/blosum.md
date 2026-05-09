---
name: blosum
category: sequence-analysis
description: Computes and retrieves BLOSUM (BLOcks SUbstitution Matrix) scoring matrices for amino acid sequence alignment. Supports multiple BLOSUM variants and output formats.
tags:
  - sequence-alignment
  - protein-analysis
  - scoring-matrix
  - bioinformatics
  - amino-acids
author: AI-generated
source_url: https://emboss.readthedocs.io/en/latest/programs/EMBOSS/blosum
---

## Concepts

- BLOSUM matrices score the likelihood of one amino acid substituting for another based on observed evolutionary changes in conserved protein sequence blocks. The numeric suffix (e.g., BLOSUM62) indicates the percent identity threshold used during matrix construction from the BLOCKS database.
- The blosum tool accepts a numeric argument to select a specific matrix variant (45, 50, 62, 65, 80, 90, 95, or 100), with BLOSUM62 being the most widely used for general protein similarity searches because it balances sensitivity and specificity for moderately conserved sequences.
- Output can be directed to stdout in standard substitution matrix format (space-separated or column-aligned), XML format, or serialized to a file for later use in alignment programs such as needle, water, or external tools like BLAST.
- BLOSUM matrices are symmetric—scores for amino acid pair (A, B) equal (B, A)—and include a special score for identical residues vs. gap penalties, which are governed by separate gap scoring parameters in alignment algorithms.

## Pitfalls

- Omitting the matrix number argument defaults to an unintended BLOSUM variant, producing alignment scores that may not match published results or established pipelines, leading to non-reproducible analysis.
- Failing to redirect stdout to a file causes the matrix to print to the terminal, making it difficult to capture for downstream alignment commands that require an input matrix file.
- Attempting to use BLOSUM matrices for nucleotide sequence alignment yields meaningless scores since these matrices are derived exclusively from amino acid substitution frequencies in protein families.
- Misinterpreting XML output as plain text matrix format when parsing programmatically results in malformed data ingestion, as XML includes tags and attributes that require proper parsing.

## Examples

### Generate the standard BLOSUM62 substitution matrix

**Args:** `62`
**Explanation:** Specifying `62` outputs the widely-used BLOSUM62 matrix, which is derived from sequences with at most 62% identity and is the default choice for most protein similarity and database search applications.

### Generate BLOSUM45 matrix for distantly related proteins

**Args:** `45`
**Explanation:** Specifying `45` produces a matrix optimized for highly diverged protein sequences, providing higher sensitivity but lower specificity for remote homology detection compared to matrices with higher thresholds.

### Save a BLOSUM80 matrix to a file for repeated use

**Args:** `80 -outfile blosum80.mat`
**Explanation:** Specifying `80` with an output file flag redirects the computed matrix to a persistent file, enabling reuse across multiple alignment runs without recomputing.

### Output BLOSUM62 matrix in XML format

**Args:** `62 -xml`
**Explanation:** Adding the XML flag wraps the matrix output in XML tags, which is required when integrating with certain pipeline frameworks or tools that consume structured matrix data.

### Generate BLOSUM50 matrix with verbose column headers

**Args:** `50 -verbose`
**Explanation:** Enabling verbose output includes column headers and metadata alongside the numeric matrix, assisting in verification that the correct matrix variant and parameters were used.