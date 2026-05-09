---
name: basic
category: Utilities
description: A lightweight bioinformatics utility for basic sequence manipulation, format conversion, and text processing operations. Provides simple, foundational operations for DNA/RNA/protein sequence data.
tags: [sequence, manipulation, format-conversion, utilities, text-processing]
author: AI-generated
source_url: https://github.com/username/basic
---

## Concepts
- The tool operates on standard sequence formats (FASTA, FASTQ, plain text) and can convert between them using simple command-line flags
- Input is accepted from stdin or file arguments, and output goes to stdout by default unless an output file is specified with appropriate flags
- The tool processes sequences line-by-line or in batch mode depending on flags, maintaining the order of input records in the output
- Sequence data is treated as case-insensitive by default (A, T, G, C, U are equivalent to a, t, g, c, u) unless a case-sensitive flag is provided

## Pitfalls
- Omitting the output flag when processing large files causes results to print to stdout, which can be lost in terminal buffers; always redirect to a file or use the output flag
- Using the wrong format flag (e.g., specifying FASTQ input when the file is FASTA) results in parsing errors or silent data corruption without warnings
- Not specifying a newline character when processing Windows-style line endings (CRLF) can cause incomplete sequence reads or merge artifacts in output
- Forgetting to handle empty sequences or header-only files results in error messages or empty output files without clear indication of what went wrong

## Examples
### Convert a FASTA file to FASTQ format
**Args:** `--input sequence.fasta --output sequence.fastq --format fastq`
**Explanation:** Reads sequences from a FASTA file and writes them to FASTQ format with placeholder quality scores.

### Extract only sequence headers from a multi-FASTA file
**Args:** `--input sequences.fasta --headers-only`
**Explanation:** Outputs only the header lines without the associated sequence data, useful for creating index files.

### Reverse complement DNA sequences
**Args:** `--input dna.fasta --reverse-complement`
**Explanation:** Transforms each input sequence into its reverse complement, swapping A↔T and G↔C while preserving strand direction.

### Uppercase all sequence data
**Args:** `--input mixed.fasta --uppercase --output uppercase.fasta`
**Explanation:** Converts all lowercase nucleotides to uppercase, ensuring uniformity for downstream tools requiring consistent case.

### Extract sequences by length threshold
**Args:** `--input sequences.fasta --min-length 100 --output long_sequences.fasta`
**Explanation:** Filters input sequences, keeping only those with 100 or more bases, useful for removing short reads or adapters.

### Count nucleotide composition
**Args:** `--input sequence.fasta --count`
**Explanation:** Reports the total counts of each nucleotide (A, T, G, C, U) across all sequences in the input file.

### Convert multi-line FASTA to single-line format
**Args:** `--input multiline.fasta --single-line --output oneline.fasta`
**Explanation:** Collapses sequences that span multiple lines into single-line format per sequence record, the inverse of line-wrapping.

### Print sequence lengths only
**Args:** `--input sequences.fasta --lengths`
**Explanation:** Outputs a two-column report showing header and sequence length for each record, useful for quality control and filtering decisions.

### Filter sequences by header pattern
**Args:** `--input sequences.fasta --filter-header "sample1*" --output filtered.fasta`
**Explanation:** Retains only sequences whose headers start with "sample1", supporting basic pattern matching with wildcards.

### Merge multiple FASTA files into one
**Args:** `--input file1.fasta --inputfile file2.fasta --output merged.fasta`
**Explanation:** Combines sequences from multiple input files into a single output file in the order specified by the flags.

---