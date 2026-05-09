---
name: cobra
category: Variant Calling
description: A haplotype-based variant caller that uses combinatorial assembly methods to detect genetic variants including SNPs, indels, and structural rearrangements from aligned sequencing reads.
tags:
  - variant-calling
  - genomics
  - haplotyping
  - snp-detection
  - structural-variants
  - germline-variants
  - somatic-variants
author: AI-generated
source_url: https://github.com/bioinformatics-tools/cobra
---

## Concepts

- **Haplotype Assembly Model**: Cobra performs local assembly of reads into candidate haplotypes before variant calling, which improves accuracy in complex regions with multiple close variants. The assembler builds a De Bruijn-style graph from k-mers extracted from aligned reads, then extracts maximal unitigs as candidate haplotypes.

- **Input/Output Formats**: Cobra accepts BAM/CRAM files as input (aligned reads) and outputs VCF files for variants. It also accepts a reference genome in FASTA format. The tool requires indexed BAM files (with .bai or .crai index) and will report an error if alignment indices are missing.

- **ploidy and Sample Handling**: Cobra models ploidy explicitly during haplotype reconstruction, allowing accurate variant calling in polyploid organisms or in regions with allelic imbalance. When analyzing multiple samples, Cobra can perform joint variant calling to improve sensitivity for rare variants across the cohort.

- **Quality Score Recalibration**: Built-in Base Quality Score Recalibration (BQSR) adjusts base call quality scores before variant detection, which reduces false positives from systematic sequencing errors. The tool applies a covariant motif analysis to identify and correct context-specific bias in quality scores.

## Pitfalls

- **Missing Index Files**: Running Cobra on an unindexed BAM file produces a non-obvious error that halts processing. Always verify that .bai or .crai files exist in the same directory as the input BAM before launching a run.

- **Incorrect Reference Compatibility**: Supplying a BAM aligned to a different reference genome version than specified in the FASTA reference causes silent false negatives where real variants go undetected without warning messages. Confirm reference consistency using samtools idxstats or md5 checksum verification.

- **Memory Exhaustion on Large Genomes**: Cobra's assembler constructs graphs proportional to genome size and read depth, which can exceed available RAM when processing whole-genome data from large genomes (human size or larger) without explicit memory limits. Use the `--max-memory` flag to constrain heap usage.

- **Overlooking Heterozygosity Parameters**: Default heterozygosity settings may not match the biology of your sample, particularly for highly inbred organisms or haploid pathogens, leading to under-calling of homozygous variants. Adjust `--heterozygosity-rate` based on known population genetics before running.

## Examples

### Call variants from a single BAM file with default settings

**Args:** `call --ref hg38.fa --bam sample1.bam --out variants.vcf`
**Explanation:** This invokes the standard variant calling pipeline on an aligned BAM file, using the provided reference to anchor haplotypes and output all discovered variants to a VCF file.

### Perform joint variant calling across multiple samples

**Args:** `call --ref hg38.fa --bam sample1.bam sample2.bam sample3.bam --joint --out cohort_variants.vcf`
**Explanation:** Joint calling combines evidence across multiple samples to improve sensitivity for low-frequency alleles and to distinguish true variants from sequencing errors present in only one sample.

### Enable somatic variant detection with paired tumor-normal analysis

**Args:** `call --ref hg38.fa --tumor tumor.bam --normal normal.bam --somatic --out somatic_variants.vcf`
**Explanation:** The somatic mode applies specialized filtering that accounts for the expected allelic fraction pattern in tumor samples, distinguishing somatic mutations from germline variants and sequencing artifacts.

### Adjust ploidy for polyploid organism analysis

**Args:** `call --ref wheat_ref.fa --bam wheat_sample.bam --ploidy 6 --out wheat_variants.vcf`
**Explanation:** Setting ploidy to 6 informs the haplotype assembler that up to 6 distinct alleles may exist at heterozygous sites, which is essential for polyploid wheat where homeologous chromosomes can contain different variants.

### Limit memory usage to prevent OOM on large datasets

**Args:** `call --ref hg38.fa --bam large_wgs.bam --max-memory 32gb --out variants.vcf`
**Explanation:** Constraining memory to 32 GB forces Cobra to process genomic regions in smaller chunks, trading some speed for stability when running on machines with limited RAM or when analyzing high-depth whole-genome data.

### Generate phased haplotypes with read-backed phasing

**Args:** `call --ref hg38.fa --bam phased_input.bam --phasing --out phased_variants.vcf`
**Explanation:** The phasing mode extends variant calling to produce phase blocks that assign alleles to specific haplotypes, using read evidence to connect heterozygous sites within phasing blocks.

### Enable Base Quality Score Recalibration before variant detection

**Args:** `call --ref hg38.fa --bam raw_quality.bam --bqsr known_sites.vcf --out recal_variants.vcf`
**Explanation:** BQSR uses a list of known polymorphic sites to train a model that corrects systematic base quality errors, reducing false positive variant calls in regions with sequencing context bias.

### Set minimum alternate allele fraction threshold for low-frequency variant detection

**Args:** `call --ref hg38.fa --bam cfDNA.bam --min-af 0.01 --out lowaf_variants.vcf`
**Explanation:** Lowering the minimum alternate allele fraction to 1% enables detection of circulating tumor DNA or low-frequency somatic variants that would be missed by default sensitivity settings tuned for germline calling.