---
name: calitas
category: Variant Calling
description: A read-backed variant caller for detecting SNPs and small indels from aligned sequencing reads. Calitas uses Bayesian models to call variants from BAM/CRAM files against a reference genome and outputs standard VCF format.
tags: [variant-calling, snp, indel, vcf, bam, genomics]
author: AI-generated
source_url: https://github.com/bgi-calitas/calitas
---

## Concepts

- Calitas operates on position-level read data, requiring a sorted BAM/CRAM file aligned to a reference genome. The input alignment must be coordinate-sorted and indexed (BAI/CSI) for efficient random access.
- The tool outputs variants in VCF 4.2 format, including genotype calls (GT), allele depth (AD), read coverage (DP), genotype quality (GQ), and Phred-scaled likelihoods (PL) for each sample.
- Calitas employs a Bayesian heterozygous prior (default 0.001) and accounts for sequencing errors via base quality scores embedded in the BAM file. The prior can be adjusted per-sample via a platform string in the VCF header.
- The variant calling model considers read orientation (forward/reverse strand balance) and uses a somatic filter for paired tumor-normal analyses. Germline mode applies population frequency databases for common variant suppression.
- Base alignment quality (BAQ) is computed internally to reduce false positives from indel realignment regions. Disabling BAQ via flags increases sensitivity but may inflate false call rates in repetitive regions.

## Pitfalls

- Providing an unsorted or unindexed BAM file causes calitas to fail with ambiguous I/O errors. Always run `samtools index` on the alignment before variant calling and verify sorting with `samtools view -h input.bam | head`.
- Mismatched chromosome names between the BAM and reference FASTA (e.g., "chr1" vs "1") causes zero reads to be called at all positions, producing an empty VCF. Use `samtools view -H input.bam` and `samtools faidx ref.fa` to confirm exact naming conventions.
- Setting minimum coverage too low (e.g., below 5x) in high-depth sequencing data generates thousands of false heterozygous calls at low-frequency sequencing errors. Calitas defaults assume 10x minimum; adjust based on actual depth.
- Using an outdated or incomplete reference genome (missing decoy contigs or HLA sequences) leads to read misalignment and artefactual variants in pseudoautosomal and repetitive regions. Always use a primary reference with decoy sequences.
- Specifying the wrong library strategy (e.g., RNA-seq flags for DNA-seq data) skews the error model and undercalls true variants. Ensure the `--platform` flag matches the sequencing chemistry (ILLUMNA, BGISEQ, etc.).

## Examples

### Call variants from a whole-genome BAM with default settings
**Args:** --input sample.bam --reference ref.fa --output variants.vcf
**Explanation:** This runs calitas in standard germline mode, using default filtering thresholds (minimum depth 10x, minimum quality 20, Bayesian heterozygous prior 0.001) and outputting all passing SNP and indel calls.

### Call variants with stringent quality and coverage filters
**Args:** --input sample.bam --reference ref.fa --output strict.vcf --min-depth 30 --min-quality 50 --min-alt-frac 0.2
**Explanation:** This applies a 30x minimum coverage, Phred quality 50 threshold, and 20% minimum alternate allele fraction, reducing false positives in high-depth sequencing runs at the cost of missing true low-frequency variants.

### Run in multi-threaded mode to accelerate large genomes
**Args:** --input sample.bam --reference ref.fa --output variants.vcf --threads 8 --region "chr1:1-250000000"
**Explanation:** This invokes 8 parallel threads and restricts processing to chromosome 1, enabling faster analysis of whole-genome datasets by partitioning work across genomic regions.

### Enable somatic mode for tumor-normal paired analysis
**Args:** --input tumor.bam --normal normal.bam --reference ref.fa --output somatic.vcf --mode somatic --min-t-alt-frac 0.05
**Explanation:** This runs calitas in somatic mode, computing somatic mutations present in the tumor but absent or below threshold in the normal, with a 5% minimum tumor alternate fraction for variant acceptance.

### Export raw genotype likelihoods for external filtering
**Args:** --input sample.bam --reference ref.fa --output raw.vcf --output-alleles --blind --no-gq-filter
**Explanation:** This exports all candidate positions including failing quality filters, with raw genotype likelihoods instead of filtered GQ values, enabling custom filtering downstream without recalcalling.