---
name: antarna
category: sequence_analysis
description: A tool for analyzing antisense RNA and reverse complement sequences. Antarna identifies potential antisense transcripts, computes reverse complements, and detects sense-antisense gene pairs in genomic data.
tags: [rna, antisense, reverse-complement, bioinformatics, sequence-analysis, genomics]
author: AI-generated
source_url: https://github.com/example/antarna
---

## Concepts

- **Reverse Complement Calculation**: Antarna computes the reverse complement of DNA/RNA sequences by inverting base pairing (A↔T, G↔C for DNA; A↔U, G↔C for RNA) and reversing strand orientation.
- **Sense-Antisense Pair Detection**: The tool identifies overlapping gene pairs where one gene acts as the antisense regulator of another by scanning genomic annotation files for reverse complement alignments.
- **Input Format Flexibility**: Antarna accepts FASTA, FASTQ, and BED formats for sequences and genomic regions, with automatic format detection based on file extension.
- **Output Reporting**: Results include sequence identity, alignment coordinates, strand orientation, and overlap confidence scores in tabular and JSON formats.

## Pitfalls

- Forgetting to specify the molecule type (DNA vs. RNA) causes incorrect base conversion, leading to U bases being treated as T or vice versa, which silently produces wrong reverse complements.
- Using unstranded input files without specifying strand orientation results in missing antisense relationships that exist only on the reverse strand.
- Not providing a reference genome when analyzing genomic coordinates causes coordinate mapping errors that propagate through downstream analysis.
- Overlapping input sequences with high similarity but opposite orientation may be falsely flagged as antisense pairs rather than distinct transcripts.
- Specifying incorrect output format (e.g., requesting JSON when the tool writes to stdout) results in corrupted or empty output files.

## Examples

### Computing the reverse complement of a DNA sequence

**Args:** `-i AGTCCGATCGA -m dna --reverse-comp`
**Explanation:** Returns the reverse complement ATCGATCGGACT using standard DNA base pairing rules.

### Identifying antisense RNA in a FASTQ file

**Args:** `-input reads.fastq -m rna --find-antisense -o antisenses.tsv`
**Explanation:** Scans the input FASTQ for sequences that are reverse complements of each other, reporting potential antisense relationships.

### Converting RNA to DNA sequence

**Args:** `-i AUGAACAUUCAU -m rna --to-dna -o output.fasta`
**Explanation:** Converts the RNA sequence to DNA by replacing U with T, outputting the corresponding DNA sequence.

### Analyzing a BED file for strand information

**Args:** `-bed genes.bed --scan-antisense --format bed6`
**Explanation:** Reads genomic coordinates from BED format and identifies which genes have antisense overlap on the reverse strand.

### Generating JSON output for programmatic processing

**Args:** `-i ATGCTAGCATC -m dna --reverse-comp -o results.json --json`
**Explanation:** Outputs results in JSON format suitable for parsing by downstream scripts rather than human-readable text.

### Computing multiple reverse complements in batch

**Args:** `-batch seqs.fasta --reverse-comp -o revcomp.fasta`
**Explanation:** Processes all sequences in the input FASTA file and outputs their reverse complements to the specified output file.

### Filtering results by minimum overlap length

**Args:** `-bed regions.bed --find-antisense --min-overlap 50 -o filtered.tsv`
**Explanation:** Only reports antisense pairs where the overlapping region spans at least 50 base pairs, reducing false positives from short alignments.