---
name: bis-snp
category: Variant Calling / Epigenomics
description: Bis-SNP is a bisulfite sequencing variant and methylation caller that simultaneously identifies SNPs and DNA methylation sites from bisulfite-converted sequencing data using Bayesian inference. It processes aligned BS-Seq BAM files and outputs VCF for SNPs and BED for methylation calls.
tags:
  - bisulfite-sequencing
  - snp-calling
  - methylation-calling
  - epigenomics
  - bs-seq
  - vcf
  - bayesian
author: AI-generated
source_url: https://github.com/dani class/bis-snp
---

## Concepts

- **Bisulfite Alignment Input**: Bis-SNP requires BAM files from bisulfite-converted reads aligned to a reference genome. The aligned reads must use a bisulfite-aware aligner (such as BISMARK or BSMAP) because standard aligners do not handle the C-to-T conversion that occurs during bisulfite treatment. Input BAM files must be sorted and indexed.
- **Simultaneous SNP and Methylation Calling**: The tool uses a Bayesian model to jointly estimate genotype states and methylation levels at each cytosine position. The model incorporates the bisulfite conversion rate (typically 95-99%) to correctly interpret C-to-T mismatches as either SNPs or incomplete conversion events. Output includes VCF files for SNPs and BED files for methylation calls at single-base resolution.
- **Genome Indexing with bis-snp-build**: Before running Bis-SNP, the reference genome must be indexed using the companion `bis-snp-build` command, which creates a special index optimized for bisulfite-aware variant detection. This index differs from standard genomic indices and is required for correct read masking during variant calling.

## Pitfalls

- **Using Non-Bisulfite-Aligned BAM Files**: If the input BAM contains reads aligned by a standard aligner (e.g., BWA, Bowtie2 without bisulfite mode), the C-to-T mismatch pattern will be misinterpreted as high SNP rates or failed calling. This produces spurious SNP calls in CpG islands and near repeat regions, wasting downstream validation effort.
- **Forgetting to Run bis-snp-build**: Attempting to call variants without the bis-snp-built genome index causes the tool to fail or produce unreliable results because the index files are missing. Re-running the full pipeline from the beginning wastes hours of compute time on large cohorts.
- **Insufficient Sequencing Depth**: Bis-SNP requires adequate coverage to distinguish genuine methylation from stochastic sampling noise. Regions with fewer than 5-10x coverage generate inconsistent methylation calls with high variance, leading to false conclusions about differential methylation in biological comparisons.
- **Misconfigured Conversion Rate**: Specifying an incorrect bisulfite conversion rate (via the `-convRate` parameter) skews the Bayesian posterior probabilities for methylation. This systematically biases all methylation estimates, making hypomethylated regions appear normethylated or vice versa.
- **Ignoring Read Directionality in RRBS Data**: Reduced Representation Bisulfite Sequencing (RRBS) reads from opposite DNA strands have different cytosine contexts. Failing to specify both `-topStrand` and `-bottomStrand` modes for RRBS results in missing calls for one strand, effectively halving the usable coverage for CpG sites.

## Examples

### Call SNPs and methylation from a whole-genome bisulfite BAM
**Args:** `-I aligned_bs_reads.bam -D genome.fa -o snp_methylation_output -vcf output.vcf`
**Explanation:** This runs Bis-SNP on a whole-genome bisulfite sequencing BAM, outputting both a BED file of methylation calls and a VCF of SNPs to the specified prefix.

### Build a bisulfite-aware genome index for variant calling
**Args:** `genome.fa`
**Explanation:** This uses the bis-snp-build companion to create the required index files from the reference genome, enabling accurate read masking during subsequent Bis-SNP runs.

### Specify a non-default bisulfite conversion rate
**Args:** `-I sample.bam -D genome.fa -o results -convRate 0.98`
**Explanation:** Setting the conversion rate to 98% adjusts the Bayesian priors to account for imperfect bisulfite conversion, reducing false SNP calls caused by unconverted cytosines.

### Call variants with separate strand handling for RRBS data
**Args:** `-I rrbs_sample.bam -D genome.fa -o rrbs_results -topStrand -bottomStrand`
**Explanation:** Using both strand flags ensures that both Watson and Crick strand reads are processed independently, which is critical for RRBS libraries where coverage is inherently asymmetric.

### Limit memory usage for large genomes
**Args:** `-I large_sample.bam -D large_genome.fa -o output -maxMem 4000`
**Explanation:** Restricting Bis-SNP to 4 GB of RAM prevents out-of-memory errors on systems with constrained resources, though it may slow processing for chromosome-scale genomes.

### Output only methylation calls without SNP genotyping
**Args:** `-I sample.bam -D genome.fa -o meth_only -methOut`
**Explanation:** Using the methylation-only output flag produces a BED file focused solely on cytosine methylation levels without generating SNP VCF records, which is useful for downstream methylation analysis pipelines.

### Process multiple BAM files in batch mode
**Args:** `-list sample_list.txt -D genome.fa -o batch_output`
**Explanation:** Providing a file containing one BAM path per line enables batch processing of multiple samples simultaneously, streamlining cohort-level bisulfite variant analysis.