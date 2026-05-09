---
name: constava
category: sequence_analysis
description: A command-line tool for calculating sequence conservation scores across genomic regions using multiple alignment methods. Constava accepts aligned sequences in FASTA format or genomic coordinates in BED format, computes per-position conservation metrics, and outputs results in standard bioinformatics formats.
tags:
  - conservation
  - sequence-analysis
  - genomics
  - multiple-alignment
  - bedtools
  - variant-annotation
author: AI-generated
source_url: https://github.com/constava/constava
---

## Concepts

- **Input flexibility**: Constava accepts two primary input types — pre-aligned sequences in multi-FASTA format (with `--fasta`) or genomic coordinates in BED format (with `--bed`). For BED inputs, the tool automatically retrieves reference sequences from a supplied genome assembly.
- **Conservation algorithms**: The tool computes conservation using three methods: phastCons (probabilistic model), phyloP (pairwise likelihood ratios), and sinke (simple_kmer-based entropy). Use `--method phastCons` for standard conservation or `--method phyloP` for accelerated evolutionary rate estimation.
- **Output formats**: Results export to CSV (default), BEDGraph (with `--out-format bedgraph`), or JSON (with `--out-format json`). The CSV output includes columns: chrom, start, end, ref, alt, score, and rank. BEDGraph format enables direct visualization in genome browsers.

## Pitfalls

- **Providing only a single sequence**: Constava requires multiple aligned sequences to compute conservation. Using `--fasta` with a single sequence file will error with "Insufficient sequences for conservation calculation — minimum 3 required." Use synthetic outgroups or related species to meet this requirement.
- **Mismatched chromosome naming**: Genome assemblies use different naming Conventions (chr1 vs 1, chrM vs MT). Supplying a BED file with "chr1" notation to an assembly indexed as "1" causes zero-length output. Use `--genome` to specify the exact assembly and verify contig names beforehand.
- **Ignoring strand orientation**: For BED inputs with gene features, failing to set `--strand` treats both + and - orientations identically, inverting conservation scores for antisense genes. This leads to incorrect downstream variant effect interpretations. Always specify strand when analyzing coding regions.

## Examples

### Calculate conservation for a multi-FASTA alignment
**Args:** `--fasta alignments/mammal_fox4.fa --method phastCons --out conservation_scores.csv`
**Explanation:** This runs conservation analysis on a pre-aligned multi-FASTA file using the phastCons probabilistic model, outputting per-position scores to a CSV file for downstream variant scoring.

### Analyze a BED region with strand specificity
**Args:** `--bed genes/exon21.bed --genome hg38 --strand + --method phyloP --out exon21_conservation.csv`
**Explanation:** This processes the genomic regions in exon21.bed against the hg38 assembly using only the forward strand and phyloP scoring, filtering results to the sense orientation before writing output.

### Export results in BEDGraph format
**Args:** `--fasta alignments/primates.fa --out-format bedgraph --out visualized.bedgraph`
**Explanation:** This converts conservation scores directly to BEDGraph format, enabling immediate visualization in UCSC Genome Browser or IGV without additional liftOver or format conversion steps.

### Use k-mer entropy scoring method
**Args:** `--fasta alignments/virus_genomes.fa --method sinke --window 11 --out entropy_scores.csv`
**Explanation:** This applies the simple k-mer entropy method with an 11-base window, useful for rapidly scanning viral sequences where probabilistic models lack training data.

### Process multiple BED files in batch
**Args:** `--bed "regions/*.bed" --genome mm10 --method phastCons --out-batch conservation_results/`
**Explanation:** This iterates over all BED files in the regions directory, computing conservation independently for each and writing separate output files to the specified directory with inherited filenames.