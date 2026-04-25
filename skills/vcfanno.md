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
**Args:** `-p 8 config.toml input.vcf.gz > annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; config.toml defines gnomAD VCF source; input.vcf.gz bgzipped input VCF; > annotated.vcf output redirected to file

### annotate variants with ClinVar pathogenicity and a custom BED file
**Args:** `-p 16 clinvar_bed_config.toml input.vcf.gz | bgzip > annotated.vcf.gz`
**Explanation:** vcfanno command; -p 16 uses 16 threads; clinvar_bed_config.toml config with ClinVar VCF and BED sources; input.vcf.gz bgzipped input VCF; | bgzip pipes to bgzip for compressed output; annotated.vcf.gz output file

### add a flag for variants overlapping a BED region of interest
**Args:** `-p 8 regions_config.toml input.vcf.gz > flagged.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; regions_config.toml config with op = ['flag']; input.vcf.gz bgzipped input VCF; > flagged.vcf output VCF with boolean INFO field for region overlap

### compute mean coverage at each variant position from a BAM file
**Args:** `-p 8 bam_config.toml input.vcf.gz > coverage_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; bam_config.toml config with BAM annotation op = ['mean']; input.vcf.gz bgzipped input VCF; > coverage_annotated.vcf output VCF; uses BAM index for fast access

### use a Lua postannotation to combine scores into a final filter
**Args:** `-p 8 -lua filters.lua combined_config.toml input.vcf.gz > filtered_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; -lua filters.lua Lua script for postannotation; combined_config.toml config file; input.vcf.gz bgzipped input VCF; > filtered_annotated.vcf output VCF; Lua script can combine gnomAD AF and CADD scores

### annotate indels with COSMIC and output only annotated variants
**Args:** `-p 8 cosmic_config.toml input.vcf.gz | bcftools view -f PASS > cosmic_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; cosmic_config.toml config with COSMIC VCF source; input.vcf.gz bgzipped input VCF; | bcftools view -f PASS pipes to bcftools for PASS filter; > cosmic_annotated.vcf output VCF

### annotate with multiple population frequencies
**Args:** `-p 8 populations_config.toml input.vcf.gz > pop_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; populations_config.toml config with gnomAD, ExAC, 1000G frequencies; input.vcf.gz bgzipped input VCF; > pop_annotated.vcf output VCF

### annotate variants with CADD scores
**Args:** `-p 8 cadd_config.toml input.vcf.gz > cadd_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; cadd_config.toml config with CADD BED file; input.vcf.gz bgzipped input VCF; > cadd_annotated.vcf output VCF; op = ['mean'] for average score

### annotate with conservation scores (PhyloP)
**Args:** `-p 8 phylop_config.toml input.vcf.gz > phylop_annotated.vcf`
**Explanation:** vcfanno command; -p 8 uses 8 threads; phylop_config.toml config with PhyloP scores; input.vcf.gz bgzipped input VCF; > phylop_annotated.vcf output VCF; high scores indicate conserved regions

### create minimal annotation config file
**Args:** `echo '[[annotation]]
file="annotations.bed.gz"
fields=[4]
ops=["first"]
names=["annotation_name"]' > config.toml`
**Explanation:** echo command; creates minimal TOML config; file annotations.bed.gz must be bgzipped and tabix-indexed; fields=[4] column to extract; ops=["first"] operation; names=["annotation_name"] output INFO field name

### validate annotation config file
**Args:** `-p 1 config.toml input.vcf.gz -dry-run`
**Explanation:** vcfanno command; -p 1 uses 1 thread; config.toml config file to validate; input.vcf.gz bgzipped input VCF; -dry-run validates config without running annotation; checks file paths and operations
