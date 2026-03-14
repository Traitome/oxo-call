---
name: cnvkit
category: variant-calling
description: Copy number variant detection and visualization from targeted sequencing data (WES/panels)
tags: [cnv, copy-number, wes, targeted-sequencing, tumor, somatic, vcf]
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

## Pitfalls

- For WES, --targets (-t) BED file is required — WGS can run without it.
- Without matched normal, a pooled normal reference can be used but accuracy decreases.
- CNVkit requires all BAM files to be sorted and indexed.
- The access file (--access) restricts analysis to mappable regions — improves specificity.
- Integer copy number calling assumes diploid baseline — adjust with --purity and --ploidy for tumor.
- CNVkit is designed for targeted sequencing; for WGS long-read CNVs use other tools.

## Examples

### run CNVkit batch workflow for tumor-normal WES
**Args:** `batch tumor.bam --normal normal.bam --targets targets.bed --annotate refFlat.txt --fasta reference.fa --access access.hg38.bed --output-reference normal_reference.cnn --output-dir cnvkit_output/ -p 8`
**Explanation:** --normal matched normal BAM; --targets WES regions; -p 8 threads; outputs CNV profiles

### run CNVkit on tumor-only WES with pre-built reference
**Args:** `batch tumor.bam --reference normal_reference.cnn --targets targets.bed --output-dir cnvkit_tumor_only/ -p 4`
**Explanation:** --reference pre-built normal reference .cnn; use for tumor-only analysis

### visualize CNV scatter plot
**Args:** `scatter tumor.cnr -s tumor.cns -o cnv_scatter.pdf`
**Explanation:** -s segmentation file; -o output PDF with genome-wide CNV scatter plot

### call integer copy numbers from segments
**Args:** `call tumor.cns -o tumor.call.cns --center median --purity 0.8`
**Explanation:** --purity 0.8 sets tumor purity for copy number adjustment; --center median for centering
