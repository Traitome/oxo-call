---
name: bandwagon
category: population-genomics
description: Estimates changes in allele frequencies in viral populations from deep sequencing time-series data using Bayesian inference. Designed for analyzing viral quasispecies evolution and detecting selective pressures.
tags: allele-frequency, viral-evolution, time-series, deep-sequencing, bayesian-inference, quasispecies
author: AI-generated
source_url: https://github.com/mlafeldt/bandwagon
---

## Concepts

- Bandwagon processes aligned sequencing reads (BAM/SAM) across multiple time points to infer allele frequency trajectories. The input must be coordinate-sorted and indexed BAM files, with a corresponding variant list in BED or VCF format specifying the genomic positions to track.
- The tool outputs posterior distributions for allele frequencies at each time point, including mean, median, and 95% credible intervals. These statistics are written to stdout in tab-delimited format, making them easily parseable for downstream visualization or statistical analysis.
- Bandwagon uses a Bayesian state-space model where the frequency evolution is modeled as a random walk with a diffuse prior. Users can specify the prior strength via the `--prior-strength` flag to control how much the data drives the estimates versus the prior belief.
- The tool supports multiple alleles at a single locus (multi-allelic sites), with automatic grouping of reads by allele identity at each genomic position. Nucleotide ambiguities in the reference are handled appropriately.

## Pitfalls

- Providing unsorted or unindexed BAM files causes the tool to fail silently or produce incorrect allele assignments. Always ensure alignments are coordinate-sorted with `samtools sort` and indexed with `samtools index` before running bandwagon.
- Mismatched read groups between time-point BAM files and the variant definition file leads to zero counts for all alleles. Verify that the chromosome names and coordinates in your BED/VCF file exactly match those in the alignment files.
- Running bandwagon with insufficient sequencing depth at early time points produces wide credible intervals that may mislead evolutionary interpretations. Check that minimum read coverage thresholds are met in your input data.
- Using a `--prior-strength` value that is too high (greater than 100) will cause the posterior estimates to be dominated by the prior, effectively ignoring the observed data and producing flat frequency trajectories.
- Ignoring the `--output-file` flag and redirecting stdout when running on large datasets may cause memory buffering issues; always write results directly to file when processing many loci.

## Examples

### Estimate allele frequency trajectory at a single locus
**Args:** --bam-list sample1.bam,sample2.bam,sample3.bam --variant-positions chr1:45223:A:G --output-file trajectories.tsv
**Explanation:** Specifies three time-point BAM files and a single variant position to track a frequency change over time; outputs a tab-separated file with frequency estimates and credible intervals for each time point.

### Track multiple variants across a genomic region
**Args:** --bam-list early.bam,late.bam --variant-file variants.bed --output-file multi_variant.tsv
**Explanation:** Uses a BED file containing multiple variant positions to simultaneously estimate frequency trajectories for all variants in the specified region, useful for scanning for selective sweeps.

### Adjust Bayesian prior strength for conservative estimates
**Args:** --bam-list t1.bam,t2.bam,t3.bam --variant-positions chrX:98765:T:C --prior-strength 50 --output-file conservative.tsv
**Explanation:** Increases the prior strength to 50, which shrinks estimates toward the prior mean (typically 0.5 for neutral sites), producing more conservative frequency change estimates with narrower intervals.

### Specify minimum read coverage filter
**Args:** --bam-list t1.bam,t2.bam,t3.bam --variant-positions chr2:33210:G:T --min-depth 100 --output-file filtered.tsv
**Explanation:** Sets a minimum read coverage threshold of 100 reads at each time point; positions with fewer reads are excluded from output, preventing low-confidence estimates from influencing downstream analysis.

### Run with predefined reference allele frequency prior
**Args:** --bam-list t1.bam,t2.bam --variant-file known_variants.vcf --prior-strength 20 --prior-freq 0.1 --output-file biased_result.tsv
**Explanation:** Initializes the prior frequency at 0.1 rather than the default 0.5, useful when biological knowledge suggests the allele is rare before observing the data, such as for a known resistance mutation.