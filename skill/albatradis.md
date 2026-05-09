---
name: albatradis
category: Genomics / Transposable Element Analysis
description: A bioinformatics tool for analyzing transposable element (TE) insertions from VCF data, computing population statistics, and detecting lineage-specific insertions.
tags: [transposable-elements, vcf, polymorphism, population-genomics, te-analysis, genomics, variants]
author: AI-generated
source_url: https://github.com/Stephentheweb/albatradis
---

## Concepts

- **VCF-based TE polymorphism model**: albatradis parses VCF files containing transposable element insertions, where each sample genotype indicates presence (1/1), absence (0/0), or heterozygous (0/1) status of a TE at a specific genomic coordinate.
- **Population frequency calculations**: The tool computes allele frequencies, genotype counts, and Hardy-Weinberg equilibrium statistics across populations defined in the sample metadata.
- **Lineage-specific insertion detection**: By comparing insertion frequencies between predefined population groups, albatradis identifies TE insertions that are specific to certain lineages or populations.
- **Output formats**: Results are typically exported as TSV tables containing genomic coordinates, insertion characteristics, frequency data, and statistical significance values.

## Pitfalls

- **Using default population groups without defining samples**: Running analysis without properly configuring population groupings via `--population` or metadata files will cause all samples to be treated as a single population, eliminating the ability to detect lineage-specific insertions.
- **Failing to filter low-quality insertions**: Not applying minimum read depth (`--min-reads`) or genotype quality (`--min-qual`) thresholds can include false-positive TE calls, skewing frequency calculations and leading to incorrect conclusions about insertion polymorphism.
- **Mismatched reference genome versions**: Using VCF files aligned to a different genome build than specified in `--genome` will produce incorrect coordinate mappings and annotations, as TE insertion positions are genome-build specific.
- **Overlooking missing genotypes as uncertain**: Treating missing genotypes (./.) as homozygous absent without the `--missing-as-null` or `--treat-missing` flag underestimates allele frequencies, particularly in low-coverage datasets.

## Examples

### Calculate allele frequencies for all TE insertions across a population
**Args:** `calculate-frequencies --input calls.vcf --output allele_freqs.tsv --population PopA`
**Explanation:** This computes allele frequencies for each transposable element insertion by counting genotypes in the specified population, outputting a tab-separated file with frequency values for all TE loci.

### Identify lineage-specific TE insertions between two populations
**Args:** `lineage-specific --input calls.vcf --pop1 EastGroup --pop2 WestGroup --min-freq-diff 0.5 --output lineage_tes.tsv`
**Explanation:** This compares insertion frequencies between two predefined population groups and outputs TE insertions with frequency differences exceeding 0.5, indicating potential lineage-specific or population-private insertions.

### Filter TE calls requiring minimum read support
**Args:** `filter --input raw_calls.vcf --min-reads 10 --min-qual 30 --output filtered_calls.vcf`
**Explanation:** This removes TE insertions from the VCF that have fewer than 10 read supports or genotyping quality scores below 30, reducing false-positive detections in downstream analyses.

### Generate genome-wide TE insertion summary statistics
**Args:** `summary-stats --input calls.vcf --by-chromosome --output te_summary.tsv`
**Explanation:** This produces summary statistics including total insertion counts, genotype distributions, and heterozygosity levels segmented by chromosome, useful for quality assessment and dataset characterization.

### Export TE insertions in BED format for genomic visualization
**Args:** `to-bed --input calls.vcf --min-allele-freq 0.05 --output te_insertions.bed`
**Explanation:** This converts TE insertion coordinates from the VCF to BED format, including only insertions with allele frequency ≥5% for visualization in genome browsers like IGV or UCSC.

### Analyze private insertions present only in one population
**Args:** `private-insertions --input calls.vcf --target-population UniquePop --reference-pops Background --output private_tes.tsv`
**Explanation:** This identifies TE insertions that are fixed or nearly fixed in one population while absent in reference populations, useful for tracing recent TE activity specific to certain lineages.