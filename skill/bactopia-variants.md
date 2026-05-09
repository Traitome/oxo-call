---
name: bactopia-variants
category: variant-analysis
description: A Nextflow-based pipeline for processing bacterial wholegenome sequencing data to detect and characterize genetic variants including SNPs, indels, and accessory gene alterations.
tags: bacteria, variant-calling, snp, genomics, nextflow, ngs, wgs
author: AI-generated
source_url: https://github.com/bactopia/bactopia-variants
---

## Concepts

- **Input Data Model**: bactopia-variants accepts paired-end Illumina FASTQ files (or pre-aligned BAM files) and processes them through Bcftools for variant calling. For FASTQ inputs, reads are aligned to a reference using Bwa-Mem before variant calling.
- **Output Formats**: The pipeline generates multiple output files including a compressed VCF file (`variants.vcf.gz`), an annotated VCF with variant quality scores, and a summary TSV report containing core genome SNP distances and allele frequencies.
- **Parameter Architecture**: Configuration is managed through Nextflow's `--paramfile` (YAML/JSON) or CLI arguments. Key parameters include `--reference` (FASTA with Bwa-Mem indices), `--sample` (sample name), and `--outdir` (output directory).
- **Companion Binaries**: The bactopia suite includes `bactopia-variants-build` for constructing reference indices and `bactopia-variants-collect` for aggregating variant calls across multiple samples into a pan-genome matrix.
- **Integration Points**: Variants are annotated with SnpEff for functional consequences and matched against curated databases (CARD, ResFinder) for antimicrobial resistance gene identification.

## Pitfalls

- **Reference Index Mismatch**: Providing a reference FASTA without pre-built Bwa-Mem indices (using `.bwt`, `.pac`, `.sa`, `.ann`, `.amb` files) causes immediate alignment failure. Always run `bactopia-variants-build` on your reference before variant calling.
- **Insufficient Read Depth**: Samples with genome-wide coverage below 10x may produce unreliable genotype calls in the VCF output, leading to falsepositive SNP predictions in downstream phylogenetic analysis.
- **Incorrect Sample Encoding**: Using special characters (spaces, semicolons, brackets) in the `--sample` name argument breaks downstream file naming and causes the pipeline to fail when creating output directories.
- **Missing GATK/VcfTools Filtering**: Not applying appropriate read-depth (`-G std`) or quality (`-q 30`) filters produces a VCF with low-confidence variants that inflate the SNP matrix and distort population structure inference.
- **Workflow Resource Mismatch**: Running the default pipeline on compute nodes with less than 8GB RAM causes out-of-memory errors when processing multiple samples in parallel; adjust `--max_memory` in the Nextflow config.

## Examples

### Run variant calling on a single FASTQ sample using a prepared reference
**Args:** --sample SAMN001 --reads1 R1.fastq.gz --reads2 R2.fastq.gz --reference reference.fa --outdir variants_output
**Explanation:** Executes the complete variant calling pipeline for one paired-end bacterial sample against a pre-indexed reference genome, generating SNP calls in the output directory.

### Use pre-aligned BAM input instead of FASTQ
**Args:** --sample SAMN001 --bam alignments.bam --reference reference.fa --outdir variants_output --skip_alignment
**Explanation:** Bypasses the alignment step when you already have a sorted BAM file, directly performing variant calling with Bcftools mpileup and call.

### Adjust minimum variant quality and read depth filters
**Args:** --sample SAMN001 --reads1 R1.fastq.gz --reads2 R2.fastq.gz --reference reference.fa --outdir variants_output --min_qual 60 --min_depth 15
**Explanation:** Applies stricter filtering thresholds (Phred quality 60 and minimum 15 reads per base) to reduce false-positive variant calls in the output VCF.

### Run variant calling with SNP annotation using SnpEff
**Args:** --sample SAMN001 --reads1 R1.fastq.gz --reads2 R2.fastq.gz --reference reference.fa --outdir variants_output --snpeff_db Salmonella
**Explanation:** Annotates detected variants with functional consequences (synonymous/nonsynonymous, frameshifts) using the SnpEff database for the target species.

### Generate a core genome SNP alignment for phylogenetics
**Args:** --sample SAMN001 --reads1 R1.fastq.gz --reads2 R2.fastq.gz --reference reference.fa --outdir variants_output --core_genome --filter_ recomb
**Explanation:** Filters out recombinant regions and outputs a core genome SNP alignment suitable for constructing phylogenetic trees with IQ-TREE or RAxML.

### Build reference indices for a new bacterial genome
**Args:** --reference NEW_REF.fa --outdir ref_index_output
**Explanation:** Creates all required Bwa-Mem index files (`.bwt`, `.pac`, `.sa`, `.ann`, `.amb`) and sequence dictionary using Samtools dict for the new reference genome.