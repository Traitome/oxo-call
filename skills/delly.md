---
name: delly
category: variant-calling
description: Integrated structural variant prediction at single-nucleotide resolution using short-read sequencing
tags: [structural-variant, sv, deletion, inversion, duplication, translocation, vcf]
author: oxo-call built-in
source_url: "https://github.com/dellytools/delly"
---

## Concepts

- DELLY calls SVs (deletions, duplications, inversions, translocations) by paired-end and split-read analysis.
- DELLY call is the primary subcommand; use -o for BCF output (preferred over VCF for speed).
- For somatic calling, use matched normal + tumor BAMs and apply germline filter: delly call → delly filter.
- Population SV calling: call per sample → merge with delly merge → re-genotype all samples at merged sites.
- Use -g for reference FASTA; -x for an exclusion list of repetitive regions (improves specificity).
- SV type is called jointly; no need to split by SV type in most workflows.
- DELLY outputs BCF by default (binary VCF); convert to VCF with bcftools view.

## Pitfalls

- DELLY requires paired-end Illumina data — it does not work with long reads.
- Input BAMs must be coordinate-sorted with read groups set.
- The reference FASTA must match the genome build used for alignment.
- Without the exclusion list (-x), DELLY calls many false positives in repetitive regions.
- DELLY BCF output requires bcftools for processing — install bcftools alongside DELLY.
- Population genotyping workflow is multi-step — skipping the merge and re-genotyping steps reduces sensitivity.

## Examples

### call structural variants from a single sample
**Args:** `call -g reference.fa -o sample_svs.bcf sample.bam`
**Explanation:** -g reference FASTA; -o output BCF; input BAM as positional argument

### call SVs with repetitive region exclusion list
**Args:** `call -g reference.fa -x hg38.excl -o sample_svs.bcf sample.bam`
**Explanation:** -x exclusion list (e.g., human.hg38.excl.tsv) reduces false positives in repetitive regions

### call somatic SVs from tumor-normal pair
**Args:** `call -g reference.fa -x hg38.excl -o somatic_svs.bcf tumor.bam normal.bam`
**Explanation:** list tumor first then normal; apply delly filter with -f somatic afterward for somatic-only calls

### filter somatic SVs from DELLY output
**Args:** `filter -f somatic -o somatic_filtered.bcf -s samples.tsv somatic_svs.bcf`
**Explanation:** -f somatic filters for somatic calls; -s samples.tsv specifies tumor/normal sample names and types

### merge per-sample SV calls for population analysis
**Args:** `merge -o merged_sites.bcf sample1.bcf sample2.bcf sample3.bcf`
**Explanation:** merge SV sites across samples; then re-genotype all samples at merged sites with delly call -v
