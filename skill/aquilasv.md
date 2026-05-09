---
name: aquilasv
category: Structural Variant Calling
description: A haplotype-resolved structural variant (SV) caller designed for long-read sequencing data (Oxford Nanopore and PacBio). It integrates split-read, read-depth, and read-pairing signals to call allele-resolved insertions, deletions, inversions, duplications, and translocations across phased or unphased genomes.
tags:
  - structural-variants
  - long-reads
  - nanopore
  - pacbio
  - haplotype-resolved
  - sv-calling
  - vcf
author: AI-generated
source_url: https://github.com/biologics-lab-aquila/aquilasv
---

## Concepts

- **Input formats**: aquilasv consumes aligned long-read BAM/CRAM files alongside a reference genome. It relies on CIGAR strings and soft-clipping patterns to detect split-read signals, and on per-window depth statistics for read-depth-based SV inference. Ensure reads are aligned with a mapper that preserves soft-clips (e.g., `minimap2` with `-Y` flag).
- **Haplotype-resolved calling**: aquilasv can leverage phasing information from haplotagged reads or phased VCFs (`.phase` files) to assign SV alleles to specific haplotypes, producing separate genotype calls per haplotype in the output VCF.
- **Output formats**: Primary output is a VCF file containing genotype-level SV calls with INFO fields encoding SV type (`SVTYPE=INS/DEL/INV/DUP/TRA`), precise breakpoints, allele frequencies, and haplotype attribution. A companion tabix-indexed `.tbi` file is generated for direct querying.
- **Technology-aware parameter sets**: aquilasv ships with preset parameter profiles for Oxford Nanopore (`--preset ont`) and PacBio HiFi (`--preset pacbio-hifi`). Selecting the correct preset substantially affects sensitivity and precision because signal noise profiles differ between platforms.
- **Multi-sample joint calling**: When multiple BAMs are supplied simultaneously, aquilasv performs joint genotyping across samples, enabling shared SV reports and cross-sample allele frequency estimation in the INFO field.

## Pitfalls

- **Providing unphased BAM data without specifying `--unphased`**: By default, aquilasv assumes phasing information exists in the reads or an external `.phase` file. On unphased data, this causes the variant calling engine to stall or produce nonsensical haplotype assignments, degrading call quality. Always add `--unphased` when no phasing is available.
- **Running with mismatched reference builds**: aquilasv checks that the BAM header SQ lines match the reference genome MD5 checksum. A mismatch causes silent read-to-reference misalignment and results in an empty or near-empty VCF with no warning, making downstream analysis appear clean but be completely incorrect.
- **Using a preset for the wrong sequencing platform**: Supplying `--preset ont` for PacBio HiFi data (or vice versa) produces systematically biased allele frequency estimates because the read-depth noise model parameters are calibrated per-platform. Always verify the input read technology before selecting a preset.
- **Omitting the `--min-sv-size` threshold when focusing on small SVs**: The default minimum SV size is 50 bp, which silently drops all variants below this threshold. Researchers targeting micro-indels or small insertions under 50 bp will receive no calls unless `--min-sv-size` is explicitly set to a lower value (e.g., `20`).
- **Insufficient read coverage producing false positives**: aquilasv requires a minimum anchor depth for split-read confirmation. With coverage below 10×, the caller enters a permissive mode that inflates false-positive rates, especially for insertions and complex rearrangements. Always check per-sample coverage before interpreting results.

## Examples

### Call SVs from a single Nanopore BAM file
**Args:** `-b sample.bam -r GRCh38.fa -o output.vcf --preset ont`
**Explanation:** This runs aquilasv in standard mode on a Nanopore-aligned BAM using the ont preset, outputting a VCF with phased genotype calls if phasing is detected in the reads.

### Call SVs from unphased PacBio HiFi BAM data
**Args:** `-b pb_sample.bam -r GRCh38.fa -o output.vcf --preset pacbio-hifi --unphased`
**Explanation:** The `--unphased` flag disables haplotype assignment logic so aquilasv does not wait for phasing tags that do not exist, producing diploid genotype calls instead of per-haplotype calls.

### Set a custom minimum SV size for small indel detection
**Args:** `-b sample.bam -r GRCh38.fa -o output.vcf --preset ont --min-sv-size 20 --max-sv-size 500`
**Explanation:** By default aquilasv calls SVs ≥ 50 bp; setting `--min-sv-size 20` captures micro-indels and small insertions that would otherwise be excluded from the output VCF.

### Joint calling across two samples for shared SV reporting
**Args:** `-b sample1.bam sample2.bam -r GRCh38.fa -o joint_output.vcf --preset ont --joint`
**Explanation:** The `--joint` flag enables cross-sample genotyping so shared SVs are reported with consistent breakpoints and cross-sample allele frequency annotations in the INFO field.

### Filter output to high-confidence calls only
**Args:** `-b sample.bam -r GRCh38.fa -o output.vcf --preset ont --min-af 0.05 --min-qual 30`
**Explanation:** The `--min-af` discards rare alleles below 5% allele frequency and `--min-qual` removes low-quality calls with QUAL scores below 30, leaving only high-confidence variants for downstream validation.