---
name: hap_py
category: variant-calling
description: Variant comparison and benchmarking tool for evaluating SNP/indel caller performance against truth sets
tags: [benchmarking, variant-calling, vcf, precision, recall, giab, evaluation]
author: oxo-call built-in
source_url: "https://github.com/Illumina/hap.py"
---

## Concepts

- hap.py compares a called VCF to a truth/benchmark VCF and reports precision, recall, and F1-score.
- Standard use case: benchmarking variant callers against GIAB (Genome in a Bottle) truth sets.
- hap.py -r reference.fa truth.vcf.gz query.vcf.gz -o output_prefix --engine vcfeval
- Use --confident-regions (-f) to restrict comparison to high-confidence regions (GIAB bed files).
- Output: output_prefix.summary.csv with SNP and indel precision/recall/F1 metrics.
- Use --engine xcmp for fast comparison or --engine vcfeval (recommended) for accurate complex variant handling.
- GIAB benchmark files: HG001-HG007 truth VCFs and high-confidence BED files available from NCBI.
- Use --threads N for parallel processing.
- --roc generates ROC curve data for different quality thresholds; use --roc-filter to specify which VCF filter to use for thresholding.
- --write-vcf (-V) outputs an annotated VCF with TP/FP/FN/UNK classifications for each variant.
- --write-counts (-X) outputs extended.csv with detailed per-category counts (subtype, genotype, stratification).
- --stratification enables region-based stratification for detailed performance analysis (e.g., exome vs genome).
- --preprocess-truth normalizes the truth set using bcftools norm before comparison; recommended for consistent variant representation.
- --unhappy outputs additional debug information about comparison failures.
- Input VCFs must be bgzipped and tabix-indexed; hap.py will fail otherwise.

## Pitfalls

- Reference genome must match the genome build of both truth and query VCFs.
- The confident regions BED (-f) is critical — without it, comparisons include low-confidence regions.
- hap.py requires bgzipped and tabix-indexed input VCF files.
- Multi-allelic records should be split before hap.py for accurate comparison.
- hap.py is a benchmarking tool — it compares, not calls variants.
- --roc-filter requires the filter to exist in the query VCF; using non-existent filter names causes errors.
- --engine vcfeval requires RTG Tools to be installed separately; if unavailable, use --engine xcmp instead.
- Stratification BED files must not contain track headers; plain BED format only.
- Without --write-vcf, the annotated VCF output is not produced; only summary statistics are available.
- --preprocess-truth modifies the truth set; do not use if you need to preserve original truth variant representation.
- Confident regions (-f) and target regions (-T/-R) serve different purposes: -f marks high-confidence truth regions, -T restricts comparison to specific genomic regions.
- som.py is for somatic variant comparison (ignores genotypes); do not use for germline variant benchmarking.

## Examples

### benchmark a variant caller VCF against GIAB truth set
**Args:** `-r reference.fa GIAB_truth.vcf.gz query_calls.vcf.gz -o benchmark_results --engine vcfeval -f HG001_highconf.bed --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident regions BED; --threads 8 parallel processing

### benchmark with Stratification for SNPs and indels separately
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o results --engine vcfeval -f confident.bed --report-prefix detailed_report --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --report-prefix generates detailed per-category report; --threads 8 parallel processing

### generate ROC curve data for quality threshold analysis
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o roc_results --engine vcfeval -f confident.bed --roc QUAL --roc-filter PASS --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --roc QUAL uses QUAL field for ROC; --roc-filter PASS computes ROC for PASS variants; --threads 8 parallel processing; outputs ROC data for precision/recall curves

### output annotated VCF with TP/FP/FN classifications
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o annotated --engine vcfeval -f confident.bed --write-vcf --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --write-vcf (-V) produces annotated VCF with variant classifications; --threads 8 parallel processing; useful for investigating specific false positives/negatives

### use stratification for region-specific performance metrics
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o stratified --engine vcfeval -f confident.bed --stratification strat_regions.tsv --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --stratification TSV file with region name and BED file pairs; --threads 8 parallel processing; computes precision/recall per region (e.g., exome, CDS, low-complexity)

### preprocess truth set for consistent variant representation
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o preprocessed --engine vcfeval -f confident.bed --preprocess-truth --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --preprocess-truth normalizes truth variants using bcftools norm; --threads 8 parallel processing; ensures consistent variant representation between truth and query

### use fast xcmp engine for quick benchmarking
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o fast_results --engine xcmp -f confident.bed --threads 16`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine xcmp is faster than vcfeval but less accurate for complex variants; -f confident BED; --threads 16 parallel processing; suitable for quick iteration during pipeline development

### restrict analysis to specific target regions
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o targeted --engine vcfeval -f confident.bed -T exome_regions.bed --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; -T restricts comparison to target regions only; --threads 8 parallel processing; different from -f (confident regions): -T limits where to look, -f marks high-confidence truth

### output extended counts for detailed analysis
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o extended --engine vcfeval -f confident.bed --write-counts --threads 8`
**Explanation:** hap.py command; -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident BED; --write-counts (-X) outputs extended.csv with per-subtype (ti/tv, indel lengths) and per-genotype (het/hom) counts; --threads 8 parallel processing
