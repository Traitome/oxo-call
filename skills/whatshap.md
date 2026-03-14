---
name: whatshap
category: variant-calling
description: Fast and accurate read-based phasing of heterozygous SNPs and structural variants
tags: [phasing, haplotype, long-read, nanopore, pacbio, illumina, vcf, SNP]
author: oxo-call built-in
source_url: "https://whatshap.readthedocs.io/"
---

## Concepts

- WhatsHap phases heterozygous variants into haplotypes using sequencing reads spanning multiple variants.
- Primary command: whatshap phase — phases VCF variants using reads from BAM.
- Use --reference for the reference FASTA (required for CRAM input).
- Use --output for phased VCF; -o is also accepted.
- WhatsHap works best with long reads (ONT/PacBio) which span multiple SNPs.
- whatshap stats computes phasing statistics; whatshap haplotag tags reads with haplotype origin.
- For population phasing, statistical methods (SHAPEIT4) are preferred; WhatsHap for read-based phasing.
- Use --ignore-read-groups to merge reads from multiple samples in one BAM.

## Pitfalls

- WhatsHap requires phase-informative reads — short reads (Illumina) may not span multiple SNPs.
- Input VCF must be for a single sample — multi-sample VCFs need to be split per sample.
- Without --reference, CRAM files cannot be processed.
- WhatsHap does not phase insertions/deletions as accurately as SNPs.
- The input VCF must be bgzipped and tabix-indexed.

## Examples

### phase variants using long reads (ONT/PacBio)
**Args:** `phase --output phased.vcf.gz --reference reference.fa variants.vcf.gz long_reads.bam`
**Explanation:** --output phased VCF; --reference genome; variants.vcf.gz input unphased; long_reads.bam provides phase info

### phase variants using Illumina short reads
**Args:** `phase --output phased.vcf.gz --reference reference.fa variants.vcf.gz illumina.bam`
**Explanation:** Illumina can phase nearby variants; long reads give much longer phase blocks

### tag reads with haplotype information after phasing
**Args:** `haplotag --output haplotagged.bam --reference reference.fa phased.vcf.gz sorted.bam`
**Explanation:** haplotag adds HP tag to reads assigning them to haplotype 1 or 2; useful for read visualization

### compute phasing statistics from a phased VCF
**Args:** `stats phased.vcf.gz`
**Explanation:** outputs N50 phase block length, number of phased heterozygous variants
