---
name: clipper
category: Peak Calling / Genomics
description: A tool for identifying enriched regions in genomic assays by calling peaks against background controls. Performs statistical peak detection onwiglx or bigWig signal tracks.
tags: [peak-calling, chip-seq, clip-seq, enrichment, genomic-regions]
author: AI-generated
source_url: https://github.com/taxtonu/clipper
---

## Concepts

- Clipper operates on genome browser signal tracks (bigWig or bedGraph format) and identifies genomic regions whose signal exceeds a statistically determined threshold relative to a background or input control track.
- The tool requires exactly two input files: a treatment (immunoprecipitation or marked ChIP) file and a matching control (input or IgG) file, processed with `--treatment` and `--control` flags respectively.
- Output is written in BED6 or narrowPeak format containing chromosome, start, end, peak name, score, and strand columns, where the score reflects signal enrichment significance (-log10 p-value).
- Clipper supports whole-genome chromosome naming conventions via the `--chrom` flag, and can filter results to specific genomic regions for focused analysis (e.g., promoter regions).
- The false discovery rate is controlled by the `--qval` threshold flag (default 0.05), where peaks with q-value below this threshold are reported as significant.

## Pitfalls

- Running clipper with a single input file instead of providing both treatment and control results in an error "Must provide both treatment and control data," causing the analysis to fail without producing output.
- Specifying incorrect chromosome names (e.g., "chr1" when the bigWig uses "1") causes silent filtering where no peaks are called, producing an empty output file despite valid signal data existing.
- Using insufficient read depth in input files leads to overly stringent peak calls or no peaks detected, with the consequence of missing genuine binding sites in downstream analysis.
- Forgetting to index the bigWig file (.bai) causes clipper to fail with a "file not sorted" error, preventing any peak detection from occurring.
- Specifying an excessively low `--qval` threshold (e.g., 0.001) may result in no peaks passing the significance filter, yielding an empty output file even for high-quality data.

## Examples

### Calling peaks from ChIP-seq treatment vs input control
**Args:** `--treatment treatment.bigWig --control input.bigWig --species hg38 --outfile peaks.bed`
**Explanation:** The standard peak-calling invocation compares enriched IP signal against input control to identify statistically significant binding regions.

### Filtering peaks to a specific genomic region
**Args:** `--treatment chip.bigWig --control input.bigWig --chrom chr1:1000000-2000000 --species hg38 --outfile chr1_peaks.bed`
**Explanation:** Specifying `--chrom` with chromosome and coordinates limits peak detection to that interval, useful for focused analysis or testing.

### Adjusting significance threshold for relaxed peak calling
**Args:** `--treatment chip.bigWig --control input.bigWig --species hg38 --qval 0.1 --outfile relaxed_peaks.bed`
**Explanation:** A higher q-value threshold (0.1 instead of default 0.05) reports more peaks including weaker enrichments, appropriate for exploratory analysis.

### Saving both significant and all candidate peaks
**Args:** `--treatment chip.bigWig --control input.bigWig --species hg38 --save-region --outfile all_candidates.bed`
**Explanation:** The `--save-region` flag writes all candidate peaks above the background threshold regardless of q-value filtering, preserving data for manual inspection.

### Specifying a custom output filename
**Args:** `--treatment H3K27ac.bigWig --control Input.bigWig --species mm10 --outfile H3K27ac_peaks.narrowPeak`
**Explanation:** The `--outfile` flag specifies the output path and filename, allowing control over naming conventions for downstream processing.

### Running clipper with explicit genome annotation
**Args:** `--treatment IP.bigWig --control IgG.bigWig --genome hg38 --outfile called_peaks.bed`
**Explanation:** The `--genome` or `--species` flag (depending on version) must match the reference used to align the input files; mismatches produce coordinate errors.

### Specifying chromosome naming convention
**Args:** `--treatment chip.bigWig --control input.bigWig --species hg38 --chrom chr --outfile peaks.bed`
**Explanation:** The `--chrom` flag sets the naming convention ("chr" for UCSC or "" for Ensembl), required when chromosome prefixes in the bigWig differ from expectations.