---
name: arcas-hla
category: HLA Typing / Immunogenomics
description: A computational tool for high-resolution HLA genotyping from RNA-seq or DNA-seq data. Aligns sequencing reads to a database of known HLA alleles and reports predicted HLA class I and class II genotypes.
tags:
  - HLA typing
  - immunogenomics
  - NGS analysis
  - allele calling
  - genetics
author: AI-generated
source_url: https://github.com/HammondLabs/arcas-hla
---

## Concepts

- **Input Format:** arcas-hla accepts raw FASTQ files (single-end or paired-end) generated from RNA-seq or DNA-seq experiments targeting the HLA region. Quality scores must be in standard Phred format.
- **Data Model:** The tool uses a reference database of known HLA alleles (HLA-A, HLA-B, HLA-C for class I; HLA-DR, HLA-DQ, HLA-DP for class II) and aligns reads using alignment algorithms to determine the most likely genotype.
- **Output Formats:** Results are typically reported in JSON or tabular text format, listing called HLA alleles with associated confidence scores or read support. Output files may include genotype calls, allele frequencies, and alignment statistics.
- **Workflow Stages:** The typical analysis involves read extraction (filtering reads mapping to the HLA region), alignment to allele references, and genotype inference using maximum likelihood or Bayesian approaches.

## Pitfalls

- **Using whole-genome FASTQ without extraction:** Providing unfiltered FASTQ files containing all sequencing reads will drastically slow down analysis and may cause memory issues because the tool attempts to align the entire dataset to HLA references rather than just HLA-targeted reads.
- **Specifying incorrect read layout:** Failing to indicate whether inputs are single-end or paired-end can lead to failed alignments or incorrect genotype calls, as the alignment and pairing logic depends on this parameter.
- **Outdated HLA allele database:** Using an old or custom reference database without updating to current IMGT/HLA database releases may result in missing newly discovered alleles and less accurate genotyping for diverse populations.
- **Insufficient read depth:** Submitting samples with very low coverage of the HLA region (fewer than 10-20 reads per allele) can produce unreliable or no genotype calls, leading to false negative results.

## Examples

### HLA genotyping from RNA-seq paired-end data
**Args:** genotype --sample NA12878 --fq1 NA12878_R1.fq.gz --fq2 NA12878_R2.fq.gz --outdir results/
**Explanation:** Runs the full genotype workflow on paired-end RNA-seq reads, outputting results for sample NA12878 to the specified directory.

### Extract HLA reads from whole-genome FASTQ
**Args:** extract --fq1 sample_R1.fq.gz --fq2 sample_R2.fq.gz --ref ref/hla_ref.fa --output sample_hla.fq
**Explanation:** Filters and extracts reads mapping to the HLA region from larger FASTQ files using the provided reference sequence.

### Align extracted reads to HLA allele reference
**Args:** align --sample sample_hla.fq --ref hla_alleles.fa --output sample_align.bam
**Explanation:** Aligns pre-extracted HLA reads to a database of known HLA allele sequences, producing a BAM file for downstream analysis.

### Specify single-end reads for genotyping
**Args:** genotype --sample HG001 --fq1 HG001_single.fq.gz --layout se --outdir out/
**Explanation:** Runs genotyping on single-end sequencing data, explicitly specifying the layout to ensure correct read pairing behavior.

### Control alignment threads and memory
**Args:** align --sample reads.fq.gz --ref ref.fa --output align.bam --threads 8 --mem 16g
**Explanation:** Aligns reads using 8 computational threads and limiting memory usage to 16GB for resource-constrained environments.