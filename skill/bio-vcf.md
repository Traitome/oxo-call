---
name: bio-vcf
category: variant-calling
description: A Ruby-based VCF file parser and filter for extracting, transforming, and analyzing variant call data at site and sample levels
tags:
  - vcf
  - variant-filtering
  - genomics
  - bioinformatics
author: AI-generated
source_url: https://github.com/pjotrp/bio-vcf
---

## Concepts

- bio-vcf reads standard VCF v4.x files, distinguishing between site-level annotations (CHROM, POS, ID, REF, ALT, QUAL, FILTER, INFO) and sample-level genotype data (FORMAT column plus per-sample GT, AD, DP fields)
- Filters are applied separately at site level (affecting INFO/FILTER fields across all samples) versus sample level (per-sample GT or DP thresholds that may retain the site but exclude specific samples)
- Output can be redirected to stdout for piping into other tools, or written directly to file with appropriate flags; by default, unfiltered lines are printed to STDOUT

## Pitfalls

- Applying a sample-level numeric threshold without specifying the correct sample name silently skips records where that sample has no data, producing unexpected empty output
- Using chromosome names that include the "chr" prefix (e.g., chr1) when the VCF uses bare numeric names causes all region queries to return zero records
- Confusing QUAL (site-level Phred score) with FILTER: QUAL is a numeric annotation, not a hard filter, so a filter on QUAL may pass variants that should be excluded by site-specific standards
- Modifying the VCF with output flags without specifying --output/-o can overwrite the original file if shell redirection is used inadvertently

## Examples

### Filter variants in chromosome 1 between positions 1000000 and 2000000
**Args:** `-r 1:1000000-2000000 input.vcf.gz`
**Explanation:** Region flags specify genomic coordinates using colon notation; without this range, all chromosomes in the file are processed

### Keep only variants with QUAL score above 30
**Args:** `--filter QUAL>30 input.vcf.gz`
**Explanation:** QUAL represents site-level Phred-scale confidence; filtering on this threshold removes low-confidence calls before sample-level analysis

### Extract records where sample NA12878 has heterozygous genotype
**Args:** `--filter NA12878:GT=het input.vcf.gz`
**Explanation:** Sample-specific filtering uses the format SAMPLE:FIELD=VALUE syntax; heterozygous means one alt and one ref allele present

### Require a minimum read depth of 20 for sample HG001
**Args:** `--filter HG001:DP>20 input.vcf.gz`
**Explanation:** DP is the per-sample depth field; low-depth genotypes may indicate low confidence or alignment artifacts and should be excluded from downstream analysis

### Output only INFO field values without sample genotypes
**Args:** `-s --no-sample-genotypes input.vcf.gz`
**Explanation:** This combination of flags prints site-level information while stripping per-sample columns, producing a condensed site-only VCF

### Remove any record flagged as low-quality in the FILTER column
**Args:** `--filter FILTER=PASS input.vcf.gz`
**Explanation:** When FILTER contains values beyond the default PASS, records have been flagged by upstream callers; requiring PASS retains only fully unfiltered variants

### Write filtered results to a new file instead of stdout
**Args:** `-o filtered_output.vcf --filter DP>10 input.vcf.gz`
**Explanation:** The -o flag redirects output to the specified file path rather than piping to the next command in the pipeline