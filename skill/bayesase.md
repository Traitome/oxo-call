---
name: bayesase
category: sequence_analysis
description: A Bayesian probabilistic model-based tool for sequence analysis, capable of performing sequence alignment, variant calling, and haplotype estimation using Bayesian inference algorithms. The tool models uncertainty in alignment and variant detection through posterior probability distributions.
tags:
- bayesian-inference
- sequence-analysis
- variant-calling
- haplotypes
- probabilistic-models
- bioinformatics
author: AI-generated
source_url: https://github.com/bayesase/bayesase
---

## Concepts

- **Input Formats:** bayesase accepts FASTA, FASTQ, and BAM alignment files as primary inputs. It supports both single-end and paired-end sequencing data, and can process multiple alignment files simultaneously when provided as a comma-separated list.
- **Output Formats:** The tool generates VCF (Variant Call Format) files for variant calling results, as well as custom TAB-delimited files containing posterior probability scores for each called variant. A summary report in TSV format is also produced by default.
- **Bayesian Model:** The tool uses a probabilistic graphical model with a Dirichlet process prior to handle uncertainty in variant allele frequencies. The model estimates posterior probabilities for each potential genotype at each genomic position, requiring a reference genome FASTA file for read alignment context.
- **Statistical Thresholds:** By default,bayesase calls variants with a posterior probability threshold of 0.9 (90% confidence). This threshold is adjustable via the posterior-cutoff parameter. The tool reports all positions with their associated posterior probabilities, regardless of the threshold.

## Pitfalls

- **Reference Genome Mismatch:** Using a reference genome that does not match the sample species or strain leads to false positive variant calls. This occurs because mismapped reads create artificial mismatches that the Bayesian model interprets as valid variants. Always verify the reference genome build/version matches your sample.
- **Insufficient Sequencing Depth:** Low-coverage data (below 10x) produces unreliable posterior probability estimates, especially for heterozygous variants. The Bayesian model still produces output but with wider confidence intervals. Specifying low-depth regions in the exclusion BED file helps, but results may be statistically underpowered.
- **Ignoring Population Structure:** When analyzing multiple samples, failing to account for population structure in the prior distribution leads to biased allele frequency estimates. The tool includes a population-prior flag that should be used for cohort analyses to avoid overestimating rare variants.
- **Missing Read Group Information:** BAM files without proper read group annotations cause sample misattribution in multi-sample analyses. The Bayesian model will combine reads from different samples incorrectly, producing inflated heterozygosity rates. Always validate read group fields before running analyses.

## Examples

### Call variants from an aligned BAM file using the default settings
**Args:** --input aligned_reads.bam --reference ref_genome.fa --output variants.vcf
**Explanation:** This runs variant calling with default parameters, including the 0.9 posterior probability threshold and standard read filtering. The output includes only variants that meet the posterior cutoff.

### Analyze multiple samples simultaneously for cohort study
**Args:** --input sample1.bam,sample2.bam,sample3.bam --reference ref_genome.fa --output cohort_variants.vcf --population-prior --min-coverage 20
**Explanation:** This enables population-based prior distribution modeling across three samples, which improves allele frequency estimates for rare variants. The min-coverage filter excludes low-confidence positions.

### Adjust the posterior probability threshold for stringent variant calling
**Args:** --input sample.bam --reference ref_genome.fa --output strict_variants.vcf --posterior-cutoff 0.99 --min-read-quality 30
**Explanation:** Using a 0.99 threshold requires 99% confidence for variant calls, drastically reducing false positives but potentially missing true positives. The min-read-quality further filters low-quality base calls.

### Generate detailed posterior probability output for all genomic positions
**Args:** --input sample.bam --reference ref_genome.fa --output all_positions.tsv --report-all-positions --posterior-cutoff 0.0
**Explanation:** Setting the posterior-cutoff to 0.0 ensures all positions are reported, regardless of variant confidence. This generates the full posterior distribution file for downstream statistical analysis.

### Exclude problematic genomic regions from analysis
**Args:** --input sample.bam --reference ref_genome.fa --output filtered_variants.vcf --exclude-bed problem_regions.bed --min-variant-quality 50
**Explanation:** The exclude-bed parameter skips variant calling in specified regions such as segmental duplications or known problematic loci. The min-variant-quality filter removes low-quality variant candidates.

### Run in parallel on multiple cores for faster processing
**Args:** --input large_sample.bam --reference ref_genome.fa --output variants.vcf --threads 8 --memory 16G
**Explanation:** Multi-threaded execution distributes the Bayesian model's markov chain computation across 8 processing threads, significantly reducing runtime. Memory allocation of 16GB prevents out-of-memory errors on large genomes.

### Perform haplotype assembly using linkage information
**Args:** --input sample.bam --reference ref_genome.fa --output haplotypes.vcf --run-haploassembly --linkage-disequilibrium
**Explanation:** This enables haplotype phasing using linkage disequilibrium information from the read alignments, producing more accurate haplotype resolved variant calls for population genetics analyses.