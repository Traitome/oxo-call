---
name: altair-mf
category: Sequence Analysis
description: A bioinformatics tool for discovering and analyzing conserved motifs in multi-fasta sequence datasets. Performs statistical enrichment analysis to identify statistically significant sequence patterns across DNA, RNA, or protein sequences.
tags: [motif-finding, sequence-analysis, fasta, bioinformatics, enrichment, pattern-discovery]
author: AI-generated
source_url: https://github.com/altair-mf/altair-mf
---

## Concepts

- **Input Format**: altair-mf accepts standard multi-fasta files (.fa, .fasta, .faa) as primary input, where sequences are delimited by lines starting with '>' followed by the sequence identifier and optional description.
- **Statistical Model**: The tool uses a hypergeometric distribution model to calculate motif enrichment p-values against a background distribution, typically derived from a random genomic model of the same nucleotide or amino acid composition.
- **Output Modes**: Results are provided in three formats: tabular text output (default), JSON for programmatic consumption, and HTML for interactive visualization of motif positions and enrichment scores.
- **Companion Binary**: The companion tool `altair-mf-build` constructs custom background models from user-provided FASTA files, enabling species-specific or locus-specific motif enrichment analysis.

## Pitfalls

- **Empty Sequence Filtering**: Sequences containing only ambiguous characters (N, X, -) are automatically filtered, which may reduce sample size unexpectedly and alter statistical significance if not explicitly logged.
- **Case Sensitivity**: DNA/RNA sequences are treated as case-insensitive by default, but mixed-case inputs may cause inconsistent motif matching if the motif pattern case does not match the input case convention.
- **Statistical Threshold Misinterpretation**: Setting e-value thresholds incorrectly may filter out biologically meaningful motifs or return excessive false positives; lower e-values indicate stricter significance criteria.
- **Background Model Mismatch**: Using an inappropriate background model (e.g., a generic random model for organism-specific analysis) leads to inflated enrichment scores for motifs that reflect compositional bias rather than functional conservation.

## Examples

### Identify motifs in a protein multi-fasta file
**Args:** -i proteins.faa -m "WDD[ES]" --evalue 0.01 --output-format json
**Explanation:** This searches for the motif WDD followed by either glutamic acid (E) or serine (S) in protein sequences, filtering results with statistical significance better than e-value 0.01 and outputting in JSON format.

### Find DNA motifs with custom background model
**Args:** -i promoters.fa -m "TATAAA" --background promoters_bg.fa --stat enrichment
**Explanation:** This discovers the classic TATA box motif in promoter sequences using a custom background model built from provided promoter sequences rather than a generic background.

### Generate HTML visualization report
**Args:** -i transcription_factors.fa -m "锌finger" --html-report motif_report.html
**Explanation:** This identifies zinc finger binding motifs in transcription factor sequences and produces an interactive HTML visualization showing motif positions and statistical enrichment.

### Run with organism-specific nucleotide composition
**Args:** -i yeast_genes.fa -m "GCWGCW" --species saccharomyces_cerevisiae --evalue 0.001
**Explanation:** This searches for the GC-rich motif pattern specific to yeast genes, using the yeast genome nucleotide composition as the statistical background for accurate enrichment calculation.

### Extract top 10 most significant motifs
**Args:** -i chip_peaks.fa --auto-motif --top-n 10 --output top_motifs.txt
**Explanation:** This performs de novo motif discovery without a predefined motif pattern, automatically extracting and ranking the top 10 statistically significant enriched motifs from ChIP-seq peaks.