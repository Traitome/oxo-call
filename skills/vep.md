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
- --canonical annotates only canonical transcripts; reduces output size.
- --symbol adds gene symbols (HGNC) to output.
- --biotype adds transcript biotype to output.
- --regulatory annotates regulatory region variants.
- --plugin allows custom annotation plugins (e.g., CADD, dbNSFP).

## Pitfalls
- VEP without --cache uses REST API (slow); always use --cache for production runs.
- The cache version must match the genome build and Ensembl release used for analysis.
- Without --vcf flag, VEP outputs in its own text format, not standard VCF.
- --everything enables many annotations but increases runtime; select specific flags for speed.
- Multi-allelic VCF records should be split before VEP for accurate per-allele annotation.
- VEP adds 'CSQ' INFO field; bcftools +split-vep plugin simplifies CSQ field parsing.
- --canonical may miss important non-canonical transcripts; use with caution.
- --regulatory requires regulatory build cache; not available for all species.
- --plugin requires plugin files to be downloaded and configured separately.
- Cache files are large (~10-50 GB); ensure sufficient disk space.

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

### annotate only canonical transcripts
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --canonical --symbol --fork 8 --offline`
**Explanation:** --canonical annotates only canonical transcripts; --symbol adds gene symbols

### annotate with transcript biotypes
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --biotype --fork 8 --offline`
**Explanation:** --biotype adds transcript biotype (protein_coding, lncRNA, etc.) to output

### annotate regulatory regions
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --regulatory --fork 8 --offline`
**Explanation:** --regulatory annotates variants in regulatory regions (promoters, enhancers)

### annotate with CADD scores using plugin
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --plugin CADD,whole_genome_SNVs.tsv.gz --fork 8 --offline`
**Explanation:** --plugin CADD adds CADD pathogenicity scores; requires CADD database files

### annotate with custom BED file
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --custom custom_regions.bed.gz,custom,bed,overlap,0 --fork 8 --offline`
**Explanation:** --custom annotates with user-provided BED file; useful for custom regions

### output tabular format instead of VCF
**Args:** `--input_file variants.vcf --output_file annotated.txt --cache --dir_cache /path/to/cache/ --assembly GRCh38 --tab --fork 8 --offline`
**Explanation:** --tab outputs tab-delimited format; easier to parse than VCF CSQ field

### annotate per gene instead of per transcript
**Args:** `--input_file variants.vcf --output_file annotated.vcf --vcf --cache --dir_cache /path/to/cache/ --assembly GRCh38 --per_gene --fork 8 --offline`
**Explanation:** --per_gene outputs one annotation per gene instead of per transcript
