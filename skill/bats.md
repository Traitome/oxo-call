---
name: bats
category: sequence_analysis
description: A bioinformatics tool for analyzing base-level sequence variations and detecting structural variants using read-level evidence.
tags: [variant-calling, sequence-analysis, structural-variants, genomics]
author: AI-generated
source_url: https://github.com/bats-tools/bats
---

## Concepts

- **Input formats**: BATS accepts aligned BAM/CRAM files as primary input, along with a reference genome in FASTA format for variant context analysis.
- **Data model**: The tool operates on read pileups and generates variant calls with associated quality scores, coverage depth, and read-level supporting evidence.
- **Output**: Produces VCF/BED files containing annotated variants, including information about variant type (SNV, indel, structural rearrangement), allele frequency, and read support metrics.
- **Key behaviors**: Performs statistical filtering based on minimum read coverage thresholds and applies Bayesian probability models to distinguish true variants from sequencing artifacts.

## Pitfalls

- **Using unindexed BAM files**: If the input BAM file lacks a corresponding index (.bai), BATS will fail to parse read alignments, resulting in runtime errors and incomplete variant detection.
- **Specifying incompatible reference versions**: Providing a reference genome version that differs from the alignment reference will cause positional discrepancies and produce false or missed variant calls.
- **Ignoring quality score thresholds**: Running without setting appropriate minimum quality filters (QUAL/DP) will generate excessive false-positive variants, especially in low-complexity genomic regions.
- **Processing compressed files incorrectly**: Attempting to stream gzipped VCF output without proper pipe handling results in corrupted output files that cannot be validated by downstream tools.

## Examples

### Call variants from an aligned BAM file
**Args:** -i sample1.bam -r hg38.fa -o variants.vcf
**Explanation:** Directs BATS to analyze sample1.bam using hg38 reference genome and output variant calls to variants.vcf.

### Filter variants by minimum read depth
**Args:** -i sample.bam -r hg38.fa --min-depth 10 -o filtered.vcf
**Explanation:** Requires at least 10 supporting reads at each position before calling a variant, reducing false positives from low-coverage regions.

### Enable heterozygous variant detection
**Args:** -i sample.bam -r hg38.fa --het-ratio 0.3 -o het_calls.vcf
**Explanation:** Sets the minimum allele ratio threshold to 0.3 for detecting heterozygous variants rather than homozygosity.

### Output in compressed BCF format
**Args:** -i sample.bam -r hg38.fa -O bcf -o variants.bcf
**Explanation:** Writes binary compressed variant calls directly to BCF format for efficient storage and faster downstream processing.

### Generate variant statistics report
**Args:** -i sample.bam -r hg38.fa --stats-report -o variants.vcf
**Explanation:** Produces an additional report file containing summary statistics about transition/transversion ratios, coverage distribution, and variant density per chromosome.