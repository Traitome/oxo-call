---
name: macs2
category: epigenomics
description: Model-based Analysis for ChIP-Seq — peak caller for ChIP-seq and ATAC-seq experiments
tags: [chip-seq, atac-seq, peak-calling, epigenomics, chromatin, histone, transcription-factor]
author: oxo-call built-in
source_url: "https://macs3-project.github.io/MACS/"
---

## Concepts

- MACS2 models the ChIP enrichment pattern to distinguish true peaks from background noise.
- Use 'macs2 callpeak' as the main subcommand; -t for treatment (ChIP/ATAC); -c for control (input/IgG).
- Use -g hs for human (hg19/hg38) or -g mm for mouse; these set effective genome sizes for q-value calculation.
- Use --broad for histone marks with broad enrichment (H3K27me3, H3K9me3); default is narrow peaks (TFs, H3K4me3).
- ATAC-seq: use --nomodel --shift -100 --extsize 200 for nucleosome-free regions; or MACS3 for native ATAC support.
- Output files: <name>_peaks.narrowPeak or <name>_peaks.broadPeak, <name>_summits.bed, <name>_peaks.xls.
- Use -q 0.05 (default) for FDR threshold or -p 1e-5 for p-value threshold.
- For paired-end ATAC-seq, use -f BAMPE to call peaks from fragment size distribution.

## Pitfalls

- Forgetting -c (input control) leads to high false positive rates in ChIP-seq — always include input/IgG control.
- Using -g for the wrong organism gives incorrect q-values — match genome size to your reference.
- For ATAC-seq, using default ChIP-seq parameters without --nomodel/shift adjustment misses NFR peaks.
- MACS2 expects single-end BAM by default; for paired-end BAM use -f BAMPE.
- The narrowPeak format columns 7-9 are enrichment and signalValue; column 10 is peak summit offset from start.
- Without --keep-dup auto, MACS2 removes duplicates — for ATAC-seq use --keep-dup all after separate dedup.

## Examples

### call narrow peaks from ChIP-seq data with input control
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample_chip -q 0.05 --outdir chip_peaks/`
**Explanation:** -t treatment; -c control; -g hs for human; -n output name prefix; -q FDR threshold

### call broad peaks for histone mark (H3K27me3) ChIP-seq
**Args:** `callpeak -t h3k27me3.bam -c input.bam -f BAM -g hs --broad --broad-cutoff 0.1 -n h3k27me3 --outdir broad_peaks/`
**Explanation:** --broad for histone marks with diffuse enrichment; --broad-cutoff 0.1 for broad peak FDR threshold

### call ATAC-seq peaks using nucleosome-free region model
**Args:** `callpeak -t atac.bam -f BAM -g hs --nomodel --shift -100 --extsize 200 -n atac_sample -q 0.05 --outdir atac_peaks/`
**Explanation:** --nomodel --shift -100 --extsize 200 is the standard ATAC-seq NFR peak calling setting

### call peaks from paired-end ATAC-seq BAM
**Args:** `callpeak -t atac_pe.bam -f BAMPE -g hs -n atac_pe_sample -q 0.05 --outdir atac_pe_peaks/`
**Explanation:** -f BAMPE uses actual fragment sizes from paired-end BAM; better for ATAC-seq peak calling

### call peaks without control for ATAC-seq open chromatin
**Args:** `callpeak -t atac.bam -f BAM -g hs --nomodel --shift -100 --extsize 200 --keep-dup all -n open_chromatin --outdir atac_out/`
**Explanation:** --keep-dup all prevents duplicate removal (do separately if needed); no -c for ATAC without control
