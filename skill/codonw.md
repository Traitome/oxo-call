---
name: codonw
category: Bioinformatics - Sequence Analysis - Codon Usage
description: A program for analyzing codon usage bias in DNA sequences, calculating indices like CAI (Codon Adaptation Index), ENC (Effective Number of Codons), GC content, and performing correspondence analysis to detect patterns of gene expression or evolutionary pressure.
tags: codon-usage, dna-analysis, gene-expression, cai, enc, bioinformatics, sequence-analysis
author: AI-generated
source_url: https://github.com/hyphaltip/codonw
---

## Concepts

- **Input Format**: codonw accepts DNA sequences in FASTA format (single or multi-sequence files). Sequences must be in frame with the coding region; incomplete codons at the end of sequences are automatically discarded from calculations.
- **Codon Usage Indices**: The tool calculates multiple metrics including CAI (Codon Adaptation Index) which requires a reference codon usage table (e.g., from highly expressed genes), ENC (Effective Number of Codons) measuring codon usage uniformity, GC content overall and at third codon positions (GC3s), and synonymous codon usage frequencies.
- **Output Options**: Results can be generated as human-readable text reports or machine-parseable CSV format. The tool also supports Correspondence Analysis (COA) to reduce codon usage patterns to principal coordinates for visualization and clustering.
- **Reference Tables**: For CAI calculations, codonw uses predefined codon usage tables for model organisms (E. coli, S. cerevisiae, C. elegans, etc.) or accepts custom reference tables. Without an appropriate reference, CAI values will be meaningless.

## Pitfalls

- **Unaligned or Frame-Shifted Sequences**: Submitting sequences that are not properly aligned to the coding frame causes incorrect codon identification, leading to entirely invalid results and misleading biological conclusions.
- **Mixed Reading Frames**: Sequences containing multiple reading frames mixed together (e.g., from genomic DNA with introns) produce nonsensical codon usage statistics since codons are extracted from incorrect positions.
- **Small Sequence Sets**: Analyzing fewer than 10-20 genes provides insufficient data for reliable statistical conclusions, particularly for correspondence analysis which requires adequate sample density.
- **Missing Stop Codons**: Stop codons (TAA, TAG, TGA) are automatically excluded from codon usage calculations; however, if your input sequences contain these at unexpected positions due to annotation errors, the codon count will be off.
- **Inconsistent Sequence Quality**: Sequences with ambiguous nucleotides (N, R, Y, etc.) or contamination from non-coding regions skew codon usage bias measurements significantly.

## Examples

### Calculate CAI for a set of gene sequences using an E. coli reference
**Args:** -coa -enc -finput.fasta -codon -refEcoli
**Explanation:** This runs multiple analyses including the Codon Adaptation Index using the E. coli reference table, correspondence analysis, and effective number of codons, outputting results for all sequences in input.fasta.

### Analyze GC content and GC3s only for quick composition check
**Args:** -gc -gc3s -finput.fasta -outfile results.txt
**Explanation:** This calculates overall GC content and GC at third codon positions without computing full codon usage metrics, useful for quick compositional analysis of coding sequences.

### Export all codon usage data to CSV for downstream statistical analysis
**Args:** -csv -finput.fasta -outfile codon_usage.csv
**Explanation:** This outputs detailed per-codon usage frequencies in CSV format, making the data suitable for import into R, Python, or other statistical environments for custom analyses.

### Generate a Correspondence Analysis plot for clustering genes by codon usage
**Args:** -coa -finput.fasta -out coords.txt
**Explanation:** This performs correspondence analysis on codon usage patterns and outputs principal coordinates for visualization, allowing identification of genes with similar codon usage bias.

### Calculate ENC for individual coding sequences to assess expression prediction potential
**Args:** -enc -finput.fasta -outfile enc_values.txt
**Explanation:** This computes the Effective Number of Codons for each sequence, where lower ENC values indicate stronger codon bias often correlated with higher predicted expression levels.