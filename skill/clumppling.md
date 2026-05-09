---
name: clumppling
category: Variant Analysis
description: A bioinformatics tool for clustering and aggregating genomic variants based on physical proximity, linkage disequilibrium, or custom grouping criteria. Processes VCF or BCF files to identify statistically significant variant clumps, calculate cluster-level statistics, and output merged variant representations for downstream analysis.
tags: [variant-clustering, vcf-processing, linkage-disequilibrium, genomic-analysis, variant-aggregation,群体遗传学]
author: AI-generated
source_url: https://github.com/clumppling/clumppling
---

## Concepts

- **Input Formats**: clumppling accepts VCF (Variant Call Format) and BCF (Binary VCF) files as primary input. Files may be uncompressed or compressed with bgzip. The tool automatically detects the format by examining the file header.
- **Clustering Modes**: The tool supports three clustering algorithms—`spatial` (based on genomic coordinate distance), `ld` (linkage disequilibrium threshold), and `custom` (user-defined variant groups via BED intervals). Mode selection directly affects which variants are grouped together.
- **Output Formats**: Results are emitted in VCF (with cluster annotations in INFO fields), JSON (cluster summaries), or TSV (variant-to-cluster mappings). The output format flag determines the structure—VCF preserves compatibility with downstream tools like GATK or PLINK.
- **Statistical Aggregation**: Within each cluster, clumppling computes aggregated statistics including cluster size, minor allele frequency (MAF) range, and confidence intervals. These are populated in the INFO field using the `..` prefix convention (e.g., `CLUP_MAF_MIN=0.01`).

## Pitfalls

- **Specifying an invalid clustering mode**: Using a mode other than `spatial`, `ld`, or `custom` causes the tool to exit with an ambiguous error. The clusters will not be generated, and downstream analyses dependent on cluster IDs will fail.
- **Failing to index input VCF/BCF files**: Without a corresponding `.tbi` (tabix) index, clumppling cannot perform random access and may either abort or process only the header, producing empty output files that silently pass validation.
- **Setting contradictory distance and MAF thresholds**: If `--ld-threshold` and `--max-gap` are both specified in `ld` mode, the tool uses only the ld-threshold, ignoring max-gap, which may produce unexpected cluster boundaries and mislead interpretation.
- **Outputting to a pre-existing file without overwrite permission**: The tool refuses to overwrite existing output files by default. If the output path exists, clumppling fails with a permission error, breaking pipelines that assume automatic overwriting.

## Examples

### Cluster variants within 500bp genomic distance
**Args:** `--input variants.vcf.gz --mode spatial --max-gap 500 --output spatial_clusters.vcf`
**Explanation:** This runs clumppling in spatial mode, grouping variants whose genomic positions differ by at most 500 base pairs into the same cluster, and writes annotated clusters to the output VCF.

### Cluster variants using LD threshold of 0.7
**Args:** `--input genome.vcf.gz --mode ld --ld-threshold 0.7 --output ld_clusters.vcf`
**Explanation:** In ld mode, clumppling uses linkage disequilibrium to group variants with pairwise correlation ≥0.7, creating biologically meaningful variant clumps for eQTL or association analysis.

### Use custom BED intervals forclustering
**Args:** `--input calls.vcf.gz --mode custom --regions peaks.bed --output custom_clusters.vcf`
**Explanation:** This mode restricts clustering to variants overlapping the user-provided BED intervals, useful for focusing analysis on specific genomic regions like candidate loci.

### Export cluster summaries as JSON
**Args:** `--input variants.vcf.gz --mode spatial --max-gap 1000 --output-summary clusters.json --format json`
**Explanation:** By specifying JSON output format, clumppling produces a machine-readable summary of cluster sizes, MAF ranges, and variant counts, suitable for downstream programmatic processing.

### Generate TSV mapping table
**Args:** `--input variants.vcf.gz --mode ld --ld-threshold 0.8 --output mapping.tsv --format tsv`
**Explanation:** The TSV format outputs a simple two-column table mapping each variant ID to its cluster ID, which can be loaded into R or Python for visualization or further statistical testing.