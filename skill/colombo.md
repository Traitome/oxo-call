---
name: colombo
category: haplotype_phasing
description: A tool for haplotype phasing, mosaic mutation detection, and trio-based inheritance analysis from sequencing data.
tags: [haplotype, phasing, genomics, vcf, bam, trio-analysis, mosaic-detection]
author: AI-generated
source_url: https://github.com/bcgsc/colombo
---

## Concepts

- Colombo operates on aligned sequencing reads (BAM/CRAM) and a reference genome to phase haplotypes and detect mosaic mutations, producing phased VCF files as primary output.
- Input requires a coordinate-sorted, indexed BAM file paired with a reference FASTA (with .fai index) and optionally a pedigree file for trio-based phasing to improve accuracy.
- The tool supports multiple analysis modes: standard phasing, trio phasing (using parental BAMs), mosaic mutation detection, and result merging across samples.
- Output formats include phased VCF (.vcf.gz), allelic depth tables, and JSON summaries of phasing statistics and confidence scores.
- Confidence scoring is based on read coverage, heterozygosity patterns, and linkage disequilibrium between variant sites within phased blocks.

## Pitfalls

- Using an unsorted or unindexed BAM file causes silent failures or missing variants in output, as Colombo requires coordinate-sorted and indexed inputs for efficient read traversal.
- Specifying incorrect sample names in trio mode results in wrong parental relationships, leading to flipped phasing calls that propagate errors to all downstream analysis.
- Running without sufficient memory for large cohorts causes OOM crashes; use the `--batch-size` flag to limit chromosomes processed concurrently.
- Misinterpreting low-confidence phased blocks as errors—the tool flags low-confidence calls but retains them; filtering these requires explicit use of a quality threshold.
- Forgetting to create output directory results in silent failures if the specified output path does not exist.

## Examples

### Phase haplotypes for a single sample using default settings
**Args:** phase -bam sample.bam -ref GRCh38.fa -sample NA12878 -out results/phased
**Explanation:** This runs standard haplotype phasing on a single individual, outputting phased VCF files to the specified directory.

### Phase using trio data with parental BAMs for improved accuracy
**Args:** phase -bam child.bam -ref GRCh38.fa -trio -father-bam father.bam -mother-bam mother.bam -pedigree family.ped -out results/trio_phased
**Explanation:** Trio mode leverages Mendelian inheritance constraints to resolve ambiguous heterozygous calls in the child sample.

### Detect mosaic mutations in a cancer sample against matched normal
**Args:** detect -phased phased.vcf.gz -tumor-bam tumor.bam -normal-bam normal.bam -min-vaf 0.02 -out mosaic_calls
**Explanation:** This identifies low-frequency variant alleles suggestive of mosaic or somatic origin by comparing allelic fractions between tumor and normal.

### Process multiple samples in batch mode for cohort analysis
**Args:** batch -sample-list samples.txt -ref GRCh38.fa -out cohort_results -threads 8
**Explanation:** Batch mode processes multiple BAM files in parallel, generating a unified VCF and per-sample statistics for population-scale phasing.

### Merge phased results across sequencing runs for the same individual
**Args:** merge -inputs run1_phased run2_phased -ref GRCh38.fa -out merged_phased
**Explanation:** Merging improves phasing completeness and confidence by combining read evidence from multiple sequencing experiments targeting the same sample.