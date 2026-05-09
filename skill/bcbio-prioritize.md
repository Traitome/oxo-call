---
name: bcbio-prioritize
category: variant-analysis
description: A tool for prioritizing and ranking genomic variants from VCF files based on clinical relevance, annotations, and custom scoring schemes. Part of the bcbio-nextgen pipeline ecosystem for validated, reproducible analyses.
tags: [vcf, variant-prioritization, genomics, clinical, annotation, bcbio]
author: AI-generated
source_url: https://github.com/chapmanb/bcbio-nextgen
---

## Concepts

- **VCF I/O Format**: bcbio-prioritize accepts standard VCF 4.x files as input, preserving INFO and FORMAT fields from the original file. It outputs a sorted and annotated VCF with a new `bcbio_score` INFO field added to each variant.

- **Prioritization Criteria**: Variants can be ranked using multiple built-in criteria including: pathogenicity predictions (CADD, SIFT, PolyPhen), allele frequency from population databases (ExAC, gnomAD), functional impact (exonic, splice site, UTR), and custom user-defined scoring weights.

- **Integration with GEMINI**: When using `--database` or `-d`, the tool queries a GEMINI database to pull pre-loaded annotations. This requires the GEMINI wrapper script `bcbio-prioritize-gemini` to have been run beforehand to initialize the database with your VCF.

- **Companion Build Script**: The companion binary `bcbio-prioritize-build` constructs pre-computed index files (`.pri.db`) for rapid prioritization without per-run database initialization. Use this when processing large cohorts repeatedly.

- **Priority Levels**: Output variants are tagged with a priority tier in the INFO field: `TIER1` (known pathogenic, rare, high-impact), `TIER2` (likely pathogenic or rare), `TIER3` (uncertain significance), and `TIER4` (benign/common).

## Pitfalls

- **Missing GEMINI Database Initialization**: Running prioritization with database queries before creating a GEMINI database produces empty or default scores. The consequence is that variants retain original annotations only, defeating the purpose of prioritization.

- **Incorrect VCF Coordinate Sorting**: If the input VCF is not coordinate-sorted (and indexed with `bcftools sort`), the tool may silently drop or misorder records. Always sort and index input VCFs with `bcftools sort -O z -o input.sorted.vcf.gz` before passing to bcbio-prioritize.

- **Overlapping Variants Not Handled**: bcbio-prioritize does not resolve overlapping deletions or complex structural variants automatically. Passing a VCF with such variants results in conflicting priority scores. Use `bcftools norm -Oz -f reference.fa` to split complex alleles first.

- **Score Weight Mismatch**: Custom scoring schemes specified with `--weights` must sum to 1.0. Specifying weights that sum to a different value produces normalized but unpredictable rankings where relative importance is distorted.

- **Large-Scale Memory Exhaustion**: Prioritizing whole-exome or whole-genome VCFs without `--chunk` or `-c` splits loads entire datasets into memory. For GRCh38 WGS data, expect >16GB RAM usage; on constrained systems this causes OOM termination.

## Examples

### Prioritize a VCF using CADD and allele frequency criteria

**Args:** `-i variants.vcf.gz -o prioritized.vcf.gz --cadd 30 --max-af 0.01`
**Explanation:** This ranks variants by CADD phred score ≥ 30 and minor allele frequency ≤ 1%, outputting a sorted VCF with priority annotations.

### Prioritize variants using a pre-built GEMINI database

**Args:** `--database variants.sqlite -i cohort.vcf.gz -o prioritized_cohort.vcf.gz --max-af 0.001 --pathogenic`
**Explanation:** Queries the GEMINI database for known pathogenic variants with allele frequency