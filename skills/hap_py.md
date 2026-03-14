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

## Pitfalls

- Reference genome must match the genome build of both truth and query VCFs.
- The confident regions BED (-f) is critical — without it, comparisons include low-confidence regions.
- hap.py requires bgzipped and tabix-indexed input VCF files.
- Multi-allelic records should be split before hap.py for accurate comparison.
- hap.py is a benchmarking tool — it compares, not calls variants.

## Examples

### benchmark a variant caller VCF against GIAB truth set
**Args:** `-r reference.fa GIAB_truth.vcf.gz query_calls.vcf.gz -o benchmark_results --engine vcfeval -f HG001_highconf.bed --threads 8`
**Explanation:** -r reference; truth VCF; query VCF; -o output prefix; --engine vcfeval; -f confident regions BED

### benchmark with Stratification for SNPs and indels separately
**Args:** `-r reference.fa truth.vcf.gz query.vcf.gz -o results --engine vcfeval -f confident.bed --report-prefix detailed_report --threads 8`
**Explanation:** --report-prefix generates detailed per-category report; standard SNP/indel breakdown in summary.csv
