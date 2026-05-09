---
name: annosine2
category: RNA Editing Analysis
description: A tool for detecting and quantifying A-to-I RNA editing events from genomic and transcriptomic sequencing data. Analyzes alignments to identify inosine sites, calculates editing levels, and generates annotation reports.
tags:
  - RNA editing
  - A-to-I conversion
  - genomics
  - transcriptomics
  - alignment analysis
author: AI-generated
source_Url: https://github.com/annosine2/annosine2
---

## Concepts

- **Editing Site Detection**: Annosine2 identifies A-to-I editing events by comparing genomic DNA and RNA-seq alignments, looking for mismatches where adenosine appears as guanine in cDNA reads. The tool requires both genome-aligned BAM files and transcript-aware alignments to distinguish true editing from sequencing errors or SNPs.

- **Editing Level Calculation**: The tool calculates editing ratios by dividing reads supporting the editing event (showing G at the position) by total reads covering the site. Sites with editing levels below the configurable threshold (default 0.1) are filtered as potential noise or incomplete editing.

- **Input Format Requirements**: Annosine2 accepts coordinate-sorted BAM files as input, with read group information used for sample identification. The tool requires a reference genome FASTA file for baseline comparison and a GTF/GFF3 annotation file to restrict analysis to annotated features.

- **Output Formats**: The tool generates BED files for editing site coordinates, CSV/TSV files for editing level quantification, and optional JSON output for programmatic downstream processing. Summary statistics include total edited bases, editing site density per feature, and strand specificity metrics.

## Pitfalls

- **Missing Strand Information**: Without proper strand-aware library preparation or configuration of the `--stranded` flag, Annosine2 cannot distinguish forward-strand editing from reverse-strand transcription artifacts, leading to false positives on antisense reads.

- **Insufficient Sequencing Depth**: Sites covered by fewer than 10 reads (default threshold) will produce unreliable editing estimates. Low-coverage regions generate artificially high or low editing percentages due to random sampling bias.

- **Mismatched Reference Genome**: Using a reference genome with different chromosome naming conventions or assembly versions than the alignment file causes all positions to be reported as unmapped, producing an empty output file without error messages.

- **Confusing Input File Order**: Providing the DNA BAM before the RNA BAM when using paired-analysis mode reverses the comparison logic, causing the tool to report editing in the wrong direction and generate biologically meaningless results.

## Examples

### Align sequencing reads to a reference genome
**Args:** `align --genome hg38.fa --reads sample_R1.fastq.gz --reads sample_R2.fastq.gz --output sample.bam --threads 16`
**Explanation:** Annosine2 performs alignment using its built-in aligner with default sensitive settings optimized for detecting single-nucleotide mismatches characteristic of RNA editing.

### Detect editing sites with standard filtering
**Args:** `detect --dna-bam normal_tissue.bam --rna-bam tumor_tissue.bam --ref hg38.fa --gtf annotations.gtf --min-coverage 20 --output editing_sites.bed`
**Explanation:** The detect subcommand identifies A-to-I editing events by comparing DNA and RNA alignments, filtering sites with fewer than 20 total reads to reduce noise.

### Calculate editing levels for specific genomic regions
**Args:** `quantify --bam tumor_sample.bam --sites editing_sites.bed --ref hg38.fa --min-reads 15 --strand both --format tsv --output quantified_levels.tsv`
**Explanation:** The quantify subcommand computes editing percentages at previously identified sites, considering only reads on both strands to account for unstranded library preparation.

### Generate a summary report of editing statistics
**Args:** `report --input editing_sites.bed --annotations annotations.gtf --feature-type gene --output summary.csv --include-density`
**Explanation:** The report subcommand aggregates editing metrics by genomic feature, calculating editing site density per kilobase for comparative analysis across samples or conditions.

### Run sensitivity-optimized detection for discovery studies
**Args:** `detect --dna-bam dna_control.bam --rna-bam rna_treatment.bam --ref hg38.fa --min-coverage 5 --min-editing-level 0.05 --strand forward --sensitive --output discovery_sites.bed`
**Explanation:** The sensitive mode uses relaxed thresholds suitable for discovery studies where true editing events may have low expression or partial editing, accepting higher false-positive rates for novel finding identification.