---
name: abeona
category: Bioinformatics / Cancer Genomics / Copy Number Alteration Analysis
description: >
  Abeona (Allele-specific Binned Expression Omnibus Aggregator) is a command-line
  tool for allele-specific copy number alteration (CNA) and loss-of-heterozygosity (LOH)
  analysis in tumor samples. It integrates tumor and matched normal BAM files along
  with phased SNP reference panels to produce allele-specific copy number profiles,
  B-allele frequency (BAF) tracks, and segmented CNA calls. The tool operates in
  three stages: haplotype-aware read extraction, log-ratio calculation with allelic
  bias correction, and segmentation via a circular binary segmentation (CBS) routine.
tags:
  - allele-specific-copy-number
  - cancer-genomics
  - loss-of-heterozygosity
  - tumor-normal-analysis
  - snp-array
  - cbs-segmentation
  - BAF
  - CNA
  - genomics
  - bioinformatics
author: AI-generated
source_url: https://github.com/rikSande/Abeona
---

## Concepts

- **Tumor-Normal Paired Input Model**: Abeona requires a matched tumor and normal BAM file pair to compute allele-specific log₂ ratios. The normal sample provides the diploid baseline; without it, allelic imbalance cannot be reliably quantified.

- **Phased SNP Reference Panel**: The tool uses a tabix-indexed VCF of phased heterozygous SNPs (e.g.,dbSNP, 1000 Genomes) to define paternal and maternal alleles. These sites drive BAF computation; unphased or low-confidence SNP sets cause allele-swap errors in output.

- **Three-Stage Pipeline Architecture**: The computation flows through `(1)` haplotype-aware read counting per SNP window, `(2)` log-ratio and BAF signal generation with GC-wave correction, and `(3)` CBS segmentation producing discrete copy number segments with their integer copy number state.

- **Output Formats**: Abeona produces a primary segmented copy number file (`.cns`), a raw signal file (`.sig`), and an optional BAF track (`.baf`). The `.cns` file is tab-delimited with columns: chromosome, start, end, log₂ ratio, BAF, and copy number call (0, 1, 2, 3, 4, amp, del).

- **Allelic Bias Correction**: PCR bias and mapping bias across allele strands are modeled using mappability tracks; specifying a mappability file (`.mapp`) dramatically improves accuracy in repeat-rich regions where naive read counting is unreliable.

---

## Pitfalls

- **Using Tumor-Only Mode Without a Valid Ploidy Assumption**: Running `--mode tumor-only` skips the matched normal but requires `--ploidy` to be set explicitly. If ploidy is omitted, the tool defaults to 2.0 and produces systematically miscalled copy number states, especially in hyperdiploid tumors.

- **Mismatched Reference Genome Versions**: SNP coordinates from the reference panel must match the reference genome to which the BAM files are aligned. Cross-referencing (e.g., mixing hg19 SNP BED with a GRCh38-aligned BAM) produces empty overlap and all-zero output segments.

- **Unfiltered PCR Duplicate Reads**: If `--dedup` is not enabled, duplicate reads inflate read counts at heterozygous sites, artificially boosting BAF deviation and creating spurious amplification calls in high-coverage regions.

- **Insufficient Tumor Purity (Below 20%)**: Samples with tumor purity below ~0.20 exhibit BAF values too close to 0.5, making it impossible to distinguish diploid LOH from actual copy neutral states. The tool emits a `--min-purity` warning but proceeds regardless, producing low-confidence segments.

- **Segmented Output Used Directly for Interpretation Without Visualisation**: The `.cns` integer copy number calls are model-derived estimates and can shift across adjacent segments. Relying solely on the text file without cross-checking the `.sig` signal track or a genome browser visualisation frequently leads to incorrect focal amplification annotations.

---

## Examples

### Compute allele-specific copy number profiles from a tumor-normal BAM pair
**Args:** `--tumor tumor.bam --normal normal.bam --reference snps_GRCh38.vcf.gz --output-prefix my_sample`
**Explanation:** This is the standard paired analysis mode, where Abeona computes allelic log₂ ratios and BAF from the tumor and matched normal BAM files using the phased SNP VCF as the allelic reference, writing output files prefixed with `my_sample`.

### Run tumor-only mode with a fixed ploidy assumption of 2.5
**Args:** `--mode tumor-only --tumor tumor.bam --reference snps_GRCh38.vcf.gz --ploidy 2.5 --output-prefix hyper_tumor`
**Explanation:** When a matched normal is unavailable, this command forces a ploidy of 2.5 for baseline normalisation and emits copy number calls relative to that assumption, preventing the default diploid (2.0) ploidy from skewing the integer calls.

### Enable mappability-based allelic bias correction for a repeat-rich genome region
**Args:** `--tumor tumor.bam --normal normal.bam --reference snps_GRCh38.vcf.gz --mappability mapp_150mer_GRCh38.bed.gz --output-prefix repeat_region`
**Explanation:** Providing a mappability BED file allows Abeona to down-weight reads from low-uniqueness regions during allelic read counting, significantly reducing false amplification calls in repetitive genomic intervals.

### Generate a CBS-segmented output file with a significance threshold of 0.01
**Args:** `--tumor tumor.bam --normal normal.bam --reference snps_GRCh38.vcf.gz --seg-alpha 0.01 --output-prefix segmented_sample`
**Explanation:** Setting `--seg-alpha 0.01` tightens the CBS significance threshold so that only changepoints with p-values below 0.01 are accepted as segment boundaries, yielding fewer but more statistically robust copy number segments.

### Export raw signal and BAF tracks alongside the segmented calls for downstream visualisation
**Args:** `--tumor tumor.bam --normal normal.bam --reference snps_GRCh38.vcf.gz --export-signal --export-baf --output-prefix signal_export`
**Explanation:** The `--export-signal` and `--export-baf` flags produce `.sig` and `.baf` auxiliary files in addition to the standard `.cns`, enabling visualisation of per-SNP log₂ ratios and BAF deviation across the genome in downstream genome browsers or R/shiny dashboards.

---