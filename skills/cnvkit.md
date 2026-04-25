---
name: cnvkit
category: variant-calling
description: Copy number variant detection and visualization from targeted sequencing data (WES/panels)
tags: [cnv, copy-number, wes, targeted-sequencing, tumor, somatic, vcf, cbs, segmentation]
author: oxo-call built-in
source_url: "https://cnvkit.readthedocs.io/"
---

## Concepts

- CNVkit detects copy number variations (CNVs) from targeted sequencing (WES, panel) or WGS data.
- Main workflow: cnvkit.py batch for end-to-end processing of tumor (+/- normal) BAM files.
- The batch command handles: coverage calculation, reference creation, CNV calling, and segmentation.
- Use -n for normal BAMs; -t for target BED (WES capture regions); -f for reference FASTA.
- CNVkit outputs: .cnr (per-bin coverage ratios), .cns (segments), .call.cns (copy number calls).
- Use cnvkit.py scatter and cnvkit.py diagram for visualization.
- For somatic CNV calling, use matched normal (--normal/-n) for best accuracy.
- Segment with CBS (default) or other algorithms; call with cnvkit.py call for integer copy numbers.
- --drop-low-coverage removes bins with very low coverage to avoid false-positive deletions in poor-quality tumor samples.
- --segment-method supports CBS, flasso, haar, hmm, hmm-tumor, hmm-germline algorithms.
- --drop-outliers filters extreme outlier bins before segmentation.
- -y/--male-reference assumes male reference for sex chromosome handling.
- --smooth-cbs applies additional smoothing before CBS segmentation for increased sensitivity.

## Pitfalls

- For WES, --targets (-t) BED file is required — WGS can run without it.
- Without matched normal, a pooled normal reference can be used but accuracy decreases.
- CNVkit requires all BAM files to be sorted and indexed.
- The access file (--access) restricts analysis to mappable regions — improves specificity.
- Integer copy number calling assumes diploid baseline — adjust with --purity and --ploidy for tumor.
- CNVkit is designed for targeted sequencing; for WGS long-read CNVs use other tools.
- CNVkit has subcommands (batch, target, coverage, segment, call, scatter, etc.); use subcommand before flags.
- --drop-low-coverage is essential for poor-quality tumor samples to avoid false-positive deletions.
- CBS segmentation requires R and Rscript installed; specify path with --rscript-path if non-standard.
- --segment-method hmm-tumor and hmm-germline are specialized for respective sample types.

## Examples

### run CNVkit batch workflow for tumor-normal WES
**Args:** `batch tumor.bam --normal normal.bam --targets targets.bed --annotate refFlat.txt --fasta reference.fa --access access.hg38.bed --output-reference normal_reference.cnn --output-dir cnvkit_output/ -p 8`
**Explanation:** cnvkit.py batch subcommand; tumor.bam tumor BAM input; --normal normal.bam matched normal BAM; --targets targets.bed WES capture regions; --annotate refFlat.txt gene annotation; --fasta reference.fa reference genome; --access access.hg38.bed mappable regions; --output-reference normal_reference.cnn output reference; --output-dir cnvkit_output/ output directory; -p 8 threads; outputs CNV profiles

### run CNVkit on tumor-only WES with pre-built reference
**Args:** `batch tumor.bam --reference normal_reference.cnn --targets targets.bed --output-dir cnvkit_tumor_only/ -p 4`
**Explanation:** cnvkit.py batch subcommand; tumor.bam tumor BAM input; --reference normal_reference.cnn pre-built normal reference .cnn; --targets targets.bed WES capture regions; --output-dir cnvkit_tumor_only/ output directory; -p 4 threads; use for tumor-only analysis

### visualize CNV scatter plot
**Args:** `scatter tumor.cnr -s tumor.cns -o cnv_scatter.pdf`
**Explanation:** cnvkit.py scatter subcommand; tumor.cnr coverage ratios input; -s tumor.cns segmentation file; -o cnv_scatter.pdf output PDF with genome-wide CNV scatter plot

### call integer copy numbers from segments
**Args:** `call tumor.cns -o tumor.call.cns --center median --purity 0.8`
**Explanation:** cnvkit.py call subcommand; tumor.cns segment file input; -o tumor.call.cns output file; --purity 0.8 sets tumor purity for copy number adjustment; --center median for centering

### run batch with drop-low-coverage for poor-quality tumor
**Args:** `batch tumor.bam --normal normal.bam --targets targets.bed --fasta reference.fa --drop-low-coverage --output-dir cnvkit_output/ -p 8`
**Explanation:** cnvkit.py batch subcommand; tumor.bam tumor BAM input; --normal normal.bam matched normal BAM; --targets targets.bed WES capture regions; --fasta reference.fa reference genome; --drop-low-coverage removes bins with very low coverage; --output-dir cnvkit_output/ output directory; -p 8 threads; essential for poor-quality tumor samples to avoid false-positive deletions

### segment with alternative method (HaarSeg)
**Args:** `segment tumor.cnr -o tumor.cns -m haar --drop-outliers 5`
**Explanation:** cnvkit.py segment subcommand; tumor.cnr coverage ratios input; -o tumor.cns output segment file; -m haar uses HaarSeg algorithm (faster than CBS); --drop-outliers 5 filters extreme outliers

### create heatmap for multiple samples
**Args:** `heatmap sample1.cns sample2.cns sample3.cns -o cnv_heatmap.pdf`
**Explanation:** cnvkit.py heatmap subcommand; sample1.cns sample2.cns sample3.cns input segment files; -o cnv_heatmap.pdf output PDF; plots CNV profiles for multiple samples; useful for cohort visualization

### identify genes with copy number alterations
**Args:** `genemetrics tumor.cns -t -m 0.3 -o gainloss_genes.txt`
**Explanation:** cnvkit.py genemetrics subcommand; tumor.cns segment file input; -t for targets only; -m 0.3 minimum log2 threshold; -o gainloss_genes.txt output file; identifies genes with significant CNAs

### run WGS analysis without target BED
**Args:** `batch tumor.bam --normal normal.bam --fasta reference.fa --method wgs --output-dir cnvkit_wgs/ -p 8`
**Explanation:** cnvkit.py batch subcommand; tumor.bam tumor BAM input; --normal normal.bam matched normal BAM; --fasta reference.fa reference genome; --method wgs for whole genome sequencing; --output-dir cnvkit_wgs/ output directory; -p 8 threads; no --targets required for WGS

### segment with VCF for allele-specific analysis
**Args:** `segment tumor.cnr -o tumor.cns -v variants.vcf --sample-id tumor_sample --normal-id normal_sample`
**Explanation:** cnvkit.py segment subcommand; tumor.cnr coverage ratios input; -o tumor.cns output segment file; -v variants.vcf VCF input for allele-specific segmentation using B-allele frequencies; --sample-id tumor_sample sample identifier; --normal-id normal_sample normal identifier in VCF
