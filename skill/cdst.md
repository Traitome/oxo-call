---
name: cdst
category: Sequence Analysis
description: A bioinformatics tool for analyzing and manipulating DNA/RNA sequence data with support for multiple alignment formats, sequence filtering, and statistical analysis.
tags: [sequences, dna, rna, alignment, filtering, statistics]
author: AI-generated
source_url: https://github.com/example/cdst
---

## Concepts

- **Input Formats**: cdst accepts FASTA, FASTQ, and raw sequence formats via stdin or file input, with automatic format detection based on file extension or header markers.
- **Sequence Data Model**: Operates on nucleotide sequences with quality scores preserved when available; uses 0-based indexing internally but outputs 1-based coordinates for user convenience.
- **Filtering Logic**: Applies user-defined criteria including minimum/maximum sequence length, GC content thresholds, and pattern matching using IUPAC nucleotide codes.
- **Output Modes**: Supports plain text, JSON, and TSV export formats; streaming output available for large datasets to minimize memory footprint.

## Pitfalls

- **Quality Score Mismatch**: Treating FASTQ files as FASTA will cause quality strings to be interpreted as sequence data, leading to incorrect GC content calculations and filtering failures.
- **Memory Limits**: Attempting to load >100 million sequences without streaming mode will exhaust available RAM and crash the process, potentially losing unsaved data.
- **Coordinate Confusion**: Failing to recognize that cdst outputs 1-based coordinates while internally using 0-based indexing causes off-by-one errors when comparing results to tools like BEDTools.
- **Pattern Matching Case Sensitivity**: Using uppercase patterns on lowercase sequences (or vice versa) silently produces zero matches, masking the root cause of empty output sets.

## Examples

### Filter sequences by minimum length
**Args:** `-i sequences.fasta --min-length 50`
**Explanation:** Removes all sequences shorter than 50 nucleotides from the input file, outputting only sequences meeting the length threshold.

### Calculate GC content for all sequences
**Args:** `-i sequences.fasta --gc-content`
**Explanation:** Computes and displays the GC percentage for each sequence in the input, useful for assessing genome composition bias.

### Extract sequences matching a specific pattern
**Args:** `-i sequences.fasta --pattern "GNGTN{2,}A"`**Explanation:** Outputs sequences containing the specified degenerate nucleotide pattern, enabling motif discovery in sequence datasets.

### Convert FASTQ to FASTA format
**Args:** `-i input.fastq -o output.fasta --convert-format`
**Explanation:** Transforms input from FASTQ to FASTA by discarding quality scores, useful when downstream tools only accept FASTA input.

### Export results as JSON for downstream processing
**Args:** `-i sequences.fasta --gc-content --min-length 100 -o results.json --json`
**Explanation:** Combines filtering and formatting into a single pipeline, outputting structured JSON for programmatic analysis or integration with workflow managers.

### Stream large datasets to avoid memory overflow
**Args:** `-i large_dataset.fasta --stream --min-length 200`
**Explanation:** Processes input incrementally instead of loading all sequences at once, enabling analysis of files larger than available RAM.

### Filter by GC content threshold range
**Args:** `-i sequences.fasta --min-gc 40 --max-gc 60`
**Explanation:** Selects only sequences with GC content between 40% and 60%, useful for isolating sequences with balanced nucleotide composition.

### Append sequence statistics to existing output file
**Args:** `-i sequences.fasta --stats --append -o summary.tsv`
**Explanation:** Generates comprehensive statistics (length, GC, N-content) and appends them to a tab-delimited file without overwriting existing data.