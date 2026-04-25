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
- --distrust-genotypes allows genotype corrections during phasing (may leave variants unphased).
- --max-coverage limits coverage for read selection (default 15); higher values increase runtime.
- --mapping-quality sets minimum mapping quality for reads to be used (default 20).
- --indels enables phasing of insertions/deletions (now default in v2.0+).
- --only-snvs restricts phasing to SNVs only (reverts to pre-v2.0 behavior).
- --ped enables pedigree phasing for related samples.

## Pitfalls
- WhatsHap requires phase-informative reads — short reads (Illumina) may not span multiple SNPs.
- Input VCF must be for a single sample — multi-sample VCFs need to be split per sample.
- Without --reference, CRAM files cannot be processed.
- WhatsHap does not phase insertions/deletions as accurately as SNPs.
- The input VCF must be bgzipped and tabix-indexed.
- --distrust-genotypes may leave variants unphased if genotypes are corrected.
- --max-coverage default 15 may miss informative reads in high-coverage regions.
- --mapping-quality default 20 filters lower-quality alignments; decrease for more reads.
- Structural variants are not phased by WhatsHap.
- Pedigree phasing requires PED file with family relationships.

## Examples

### phase variants using long reads (ONT/PacBio)
**Args:** `phase --output phased.vcf.gz --reference reference.fa variants.vcf.gz long_reads.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz phased VCF output; --reference reference.fa reference genome; variants.vcf.gz input unphased VCF; long_reads.bam BAM provides phase info

### phase variants using Illumina short reads
**Args:** `phase --output phased.vcf.gz --reference reference.fa variants.vcf.gz illumina.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; variants.vcf.gz input VCF; illumina.bam short reads; Illumina can phase nearby variants; long reads give longer blocks

### tag reads with haplotype information after phasing
**Args:** `haplotag --output haplotagged.bam --reference reference.fa phased.vcf.gz sorted.bam`
**Explanation:** whatshap haplotag subcommand; --output haplotagged.bam output BAM; --reference reference.fa reference genome; phased.vcf.gz phased VCF; sorted.bam input BAM; adds HP tag for haplotype 1/2

### compute phasing statistics from a phased VCF
**Args:** `stats phased.vcf.gz`
**Explanation:** whatshap stats subcommand; phased.vcf.gz input phased VCF; outputs N50 phase block length, number of phased heterozygous variants

### phase with genotype distrust (allows corrections)
**Args:** `phase --output phased.vcf.gz --reference reference.fa --distrust-genotypes variants.vcf.gz reads.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; --distrust-genotypes allows genotype corrections; variants.vcf.gz input VCF; reads.bam phase info; may leave variants unphased

### phase with increased coverage limit
**Args:** `phase --output phased.vcf.gz --reference reference.fa --max-coverage 50 variants.vcf.gz reads.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; --max-coverage 50 uses up to 50x coverage; variants.vcf.gz input VCF; reads.bam phase info; increases runtime

### phase with lower mapping quality threshold
**Args:** `phase --output phased.vcf.gz --reference reference.fa --mapping-quality 10 variants.vcf.gz reads.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; --mapping-quality 10 includes lower-quality alignments; variants.vcf.gz input VCF; reads.bam phase info; more reads for phasing

### phase SNVs only (ignore indels)
**Args:** `phase --output phased.vcf.gz --reference reference.fa --only-snvs variants.vcf.gz reads.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; --only-snvs restricts to SNVs; variants.vcf.gz input VCF; reads.bam phase info; faster but misses indels

### pedigree phasing for trio
**Args:** `phase --output phased.vcf.gz --reference reference.fa --ped trio.ped variants.vcf.gz father.bam mother.bam child.bam`
**Explanation:** whatshap phase subcommand; --output phased.vcf.gz output VCF; --reference reference.fa reference genome; --ped trio.ped pedigree file; variants.vcf.gz input VCF; father.bam mother.bam child.bam trio BAMs; uses family relationships

### genotype variants with haplotype information
**Args:** `genotype --output genotyped.vcf.gz --reference reference.fa variants.vcf.gz reads.bam`
**Explanation:** whatshap genotype subcommand; --output genotyped.vcf.gz output VCF; --reference reference.fa reference genome; variants.vcf.gz input VCF; reads.bam reads for genotyping; computes genotype likelihoods using phasing

### split reads by haplotype
**Args:** `split --output-h1 hap1.bam --output-h2 hap2.bam --reference reference.fa phased.vcf.gz reads.bam`
**Explanation:** whatshap split subcommand; --output-h1 hap1.bam haplotype 1 BAM; --output-h2 hap2.bam haplotype 2 BAM; --reference reference.fa reference genome; phased.vcf.gz phased VCF; reads.bam input BAM

### compare two phasings
**Args:** `compare phasing1.vcf.gz phasing2.vcf.gz`
**Explanation:** whatshap compare subcommand; phasing1.vcf.gz phasing2.vcf.gz two phased VCFs to compare; evaluates agreement; useful for benchmarking
