---
name: vcfanno
category: variant-annotation
description: Fast parallel annotation of VCF files using BED, VCF, and BAM databases
tags: [vcf, annotation, variants, bed, gnomad, clinvar, parallel]
author: oxo-call built-in
source_url: "https://github.com/brentp/vcfanno"
---

## Concepts

- vcfanno annotates variants by overlapping input VCF positions with one or more annotation sources (BED, VCF, BAM) defined in a TOML config.
- The config file lists [[annotation]] blocks: each has file, fields, ops (operations), and names to define what to pull and how.
- Operations (ops) include: mean, min, max, sum, count, flag, concat, first, uniq; choose based on annotation type (numeric vs categorical).
- Lua scripts can be embedded in the config (postannotation) for custom logic combining multiple annotations into a single field.
- vcfanno parallelizes across chromosomes; set -p for the number of threads to use.
- Input VCF must be bgzipped and tabix-indexed; annotation files (BED/VCF) must also be tabix-indexed for random access.

## Pitfalls

- All annotation BED/VCF files must be bgzipped (.gz) and tabix-indexed (.tbi); plain gzip or uncompressed files cause errors.
- VCF annotation fields must exist in the INFO column of the source VCF; referencing non-existent fields silently returns empty values.
- The ops field must be an array matching the length of fields; mismatched lengths cause a fatal configuration error.
- Using concat op on numeric fields produces a string; use first or mean/max for numeric annotations.
- vcfanno does not handle multiallelic sites specially; split multiallelic sites with bcftools norm -m -any before annotation.
- Large BED files without an index are not supported; always tabix-index sorted BED files before use in vcfanno.

## Examples

### annotate a VCF with gnomAD allele frequencies
**Args:** `vcfanno -p 8 config.toml input.vcf.gz > annotated.vcf`
**Explanation:** -p 8 uses 8 threads; config.toml defines gnomAD VCF source; output to stdout, redirect to file

### annotate variants with ClinVar pathogenicity and a custom BED file
**Args:** `vcfanno -p 16 clinvar_bed_config.toml input.vcf.gz | bgzip > annotated.vcf.gz`
**Explanation:** pipe through bgzip for compressed output; config includes ClinVar VCF and custom BED annotation sources

### add a flag for variants overlapping a BED region of interest
**Args:** `vcfanno -p 8 regions_config.toml input.vcf.gz > flagged.vcf`
**Explanation:** config uses op = ['flag'] with a BED file to add a boolean INFO field when a variant overlaps a region

### compute mean coverage at each variant position from a BAM file
**Args:** `vcfanno -p 8 bam_config.toml input.vcf.gz > coverage_annotated.vcf`
**Explanation:** BAM annotation in config uses op = ['mean'] on depth field; vcfanno uses the BAM index for fast access

### use a Lua postannotation to combine scores into a final filter
**Args:** `vcfanno -p 8 -lua filters.lua combined_config.toml input.vcf.gz > filtered_annotated.vcf`
**Explanation:** -lua loads a Lua script for postannotation; the script can combine gnomAD AF and CADD scores into a custom filter tag

### annotate indels with COSMIC and output only annotated variants
**Args:** `vcfanno -p 8 cosmic_config.toml input.vcf.gz | bcftools view -f PASS > cosmic_annotated.vcf`
**Explanation:** pipe to bcftools view to keep only PASS variants after annotation; cosmic_config.toml specifies COSMIC VCF source
