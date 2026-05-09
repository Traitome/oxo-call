---
name: ceas
category: ChIP-seq Analysis
description: A tool for analyzing ChIP-seq data and annotating peaks relative to genomic features such as genes, promoters, and exons.
tags: chipseq, genomics, annotation, enrichment, peak-analysis
author: AI-generated
source_url: https://github.com/XieLab/CEAS
---

## Concepts

- **Input Format**: CEAS requires a BED file (6-column minimum: chrom, start, end, name, score, strand) containing ChIP-seq peaks or Enrichment Zones. The file must be sorted by chromosome andstart position for proper processing.
- **Genome Annotation**: CEAS needs a genome annotation file in refGene format (columns: gene name, chrom, strand, txStart, txEnd, cdsStart, cdsEnd, exonCount, exonStarts, exonEnds) either from built-in genomes (hg18, hg19, mm9) or a custom file built with ceas-build.
- **Built-in Genomes**: CEAS ships with three pre-built genomes (hg18, hg19 for human, mm9 for mouse). These can be specified directly without providing a separate annotation file.
- **Output Results**: CEAS generates summary statistics including peak distribution across genomic features (promoters, 5'UTR, coding exons, introns, 3'UTR, intergenic), pile-up profiles over gene bodies, and reports in both text and PDF formats.
- **Companion Binary ceas-build**: This tool builds custom genome annotation databases from refGene or similar annotation files, enabling CEAS to work with any organism or custom genome assembly.

## Pitfalls

- **Unsorted BED File**: Providing a BED file not sorted by chromosome and start position causes incorrect peak mapping and erroneous genomic feature assignments. Always sort BED files before running CEAS.
- **Missing Genome Annotation**: Running CEAS without specifying a built-in genome (--bg) or providing a custom annotation file (--db) results in a failure to annotate peaks relative to genes.
- **Incorrect Column Count in BED**: Using a BED file with fewer than 6 columns or misformatted coordinates leads to parsing errors and incomplete analysis results.
- **Memory Limits with Large Datasets**: Analyzing very large peak sets (hundreds of thousands of peaks) may require significant memory; insufficient RAM can cause the analysis to terminate prematurely.
- **Wrong Genome Assembly**: Using a genome annotation file that does not match the coordinate system of your BED file produces meaningless correlations between peaks and genomic features.

## Examples

### Annotate peaks using the human hg19 genome
**Args:** -g hg19 -b peaks.bed
**Explanation:** Runs CEAS with the built-in hg19 human genome annotation to assign peaks to genomic features without requiring an external annotation file.

### Analyze peaks with a custom genome annotation
**Args:** --db custom_genome.db -b peaks.bed -o results
**Explanation:** Uses a custom genome annotation database (created with ceas-build) instead of built-in genomes to annotate peaks for a non-standard organism or assembly.

### Generate PDF report with gene body enrichment
**Args:** -g hg19 -b peaks.bed --gw=5000 --genome-bin=50
**Explanation:** Calculates pile-up enrichment over gene bodies with a 5kb flanking window and 50 bins per gene, enabling visualization of peak distribution across gene length.

### Specify output directory for all results
**Args:** -g mm9 -b mouse_peaks.bed -o /home/user/ceas_output
**Explanation:** Runs CEAS on mouse mm9 genome and saves all output files (summary, enrichment tables, plots) to the specified directory rather than the current working directory.

### Run CEAS with manual annotation file
**Args:** --bg refGene_hg18.txt -b peaks.bed --name experiment1
**Explanation:** Provides a manually formatted genome annotation file (refGene format) as input instead of using built-in databases, with output files prefixed by "experiment1".