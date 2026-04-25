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
- --bdg outputs bedGraph files for visualization; use --SPMR for signal per million reads normalization.
- --trackline adds UCSC track headers to bedGraph files for browser visualization.
- --cutoff-analysis generates a table of peaks at different thresholds to help choose optimal cutoffs.
- Other subcommands: bdgpeakcall (peaks from bedGraph), bdgcmp (compare tracks), bdgdiff (differential peaks), filterdup (remove duplicates).
- --fe-cutoff sets fold enrichment cutoff for filtering peaks by signal-to-noise ratio.
- --min-length and --max-gap control minimum peak length and maximum gap between peaks for merging.
- --slocal/--llocal set small and large local lambda windows for background estimation.

## Pitfalls
- macs2 ARGS must start with a subcommand (callpeak, bdgpeakcall, bdgbroadcall, bdgcmp, bdgopt, cmbreps, bdgdiff, filterdup, predictd, pileup, randsample, refinepeak) — never with flags like -t, -c, -g. The subcommand ALWAYS comes first.
- Forgetting -c (input control) leads to high false positive rates in ChIP-seq — always include input/IgG control.
- Using -g for the wrong organism gives incorrect q-values — match genome size to your reference.
- For ATAC-seq, using default ChIP-seq parameters without --nomodel/shift adjustment misses NFR peaks.
- MACS2 expects single-end BAM by default; for paired-end BAM use -f BAMPE.
- The narrowPeak format columns 7-9 are enrichment and signalValue; column 10 is peak summit offset from start.
- Without --keep-dup auto, MACS2 removes duplicates — for ATAC-seq use --keep-dup all after separate dedup.
- --SPMR changes signal values but not p/q-value calculations; use raw counts for differential analysis.
- --broad-cutoff only applies to broad peak calling; narrow peaks still use -q or -p.
- --cutoff-analysis is slow; only use for initial parameter optimization, not production runs.
- -f BAMPE requires properly paired reads; unmapped or improperly paired reads are skipped.
- --nomodel disables model building; you must provide --extsize for fragment extension.
- Default --keep-dup 1 keeps only one duplicate; use --keep-dup all if duplicates already removed externally.

## Examples

### call narrow peaks from ChIP-seq data with input control
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample_chip -q 0.05 --outdir chip_peaks/`
**Explanation:** macs2 callpeak subcommand; -t chip.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; -n sample_chip output name; -q 0.05 FDR threshold; --outdir chip_peaks/ output directory

### call broad peaks for histone mark (H3K27me3) ChIP-seq
**Args:** `callpeak -t h3k27me3.bam -c input.bam -f BAM -g hs --broad --broad-cutoff 0.1 -n h3k27me3 --outdir broad_peaks/`
**Explanation:** macs2 callpeak subcommand; -t h3k27me3.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; --broad broad peak mode; --broad-cutoff 0.1 FDR threshold; -n h3k27me3 output name; --outdir broad_peaks/ output directory

### call ATAC-seq peaks using nucleosome-free region model
**Args:** `callpeak -t atac.bam -f BAM -g hs --nomodel --shift -100 --extsize 200 -n atac_sample -q 0.05 --outdir atac_peaks/`
**Explanation:** macs2 callpeak subcommand; -t atac.bam treatment BAM; -f BAM format; -g hs human genome size; --nomodel disable model; --shift -100 shift reads; --extsize 200 extension size; -n atac_sample output name; -q 0.05 FDR threshold; --outdir atac_peaks/ output directory

### call peaks from paired-end ATAC-seq BAM
**Args:** `callpeak -t atac_pe.bam -f BAMPE -g hs -n atac_pe_sample -q 0.05 --outdir atac_pe_peaks/`
**Explanation:** macs2 callpeak subcommand; -t atac_pe.bam treatment BAM; -f BAMPE paired-end format; -g hs human genome size; -n atac_pe_sample output name; -q 0.05 FDR threshold; --outdir atac_pe_peaks/ output directory

### call peaks without control for ATAC-seq open chromatin
**Args:** `callpeak -t atac.bam -f BAM -g hs --nomodel --shift -100 --extsize 200 --keep-dup all -n open_chromatin --outdir atac_out/`
**Explanation:** macs2 callpeak subcommand; -t atac.bam treatment BAM; -f BAM format; -g hs human genome size; --nomodel disable model; --shift -100 shift reads; --extsize 200 extension size; --keep-dup all keep duplicates; -n open_chromatin output name; --outdir atac_out/ output directory

### generate bedGraph files for UCSC genome browser visualization
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample --bdg --SPMR --trackline --outdir tracks/`
**Explanation:** macs2 callpeak subcommand; -t chip.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; -n sample output name; --bdg output bedGraph; --SPMR signal per million reads; --trackline add UCSC headers; --outdir tracks/ output directory

### analyze optimal cutoff thresholds for peak calling
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample --cutoff-analysis --outdir analysis/`
**Explanation:** macs2 callpeak subcommand; -t chip.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; -n sample output name; --cutoff-analysis analyze thresholds; --outdir analysis/ output directory

### call peaks with fold enrichment filtering
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample --fe-cutoff 2.0 -q 0.01 --outdir filtered_peaks/`
**Explanation:** macs2 callpeak subcommand; -t chip.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; -n sample output name; --fe-cutoff 2.0 fold enrichment; -q 0.01 FDR threshold; --outdir filtered_peaks/ output directory

### call peaks with custom minimum length and gap
**Args:** `callpeak -t chip.bam -c input.bam -f BAM -g hs -n sample --min-length 200 --max-gap 100 --outdir custom_peaks/`
**Explanation:** macs2 callpeak subcommand; -t chip.bam treatment BAM; -c input.bam control BAM; -f BAM format; -g hs human genome size; -n sample output name; --min-length 200 minimum peak size; --max-gap 100 merge gap; --outdir custom_peaks/ output directory

### predict fragment size from alignment data
**Args:** `predictd -i chip.bam -f BAM -g hs --outdir prediction/`
**Explanation:** macs2 predictd subcommand; -i chip.bam input BAM; -f BAM format; -g hs human genome size; --outdir prediction/ output directory

### remove duplicate reads and convert to BED
**Args:** `filterdup -i chip.bam -f BAM --keep-dup 1 -o chip_dedup.bed`
**Explanation:** macs2 filterdup subcommand; -i chip.bam input BAM; -f BAM format; --keep-dup 1 keep one duplicate; -o chip_dedup.bed output BED

### call peaks from existing bedGraph signal track
**Args:** `bdgpeakcall -i signal.bdg -c 2.0 -l 200 -g 100 -o peaks.bed`
**Explanation:** macs2 bdgpeakcall subcommand; -i signal.bdg input bedGraph; -c 2.0 cutoff; -l 200 minimum length; -g 100 maximum gap; -o peaks.bed output BED

### compare two signal tracks (ChIP vs input)
**Args:** `bdgcmp -t chip.bdg -c input.bdg -m qpois -o chip_vs_input.bdg`
**Explanation:** macs2 bdgcmp subcommand; -t chip.bdg treatment bedGraph; -c input.bdg control bedGraph; -m qpois method; -o chip_vs_input.bdg output bedGraph
