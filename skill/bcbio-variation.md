---
name: bcbio-variation
category: Variant Analysis / VCF Processing
description: A Clojure-based tool for analyzing and comparing genetic variations from multiple samples, supporting VCF format operations including filtering, allele fraction calculation, and population-level variant comparison.
tags: [vcf, variant-calling, genetic-variation, bcftools, population-genomics, java, clojure]
author: AI-generated
source_url: https://github.com/chapmanb/bcbio-variation
---

## Concepts

- **VCF as primary I/O format**: bcbio-variation operates exclusively on VCF version 4.1+ files. It can normalize variants, extract INFO and FORMAT fields, and compute population-level statistics. Always validate that input VCF files conform to the VCF specification before processing.
- **JVM-based execution with companion scripts**: bcbio-variation runs on the Java Virtual Machine and ships with companion scripts like `bcbio-variation-build` for generating population-level background frequency files. The main JAR is invoked through a Clojure launcher script that sets appropriate classpath and memory options.
- **Allele fraction and depth-based filtering**: The tool calculates allele fractions from AD (Allele Depth) FORMAT fields and can filter variants based on allele balance deviation from expected ratios. For heterozygous germline variants, the expected allele fraction is 0.5 when using heterozygous caller assumptions.
- **Population comparison mode**: bcbio-variation supports comparing query VCF files against a population background VCF to identify significant frequency differences. This requires pre-building a background VCF using companion tooling and specifying the reference with `--flip` or `--remove-ref` flags.

## Pitfalls

- **Missing Java runtime or incorrect JAVA_HOME**: If Java is not in PATH or the wrong version is installed, the tool fails with cryptic "Could not find or load main class" errors. Always verify `java -version` returns Java 8 or 11 before running.
- **Malformed or mixed-chromosome VCF input**: Passing a VCF with records from multiple references or unsorted contigs causes the tool to halt mid-processing. Ensure VCFs are coordinate-sorted and chromosome names match exactly between query and reference files.
- **Ignoring AD field absence in FORMAT column**: When filtering by allele fraction, records lacking AD FORMAT fields silently pass or produce null values depending on mode. Check that all input BAM-calling pipelines produced Allele Depth annotations before applying balance filters.
- **Insufficient memory for large population VCFs**: Population-level comparison with thousands of samples requires heap space proportional to file size. Underestimating memory causes OutOfMemoryError during load phase. Allocate at minimum 4GB for typical clinical-scale inputs.

## Examples

### Filter VCF for allele balance deviation in heterozygous calls
**Args:** `filter -c sample.vcf -f 0.3,0.7 --min-depth 20`
**Explanation:** This removes variants where the allele fraction falls outside the 0.3–0.7 range (indicating non-heterozygous calls) and requires minimum 20 reads supporting the call.

### Normalize VCF to left-aligned alleles
**Args:** `normalize sample.vcf -o normalized_output.vcf`
**Explanation:** Left-aligns indels and multi-allelic sites to their minimal representation using biallelic decomposition, ensuring compatibility with downstream tools requiring normalized input.

### Compare query variants against population background
**Args:** `compare query.vcf --flip background-population.vcf -o significant_calls.vcf`
**Explanation:** Identifies variants in query that differ significantly in allele frequency from the population background by applying Fisher's exact test and outputting only statistically-deviating sites.

### Build population frequency database from cohort
**Args:** `bcbio-variation-build --input cohort/*.vcf --output population-bg.vcf`
**Explanation:** Aggregates variant counts across multiple VCF files in the cohort directory to create a background frequency file for downstream population-aware filtering.

### Remove reference-only alleles from multisample VCF
**Args:** `remove-ref -c multisample.vcf -o filtered.vcf --min-count 2`
**Explanation:** Removes sites where all samples are homozygous reference and fewer than 2 samples support an alternate call, reducing file size for rare-variant-only analyses.

### Extract sample-specific FORMAT fields to TSV
**Args:** `export -c sample.vcf --fields AD,DP,GQ --samples sample1,sample2 -o metrics.tsv`
**Explanation:** Exports Allele Depth, Depth, and Genotype Quality values for specified samples into a tab-delimited file for downstream statistical analysis or quality control reporting.