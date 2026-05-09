---
name: chiron
category: Sequence Alignment & Comparison
description: A fast DNA/RNA sequence alignment and comparison tool using suffix tree algorithms for pairwise and multiple sequence alignments. Optimized for whole-genome comparisons and detecting structural variants.
tags:
  - sequence-alignment
  - genomics
  - pairwise-alignment
  - comparative-genomics
  - variant-detection
  - dna-analysis
author: AI-generated
source_url: https://github.com/mummer4/chiron
---

## Concepts

- **Input Formats**: Chiron accepts FASTA (.fa, .fasta) and multi-FASTA files for both reference and query sequences. Plain text format with sequence identifiers (lines starting with '>') is also supported. Sequences can be provided as raw strings without line breaks.
- **Alignment Output**: Produces SYDNEY format alignment files by default, containing coordinate mappings and alignment scores. The `-o` flag redirects output to a user-specified file path. Alignment results include cigar strings, alignment coordinates, and alignment quality scores.
- **Scoring Parameters**: Uses a substitution matrix approach with customizable match/mismatch scores. Default is +1 for matches and -2 for mismatches. The `-m` flag overrides default mismatch penalty, while `-x` sets match reward value.
- **Anchor-based Alignment**: For large sequences, chiron identifies exact matches (≤30bp by default) as anchors before extending alignments. Anchor length threshold can be adjusted using the `-l` flag to balance speed and sensitivity.
- **Output Modes**: Three primary modes are available: alignment view (`-v`), anchor list (`-a`), and minimal coordinates (`-c`). The `-t` flag enables threading for multi-threaded processing on multi-core systems.

## Pitfalls

- **Uncompressed Large Files**: Feeding gzip-compressed FASTA files (.fa.gz) directly to chiron causes parsing failures. Always decompress input files first using `gunzip` or provide uncompressed files to avoid alignment errors.
- **Insufficient Memory for Large Genomes**: Aligning entire chromosome-scale sequences without sufficient RAM results in segmentation faults. For human-genome-scale comparisons, ensure ≥16GB RAM or use the `-l` flag to increase anchor lengths and reduce memory footprint.
- **Mismatched Sequence Encodings**: Mixing upper-case and lower-case sequences can produce inconsistent results because chiron treats them as different characters by default. Standardize all input sequences to consistent case before alignment.
- **Output File Overwrites**: Using `-o output.syd` with an already-existing output file silently overwrites the previous file without confirmation. Back up important alignment results before re-running analyses.
- **Thread Count Exceeding Cores**: Specifying `-t 32` on an 8-core machine causes excessive context switching and slower performance. Always set thread count ≤ number of physical cores available.

## Examples

### Align two DNA sequences from FASTA files
**Args:** `ref.fa query.fa -o alignment.syd`
**Explanation:** Performs pairwise alignment between sequences in ref.fa and query.fa, writing results to alignment.syd in SYDNEY format.

### Increase anchor length for faster processing of large sequences
**Args:** `ref.fa query.fa -l 40 -o output.syd`
**Explanation:** Sets minimum anchor length to 40bp, reducing memory usage and processing time at the cost of potentially missing shorter conserved regions.

### Generate alignment view with coordinate information
**Args:** `ref.fa query.fa -v -o view.txt`
**Explanation:** Outputs alignment in readable view format containing coordinate mappings, matching regions, and alignment statistics.

### Adjust mismatch penalty for sensitive alignment
**Args:** `ref.fa query.fa -m -3 -o sensitive.syd`
**Explanation:** Lowers mismatch penalty to -3, making alignment more tolerant of mismatches to capture divergent homologous regions.

### Process multiple query sequences using threading
**Args:** `ref.fa queries.fa -t 8 -o multi.syd`
**Explanation:** Uses 8 threads to accelerate alignment when query file contains multiple sequences, improving throughput on multi-core systems.

### Output only anchor coordinates without full alignment
**Args:** `ref.fa query.fa -a -o anchors.txt`
**Explanation:** Extracts and lists only the exact-match anchor coordinates between sequences, useful for identifying conserved regions quickly.