---
name: bs_call
category: Variant Calling
description: Bisulfite sequencing variant caller that identifies methylated and unmethylated cytosine positions by detecting C-to-T transitions in aligned sequencing reads.
tags:
  - bisulfite-sequencing
  - methylation-calling
  - variant-calling
  - methyl-seq
  - epigenetic-variants
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bs_call
---

## Concepts

- bs_call operates on bisulfite-converted aligned reads (BAM/CRAM format) and a pre-processed bisulfite genome index to distinguish true C→T conversions from sequencing errors. The tool compares observed nucleotides against the expected bisulfite-converted reference, requiring both input files to share the exact same reference sequence and coordinate system.
- The caller emits a VCF file with INFO tags indicating strand orientation: `Forward strand` reads show C→T transitions while `Reverse strand` reads show G→A transitions due to the complementary nature of bisulfite chemistry. This strand-awareness is critical for correctly interpreting methylation status in CG, CHG, and CHH contexts.
- bs_call supports three operation modes controlled by the `--mode` flag: `snp` for single nucleotide variant calling, `cpg` for clustered CpG dinucleotide analysis, and `combined` for joint variant and methylation reporting. Each mode uses different statistical thresholds and filters embedded in the tool's data model.
- Base quality recalibration is applied automatically using scan-specific error models when the `--BQSR` flag provides a recalibration table, improving accuracy at low-quality bases that are prevalent in bisulfite data due to polymerasestalling artifacts.

## Pitfalls

- Using a non-bisulfite-converted reference genome causes massive false positive C→T calls because standard references retain genomic cytosines while bisulfite reads have been chemically converted. This results in a VCF where virtually every C position appears mutated, which cannot be filtered post-hoc.
- Specifying an incorrect strand orientation (`--strand auto`) when the library was prepared with an non-directional protocol leads to symmetric allele frequency estimates that are systematically biased by 50%, reducing sensitivity for hemi-methylated sites.
- Omitting the required `--context` parameter results in the tool defaulting to all cytosine contexts regardless of biological relevance, producing an inflated callset that includes CHH sites which may not beCpG-specific experimental questions.
- Using an outdated database build as the `--dbSNP` reference generates VCF records with stale rs identifiers that downstream methylation QTL tools cannot cross-reference, silently breaking annotation pipelines.
- Specifying `--min-depth` below 5 in high-duplication libraries produces spurious calls where PCR duplicates appear as independent observations, artifactually inflating allele frequencies by 50–200% at affected sites.

## Examples

### Call methylated SNPs in CpG context from a bisulfite BAM
**Args:** `input.bam --reference hg38_bs_convert.fa --mode snp --context CpG --output variants.vcf`
**Explanation:** This invokes SNP mode restricted to CpG dinucleotides, which are the primary target for methylation analysis, using the bisulfite-converted hg38 reference to avoid reference mismatches.

### Enable automatic strand detection for non-directional libraries
**Args:** `--bam input.bam --genome hg38_bs.fa --strand auto --minqual 20 --out calls.vcf`
**Explanation:** The `auto` strand mode handles libraries where forward and reverse reads cannot be distinguished experimentally, applying symmetric statistical tests across both orientations.

### Apply base quality recalibration to reduce polymerase-stalling artifacts
**Args:** `--input sample.cram --ref hg38.fa --BQSR recal.table --min-depth 10 --filter-lowmq --out recal_variants.vcf`
**Explanation:** The recalibration table corrects systematic errors introduced by bisulfite-induced polymerase stalling at consecutive cytosines, which inflate base quality scores and cause false variant calls.

### Jointly call SNPs and CpG clusters for WGBS analysis
**Args:** `--bam wgbs_sample.bam --genome hg38_bs.fa --mode combined --context CpG --min-cpg-count 2 --out wgbs_combined.vcf`
**Explanation:** Combined mode reports both single nucleotide variants and clustered CpG dinucleotides in a single VCF, facilitating downstream methylation ratio calculations without file merging.

### Export allelic methylation fractions with custom allele frequency thresholds
**Args:** `--input chip_bs.bam --ref hg38_bs.fa --mode snp --allele-freq 0.1 --max-allele-freq 0.9 --out het_sites.vcf`
**Explanation:** Restricting allele frequencies to heterozygous ranges (0.1–0.9) filters homozygous methylated and unmethylated sites, isolating allele-specific methylation for imprinting analysis.

### Call variants excluding known polymorphic CpG sites
**Args:** `--bam tumor_bs.bam --genome hg38_bs.fa --mode snp --context CpG --dbSNP hg38_snp155.vcf --exclude-polymorphic --out somatic_vars.vcf`
**Explanation:** The `--exclude-polymorphic` flag removes sites overlapping dbSNP entries, preventing germline polymorphisms from confounding somatic methylation analysis in cancer studies.