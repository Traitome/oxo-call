---
name: biopet-vcfstats
category: variant-analysis
description: Generates comprehensive statistical reports and quality metrics from VCF files, including genotype distributions, variant types, sample comparisons, and callable region statistics.
tags:
  - vcf
  - variant-calling
  - quality-control
  - genomics
  - biopet
  - statistics
  - bioinformatics
author: AI-generated
source_url: https://biopet.readthedocs.io/en/latest/tools/vcfstats/
---

## Concepts

- **VCF Input Format**: biopet-vcfstats accepts standard VCF v4.0+ files (compressed with bgzip or uncompressed) containing variant calls. The tool parses INFO fields (e.g., DP, QD, FS), FORMAT fields (e.g., AD, DP, GQ), and per-sample genotype calls to compute distribution metrics across all records.
- **Multi-Sample Analysis**: When given a multi-sample VCF, biopet-vcfstats computes sample-level statistics independently and cross-sample metrics such as sample heterozygosity ratios, missing call rates per sample, and allele balance distributions grouped by sample.
- **Output Report Structure**: The tool generates an HTML report with interactive charts (stored alongside a JSON summary file) containing sections for variant type breakdown (SNP, indel, MNV, structural), transition/transversion ratios, depth distribution histograms, genotype quality score plots, and a callable loci summary derived from the input VCF header annotations.
- **Configurable Thresholds**: Statistics thresholds (e.g., minimum read depth, minimum genotype quality) can be configured via the tool's settings config file to affect which variants are flagged in "low-quality" summary tables in the report.

## Pitfalls

- **Mismatched chromosome naming conventions**: If the VCF uses chromosome names like "chr1" but a reference or mask file uses "1" (or vice versa), depth and callable region statistics will be silently skewed or produce zero-coverage regions, leading to misleading quality assessments.
- **Compressed VCF without index**: Providing a bgzip-compressed VCF (.vcf.gz) without a corresponding .tbi tabix index causes biopet-vcfstats to fail at runtime with an index-related error, requiring you to re-run tabix on the file first.
- **Mixed variant allele representation**: VCFs with mixed multi-allelic records decomposed inconsistently across samples (some sites as biallelic, others as multi-allelic rows) will cause genotype balance statistics to be incomparable between samples, producing confusing ratios in the report that may be misinterpreted as sample quality issues.
- **Header-only callable region annotations**: When the VCF header lacks required annotations (e.g., no DP histogram bins or missing "callable" FREELIB/CALLABLE annotations in INFO), biopet-vcfstats silently omits the callable region section rather than warning the user, making the report appear incomplete.
- **Large VCF memory consumption**: Processing a VCF with millions of records (e.g., WGS whole-genome data) without adjusting JVM heap memory settings can cause out-of-memory failures; the tool's companion wrapper (biopet-vcfstats-queue) requires explicit memory allocation flags to handle such inputs.

## Examples

### Generate a basic statistics report from a single-sample VCF

**Args:** `-V input.vcf.gz -o report_output_dir`
**Explanation:** This runs biopet-vcfstats on a bgzip-compressed single-sample VCF file, producing an HTML report and JSON summary in the specified output directory.

### Generate statistics for a multi-sample VCF with sample comparison metrics

**Args:** `-V multisample.vcf.gz -o multisample_report --ignore-filtered`
**Explanation:** Using `--ignore-filtered` excludes records marked as low-quality (FILTER != PASS), ensuring cross-sample statistics reflect only high-confidence variant calls across all samples in the dataset.

### Run statistics with a custom settings file to adjust quality thresholds

**Args:** `-V variants.vcf -o filtered_report -s quality_thresholds.config`
**Explanation:** The `-s` flag points to a BIOPET settings/config file that overrides default minimum DP, GQ, and allele balance thresholds, producing a report that flags variants meeting your project's specific quality criteria rather than defaults.

### Generate a report from an uncompressed VCF for immediate inspection

**Args:** `-V raw_variants.vcf -o quick_report`
**Explanation:** Running on an uncompressed VCF avoids the need to pre-index the file, useful for quick initial quality checks during active variant calling before compression and archival.

### Generate statistics and save JSON output for downstream script integration

**Args:** `-V cohort.vcf.gz -o cohort_stats --json`
**Explanation:** The `--json` flag ensures the tool emits a structured JSON summary alongside the HTML report, enabling programmatic parsing of variant counts, Ti/Tv ratios, and sample heterozygosity values in automated pipelines.