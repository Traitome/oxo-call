---
name: vep
category: variant-annotation
description: Ensembl Variant Effect Predictor — annotate variants with functional consequences, SIFT, PolyPhen, and more
tags: [variant-annotation, vcf, effect-prediction, ensembl, sift, polyphen, functional]
author: oxo-call built-in
source_url: "https://www.ensembl.org/vep"
---

## Concepts

- VEP annotates VCF variants with effects on transcripts, regulatory regions, SIFT/PolyPhen scores, and custom annotations.
- Use --input_file (-i) for input VCF; --output_file (-o) for output; --cache for offline annotation with Ensembl cache.
- Download VEP cache: vep_install -a cf -s homo_sapiens -y GRCh38 -c /path/to/cache/
- Use --everything to enable all standard VEP annotations including SIFT, PolyPhen, gnomAD, ClinVar.
- The --format vcf flag is required for VCF input; --vcf flag outputs VCF (adds CSQ INFO field).
- Use --fork N for parallel processing; --buffer_size for memory efficiency.
- The CSQ field format: Consequence|Feature_type|Feature|SYMBOL|Gene|HGVSc|HGVSp|SIFT|PolyPhen|...
- Use --pick or --per_gene to get one annotation per variant/gene instead of all transcripts.

## Pitfalls

- VEP without --cache uses REST API (slow); always use --cache for production runs.
- The cache version must match the genome build and Ensembl release used for analysis.
- Without --vcf flag, VEP outputs in its own text format, not standard VCF.
- --everything enables many annotations but increases runtime; select specific flags for speed.
- Multi-allelic VCF records should be split before VEP for accurate per-allele annotation.
- VEP adds 'CSQ' INFO field; bcftools +split-vep plugin simplifies CSQ field parsing.

## Examples

### annotate VCF variants with VEP using offline cache
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --fork 8 --offline`
**Explanation:** --cache offline mode; --vcf output VCF; --assembly GRCh38; --fork 8 parallel; --offline for speed

### annotate with all standard functional predictions
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --everything --fork 8 --offline`
**Explanation:** --everything enables SIFT, PolyPhen, gnomAD AF, ClinVar, dbSNP, and all standard annotations

### annotate and pick single most severe consequence per variant
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --pick --fork 8 --offline`
**Explanation:** --pick selects one annotation per variant (most severe consequence); reduces output size

### annotate with gnomAD population frequencies
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --af_gnomad --fork 8 --offline`
**Explanation:** --af_gnomad adds gnomAD allele frequencies for all populations; useful for filtering rare variants
