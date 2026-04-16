---
name: delly
category: variant-calling
description: Integrated structural variant prediction at single-nucleotide resolution using short-read and long-read sequencing
tags: [structural-variant, sv, deletion, inversion, duplication, translocation, vcf, bcf, cnv, long-read, pacbio, nanopore]
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
- Six subcommands: call (short-read SV), lr (long-read SV), merge (merge sites), filter (somatic/germline), cnv (CNV calling), classify (CNV filtering).
- delly lr supports PacBio (-y pb) and Oxford Nanopore (-y ont) long reads.
- delly cnv requires a mappability map (-m) for accurate copy-number estimation.
- -t SVTYPE filters to specific SV types: DEL, INS, DUP, INV, BND (translocation), or ALL.
- PRECISE variants have split-read support; IMPRESE variants rely on paired-end only.
- FILTER:PASS indicates high-quality calls; FILTER:LowQual indicates low confidence.

## Pitfalls

- CRITICAL: delly ARGS must start with a subcommand (call, filter, merge, lr, cnv, classify) — never with flags like -g, -o, -x. The subcommand ALWAYS comes first.
- Input BAMs must be coordinate-sorted, indexed, and duplicate-marked with read groups set.
- The reference FASTA must match the genome build used for alignment.
- Without the exclusion list (-x), DELLY calls many false positives in repetitive regions.
- DELLY BCF output requires bcftools for processing — install bcftools alongside DELLY.
- Population genotyping workflow is multi-step — skipping the merge and re-genotyping steps reduces sensitivity.
- For somatic calling, list tumor BAM first, then normal BAM in the command.
- delly cnv requires a mappability map; without it, CNV calling will fail.
- Germline filter requires at least 20 unrelated samples for reliable filtering.
- PRECISE variants are more reliable than IMPRECISE variants; consider filtering for PRECISE only.
- For PacBio long reads, use delly lr with --norealign_reads --vsc_min_fraction_indels 0.12 flags.
- BND type represents inter-chromosomal translocations; use INFO/CHR2 for the second chromosome.

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

### call SVs from PacBio long reads
**Args:** `lr -y pb -g reference.fa -o pacbio_svs.bcf sample.bam`
**Explanation:** -y pb specifies PacBio technology; delly lr is for long-read SV discovery

### call SVs from Oxford Nanopore reads
**Args:** `lr -y ont -g reference.fa -o ont_svs.bcf sample.bam`
**Explanation:** -y ont specifies Oxford Nanopore technology

### call only deletions (skip other SV types)
**Args:** `call -t DEL -g reference.fa -o dels.bcf sample.bam`
**Explanation:** -t DEL restricts calling to deletions only; speeds up analysis when only deletions are needed

### genotype merged SV sites across samples
**Args:** `call -g reference.fa -v merged_sites.bcf -o genotyped.bcf sample.bam`
**Explanation:** -v specifies input VCF/BCF with SV sites to genotype; used in population workflows after merge

### call copy-number variants with mappability map
**Args:** `cnv -g reference.fa -m mappability.map.gz -o cnv.bcf sample.bam`
**Explanation:** delly cnv for CNV calling; -m provides mappability map for accurate copy-number estimation

### filter for germline SVs (requires 20+ samples)
**Args:** `filter -f germline -o germline.bcf merged_samples.bcf`
**Explanation:** -f germline filters for germline SVs; requires merged data from at least 20 unrelated samples

### filter for PRECISE variants only
**Args:** `filter -p -o precise.bcf input.bcf`
**Explanation:** -p filters for PASS and PRECISE variants; more reliable than IMPRECISE calls

### convert BCF to VCF for viewing
**Args:** `bcftools view delly.bcf > delly.vcf`
**Explanation:** DELLY outputs BCF by default; use bcftools view to convert to human-readable VCF
