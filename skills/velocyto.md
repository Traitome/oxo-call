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
**Explanation:** velocyto run10x subcommand; -m repeat_masker.bed masks repeats; /path/to/cellranger_output CellRanger directory; genes.gtf annotation GTF; uses filtered barcodes automatically

### run velocyto on a 10x BAM with explicit barcode file
**Args:** `run -b filtered_barcodes.tsv -o loom_output/ --samtools-threads 8 cellranger_output/possorted_genome_bam.bam genes.gtf`
**Explanation:** velocyto run subcommand; -b filtered_barcodes.tsv explicit barcodes; -o loom_output/ output directory; --samtools-threads 8 parallel BAM reading; cellranger_output/possorted_genome_bam.bam input BAM; genes.gtf annotation GTF; reads CB and UB tags

### run velocyto on Smart-seq2 plate data
**Args:** `run-smartseq2 -o smartseq_loom/ -m repeat_masker.bed cells/*.bam genes.gtf`
**Explanation:** velocyto run-smartseq2 subcommand; -o smartseq_loom/ output directory; -m repeat_masker.bed masks repeats; cells/*.bam glob of per-cell BAMs; genes.gtf annotation GTF; creates one loom file

### run velocyto on a general BAM with cell barcodes from inDrops
**Args:** `run -b barcodes.txt -e experiment_name -o loom_output/ sample.bam genes.gtf`
**Explanation:** velocyto run subcommand; -b barcodes.txt cell barcodes file; -e experiment_name loom file prefix; -o loom_output/ output directory; sample.bam input BAM; genes.gtf annotation GTF

### run velocyto with repeat masking and multiple threads
**Args:** `run10x -m hg38_rmsk.bed --samtools-threads 16 /path/to/cellranger_output Homo_sapiens.GRCh38.gtf`
**Explanation:** velocyto run10x subcommand; -m hg38_rmsk.bed repeat masker BED; --samtools-threads 16 speeds up BAM reading; /path/to/cellranger_output CellRanger directory; Homo_sapiens.GRCh38.gtf annotation GTF

### run velocyto on multiple 10x samples with per-sample output loom files
**Args:** `run10x -m repeat_masker.bed -o sample2_loom/ /path/to/sample2_cellranger_output genes.gtf`
**Explanation:** velocyto run10x subcommand; -m repeat_masker.bed masks repeats; -o sample2_loom/ output directory; /path/to/sample2_cellranger_output CellRanger directory; genes.gtf annotation GTF; repeat per sample then merge loom files

### run velocyto with custom barcode whitelist
**Args:** `run -b custom_barcodes.txt -o loom_output/ sample.bam genes.gtf`
**Explanation:** velocyto run subcommand; -b custom_barcodes.txt custom barcode list; -o loom_output/ output directory; sample.bam input BAM; genes.gtf annotation GTF; useful for non-standard protocols

### run velocyto on Drop-seq data
**Args:** `run -b dropseq_barcodes.txt -e dropseq_experiment sample.bam genes.gtf`
**Explanation:** velocyto run subcommand; -b dropseq_barcodes.txt explicit barcodes; -e dropseq_experiment experiment name; sample.bam input BAM; genes.gtf annotation GTF

### merge multiple loom files from different samples
**Args:** `loompy combine sample1.loom sample2.loom sample3.loom combined.loom`
**Explanation:** loompy combine command; sample1.loom sample2.loom sample3.loom input loom files; combined.loom output loom file; run after velocyto processing

### run velocyto with increased memory for large datasets
**Args:** `run10x -m repeat_masker.bed --samtools-memory 4000 /path/to/cellranger_output genes.gtf`
**Explanation:** velocyto run10x subcommand; -m repeat_masker.bed masks repeats; --samtools-memory 4000 increases memory buffer for BAM reading; /path/to/cellranger_output CellRanger directory; genes.gtf annotation GTF

### extract spliced counts only from loom file
**Args:** `python -c "import loompy; ds = loompy.connect('output.loom'); print(ds.layers['spliced'][:])"`
**Explanation:** python command; loompy.connect('output.loom') opens loom file; ds.layers['spliced'][:] extracts spliced count matrix
