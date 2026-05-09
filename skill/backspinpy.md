---
name: backspinpy
category: Sequence Manipulation
description: A Python tool for reversing DNA/RNA sequences, computing reverse complements, and analyzing sequence inversions in genomic data.
tags: [bioinformatics, sequence-analysis, reverse-complement, genomics, dna, rna]
author: AI-generated
source_url: https://github.com/backspinpy/backspinpy
---

## Concepts

- **Input formats**: Accepts FASTA (.fasta, .fa), FASTQ (.fq), plain text (.txt), and CSV (.csv) containing sequence data. Each sequence must be on a single line or properly formatted with header lines.
- **Output modes**: Supports three output formats: standard (reversed sequence), reverse-complement (DNA/RNA with complement and reversed), and JSON for programmatic integration.
- **Sequence type detection**: Automatically detects DNA vs RNA sequences by detecting uracil (U) vs thymine (T) bases. Mixed bases or ambiguity codes (N, R, Y, etc.) are preserved unless explicitly filtered.
- **Companion binary**: Use `backspinpy-build` for creating index databases for batch processing of large sequence collections.

## Pitfalls

- **Using DNA flags with RNA sequences**: Applying `--dna-mode` to RNA sequences will incorrectly convert uracil to thymine, corrupting the sequence. Always let auto-detection handle this or use `--rna-mode` explicitly.
- **Ignoring case sensitivity**: The tool is case-sensitive; lowercase bases may be treated as invalid characters if `--strict` mode is enabled, leading to silent filtering or errors.
- **Wrong output overwrite**: Not specifying `-o/--output` when processing multiple sequences will overwrite files sequentially. Always use output directories (`-d/--dir`) for batch operations.
- **Ambiguity code handling**: Default behavior preserves ambiguity codes but may not analyze them correctly in downstream tools that expect resolved bases.

## Examples

### Reverse a single DNA sequence from FASTA input
**Args:** `-i sequence.fasta --reverse -o reversed.fasta`
**Explanation:** Reads the DNA sequence from a FASTA file and outputs the reversed sequence (5'→3' becomes 3'→5') to the specified output file.

### Compute reverse complement of an RNA sequence
**Args:** `-i mrna.fasta --rev-comp --rna-mode -o output.fasta`
**Explanation:** Calculates the reverse complement of an RNA sequence, replacing A↔U, G↔C and reversing the order, preserving the RNA base uracil.

### Batch process all FASTA files in a directory
**Args:** `-d input_dir/ --rev-comp --dir -o output_dir/`
**Explanation:** Processes all sequence files in the input directory, applying reverse complement to each and writing results to the output directory with preserved filenames.

### Export results in JSON format
**Args:** `-i sequences.fasta --rev-comp --json -o results.json`
**Explanation:** Outputs reverse-complemented sequences in JSON format, making it suitable for pipeline integration and programmatic parsing.

### Build index for large sequence database
**Args:** `-i genome.fa --build-index -o genome.bspidx`
**Explanation:** Uses the companion binary `backspinpy-build` to create an indexed database of the genome for fast lookup operations in subsequent analyses.

### Filter and reverse sequences with ambiguity code handling
**Args:** `-i mixed.fasta --rev-comp --filter-ambiguity -o clean.fasta`
**Explanation:** Removes or handles ambiguity codes (N, R, Y, etc.) during reversal, outputting only resolved bases for downstream applications.