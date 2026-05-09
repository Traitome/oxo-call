---
name: bis-snp-utils
category: Genomics - Bisulfite Sequencing & SNP Calling
description: Bioinformatics utilities for processing bisulfite sequencing data and identifying single nucleotide polymorphisms (SNPs). Supports common formats like BAM, VCF, and FASTA for methylation analysis and variant detection workflows.
tags:
- bisulfite sequencing
- SNP calling
- methylation
- variant detection
- genomics
- BAM
- VCF
- DNA methylation
author: AI-generated
source_url: https://github.com/bis-snp-utils
---

## Concepts

- **Input Format**: bis-snp-utils typically processes aligned bisulfite sequencing reads in BAM/CRAM format, where cytosines have been converted to uracils (read as thymine) during library preparation.
- **Data Model**: The tool distinguishes between methylated CpGs and SNPs by analyzing base-level coverage and allele frequencies at cytosine positions in the genome.
- **Companion Binaries**: Common companion utilities include `bis-snp-utils-build` for generating genome indexes, and `bis-snp-utils-call` for variant and methylation calling from processed alignments.
- **Output Formats**: Results are typically emitted in VCF format for SNPs and BED or Wiggle format for methylation levels at CpG sites.
- **Key Behavior**: The tool requires a reference genome in FASTA format and may accept methylation-aware aligner outputs from tools like BSMAP, BatMeth2, or Bismark.

## Pitfalls

- **Mismatched Reference Genome**: Using a reference genome index that does not match the alignment BAM file leads to malformed coordinate mappings and incorrect SNP positions.
- **Duplicate File Extensions**: Specifying output filenames that overwrite input files causes data loss always verify output paths before execution.
- **Insufficient Coverage**: Low sequencing depth at CpG sites produces unreliable methylation percentages and may cause false positive or false negative SNP calls.
- **Incompatible Aligner Data**: Feeding outputs from non-bisulfite-aware aligners (standard BWA or Bowtie2 without bisulfite conversion) produces incorrect methylation estimates because unconverted cytosines remain.
- **Memory Overflow with Large Genomes**: Processing whole-genome bisulfite datasets without specifying memory limits may cause crashes on systems with limited RAM.

## Examples

### Build a reference genome index for bisulfite analysis

**Args:** build -reference hg38.fa -out hg38_bis

**Explanation:** Creates a searchable index of the hg38 reference genome optimized for bisulfite alignment data processing.

### Call SNPs from a bisulfite-aligned BAM file

**Args:** call -input sample.bam -reference hg38.fa -output sample_snps.vcf

**Explanation:** Analyzes the provided BAM file against the reference genome and outputs variant positions in VCF format.

### Filter low-confidence SNP calls by read depth

**Args:** filter -input raw_snps.vcf -min-depth 10 -output filtered_snps.vcf

**Explanation:** Removes SNP calls with fewer than 10 supporting reads to reduce false positives in downstream analysis.

### Extract methylation levels at CpG sites

**Args:** meth -input sample.bam -reference hg38.fa -output cpgs.bed -context CG

**Explanation:** Calculates methylation percentages at cytosine-guanine dinucleotides and writes results in BED format.

### Generate a summary report of methylation statistics

**Args:** stats -input sample.bam -reference hg38.fa -report summary.txt

**Explanation:** Produces a text report containing aggregate metrics such as mean methylation and coverage distribution.

### Combine multiple sample VCF files into a merged dataset

**Args:** merge -inputs sample1.vcf sample2.vcf sample3.vcf -output combined.vcf

**Explanation:** Concatenates variant calls from several samples into a single VCF file for comparative analysis.

### Convert VCF output to PLINK format for population genetics

**Args:** convert -input combined.vcf -format plink -out merged

**Explanation:** Transforms VCF variant data into PLINK PED/MAP format for GWAS or population structure analysis.