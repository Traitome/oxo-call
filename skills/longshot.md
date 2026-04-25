---
name: longshot
category: variant-calling
description: Long-read SNP genotyper that leverages haplotype phasing for accurate diploid variant calling
tags: [variant-calling, snp, long-read, nanopore, pacbio, haplotype, phasing, vcf]
author: oxo-call built-in
source_url: "https://github.com/pjedge/longshot"
---

## Concepts
- Longshot calls SNPs from long-read sequencing (ONT/PacBio) with haplotype-aware phasing.
- Longshot uses long reads' haplotype information to distinguish homozygous from heterozygous variants.
- Input: sorted indexed BAM and reference FASTA; output: VCF.
- Use -A for auto-detect; -s for sample name in VCF output.
- Longshot is designed for diploid organisms; outputs phased heterozygous SNPs.
- Use -r for region-specific calling; -c for minimum coverage; -q for minimum mapping quality.
- Longshot works best on high-coverage (20x+) data from ONT or PacBio platforms.
- -O outputs haplotype-tagged BAM with HP:i:1 and HP:i:2 tags for read separation.
- Coverage filters: -c (min_cov, default 6), -C (max_cov, default 8000), -A (auto-calculate max).
- Quality thresholds: -q (min_mapq, default 20), -a (min_allele_qual, default 7.0), -y (hap_assignment_qual, default 20.0).
- Variant detection: -e (min_alt_count, default 3), -E (min_alt_frac, default 0.125), -Q (potential_snv_cutoff, default 20.0).
- Haplotype parameters: -m (max_snvs per cluster, default 3), -L (hap_converge_delta, default 0.0001).
- Alignment options: -S (stable_alignment, slower but more accurate), -x (max_alignment, max scoring instead of pair HMM).
- -n disables HapCUT2 phasing; useful for genotyping only without haplotype assembly.

## Pitfalls
- Longshot is for SNP calling only — use Sniffles or PBSV for structural variants.
- Input BAM must be sorted and indexed with samtools index.
- Very low coverage (<10x) reduces Longshot accuracy significantly.
- Longshot outputs phased genotypes — use carefully if unphased calls are needed.
- For indels, use other tools (GATK, DeepVariant) in addition to Longshot.
- -q is minimum mapping quality (MAPQ), not base quality; base quality filtering is implicit.
- -m controls max SNVs per cluster (default 3), not minimum coverage; use -c for coverage.
- 2^m haplotypes are aligned per read for cluster size m; -m > 5 causes exponential slowdown.
- -A (auto_max_cov) is slower but recommended for variable coverage regions.
- -S (stable_alignment) uses logspace pair HMM; slower but may help with systematic errors.
- High coverage regions (>8000x) are skipped by default; increase -C or use -A for ultra-deep.
- -O (out_bam) removes existing HP/PS tags before adding new ones; backup original BAM if needed.

## Examples

### call SNPs from Oxford Nanopore aligned reads
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF

### call SNPs restricted to a specific region
**Args:** `-b sorted.bam -f reference.fa -o chr1_snps.vcf -r chr1:1000000-2000000`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o chr1_snps.vcf output VCF; -r chr1:1000000-2000000 genomic region

### call SNPs with minimum coverage filter
**Args:** `-b sorted.bam -f reference.fa -o snps_filtered.vcf -c 10 -q 20`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps_filtered.vcf output VCF; -c 10 minimum coverage; -q 20 minimum mapping quality

### output haplotype-separated BAM for downstream analysis
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -O haplotagged.bam`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -O haplotagged.bam output BAM with HP:i:1/2 tags

### call SNPs with auto-coverage for variable depth regions
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -A`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -A auto-calculate max coverage

### genotype SNPs without phasing
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -n`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -n disable HapCUT2 phasing

### call SNPs with strict allele observation filters
**Args:** `-b sorted.bam -f reference.fa -o snps_strict.vcf -e 5 -E 0.2 -a 10.0`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps_strict.vcf output VCF; -e 5 minimum alt observations; -E 0.2 minimum alt fraction; -a 10.0 minimum allele quality

### call SNPs with stable alignment for systematic errors
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -S`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -S stable alignment mode

### call SNPs in high-coverage region with increased max coverage
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -C 20000`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -C 20000 max coverage threshold

### force overwrite existing output files
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf -F`
**Explanation:** longshot command; -b sorted.bam input BAM; -f reference.fa reference FASTA; -o snps.vcf output VCF; -F force overwrite
