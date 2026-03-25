---
name: velocyto
category: single-cell
description: RNA velocity analysis by counting spliced, unspliced, and ambiguous reads from scRNA-seq BAMs
tags: [rna-velocity, single-cell, splicing, 10x, smart-seq2, loom, scrnaseq]
author: oxo-call built-in
source_url: "http://velocyto.org/velocyto.py/tutorial/index.html"
---

## Concepts

- velocyto counts reads mapping to introns (unspliced) and exons (spliced) to estimate RNA velocity — the rate of change in gene expression.
- velocyto run10x is a convenience wrapper for 10x Chromium data using the CellRanger output directory structure.
- velocyto run-smartseq2 processes Smart-seq2 plate-based data where each cell is a separate BAM file.
- Output is a .loom file containing spliced (S), unspliced (U), and ambiguous (A) count matrices per cell.
- A GTF annotation file is required; it must match the genome used for alignment; gene models define exon/intron boundaries.
- velocyto run is the general-purpose command for any technology; --bcfile provides cell barcodes for droplet-based protocols.

## Pitfalls

- Input BAMs must be sorted by coordinate and indexed; name-sorted BAMs cause incorrect counting.
- For 10x data, velocyto run10x requires the CellRanger output directory, not just the BAM — it reads filtered barcodes automatically.
- The GTF must match the reference genome used for alignment exactly; mismatched chromosome names cause zero counts.
- Using an unfiltered barcode list inflates the cell count dramatically; use filtered_feature_bc_matrix barcodes.tsv.gz for 10x.
- Repetitive elements in the genome can be masked with a repeat masker BED (-m) to avoid artifactual unspliced counts.
- velocyto can be very slow on large datasets; consider using alevin-fry or STARsolo which compute spliced/unspliced during alignment.

## Examples

### run velocyto on a 10x CellRanger output directory
**Args:** `run10x -m repeat_masker.bed /path/to/cellranger_output genes.gtf`
**Explanation:** positional args: CellRanger dir and GTF; -m masks repeats to reduce spurious unspliced counts; uses filtered barcodes automatically

### run velocyto on a 10x BAM with explicit barcode file
**Args:** `run -b filtered_barcodes.tsv -o loom_output/ --samtools-threads 8 cellranger_output/possorted_genome_bam.bam genes.gtf`
**Explanation:** -b explicit filtered barcodes; -o output directory; positional args are BAM then GTF; reads CB and UB tags

### run velocyto on Smart-seq2 plate data
**Args:** `run-smartseq2 -o smartseq_loom/ -m repeat_masker.bed cells/*.bam genes.gtf`
**Explanation:** run-smartseq2 takes a glob of per-cell BAMs; -o output directory; creates one loom with all cells

### run velocyto on a general BAM with cell barcodes from inDrops
**Args:** `run -b barcodes.txt -e experiment_name -o loom_output/ sample.bam genes.gtf`
**Explanation:** -e sets the experiment name used as loom file prefix; -b provides cell barcodes file

### run velocyto with repeat masking and multiple threads
**Args:** `run10x -m hg38_rmsk.bed --samtools-threads 16 /path/to/cellranger_output Homo_sapiens.GRCh38.gtf`
**Explanation:** --samtools-threads 16 speeds up BAM reading; -m provides UCSC repeat masker BED to exclude repetitive loci

### run velocyto on multiple 10x samples with per-sample output loom files
**Args:** `run10x -m repeat_masker.bed -o sample2_loom/ /path/to/sample2_cellranger_output genes.gtf`
**Explanation:** run10x per sample with -o to set output directory; repeat independently for each sample; then merge loom files with loompy
