---
name: codan
category: Sequence Analysis / Codon Usage
description: EMBOSS tool for analyzing codon usage patterns in DNA sequences. Calculates codon frequencies, effective number of codons (ENC), codon bias indices, and statistical measures of codon usage variation across input sequences.
tags:
  - codon usage
  - codon bias
  - EMBOSS
  - sequence analysis
  - genomics
  - bioinformatics
  - effective number of codons
  - ENC
author: AI-generated
source_url: https://emboss.sourceforge.net/
---

## Concepts

- **Effective Number of Codons (ENC):** The core metric calculated by codan represents the total number of codons that would produce the same observed codon usage variance if all codons were used equally. ENC values range from 20 (maximum bias, only one codon per amino acid) to 61 (no bias, equal codon usage). Lower values indicate stronger codon bias.

- **Input Format Requirements:** codan accepts nucleotide sequences in standard formats (FASTA, EMBL, GenBank) via the `-sequence` parameter. The tool automatically translates codons to amino acids to determine codon usage. Input sequences must be in reading frame with start and stop codons present for accurate analysis.

- **Codon Usage Table:** The tool uses standard genetic code tables (default is the universal genetic code, Table 1) to translate nucleotide codons to amino acids. Custom codon usage data can be provided via `-data` parameter for non-universal codes (e.g., mitochondrial genomes).

## Pitfalls

- **Using amino acid sequences instead of nucleotide sequences:** Codan requires the original nucleotide sequences, not the translated amino acid sequence. Providing amino acid input will produce meaningless results since codon usage cannot be calculated without the triplet nucleotide information.

- **Ignoring frame shifts in input sequences:** If your nucleotide sequences contain untranslated regions (UTRs), introns, or are not properly frame-aligned, the codon translation will be incorrect. Pre-process sequences to extract only the coding regions (CDS) in the correct reading frame.

- **Assuming ENC alone describes codon bias:** ENC is a summary statistic but does not capture all aspects of codon usage patterns. The tool also outputs per-codon frequencies and position-specific information; ignoring these granular outputs may cause you to miss important biological signals about gene expression or selection.

## Examples

### Analyze codon usage in a single FASTA file containing coding sequences

**Args:** `-sequence input.fasta -outfile codon_results.txt`

**Explanation:** This runs a basic codon usage analysis on all nucleotide sequences in the input file, outputting statistics including ENC values and per-codon frequencies to the specified output file.

### Perform codon analysis using a specific genetic code table

**Args:** `-sequence mitochondrial_seqs.fasta -outfile mit_codons.txt -cfile mit_codon_usage.txt`

**Explanation:** Analyzes codon usage in mitochondrial sequences using the default universal code but writes a separate codon usage frequency table to the `-cfile` output for further analysis or comparison.

### Calculate ENC and save results to a dedicated codon frequency file

**Args:** `-sequence coding_regions.fasta -outfile enc_results.txt -cfile codon_freq.txt`

**Explanation:** Writes the main analysis results (including ENC values) to `enc_results.txt` and saves the complete per-codon frequency data to `codon_freq.txt` for downstream analysis or visualization.

### Analyze multiple short sequences as individual entries

**Args:** `-sequence orfs.fasta -outfile orf_analysis.txt -single`

**Explanation:** Treats each entry in the input file as a separate sequence for analysis rather than pooling all sequences together, giving individual ENC and codon usage statistics per ORF.

### Suppress standard output and only generate codon usage frequency file

**Args:** `-sequence genes.fasta -outfile /dev/null -cfile gene_codons.txt`

**Explanation:** Discards the standard analysis output (useful for scripting) while capturing only the per-gene codon usage frequency table for batch processing or integration with other tools.