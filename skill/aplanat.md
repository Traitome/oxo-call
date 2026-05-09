---
name: aplanat
category: Variant Simulation / Test Data Generation
description: A bioinformatics tool for generating synthetic, anonymized Variant Call Format (VCF) files from a reference genome. Used for benchmarking variant calling pipelines, testing bioinformatics workflows, and creating realistic test datasets without exposing real genetic data.
tags: [vcf, variant-calling, simulation, test-data, genomics, benchmarking, synthetic-data, anonymization]
author: AI-generated
source_url: https://github.com/koelling/aplanat
---

## Concepts

- **Synthetic VCF Generation**: aplanat creates realistic variant calls (SNPs and indels) by simulating mutations along a reference genome. Variants are placed randomly across the genome following configurable density models, enabling generation of test datasets with known ground truth.
- **Reference-Based Simulation**: The tool requires an input FASTA reference genome. It uses the reference sequence context to generate biologically plausible variants that respect local sequence composition and avoid creating invalid variant calls.
- **Anonymized Output**: Generated VCF files contain no real patient data, making them safe for sharing, pipeline testing, and public benchmarking datasets. This addresses data privacy concerns in genomics research and development.
- **Configurable Mutation Models**: Users can control variant type distribution (transitions vs. transversions), allele frequencies, indel length distributions, and regional mutation rates to tailor synthetic datasets to specific testing requirements.

## Pitfalls

- **Unrealistic Variant Density**: Setting mutation rates too high creates implausibly dense variant files that may cause downstream tools to fail or behave abnormally. For human-like data, use rates around 0.001-0.01 variants per base pair, not 1:1.
- **Reference Genome Mismatch**: Using a reference genome that differs from your downstream pipeline's reference will cause validation failures. Always ensure the aplanat reference matches exactly the reference used in your variant calling workflow.
- **_seed Not Set for Reproducibility**: By default, random variant placement changes between runs. Failing to set a seed (`--seed`) makes it impossible to reproduce exactly the same synthetic dataset for regression testing.
- **Chromosome Naming Inconsistency**: Generated VCFs use chromosome names from the input FASTA. If your reference uses "chr1" but downstream tools expect "1" (or vice versa), data loading will fail. Audit chromosome naming conventions before generating test data.

## Examples

### Generate a basic synthetic VCF file from a reference genome
**Args:** `--reference ref.fa --output synthetic.vcf`
**Explanation:** This creates a VCF file with randomly distributed variants based on the provided reference FASTA file. The default settings produce a modest number of realistic-looking SNPs and small indels.

### Generate variants with a specific number of SNPs
**Args:** `--reference ref.fa --number 10000 --type snp --output snps_only.vcf`
**Explanation:** This generates exactly 10,000 single nucleotide polymorphisms (SNPs) distributed across the reference genome, producing a clean test file containing only SNPs for focused testing.

### Set a random seed for reproducible test data
**Args:** `--reference ref.fa --seed 42 --output reproducible.vcf`
**Explanation:** By specifying a seed value, the random number generator produces the same variant positions every time, ensuring reproducible synthetic data across multiple runs and pipeline versions.

### Generate indels with controlled length distribution
**Args:** `--reference ref.fa --type indel --indel-length-max 50 --output indels.vcf`
**Explanation:** This creates insertions and deletions with maximum length of 50 base pairs, useful for testing indel handling capabilities in variant callers and annotation tools.

### Create a multi-sample VCF with defined sample names
**Args:** `--reference ref.fa --samples sample1,sample2,sample3 --output multisample.vcf`
**Explanation:** This generates a VCF file containing multiple samples (sample1, sample2, sample3), each with independently simulated variant calls, suitable for testing joint calling pipelines.

### Generate variants with specific transition/transversion ratio
**Args:** `--reference ref.fa --ti-tv-ratio 2.0 --output ti_tv_controlled.vcf`
**Explanation:** Setting a transition-to-transversion ratio of 2.0 mimics the biologically expected ratio in human exomes (approximately 2:1), producing more realistic variant distributions for benchmarking.

### Generate variants only in specific genomic regions
**Args:** `--reference ref.fa --regions chr1:1000000-5000000 --output exonic_region.vcf`
**Explanation:** Restricting variant simulation to a specific genomic region creates a targeted test file focused on a particular locus, useful for testing region-specific analysis workflows.