---
name: cigar
category: sequence_analysis
description: Parse, validate, and manipulate CIGAR (Compact Idiosyncratic Gapped Alignment Representation) strings commonly found in SAM/BAM alignment files. Converts between CIGAR formats, calculates aligned lengths, extracts read segments, and performs operations on aligned sequences.
tags:
- cigar
- sam
- bam
- alignment
- sequence-analysis
- read-alignment
- genomics
author: AI-generated
source_url: https://github.com/lh3/bwa
---

## Concepts

- **CIGAR String Format**: A CIGAR string uses operators (M for match/mismatch, I for insertion, D for deletion, S for soft-clipping, H for hard-clipping, N for skip, P for pad, = for exact match, X for mismatch) to represent sequence alignment. Example: `10M5I20M` means 10 matching bases, then 5 inserted bases, then 20 matching bases.
- **Input/Output Formats**: The tool accepts SAM format alignments with CIGAR strings in column 6, and can output plain CIGAR strings, JSON, or SAM format. It can also read from BAM files when paired with samtools.
- **Alignment Length Calculation**: The "query" length (read length including soft-clips) differs from the "reference" length (bases consumed on the reference). `M`, `=`, `X` consume reference bases; `I`, `S`, `H` do not. Use `cigar` to compute both values correctly.
- **Operation Symbols**: In SAM format, `=` and `X` are preferred over `M` (deprecated) to distinguish between matching and mismatching bases. The tool supports both old-style and new-style CIGAR symbols.

## Pitfalls

- **Confusing Query vs Reference Length**: Using `M` alone doesn't distinguish matches from mismatches. If you calculate reference-consumed bases by summing all numeric values, you'll incorrectly include insertions (`I`) which don't consume reference bases. Always parse the full CIGAR string.
- **Ignoring Hard-Clipped Sequences**: Hard-clipped bases (`H`) are not present in the SEQ field of a SAM record. If your pipeline expects them, the aligned sequence will be shorter than expected. The tool correctly handles this distinction.
- **Invalid CIGAR Characters**: Using deprecated or invalid operators (like lowercase letters or unsupported symbols) will cause parsing failures in downstream tools. Always validate CIGAR strings before processing.
- **Assuming All Alignments Use Same Operators**: Different aligners use different CIGAR representations. BWA-MEM uses `=`/`X` while older aligners may use `M`. The tool normalizes these but you must verify which representation your downstream analysis expects.

## Examples

### Validate a CIGAR string format
**Args:** `validate "25M5D10M"`
**Explanation:** Checks whether the CIGAR string contains valid operators and properly formatted numbers, returning an error if invalid characters or malformed operators are found.

### Convert CIGAR to JSON representation
**Args:** `parse --format json "10M5I20M3D5M"`
**Explanation:** Outputs a structured JSON showing each operation with its length and operator type, useful for programmatic processing of alignment data.

### Calculate reference-consumed length
**Args:** `length --reference "10S5M3I15M2D5M"`
**Explanation:** Computes the number of bases consumed on the reference genome (matches, deletions, skips) excluding insertions and soft-clipped bases.

### Calculate query sequence length
**Args:** `length --query "3S10M5I15M"`
**Explanation:** Computes the total read length including soft-clipped bases and insertions, matching what's stored in the SEQ field.

### Extract soft-clipped bases from a read
**Args:** `extract --soft-clip "10S90M" --sequence "AACCGTACCC...longsequence"`
**Explanation:** Extracts the first 10 bases that are soft-clipped, which may contain useful adapter or barcode sequences not aligned to the reference.

### Convert deprecated M operators to = and X
**Args:** `convert --style new "50M"`
**Explanation:** Replaces deprecated `M` operators with the more precise `=` (match) or `X` (mismatch) based on sequence comparison, improving compatibility with modern tools.

### Filter alignments by minimum aligned length
**Args:** `filter --min-ref-length 50 "10S90M" --min-ref-length 50 "5S45M" --min-ref-length 50 "100M"`
**Explanation:** The first two alignments are filtered out because their reference-consumed lengths are below 50, while the third passes the filter.