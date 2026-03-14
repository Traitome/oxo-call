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
- Use -r for region-specific calling; -m for minimum coverage; -q for minimum base quality.
- Longshot works best on high-coverage (20x+) data from ONT or PacBio platforms.

## Pitfalls

- Longshot is for SNP calling only — use Sniffles or PBSV for structural variants.
- Input BAM must be sorted and indexed with samtools index.
- Very low coverage (<10x) reduces Longshot accuracy significantly.
- Longshot outputs phased genotypes — use carefully if unphased calls are needed.
- For indels, use other tools (GATK, DeepVariant) in addition to Longshot.

## Examples

### call SNPs from Oxford Nanopore aligned reads
**Args:** `-b sorted.bam -f reference.fa -o snps.vcf`
**Explanation:** -b BAM input; -f reference FASTA; -o output VCF with phased SNPs

### call SNPs restricted to a specific region
**Args:** `-b sorted.bam -f reference.fa -o chr1_snps.vcf -r chr1:1000000-2000000`
**Explanation:** -r restricts calling to specified genomic region

### call SNPs with minimum coverage filter
**Args:** `-b sorted.bam -f reference.fa -o snps_filtered.vcf -m 10 -q 20`
**Explanation:** -m 10 minimum coverage; -q 20 minimum base quality threshold
