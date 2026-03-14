---
name: chromap
category: epigenomics
description: Ultrafast short-read aligner for single-cell ATAC-seq and ChIP-seq with built-in barcode processing
tags: [atac-seq, chip-seq, alignment, single-cell, barcode, epigenomics, fast]
author: oxo-call built-in
source_url: "https://github.com/haowenz/chromap"
---

## Concepts

- Chromap is an ultrafast aligner specifically designed for ATAC-seq, ChIP-seq, and scATAC-seq data.
- Use --preset atac for ATAC-seq; --preset chip for ChIP-seq; --preset hic for Hi-C data.
- Build index: chromap -i -r genome.fa -o genome.index
- Align: chromap --preset atac -x genome.index -r genome.fa -1 R1.fq -2 R2.fq -o output.bed
- For scATAC-seq with barcodes, use -b barcode.fq and --barcode-whitelist whitelist.txt.
- Chromap outputs in BED, SAM, or pairs format; use -q for minimum MAPQ filter.
- Chromap is 10-100x faster than BWA+Picard for ATAC-seq peak calling workflows.

## Pitfalls

- Chromap requires its own index (not BWA or Bowtie2 indices) — build with chromap -i.
- For ATAC-seq, use --preset atac to handle fragment length distribution properly.
- Chromap output is fragments/BED by default, not BAM — convert if BAM is required.
- For scATAC-seq, the barcode FASTQ (-b) must be correctly specified with position.
- Chromap deduplicates by default — use --no-remove-pcr-duplicates to disable.

## Examples

### build Chromap genome index
**Args:** `-i -r genome.fa -o genome.index`
**Explanation:** -i index mode; -r reference FASTA; -o index output file

### align paired-end ATAC-seq reads with Chromap
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -o fragments.bed -t 16`
**Explanation:** --preset atac; -x index; -r reference; -1/-2 paired reads; -o output BED; -t threads

### process single-cell ATAC-seq with barcodes
**Args:** `--preset atac -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -b barcode.fastq.gz --barcode-whitelist whitelist.txt -o scatac_fragments.bed -t 16`
**Explanation:** -b barcode FASTQ; --barcode-whitelist valid barcode list; outputs fragments per cell

### align ChIP-seq reads with Chromap
**Args:** `--preset chip -x genome.index -r genome.fa -1 R1.fastq.gz -2 R2.fastq.gz -o chip_aligned.bed -t 16`
**Explanation:** --preset chip for ChIP-seq alignment settings
