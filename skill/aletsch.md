---
name: aletsch
category: variant_calling
description: A local assembly-based variant caller designed for pooled sequencing data. Aletsch detects genetic variants by performing localized de novo assembly of reads spanning variant sites, making it particularly effective for identifying low-frequency alleles in pooled samples from population studies.
tags:
- variant_calling
- pooled_sequencing
- local_assembly
- vcf_output
- snp_detection
- indel_detection
- genomics
author: AI-generated
source_url: https://github.com/gt1/aletsch
---

## Concepts

- **Pooled Sample Variant Calling**: Aletsch is optimized for analyzing pooled sequencing data where DNA from multiple individuals is combined into a single sequencing library. It estimates allele frequencies and detects variants present at any frequency in the pool, making it ideal for population genetics studies.
- **Local De Novo Assembly**: Unlike traditional aligners that rely on reference-based placement, Aletsch constructs local assemblies of reads overlapping suspected variant regions, allowing detection of novel alleles and complex variants that standard mappers may miss or misalign.
- **Input/Output Formats**: Aletsch accepts aligned BAM or CRAM files as input and outputs standard VCF (Variant Call Format) files containing variant calls with associated quality scores, allele frequencies, and read support information.
- **Bayesian Probabilistic Model**: The variant calling engine uses Bayesian inference to calculate posterior probabilities for each possible genotype at each position, incorporating read evidence, prior allele frequencies, and sequencing error models.

## Pitfalls

- **Unfiltered BAM Files**: Using BAM files without proper preprocessing (e.g., duplicate marking, read grouping) can lead to inflated allele frequency estimates because PCR duplicates create artificial read redundancy that mimics high read coverage.
- **Ignoring Read Group Tags**: When BAM files contain multiple read groups (e.g., from different sequencing runs or libraries), failure to specify `-- RG` parameters can cause aletsch to treat all reads as a single pool, potentially biasing frequency estimates.
- **Insufficient Pool Size Settings**: The `--pool-size` parameter must accurately reflect the number of individuals in the pool. Underestimating this value causes aletsch to overestimate allele frequencies, leading to false positive variant calls for rare alleles.
- **Reference Genome Incompatibility**: Using a different reference genome version than what the reads were aligned against creates systematic misalignments around indels and structural variants, causing aletsch to call spurious variants or miss true ones.

## Examples

### Call variants in a simple pooled BAM file

**Args:** `--pool-size 100 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** This runs aletsch on a BAM file from a pool of 100 diploid individuals, output standard VCF. The pool-size parameter sets the expected number of haplotypes (2 × individuals) for frequency calculations.

### Specify a known number of haplotypes

**Args:** `--pool-size 200 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** The pool-size should equal the total number of haplotypes in the pool (2 × number of sampled individuals). Here 200 haplotypes corresponds to 100 diploid individuals.

### Use multiple BAM files as input

**Args:** `--bam-input sample1.bam --bam-input sample2.bam --pool-size 100 --reference ref.fa --vcf-output variants.vcf`

**Explanation:** Multiple BAM files can be provided to call variants across several pooled samples simultaneously, with allele frequencies estimated separately for each sample.

### Set a minimum allele frequency threshold

**Args:** `--pool-size 50 --min-alternate-fraction 0.05 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** The `--min-alternate-fraction` flag filters out variant alleles with frequencies below the specified threshold (5%), reducing false positives for rare artifacts.

### Enable left-alignment of indels

**Args:** `--pool-size 100 --left-align-indels --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** The `--left-align-indels` option realigns indels to their leftmost position in the reference, standardizing representation and improving compatibility with other variant callers.

### Limit variant calling to specific genomic regions

**Args:** `--pool-size 100 --region chr1:1000000-2000000 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** The `--region` flag restricts analysis to specified genomic coordinates, useful for targeted variant calling or reducing compute time when analyzing whole-genome data in chunks.

### Specify read group to use from multi-sample BAM

**Args:** `--pool-size 50 --RG ID:rg1 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** When BAM files contain multiple read groups, specifying `--RG` selects only reads from that read group, allowing analysis of individual lanes or libraries within a larger dataset.

### Output probability scores for each genotype

**Args:** `--pool-size 100 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf --genotype-qualities`

**Explanation:** The `--genotype-qualities` flag adds genotype posterior probability scores (Gq format) to the output VCF, enabling more stringent filtering based on calling confidence.

### Adjust variant quality thresholds

**Args:** `--pool-size 100 --min-quality 30 --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** The `--min-quality` parameter sets the minimum variant quality (QUAL field in VCF) required to output a call, filtering low-confidence variants. A threshold of 30 corresponds to 99.9% accuracy.

### Use a predefined priors file for allele frequencies

**Args:** `--pool-size 100 --prior-file known_priors.tsv --bam-input sample.bam --reference ref.fa --vcf-output variants.vcf`

**Explanation:** Providing a `--prior-file` with known population allele frequencies improves calling accuracy by incorporating prior knowledge into the Bayesian model, particularly useful for well-studied populations.