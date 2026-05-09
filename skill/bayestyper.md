---
name: bayestyper
category: genomics
description: Tools for HLA and KIR genotyping from next-generation sequencing data, including read alignment, haplotype reconstruction, and variant calling
tags: [hla, kir, genotyping, ngs, sequencing, haplotyping, immunology]
author: AI-generated
source_url: https://github.com/bioinformatics-bayestyper/bayestyper
---

## Concepts

- **HLA/KIR Reference Database**: bayestyper requires a pre-built reference database containing known HLA alleles and KIR gene sequences. Use `bayestyper-build` to construct this database from FASTA inputs before running genotyping - without a valid database, the tool cannot assign alleles.

- **Input Format Flexibility**: The tool accepts FASTQ files (single or paired-end), BAM alignment files, and FASTA sequences as input. Barcoded reads should be demultiplexed beforehand or specified via `--barcodes` to enable per-sample analysis.

- **Genotype Output Formats**: Results are produced in multiple formats including VCF (for variant calling), JSON (for programmatic parsing), and human-readable text reports. Use `--output-format` to specify desired format - VCF is recommended for integration with downstream pipelines.

- **Sensitivity and Stringency Controls**: Two key parameters govern genotyping accuracy: `--min-read-depth` sets the minimum coverage required per allele call (default: 10), and `--score-threshold` controls the minimum alignment score for allele assignment (default: 30). Lower values increase sensitivity but may introduce false positives.

## Pitfalls

- **Using an Outdated Reference Database**: Running genotyping with an old or incomplete HLA/KIR database results in unassigned alleles or incorrect genotype calls. Always rebuild the reference database when new alleles are added to the ImmunoDB or IPD-KIR database - check the database version with `bayestyper-build --version`.

- **Setting Read Depth Too Low**: Specifying `--min-read-depth` below 5 creates unreliable genotype calls due to stochastic sequencing errors. This is especially problematic for heterozygous alleles where each haplotype receives half the total coverage.

- **Ignoring Read Orientation**: For paired-end reads, failure to specify `--fr` (forward-reverse) or `--rf` (reverse-forward) orientation when appropriate leads to misaligned reads and spurious variant calls. Verify library preparation chemistry before running.

- **Mismatched Read Lengths**: Providing reads shorter than the minimum length in the reference database causes alignment failures. Ensure input FASTQ read lengths meet the `--min-read-length` parameter (default: 50bp) or adjust the parameter to match your data.

## Examples

### Basic HLA genotyping from paired-end FASTQ files

**Args:** -i sample1_R1.fastq.gz sample1_R2.fastq.gz -d hla_db -o sample1_hla

**Explanation:** Performs HLA genotyping on paired-end reads using the pre-built HLA database, outputting results to the sample1_hla directory.

### Genotyping with high-sensitivity settings for low-coverage data

**Args:** -i sample2.fastq.gz -d hla_db -o sample2_sensitive --min-read-depth 3 --score-threshold 20

**Explanation:** Lowers the minimum read depth and score threshold to enable allele calling in low-coverage sequencing data, trading some specificity for increased sensitivity.

### Output results in VCF format for downstream analysis

**Args:** -i sample3_R1.fastq.gz sample3_R2.fastq.gz -d hla_db -o sample3 --output-format vcf

**Explanation:** Generates VCF-formatted output suitable for integration with variant analysis pipelines, enabling compatibility with standard bioinformatics tools.

### KIR gene genotyping using KIR-specific database

**Args:** -i kir_sample.fastq.gz -d kir_db -o kir_results --gene-range KIR2DL1,KIR2DL3,KIR3DL1

**Explanation:** Restricts genotyping to specific KIR genes in the database rather than running all available gene targets, reducing runtime and simplifying output interpretation.

### Specify forward-reverse library orientation for paired-end data

**Args:** -i pe_reads_R1.fastq.gz pe_reads_R2.fastq.gz -d hla_db -o results --orientation fr

**Explanation:** Explicitly sets the forward-reverse orientation for library fragments, ensuring proper read pairing during alignment and accurate variant detection.