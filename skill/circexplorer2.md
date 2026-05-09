---
name: CircExplorer2
category: RNA-Seq Analysis / Circular RNA Detection
description: A comprehensive tool for detecting, quantifying, and annotating circular RNAs from RNA-seq data through backspliced junction analysis.
tags:
  - circRNA
  - RNA-seq
  - backspliced junction
  - circular RNA detection
  - splicing
  - non-canonical splicing
author: AI-generated
source_url: https://github.com/YangLab/CircExplorer2
---

## Concepts

- CircExplorer2 detects circular RNAs by identifying backspliced junction (BSJ) reads—reads where one end aligns to a downstream genomic coordinate while the other end aligns to an upstream coordinate in the reverse orientation, creating a junction that spans the circular RNA's back-splice point.
- Input files include aligned RNA-seq data in SAM/BAM format, a reference genome in FASTA format, and optionally a gene annotation file in GTF format for promoter and gene body overlap analysis during the enrichment step.
- The tool operates in multiple stages: `circExplorer2` for detection with flexible back-splicing patterns, `circExplorer2-merge` for merging replicate results, and `circexplorer2- annotate` for functional annotation against reference databases like RefSeq.
- Output formats include BED files for genomic coordinates, tab-delimited files with detection statistics, and fusion-specific outputs when analyzing fusion junction candidates.
- The `--filter` option enables post-detection filtering by requiring a minimum number of back-spliced junction reads or spanning reads to classify a candidate as a high-confidence circular RNA.

## Pitfalls

- Specifying a mismatched genome build between the RNA-seq alignment file and the reference genome FASTA will cause alignment errors and produce incorrect BSJ coordinates, leading to false positive or false negative circRNA calls.
- Omitting the `--geneanno` GTF file when running detection causes the tool to skip promoter and coding sequence overlap analysis, reducing the biological interpretability of detected circRNAs.
- Setting `--low-confidence` with overly permissive thresholds (e.g., read count of 1) floods the output with sequencing artifacts and low-abundance candidates that are unlikely to represent true circular RNAs.
- Forgetting to sort and index the input BAM file with `samtools sort` and `samtools index` prior to detection causes the tool to fail or produce incomplete results since CircExplorer2 requires positional-sorted and indexed BAM files.
- Using RNA-seq data from ribosome-depleted protocols (like RNase R treatment) without adjusting expectations for extremely high circRNA abundance leads to misinterpreted expression values when compared to standard poly(A)-selected libraries.

## Examples

### Detect circular RNAs from a BAM file using a reference genome
**Args:** `detect -b test.bam -g hg38.fa -o circRNA_calls.bed`
**Explanation:** This command runs the core circRNA detection algorithm on a sorted BAM file, aligning reads against the hg38 reference genome and outputting all candidate circular RNAs with their backspliced junction coordinates in BED format.

### Detect circRNAs with gene annotation for promoter overlap analysis
**Args:** `detect -b test.bam -g hg38.fa -gtf gene_annotation.gtf -o circRNA_annotated.bed`
**Explanation:** Including the GTF annotation file enables CircExplorer2 to analyze whether circRNA back-splice points overlap with annotated promoter regions, providing biological context for each detected circular RNA.

### Filter results to retain only high-confidence circular RNAs
**Args:** `filter -b circRNA_annotated.bed -l 2 -s 2 -o high_confidence_circrnas.bed`
**Explanation:** The filter step removes candidates with fewer than 2 back-spliced junction reads and fewer than 2 spanning reads, retaining only circular RNAs supported by multiple independent reads for downstream validation.

### Merge circRNA calls from multiple replicate BAM files
**Args:** `merge -d replicate_directory/ -o merged_circrnas.bed -l 3`
**Explanation:** This command processes all BAM files in the specified directory, identifies common and unique circRNA candidates across replicates, and outputs only those detected in at least 3 of the replicates, accounting for biological variability.

### Annotate circular RNAs against a reference gene database
**Args:** `annotate -b high_confidence_circrnas.bed -gtf gene_annotation.gtf -o annotated_functions.bed`
**Explanation:** The annotate step assigns host gene identity, overlapping exons, and gene body positions to each circular RNA entry, enabling interpretation of which annotated genes produce the detected circular transcripts.