---
name: bcftools-gtc2vcf-plugin
category: variant-calling
description: A bcftools plugin that converts Illumina GTC (Genotype Call) files to VCF format. Used for transitioning array-based genotype data into standard VCF for downstream bioinformatics analysis.
tags:
  - illumina
  - genotyping
  - gtc
  - vcf
  - conversion
  - snp
  - array
  - variant-calling
author: AI-generated
source_url: https://github.com/samtools/bcftools/tree/master/plugins
---

## Concepts

- **Input format**: GTC (Genotype Call) files are binary Illumina array outputs containing SNP genotype calls, confidence scores, and coordinates. Each GTC file typically represents one sample's genotyping results.
- **Reference requirement**: The plugin requires an Illumina chip manifest file (`.bpm.json` or `.bpm`) that maps probe IDs to genomic coordinates. Without the correct manifest, SNP positions in the output VCF will be incorrect or missing.
- **Output generation**: The plugin produces a standard VCF 4.2+ file with GT (genotype), GP (genotype probability), and GQ (genotype quality) fields populated from the GTC data.
- **Plugin mechanism**: This is a bcftools plugin that must be loaded using the `--plugins` flag or placed in the plugin directory. The plugin name in bcftools commands is `gtc2vcf`.
- **Multi-sample batch processing**: When provided with multiple GTC files or a directory containing GTC files, the plugin can generate a multi-sample VCF with combined variant calls.

## Pitfalls

- **Using an incorrect or mismatched manifest file**: Supplying a manifest from a different chip version than the GTC files will cause incorrect chromosome positions and reference alleles, making downstream analysis unreliable.
- **Forgetting to index the manifest file**: Some versions require the manifest to be pre-processed with a companion tool (`bcftools_gtc2vcf-build`), and using an unindexed manifest will cause the plugin to fail with cryptic errors.
- **Confusing input GTC directory with file list**: Providing a directory path instead of an explicit file list can lead to unintended inclusion of all GTC files in a folder, creating oversized multi-sample VCFs.
- **Ignoring missing or ambiguous SNPs**: GTC files may contain SNPs where the genotype is uncertain or no-call; not filtering these with appropriate thresholds results in VCFs with low-quality genotypes that contaminate downstream analyses.
- **File permission issues**: GTC files created by Illumina software may have restricted permissions; the running user must have read access to all input GTC and manifest files.

## Examples

### Convert a single GTC file to VCF using a manifest
**Args:** `--plugin gtc2vcf --manifest