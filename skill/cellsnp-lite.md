---
name: cellsnp-lite
category: variant-calling
description: A lightweight SNP caller designed for single-cell DNA sequencing data. It genotypes variants at specified genomic positions from cell-aligned BAM files, outputting standard VCF files compatible with downstream population genetics tools.
tags:
  - single-cell
  - snp-calling
  - vcf
  - bam
  - genomics
  - variant-detection
author: AI-generated
source_url: https://github.com/single-cell-nGen/variant-calling
---

## Concepts

- **RAD (Regions of Interest) File**: cellsnp-lite requires a RAD file listing genomic positions to genotype. This is typically a simple text file with chromosome, position, and optional allele columns. The tool only calls SNPs at these predefined sites, making it efficient for targeted panels.

- **Input BAM Format**: Accepts coordinate-sorted and indexed BAM files. For multiplexed samples (e.g., 10x Genomics data), the tool reads cellular barcodes from the BC tag and assigns reads to individual cells, enabling cell-specific genotyping.

- **VCF Output with Cell-Level Genotypes**: Output is a standard VCF file where each sample column contains genotype calls for that cell. For each variant, the GT field shows 0/0, 0/1, or 1/1 based on allele support, while the AD (Allelic Depth) and other INFO fields encode read counts per allele.

- **Pileup-Based Genotyping**: The tool performs pileup at each RAD position, counting reads supporting each allele. It applies filters based on minimum coverage (--min-cov) and minimum quality (--min-qual) thresholds to make genotype calls, handling allelic dropout in single-cell data.

## Pitfalls

- **Using Unsorted or Unindexed BAM Files**: If the input BAM is not coordinate-sorted or lacks a corresponding .bai index, cellsnp-lite will fail to extract reads correctly, producing empty or corrupted VCF output with no genotype calls for any position.

- **RAD File Genomic Build Mismatch**: Providing a RAD file built for hg38 while using an hg19-aligned BAM (or vice versa) causes all positions to fail genotype matching. The tool reports zero sites processed in the log, resulting in an empty VCF without error messages.

- **Setting --min-cov Too High for Low-Coverage Data**: For single-cell data with median coverage of 1-5x per cell, setting --min-cov to 10 or higher eliminates most genotypes. The tool outputs only a few called sites, leaving downstream analysis with insufficient power for population structure or allele frequency estimation.

- **Ignoring --doublet-A Fraction in Multiplexed Data**: When processing droplet-based single-cell data, doublets (two cells in one droplet) cause heterozygous calls to appear as homozygous alternate (1/1) with inflated alternate allele counts. Not setting --doublet-A introduces systematic bias in allele frequency estimates.

- **Forgetting --gzip for Large Panels**: For targeted panels with thousands of RAD positions across hundreds of cells, uncompressed VCF output grows extremely large. This causes downstream tools like plink or vcftools to run slowly or crash due to memory constraints when loading the file.

## Examples

### Call SNPs at targeted positions from a single-cell BAM

**Args:** --bam sample_sort.bam --outDir results/ --sample SC1 --ref-genome hg38.fa --rad-file panel.rad --min-cov 1 --min-qual 20 --gzip

**Explanation:** This runs SNP genotyping at positions listed in the RAD file using reads from an individual cell's reads in the BAM, outputting a gzipped VCF.

### Process multiple cells from a 10x Genomics BAMwith barcode tags

**Args:** --bam 10x.bam --outDir vcfs/ --samples-file cell_barcodes.txt --ref-genome hg38.fa --rad-file exome.rad --min-cov 2 --doublet-A 0.1

**Explanation:** This genotypes all cells listed in the barcode file at targeted exonic positions while accounting for doublet contamination that inflates alternate allele frequencies.

### Generate a plink-ready VCF for population genetics analysis

**Args:** --bam bulk.bam --outDir popgen/ --sample NA12878 --ref-genome hg38.fa --rad-file snps_hg38.rad --min-cov 5 --min-qual 30 --gzip --format AD

**Explanation:** This outputs a VCF with allelic depth annotations required by plink for linkage disequilibrium and ancestry inference workflows.

### Run on whole-genome data without a RAD file (auto-detect variants)

**Args:** --bam scDNA.bam --outDir wgs_vcfs/ --sample WGA1 --ref-genome hg38.fa --min-cov 3 --min-qual 25 --callable

**Explanation:** This uses the tool's built-in variant calling mode to discover and genotype novel SNPs across the entire genome from low-coverage single-cell data.

### Export allele counts for custom downstream analysis

**Args:** --bam sample.bam --outDir counts/ --sample C1 --ref-genome hg38.fa --rad-file markers.txt --min-cov 1 --format OC --gzip

**Explanation:** This outputs a file with raw allele counts (observed alternative and reference reads per position) for custom filtering or allelic imbalance testing.