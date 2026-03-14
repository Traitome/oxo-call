---
name: angsd
category: population-genomics
description: Population genomics analyses from genotype likelihoods, avoiding hard genotype calling
tags: [population-genomics, genotype-likelihoods, bam, fst, thetas, sfs, snp]
author: oxo-call built-in
source_url: "http://www.popgen.dk/angsd/index.php/ANGSD"
---

## Concepts

- ANGSD works directly from BAM files using genotype likelihoods (-GL) rather than called genotypes, which is better for low-coverage data.
- -GL 1 uses the SAMtools model; -GL 2 uses GATK; -GL 1 is sufficient for most analyses and is faster.
- The site allele frequency spectrum (SFS) workflow requires two steps: -doSaf to compute per-site allele frequencies, then realSFS to fold/unfold the SFS.
- Fst and theta statistics are computed per-population: run -doSaf for each population, compute 1D SFS per population, then use realSFS to compute 2D SFS and thetaStat for sliding-window thetas.
- -doGlf 2 outputs binary BEAGLE genotype likelihood format for downstream phasing tools (BEAGLE, PCAngsd).
- Quality filters should always be applied: -minQ 20 (base quality), -minMapQ 30 (mapping quality), -remove_bads 1 to exclude improper pairs.

## Pitfalls

- Not filtering by mapping quality (-minMapQ) allows reads mapped to repetitive regions to bias allele frequency estimates.
- Using -doSaf without setting -anc (ancestral FASTA) computes a folded SFS by default; specify -anc for unfolded polarized analyses.
- ANGSD does not check that BAM files are sorted and indexed — unsorted BAMs produce incorrect or empty output silently.
- -SNP_pval threshold matters: too lenient includes invariant sites in SFS; too strict removes low-frequency variants.
- The -nThreads flag sets ANGSD threads; also set -P for samtools threads; forgetting both makes analysis very slow.
- When using -bam bam_list.txt, all BAMs must have the same read group or -uniqueOnly must be set to avoid double-counting.

## Examples

### compute genotype likelihoods and allele frequencies for a set of BAMs
**Args:** `-bam bam_list.txt -GL 1 -doGlf 2 -doMaf 1 -SNP_pval 1e-6 -minMapQ 30 -minQ 20 -nThreads 16 -out output`
**Explanation:** -doGlf 2 outputs BEAGLE format; -doMaf 1 outputs allele frequencies; -SNP_pval filters to variable sites

### compute per-site allele frequency spectrum for a single population
**Args:** `-bam pop1_bams.txt -GL 1 -doSaf 1 -anc ancestral.fasta -minMapQ 30 -minQ 20 -nThreads 16 -out pop1`
**Explanation:** -doSaf 1 computes per-site allele frequency likelihoods; -anc provides ancestral allele state for unfolded SFS

### estimate 1D site frequency spectrum from doSaf output
**Args:** `realSFS pop1.saf.idx -P 16 > pop1.sfs`
**Explanation:** realSFS uses EM algorithm to estimate the SFS from .saf.idx files; -P 16 sets parallel threads

### estimate Watterson's theta and Tajima's D in sliding windows
**Args:** `-bam pop1_bams.txt -GL 1 -doSaf 1 -doThetas 1 -pest pop1.sfs -anc ancestral.fasta -minMapQ 30 -minQ 20 -out pop1_thetas`
**Explanation:** -doThetas 1 requires -pest (prior SFS from realSFS); outputs .thetas.idx for thetaStat sliding window analysis

### compute Fst between two populations using 2D SFS
**Args:** `realSFS pop1.saf.idx pop2.saf.idx -P 16 > pop1_pop2.2dsfs && realSFS fst index pop1.saf.idx pop2.saf.idx -sfs pop1_pop2.2dsfs -fstout pop1_pop2`
**Explanation:** two-step: compute 2D SFS with realSFS, then index Fst; use realSFS fst stats for genome-wide Fst

### call SNPs and compute principal component analysis input
**Args:** `-bam bam_list.txt -GL 1 -doGlf 2 -doMaf 1 -SNP_pval 1e-6 -minMapQ 30 -minQ 20 -nInd 50 -minInd 40 -nThreads 16 -out snps_for_pca`
**Explanation:** -nInd total individuals; -minInd 40 requires genotype data in at least 40 individuals; output used with PCAngsd
