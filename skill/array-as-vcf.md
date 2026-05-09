---
name: array-as-vcf
category: Format Conversion
description: Converts genotype array data (SNP array, indel array) to Variant Call Format (VCF) for downstream genomic analysis workflows.
tags: [vcf, genotype-array, snp, conversion, variant-calling, genomics]
author: AI-Generated
source_url: https://github.com/oxo-security/array-as-vcf
---

## Concepts

- **VCF Header Requirements**: The tool automatically generates valid VCF 4.x headers including FILTER, INFO, and FORMAT fields derived from array manifest metadata. Missing or malformed headers in input files cause silent failures where output VCFs appear valid but lack essential sample-level annotations.
- **Coordinate System Handling**: Input coordinates are interpreted as 0-based by default unless `--1based` flag is specified. Mixing coordinate systems between input and reference genome causes positional shifts of 1bp across all variants, which downstream tools may not detect.
- **Sample Identifier Mapping**: Array sample names undergo automatic sanitization (spaces→underscores, special characters stripped) before VCF generation. Duplicate sanitized names trigger fatal errors only if `--allow-duplicates` is set; otherwise the first sample is retained and subsequent duplicates silently skipped.
- **Allele Encoding**: Plus/minus strand conventions from array manifests must be explicitly specified via `--strand-mode plus|forward|top`. Incorrect strand mode causes allele flips where A/T variants appear as T/A, critically affecting downstream allele frequency calculations.

## Pitfalls

- **Unspecified Reference Genome**: Omitting `--reference` causes the tool to generate VCFs with genomic coordinates but without the `##reference` meta-information header. Many downstream tools (GATK, BCFTools) reject these VCFs with cryptic "missing reference" errors.
- **Mismatched Genome Build**: Using an array manifest from hg38 with a GRCh37 reference produces silent coordinate collisions for ~2% of variants in homologous regions. These errors only surface as inflated heterozygosity in quality control metrics.
- **Duplicate Variant IDs**: When input array data contains duplicated variant identifiers (common with merged datasets), the tool assigns the same ID to multiple records without warning. This violates VCF specification and causes index corruption in downstream sorting operations.
- **Missing FORMAT Fields**: Genotype calls with no-call (./.) alleles generate VCF records without GT FORMAT fields if `--skip-nocall` is enabled. This breaks compatibility with tools expecting uniform FORMAT columns across all records.
- **Large Dataset Memory Overflow**: Processing arrays exceeding 500,000 variants without `--chunk-size` specification causes memory exhaustion on systems with less than 16GB RAM, terminating with arithmetic overflow errors instead of graceful chunked processing.

## Examples

### Convert a basic genotype array CSV to VCF using hg38 reference
**Args:** `--input genotypes.csv --reference GRCh38 --output variants.vcf`
**Explanation:** This reads the CSV file containing SNP array calls and produces a valid VCF using the GRCh38 reference sequence, applying default strand and coordinate conventions.

### Handle 1-based input coordinates from legacy platform
**Args:** `--input legacy_data.txt --1based --reference hg19 --output legacy_converted.vcf`
**Explanation:** The `--1based` flag ensures coordinates from the legacy platform are converted correctly without accidental 1bp shifts that would occur with default 0-based interpretation.

### Process array data with explicit plus-strand conventions
**Args:** `--input illumina_manifest.csv --strand-mode plus --reference GRCh38 --output strand_corrected.vcf`
**Explanation:** Specifying `--strand-mode plus` prevents allele flips for variants where the plus strand representation differs from the array manifest's default orientation.

### Chunk large datasets to prevent memory exhaustion
**Args:** `--input large_cohort.csv --reference GRCh38 --chunk-size 100000 --output batch_processed.vcf`
**Explanation:** The `--chunk-size` flag forces processing in 100,000-variant increments, enabling analysis of datasets exceeding available RAM on constrained systems.

### Generate VCF with sample-level metadata preservation
**Args:** `--input multi_sample_array.csv --preserve-metadata --reference GRCh38 --output annotated.vcf`
**Explanation:** The `--preserve-metadata` flag writes sample-level annotations (batch ID, QC metrics) into VCF header comments, maintaining auditability for clinical or regulatory workflows.