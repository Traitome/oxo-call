---
name: anise_basil
category: Sequence Analysis / Optimization
description: A bioinformatics utility for detecting and optimizing inverted repeat structures and palindromic sequences in DNA/RNA. Analyzes symmetry patterns, calculates thermodynamic stability of secondary structures, and provides optimized sequence variants with improved structural features.
tags: [sequence-analysis, inverted-repeats, palindromes, secondary-structure, dna-optimization, rna-folding]
author: AI-generated
source_url: https://github.com/anise-project/anise_basil
---

## Concepts

- **Inverted Repeat Detection**: Anise_basil identifies perfect and imperfect inverted repeats (stem-loops) by scanning input sequences for complementary reverse complements. Minimum stem length and loop size thresholds are configurable via CLI flags.
- **Thermodynamic Stability Scoring**: The tool calculates ΔG values for detected structures using nearest-neighbor parameters, allowing users to rank structures by their predicted stability in vivo.
- **Multi-sequence Input Handling**: Accepts FASTA, plain text, and GenBank formats via stdin or file input, processing multiple sequences in batch mode when --multi flag is enabled.
- **Output Formats**: Generates JSON, BED, and custom annotation formats; JSON output includes coordinates, sequences, scores, and secondary structure in dot-bracket notation.

## Pitfalls

- **Ignoring Mismatches in Stems**: Using overly short --stem-length thresholds (e.g., below 4) detects trivial repeats that form unstable structures, wasting analysis time and producing false positives in downstream applications.
- **Specifying Conflicting Output Flags**: Combining --json and --bed produces only JSON output (first Flag wins), causing users to miss expected BED annotations if they don't verify output.
- **Large Input Without Chunking**: Processing whole chromosomes or long contigs without --chunk-size may cause memory exhaustion; the tool buffers entire sequences in RAM before analysis.
- **Assuming Standard Genetic Code**: Anise_basil treats input as DNA by default unless --rna flag is set, causing incorrect complement calculations when analyzing RNA sequences.

## Examples

### Detect inverted repeats in a DNA sequence

**Args:** --fasta sequence.fasta --min-stem 6 --loop-min 3
**Explanation:** Scans sequences from file for inverted repeats with at least 6 base pairs in stem and minimum loop of 3 nucleotides, outputting all hits.

### Calculate thermodynamic stability for found structures

**Args:** --input ATCGATCG --thermo
**Explanation:** Analyzes the input sequence for inverted repeats and appends ΔG stability scores to each detected structure in the output.

### Output results in BED annotation format

**Args:** --fasta sequences.fa --min-stem 8 --bed
**Explanation:** Writes detected inverted repeats as BED features with score column indicating repeat strength, suitable for genome browsers.

### Process multiple sequences with chunked memory management

**Args:** --fasta genome.fa --chunk-size 100000 --multi
**Explanation:** Reads genome in 100kb chunks to avoid memory overflow, enables multi-sequence mode for batch processing of all contigs.

### Analyze RNA sequences for secondary structures

**Args:** --input AUGAUCGACAU --rna --min-stem 5 --json
**Explanation:** Treats input as RNA, calculates RNA-specific thermodynamic parameters, and outputs structured results in JSON format with dot-bracket notation.