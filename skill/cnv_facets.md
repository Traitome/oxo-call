---
name: cnv_facets
category: variant-analysis/copy-number
description: A high-sensitivity tool for detecting copy number variants (CNVs) and loss-of-heterozygosity (LOH) from tumor-normal BAM sequencing data using read depth and B-allele frequency analysis.
tags: [copy-number-variants, tumor-normal-analysis, facets-algorithm, bam, segmentation, loh, somatic-cnvs]
author: AI-Generated
source_url: https://github.com/mskcc/facets
---

## Concepts

- cnv_facets analyzes read depth and B-allele frequency (BAF) signals from paired tumor-normal BAM files to segment the genome into regions of copy number gain, loss, LOH, and diploid states. The algorithm uses the Facets method which jointly models log2 coverage ratios and BAF to sensitively call small focal events.
- The primary output is an `.rds` file (R data snapshot) containing the fit object, plus a tab-delimited `.cnv.txt` file with chromosome, start, end, copy number, and segment mean values for downstream interpretation.
- Key algorithmic parameters include `--tumor-purity` (fraction of tumor content, default 1.0 meaning pure sample), `--genome-build` (hg19/hg38, must match BAM), and `--normal-ploidy` (assumed ploidy of matched normal, default 2).
- For hereditary CNV analysis or matched normal comparison, always use the `--normal` flag to specify a separate normal sample BAM to baseline the log-ratio and BAF calculations; omitting this results in comparisons against a theoretical diploid baseline which reduces sensitivity for small events.
- The `--sex` parameter (male/female/unk) controls how chrX and chrY are handled for ploidy normalization; incorrectly specifying this leads to systematic copy number miscalls on sex chromosomes.

## Pitfalls

- Setting `--tumor-purity` incorrectly causes proportional under- or over-estimation of absolute copy number. For example, setting purity to 0.5 when the sample is actually 80% tumor results in all copy number calls being systematically scaled by 1.6x.
- Using a `--genome-build` that does not match the BAM sequence dictionary causes silent coordinate mismatches and empty or garbage output segments. Always verify the @SQ header lines in the BAM file.
- Running cnv_facets without sufficient read depth (recommended >30x for tumor) produces high-variance log-ratios that generate false positive focal amplifications or deletions in the segmentation output.
- Specifying `--normal` as the same file as `--tumor` for somatic analysis removes the ability to detect somatic copy number alterations; this mode is only valid for germline CNV discovery.
- The `--min-depth` threshold defaults to a value that may miss focal events smaller than 10kb in high-coverage WGS; setting this too high wastes computational time, while setting too low increases false positives.

## Examples

### Analyze tumor-normal pair with default parameters
**Args:** `--tumor sample.tumor.bam --normal sample.normal.bam --output-dir results/`
**Explanation:** Runs CNV detection comparing tumor reads against the matched normal BAM to generate log-ratio and BAF signals for segmentation, writing output files to the specified directory.

### Specify tumor purity and sex for human male sample
**Args:** `--tumor tumor.bam --normal normal.bam --purity 0.75 --sex male --genome-build hg38`
**Explanation:** Sets the tumor cellularity to 75% for corrected copy number scaling and informs the algorithm that chrY has one copy and chrX requires sex-specific ploidy normalization.

### Run with custom minimum depth threshold for targeted sequencing
**Args:** `--tumor tumor.targeted.bam --normal normal.targeted.bam --min-depth 20 --genome-build hg19`
**Explanation:** Lowers the minimum read depth per bin to 20 to accommodate targeted panels with uneven coverage, useful when uniform depth assumptions from WGS do not apply.

### Generate only the segmentation output (skip Rdata)
**Args:** `--tumor tumor.bam --normal normal.bam --skip-rdata --output-dir out/`
**Explanation:** Disables writing the .rds R data snapshot to save disk space when only the textual .cnv.txt segmentation file is needed for downstream BED interval processing.

### Adjust normal ploidy for a known aneuploid matched normal sample
**Args:** `--tumor tumor.bam --normal lohmannormal.bam --normal-ploidy 3 --purity 0.90`
**Explanation:** Specifies that the matched normal has a baseline triploid copy number (e.g., from a constitutional aneuploidy) so that the log-ratio baseline is correctly shifted for detecting somatic deviations.

### Run with verbose logging for debugging segmentation issues
**Args:** `--tumor tumor.bam --normal normal.bam --verbose --output-dir debug_out/`
**Explanation:** Enables detailed logging of the EM clustering and BAF segmentation steps to identify why small focal events are missing or why certain chromosomes have no calls.

### Process germline CNVs using tumor-only mode (no matched normal)
**Args:** `--tumor germline.bam --purity 1.0 --genome-build hg38 --normal-ploidy 2`
**Explanation:** Performs CNV discovery comparing the single sample against a theoretical diploid baseline, appropriate for germline CNV screening where no matched tissue is available.