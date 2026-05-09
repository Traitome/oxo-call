---
name: ampliconsuite
category: bioinformatics/variant-calling
description: A tool for analyzing amplicon sequencing data to characterize viral quasispecies and microbial diversity through read mapping, variant calling, and population genetics analyses.
tags: amplicon-sequencing, variant-calling, quasispecies, viral-analysis, microbiology, population-genetics, next-generation-sequencing
author: AI-generated
source_url: https://github.com/goshdu/ampliconsuite
---

## Concepts

- **Data Model**: AmpliconSuite processes paired-end or single-end FASTQ reads from amplicon PCR experiments, mapping them to a reference sequence to call variants and calculate quasispecies diversity metrics (Shannon entropy, Hamming distance, and Tajima's D).
- **Input Formats**: Accepts FASTQ/FASTA for raw reads and reference sequences; companion binary `ampliconsuite-build` creates indexed reference databases from FASTA input; outputs variants in VCF format alongside consensus FASTA and population genetics statistics.
- **Key Behaviors**: The pipeline performs adapter/primer trimming, read alignment with gap-aware scoring, variant calling using a Bayesian probabilistic model, and consensus sequence generation; downstream modules calculate diversity indices across the amplicon population.
- **Output Artifacts**: Produces consensus FASTA sequences, annotated VCF files with allele frequencies, HTML diversity reports with principal coordinate analysis, and phylogenetic trees for major haplotypes.

## Pitfalls

- **Insufficient read depth**: Calling variants below 100x depth produces unreliable genotype calls, especially for low-frequency variants below 1% frequency, leading to false positive quasispecies members.
- **Missing primer sequences**: Failing to trim PCR primers before mapping causes alignment artifacts at amplicon ends, resulting in spurious indels and skewed frequency estimates near termini.
- **Incorrect reference selection**: Using a reference sequence >5% divergent from the dominant haplotype reduces mapping sensitivity and causes allele dropout, particularly for indel-rich regions.
- **Over-aggressive quality filtering**: Setting Q-score thresholds too high (>35) discards legitimate reads from low-complexity regions, biasing diversity estimates downward artificially.

## Examples

### Map amplicon reads to a reference database
**Args:** -1 sample_R1.fastq.gz -2 sample_R2.fastq.gz -d references.fa -o alignment.sam -t 8
**Explanation:** This maps paired-end reads from amplicon PCR to an indexed reference database using 8 threads, outputting SAM format for downstream variant calling.

### Build a reference database from FASTA sequences
**Args:** build -i viral_references.fasta -o viral_index -k 11
**Explanation:** The companion binary builds a suffix array index with k-mer size 11 for efficient read alignment against multiple haplotype references.

### Call variants with minimum frequency threshold
**Args:** call -i alignment.bam -o variants.vcf -m 0.01 -q 20 -d 100
**Explanation:** Calls variants present in at least 1% of reads, requiring minimum base quality 20 and read depth 100 for a genotype call.

### Generate consensus sequences from variant calls
**Args:** consensus -i variants.vcf -r reference.fa -o consensus_seqs.fasta -f 0.5
**Explanation:** Generates consensus FASTA sequences at each variant position using the majority allele when frequency exceeds 50%.

### Calculate quasispecies diversity metrics
**Args:** diversity -i variants.vcf -o diversity_report.txt --metrics shannon,tajima,pcoa
**Explanation:** Computes Shannon entropy, Tajima's neutrality test, and principal coordinate analysis to characterize population structure.

### Filter low-quality variant calls
**Args:** filter -i raw_variants.vcf -o clean_variants.vcf --min-af 0.05 --max-af 0.95 --strandbias 0.01
**Explanation:** Removes variants with allele frequencies between 5-95% showing significant strand bias exceeding 1%, eliminating False positives from sequencing artifacts.