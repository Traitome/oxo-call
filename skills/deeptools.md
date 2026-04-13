---
name: deeptools
category: epigenomics
description: "Tools for exploring deep sequencing data: signal coverage, correlation, heatmaps, and more"
tags: [chip-seq, atac-seq, rna-seq, bigwig, coverage, heatmap, correlation, epigenomics]
author: oxo-call built-in
source_url: "https://deeptools.readthedocs.io/"
---

## Concepts

- deeptools is a suite of tools; key commands: bamCoverage, bamCompare, computeMatrix, plotHeatmap, plotProfile, multiBamSummary, plotCorrelation.
- bamCoverage converts BAM to bigWig (normalized signal track); bamCompare creates log2 ratio between two BAMs.
- Use --normalizeUsing RPKM, CPM, BPM, or RPGC for coverage normalization; RPGC requires --effectiveGenomeSize.
- computeMatrix prepares signal matrix for visualization; use scale-regions or reference-point mode.
- plotHeatmap and plotProfile visualize the matrix from computeMatrix; they require the matrix output file.
- multiBamSummary computes read count correlation across samples; use with plotCorrelation and plotPCA.
- Use -p N for multi-threading across all deeptools commands; most tools output bigWig, matrix, or plots.
- --blackListFileName removes artifactual regions (e.g., ENCODE blacklist) from coverage computation.

## Pitfalls

- BAM files must be sorted and indexed before any deeptools command.
- For ATAC-seq, use --ATACshift to correct for the +4/-5 Tn5 transposase insertion offset in bamCoverage.
- computeMatrix scale-regions and reference-point have different required arguments — check mode-specific parameters.
- plotHeatmap and plotProfile require the matrix.gz output from computeMatrix as input.
- Without --effectiveGenomeSize, RPGC normalization in bamCoverage will fail.
- bamCompare default pseudocount is 1 — affects log2 ratios when coverage is very low.

## Examples

### generate normalized bigWig coverage track with bamCoverage
**Args:** `bamCoverage -b sorted.bam -o output.bw --normalizeUsing RPKM --binSize 10 -p 8`
**Explanation:** -b input BAM; -o output bigWig; --normalizeUsing RPKM normalizes by reads per kilobase per million; --binSize 10bp resolution

### create log2 ratio (ChIP/Input) bigWig track with bamCompare
**Args:** `bamCompare -b1 chip.bam -b2 input.bam -o chip_vs_input_log2.bw --normalizeUsing RPKM --binSize 10 -p 8`
**Explanation:** -b1 treatment; -b2 control; outputs log2(ChIP/Input) bigWig for visualization

### compute signal matrix around TSS with computeMatrix
**Args:** `computeMatrix reference-point -S chip.bw -R genes.bed --referencePoint TSS -b 3000 -a 3000 -o matrix.gz -p 8`
**Explanation:** -S bigWig input; -R regions BED/GTF; -b/-a bases before/after reference point; -o matrix for plotting

### plot heatmap of signal around genomic regions with plotHeatmap
**Args:** `plotHeatmap -m matrix.gz -out heatmap.png --colorMap RdBu_r --whatToShow 'heatmap and colorbar'`
**Explanation:** -m matrix from computeMatrix; --colorMap color scheme; outputs PNG heatmap

### compute read count correlation with multiBamSummary
**Args:** `multiBamSummary bins -b sample1.bam sample2.bam sample3.bam -o readCounts.npz -p 8`
**Explanation:** bins mode computes genome-wide 10kb bin counts; output .npz matrix for plotCorrelation and plotPCA

### generate ATAC-seq normalized bigWig with bamCoverage
**Args:** `bamCoverage -b atac_sorted.bam -o atac_signal.bw --ATACshift --normalizeUsing RPGC --effectiveGenomeSize 2913022398 --binSize 10 -p 8`
**Explanation:** --ATACshift corrects for Tn5 insertion offset; --RPGC normalization with hg38 effective genome size
