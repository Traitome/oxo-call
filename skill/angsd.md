---
name: angsd
category: Population Genetics / Variant Analysis
description: A software toolkit for analyzing next-generation sequencing (NGS) data, focusing on population genetic analyses including site frequency spectrum estimation, genotype likelihoods, PCA, admixture, and association testing. Works with BAM, CRAM, and FASTA inputs.
tags:
  - population-genetics
  - sfs
  - genotype-likelihoods
  - pca
  - ngs
  - variant-calling
  - bioinformatics
author: AI-generated
source_url: https://github.com/angsd/angsd
---

## Concepts

- **Genotype Likelihoods over Called Genotypes**: angsd works with genotype likelihoods (GLs) rather than hard-called genotypes, preserving uncertainty from the sequencing data. This is critical for low-depth sequencing where traditional variant calling loses information.
- **Input Format Flexibility**: Accepts multiple input formats via the `-bam` flag (BAM/CRAM files), `-fa` flag (FASTA reference), or direct genotype likelihood files (`.glf.gz`). The format is specified with `-fai` for FASTA index or inferred from BAM headers.
- **Output is Typically Binary**: Many angsd outputs (like `.arg.gz`, `.mafs.gz`, `.ibs.gz`) are binary compressed formats requiring post-processing tools such as `realSFS`, `angsd_popgen`, or custom R/Python scripts for interpretation.
- **Reference Genome Requirement**: For most analyses, angsd requires an indexed FASTA reference (created via `samtools faidx` and `angsd-build` companion binary) to compute genotype likelihoods and estimate prior allele frequencies.

## Pitfalls

- **Forgetting to Index the Reference or BAM Files**: Running angsd without a proper FASTA index (`*.fai`) or without companion indices (e.g., `*.bai` for BAM files) causes immediate crashes. Always pre-index with `samtools faidx ref.fa` and ensure BAM files are indexed.
- **Specifying the Wrong GL Model for the Sequencing Technology**: Using `-glF 1` (Li-Ma) for Illumina data is appropriate, but applying the wrong genotype likelihood model (e.g., using SOLiD settings for Illumina) leads to biased allele frequency estimates. Verify the technology-appropriate model in the documentation.
- **Not Filtering Low-Quality Sites or Low-Depth Regions**: Failing to apply `-minMapQ`, `-minQ`, or `-setMinDepth` results in noisy estimates of SFS or Fst, as poorly sequenced regions bias the results toward false polymorphisms.
- **Mismatch Between Ancestral/Outgroup and Reference**: When estimating derived allele frequencies or SFS with an outgroup, specifying `-anc` with a reference that isn't the ancestral sequence leads to incorrect polarization of the allele frequency spectrum.

## Examples

### Estimating the Site Frequency Spectrum (SFS) from a population BAM list
**Args:** -bam bam.filelist -out out -fasta ref.fa -glF 1 -doSaf 2 -anc ref.fa
**Explanation:** This generates posterior SFS probabilities using the SAMtools genotype likelihood model (`-glF 1`) and the ancestral sequence as the outgroup for polarization. The `bam.filelist` must contain one BAM path per line.

### Computing genotype likelihoods for all sites in a BAM file
**Args:** -bam sample.bam -fasta ref.fa -glf sample.glf.gz -glF 1 -doGlf 2
**Explanation:** Writes binary genotype likelihoods (`-doGlf 2`) in gzipped format for downstream analyses like PCA or association testing. The reference is required for computing likelihoods.

### Performing Principal Component Analysis (PCA) on called genotypes
**Args:** -bam bam.filelist -out pca -fasta ref.fa -glF 1 -doGlf 2 -doMaf 1 -doPca 2 -outnorm pca_scores
**Explanation:** Implements a covariance-based PCA (`-doPca 2`) directly on genotype likelihoods, producing normalized PC scores for population structure visualization. The `angsd_build` companion is not needed here.

### Estimating Fst between two populations
**Args:** -bam pop1.filelist -bam2 pop2.filelist -out fst_pop1_pop2 -fasta ref.fa -glF 1 -doSaf 2 -fstout
**Explanation:** This calculates genome-wide Fst using the SFS-based method, outputting per-site Fst estimates in binary format. Requires the SFS for each population first (via separate `realSFS` runs).

### Calculating the inbreeding coefficient (F) for each individual
**Args:** -bam bam.filelist -out inbreeding -fasta ref.fa -glF 1 -doSaf 2 -fold -inbreedF
**Explanation:** Estimates the inbreeding coefficient per sample by examining the reduction in heterozygosity probability relative to Hardy-Weinberg expectations, using `-fold` to polarize the frequency spectrum.

### Generating a alleles-by-site matrix for downstream R analysis
**Args:** -bam bam.filelist -out mydata -fasta ref.fa -glF 1 -doGlf 2 -doMaf 1 -doMajorMinor 1
**Explanation:** Outputs a binary major/minor allele frequency table (`-doMaf 1`) that can be converted to a genotype matrix for GWAS or rare variant tests in R. Requires an additional companion script (like `angsd2vcf`) for VCF generation.