---
name: bcbio-variation-recall
category: variant-analysis
description: A tool for variant recalling and population frequency filtering that enables filtering of VCF variants against population databases like gnomAD and ExAC, supporting both germline and somatic variant analysis workflows.
tags: variant-calling, vcf, population-frequency, filtering, bcbio
author: AI-generated
source_url: https://github.com/bcbio/bcbio-nextgen
---

## Concepts

- **Input/Output Format:** bcbio-variation-recall processes VCF (Variant Call Format) files, both uncompressed and bgzip-compressed BCF. Output is typically written as bgzip-compressed VCF with an associated .tbi index, making it compatible with downstream tools like bcftools and GATK.
- **Population Frequency Filtering:** The tool annotates variants with population allele frequencies from built-in databases (gnomAD, ExAC) and can filter variants that exceed user-specified frequency thresholds, essential for prioritizing rare disease-causing variants in clinical sequencing.
- **Companion Build Tool:** The `bcbio-variation-recall-build` companion binary creates indexed population frequency databases from VCF sources, allowing users to add custom population datasets or update existing databases for use in filtering.
- **Sample-Level Recall:** bcbio-variation-recall supports pooled-sample analysis where multiple samples are analyzed together to identify variants present across a population, using cohort-wide frequency calculations rather than per-sample genotypes.

## Pitfalls

- **Missing Index Files:** Omitting the .tbi index for input VCF files causes the tool to fail with file access errors. Always ensure input VCFs are indexed with tabix before running bcbio-variation-recall.
- **Incompatible VCF Versions:** Using VCF files with non-standard or outdated format versions (pre-VCF v4.0) can cause parsing failures or silent data corruption. Validate VCF headers before processing.
- **Memory Consumption with Large Cohorts:** Processing cohort VCFs with thousands of samples can exhaust available RAM, resulting in killed processes. Use the `--num-threads` flag to limit memory usage or process in chunks.
- **Incorrect Population Database Matching:** Attempting to filter variants without a matching population database (e.g., using a database built for a different genome build) leads to incorrect frequency annotations. Verify genome build compatibility between input data and population database.

## Examples

### Filter rare variants using gnomAD population frequencies
**Args:** `--population gnomad --max-af 0.01 input.vcf.gz output.vcf.gz`
**Explanation:** This filters variants with allele frequency greater than 1% in gnomAD, retaining only rare variants suitable for disease gene discovery in rare disease analysis.

### Annotate variants without filtering
**Args:** `--population gnomad --no-filter input.vcf.gz output.vcf.gz`
**Explanation:** This adds population frequency annotations from gnomAD to the INFO field of each variant without applying any frequency-based filtering, useful for downstream custom filtering.

### Build custom population database
**Args:** `--database my_cohort_db --output /path/to/database/ /path/to/population_vcf.gz`
**Explanation:** This creates a custom recall-ready database from a population VCF file, enabling cohort-specific frequency filtering in subsequent bcbio-variation-recall runs.

### Filter with multiple population databases
**Args:** `--population gnomad --population exac --max-af 0.005 input.vcf.gz output.vcf.gz`
**Explanation:** This applies AND logic filtering, keeping variants with allele frequency below 0.5% in both gnomAD and ExAC databases, ensuring ultra-rare variant selection.

### Run multi-threaded processing for large files
**Args:** `--num-threads 4 --population gnomad input.vcf.gz output.vcf.gz`
**Explanation:** This enables parallel processing with 4 threads to speed up processing of large cohort VCF files while managing memory consumption more efficiently.

### Filter out common variants keeping only novel mutations
**Args:** `--population gnomad --max-af 0.0001 --novel-only input.vcf.gz output.vcf.gz`
**Explanation:** This filters extremely rare variants (AF