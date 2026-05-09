---
name: asqcan
category: Genomics / Copy Number Analysis
description: Allele-specific copy number alteration (AS-CNA) calling tool for genomic data. asqcan analyzes read depth and allele frequency signatures across genomic windows to identify somatic and germline copy number alterations, distinguishes between copy number loss of heterozygosity (LOH) and copy number variants (CNVs), and quantifies tumor purity and ploidy estimates from genomic sequencing data.
tags:
  - copy-number-analysis
  - allele-specific
  - genomics
  - somatic-cnv
  - read-depth
  - lohh
  - tumor-purity
  - ploidy
  - as-cna
  - sequencing
author: AI-generated
source_url: https://github.com/asqcan/asqcan
---

## Concepts

- asqcan operates on genomic windows by computing read coverage and heterozygous SNP allele frequencies to infer per-window copy number states and allelic imbalance, enabling detection of focal amplifications, deletions, LOH events, and whole-genome doubling from BAM or CRAM alignment files.
- Input requires a tumor (or test) sample alignment file plus an optional matched-normal BAM/CRAM; the normal sample is used to establish the germline heterozygous SNP baseline and dramatically improves specificity by filtering out common polymorphisms.
- The tool segments the genome into variable-length bins or uses fixed-size windows (configurable via `--bin-size`); smaller bins increase resolution but raise noise, while larger bins improve robustness in low-coverage data.
- Output includes a tab-delimited CNV calls table with chromosome, start, end, copy number state (0–5+), B-allele frequency (BAF) deviation, log2 copy ratio, and confidence scores; optionally emits a segmentation plot in SVG/PNG format.
- Tumor purity and ploidy are estimated from the distribution of BAF deviations and log2 ratios across all bins using an expectation-maximization (EM) algorithm; these estimates feed back into corrected copy number calling.

## Pitfalls

- Running asqcan without a matched-normal sample causes the tool to fall back to population allele frequency priors, which increases false-positive CNV calls for common germline variants that resemble somatic copy number events.
- Using an excessively small `--bin-size` on low-coverage sequencing (e.g., WGS below 10×) generates noisy read counts per bin, leading to spurious copy number calls and unreliable BAF measurements.
- Failing to specify the correct `--genome` (reference) build causes mismatched chromosome naming conventions between the BAM header and the built-in SNP database, resulting in empty or incorrect output with no explicit error message.
- Ignoring the `--min-snps` threshold: if fewer heterozygous SNPs fall within a bin than the minimum count, asqcan skips that bin entirely, creating gaps in output that can be mistaken for copy number neutral regions.
- Not validating tumor purity estimates from asqcan's summary metrics independently (e.g., via ABSOLUTE or ASCAT) risks propagating inaccurate purity values into downstream copy number state interpretation and clinical reporting.

## Examples

### Call copy number alterations from a tumor BAM using built-in SNP priors
**Args:** `--bam tumor_sample.bam --genome hg38 --out-prefix tumor_cna`
**Explanation:** Runs asqcan on a single tumor alignment file without a matched normal, relying on population SNP allele frequencies as the heterozygous baseline for allele-specific copy number inference.

### Call AS-CNAs from tumor BAM with a matched-normal BAM for improved specificity
**Args:** `--bam tumor_sample.bam --normal normal_sample.bam --genome hg38 --out-prefix paired_cna`
**Explanation:** Uses the normal sample to establish the true germline heterozygous SNP allele frequencies, subtracting germline signals to reveal somatic allelic imbalances more accurately.

### Adjust bin size to 500 bp for higher-resolution focal alteration detection
**Args:** `--bam tumor_sample.bam --genome hg38 --bin-size 500 --out-prefix high_res_cna`
**Explanation:** Reduces the genomic window length to 500 base pairs, increasing spatial resolution to detect small focal amplifications or deletions that would be averaged out in larger bins.

### Increase minimum SNP count per bin to reduce noise in low-coverage data
**Args:** `--bam tumor_sample.bam --genome hg38 --min-snps 10 --out-prefix robust_cna`
**Explanation:** Sets the minimum heterozygous SNP count per bin to 10, causing bins with fewer informative SNPs to be excluded from analysis, which reduces false-positive calls in low-coverage or repetitive genomic regions.

### Export segmentation plot and filtered CNV calls to BED/SEG format
**Args:** `--bam tumor_sample.bam --genome hg38 --seg-output tumor_cna.seg --plot tumor_cna_plot --out-prefix filtered_cna`
**Explanation:** Produces a SEG-format file compatible with genome visualization tools and generates a PNG plot of the segmented copy number profile and BAF track for manual review.

### Call CNAs from multiple tumor BAM files in batch mode
**Args:** `--bam-list batch_bams.txt --genome hg38 --out-dir batch_output --seg-output --plot`
**Explanation:** Processes multiple tumor samples listed in a text file in a single run, writing per-sample results to the specified output directory with optional SEG and plot outputs for each sample.