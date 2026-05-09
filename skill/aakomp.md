---
name: aakomp
category: Protein Sequence Analysis
description: A bioinformatics tool for calculating amino acid composition and composition-based statistics from protein sequences. Analyzes FASTA-formatted input to compute per-residue and aggregate amino acid frequencies,支持和输出各种格式的组成统计信息
tags:
- protein
- amino-acid
- composition
- sequence-analysis
- bioinformatics
- statistics
author: AI-generated
source_url: https://github.com/bioinformatics-tools/aakomp
---

## Concepts

- **Input Format**: Accepts protein sequences in FASTA format (single or multiple sequences), supporting both DNA-encoded and standard amino acid letter codes. The tool parses sequence headers and ignores non-alphabetic characters
- **Composition Calculation**: Computes per-amino acid frequencies (counts and percentages), generating statistics for all 20 standard amino acids plus ambiguous residue codes (B, Z, X, J). Output includes both raw counts and normalized proportions
- **Output Modes**: Provides multiple output formats including plain text tables, CSV, JSON, and a compact summary format. Supports per-sequence reporting and aggregate statistics across all input sequences
- **Sequence Filtering**: Includes options to filter sequences by length range, exclude sequences with ambiguous residues, and handle case-insensitive input. Empty or invalid sequences are skipped with warnings

## Pitfalls

- **Invalid Characters**: Sequences containing non-amino acid characters (digits, special symbols) cause parsing errors or are silently ignored, leading to incomplete composition reports without warning messages
- **Case Sensitivity**: Failing to use consistent case (tool treats 'A' and 'a' as the same amino acid) may cause unexpected behavior in certain output modes requiring exact string matching
- **Empty Input Files**: Providing an empty input file or file with only headers produces no useful output, requiring explicit handling for downstream pipeline steps that expect result files
- **Ambiguous Residue Handling**: Ambiguous codes (B, Z, X, J) are often excluded from standard composition calculations unless explicitly enabled, leading to incomplete amino acid accounts in the output
- **Large File Processing**: Processing very large FASTA files without memory management options can cause system resource exhaustion, particularly when generating detailed per-sequence reports

## Examples

### Calculate basic amino acid composition from a protein FASTA file
**Args:** input.fasta --output composition.txt
**Explanation:** Reads protein sequences from input.fasta and writes amino acid counts and percentages to composition.txt in default table format

### Output composition in CSV format for downstream analysis
**Args:** input.fasta --format csv --out result.csv
**Explanation:** Generates CSV-formatted output with columns for amino acid type, count, and percentage, suitable for Excel or R import

### Generate JSON output for programmatic integration
**Args:** proteins.fasta --format json --out stats.json
**Explanation:** Produces JSON output containing nested composition objects usable in pipelines or web applications

### Analyze only sequences with 50-500 amino acids
**Args:** proteome.fasta --min-length 50 --max-length 500 --out filtered_composition.txt
**Explanation:** Filters input sequences by length before composition calculation, excluding very short peptides or long contigs from analysis

### Exclude sequences with ambiguous residues (B, Z, X)
**Args:** dataset.fasta --no-ambiguous --out clean_composition.txt
**Explanation:** Skips any sequence containing ambiguous amino acid codes, producing composition based only on fully resolved sequences

### Generate per-sequence summary for a multi-sequence file
**Args:** sequences.fasta --per-sequence --out individual_stats.txt
**Explanation:** Outputs composition statistics for each individual sequence in the input file rather than aggregate statistics only