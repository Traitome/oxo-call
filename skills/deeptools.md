---
name: deeptools
category: epigenomics
description: "Tools for exploring deep sequencing data: signal coverage, correlation, heatmaps, and more"
tags: [chip-seq, atac-seq, rna-seq, bigwig, coverage, heatmap, correlation, epigenomics, bamcoverage, plotheatmap, mnase]
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
- --extendReads extends reads to fragment length; useful for ChIP-seq but NOT recommended for RNA-seq.
- --centerReads centers reads at fragment midpoint; sharpens signal around enriched regions.
- --smoothLength averages signal over larger window than binSize for smoothing.
- --MNase mode counts only 3 nucleotides at fragment center for nucleosome positioning.
- plotFingerprint assesses ChIP enrichment quality; separates signal from background.

## Pitfalls

- deeptools ARGS must start with a subcommand (bamCoverage, bamCompare, bigwigCompare, bigwigAverage, computeMatrix, computeMatrixOperations, plotHeatmap, plotProfile, plotEnrichment, multiBamSummary, multiBigwigSummary, plotCorrelation, plotPCA, plotFingerprint, bamPEFragmentSize, computeGCBias, correctGCBias, plotCoverage, estimateReadFiltering, alignmentSieve) — never with flags like -b, -o, -p. The subcommand ALWAYS comes first.
- BAM files must be sorted and indexed before any deeptools command.
- For ATAC-seq, use --ATACshift to correct for the +4/-5 Tn5 transposase insertion offset in bamCoverage.
- computeMatrix scale-regions and reference-point have different required arguments — check mode-specific parameters.
- plotHeatmap and plotProfile require the matrix.gz output from computeMatrix as input.
- Without --effectiveGenomeSize, RPGC normalization in bamCoverage will fail.
- bamCompare default pseudocount is 1 — affects log2 ratios when coverage is very low.
- --extendReads should NOT be used with RNA-seq data as it extends over spliced regions.
- --smoothLength must be larger than --binSize; smaller values are ignored.
- --MNase requires paired-end data and recommends --binSize 1 for nucleosome resolution.
- plotFingerprint --numberOfSamples times --binSize should be less than genome size.

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

### generate smoothed coverage track for ChIP-seq
**Args:** `bamCoverage -b chip.bam -o chip_smooth.bw --extendReads --smoothLength 150 --binSize 10 -p 8`
**Explanation:** --extendReads extends to fragment length; --smoothLength 150 averages over 150bp window for smoother signal

### centered reads for sharper peak signal
**Args:** `bamCoverage -b chip.bam -o chip_centered.bw --centerReads --binSize 10 -p 8`
**Explanation:** --centerReads centers reads at fragment midpoint; produces sharper signal around enriched regions

### MNase-seq nucleosome positioning track
**Args:** `bamCoverage -b mnase.bam -o nucleosomes.bw --MNase --binSize 1 -p 8`
**Explanation:** --MNase counts only 3 nucleotides at fragment center; --binSize 1 for nucleosome-resolution signal

### ChIP quality assessment with plotFingerprint
**Args:** `plotFingerprint -b chip.bam input.bam -plot fingerprint.png --outQualityMetrics metrics.txt -p 8`
**Explanation:** plotFingerprint assesses ChIP enrichment quality; --outQualityMetrics outputs quantitative metrics

### strand-specific RNA-seq coverage (forward strand)
**Args:** `bamCoverage -b rnaseq.bam -o forward.bw --filterRNAstrand forward --binSize 10 -p 8`
**Explanation:** --filterRNAstrand forward keeps reads from forward strand genes; useful for stranded RNA-seq analysis

### compute matrix with scale-regions mode
**Args:** `computeMatrix scale-regions -S chip.bw -R genes.bed -b 2000 -a 2000 --regionBodyLength 5000 -o matrix.gz -p 8`
**Explanation:** scale-regions stretches/shrinks all regions to same length; --regionBodyLength sets target region size

### bigWig comparison (log2 ratio)
**Args:** `bigwigCompare -b1 chip.bw -b2 input.bw -o log2ratio.bw --operation log2 --pseudocount 1 -p 8`
**Explanation:** bigwigCompare operates on existing bigWig files; --operation log2 computes log2(ChIP/Input) ratio
