---
name: bcftools
category: variant_calling
description: A versatile set of utilities for manipulating variant call format (VCF) and binary VCF (BCF) files. bcftools provides functions for viewing, filtering, calling, merging, annotating, and analyzing genetic variants from next-generation sequencing data. It is part of the SAMtools ecosystem and works seamlessly with BAM/CRAM files for integrated variant analysis workflows.
tags: [variant-calling, vcf, bcf, snp, indel, genomics, samtools, bioinformatics]
author: AI-generated
source_url: https://samtools.github.io/bcftools/
---

## Concepts

- **VCF and BCF I/O Formats**: bcftools operates on both VCF (human-readable text) and BCF (binary compressed) formats. When converting between formats, use `-O` to specify output type (v for VCF, b for BCF, z for compressed VCF). BCF files require indexing with bcftools index for efficient random access operations.
- **Zero-Based Coordinate System**: bcftools uses 0-based coordinates for genomic intervals (like BED files), unlike 1-based systems such as GFF or VCF positions themselves. When specifying regions with `-r`, the start is 0-based and end is exclusive, so `-r chr1:100-200` captures positions 100 through 199.
- **Expression-Based Filtering**: The `-i` (include) and `-e` (exclude) flags accept boolean expressions using variant annotations (e.g., `QUAL>30`, `DP>10`, `GT="het"`, `FORMAT/DP>5`). Expressions can combine multiple conditions with `&&` and `||` operators, enabling complex variant filtering in a single pass.
- **Multi-Sample Variant Calling**: bcftools call supports multi-sample calling with the `-m` flag for multiallelic caller. Use `-S` to specify sample-specific options like ploidy, and `--known` to incorporate prior variant sites from a VCF file as training sets.

## Pitfalls

- **Mismatched Index Files**: Attempting to perform random access operations (like `-r` region filtering) on a VCF/BCF file without a corresponding .tbi or .csi index produces an error or crashes silently. Always generate indexes with `bcftools index file.vcf.gz` before subsetting.
- **Forgetting to Specify Output Format**: By default, `bcftools view` outputs to BCF format, which may be unreadable in text editors. If you need a text file, explicitly specify `-Ov` (output VCF) or `-Oz` (output compressed VCF) to avoid confusion when downstream tools expect text input.
- **Confusing Sample and Population Analyses**: When merging VCF files from multiple samples, not specifying the correct sample names or using `-d` (deduplicate) can cause genotype confusion. The `--force-samples` flag prevents silent sample omission if names collide.
- **Incorrect Ploidy Settings**: Default ploidy is 1 (haploid) for the X chromosome in male mammals and 2 for autosomes. Using wrong ploidy in variant calling (via `-P`) leads to genotype miscalls, especially on sex chromosomes in sex-aware pipelines.

## Examples

### Convert a VCF file to compressed BCF format
**Args:** `-Oz -o output.vcf.gz input.vcf`
**Explanation:** This converts the text VCF to compressed VCF format (BGZF), which is smaller and indexable for random access operations.

### Filter variants with quality score above 50
**Args:** `view -i 'QUAL > 50' input.vcf -o filtered.vcf`
**Explanation:** The `-i` flag applies an expression filter, retaining only variants where the QUAL field exceeds 50, reducing false positives.

### Call variants from an aligned BAM file using the multiallelic caller
**Args:** `call -m -f GQ,GP input.bam -o variants.vcf`
**Explanation:** The `-m` flag enables the multiallelic caller which handles complex scenarios with multiple alternative alleles per site, outputting genotype qualities (GQ) and genotype probabilities (GP).

### Extract variants from a specific genomic region
**Args:** `view -r chr1:1000000-2000000 input.vcf -o region.vcf`
**Explanation:** Using zero-based coordinates, this extracts all variant records within the 1 Mb region on chromosome 1 for focused analysis.

### Merge multiple VCF files into one
**Args:** `concat -a file1.vcf file2.vcf file3.vcf -o merged.vcf`
**Explanation:** The `-a` flag performs union merging (all sites), combining variants across files while preserving sample genotypes.

### Viewonly passing records matching a specific sample genotype
**Args:** `view -s NA12878 -g het input.vcf`
**Explanation:** The `-s` selects a specific sample, and `-g het` further filters to only heterozygous genotypes for that individual.

### Compute per-sample allele counts
**Args:** `stats -s - input.vcf`
**Explanation:** The `-s -` flag reports sample-level statistics including allele counts (AC) and genotype frequencies for every variant in the file.

### Annotate variants with functional consequence predictions
**Args:** `annotate -a annotations.vcf -c ID input.vcf`
**Explanation:** This adds annotation data from a separate VCF file, populating ID fields or other specified columns for each variant.

### Sort a VCF file by genomic position
**Args:** `sort -o sorted.vcf input.vcf`
**Explanation:** bcftools sort orders records by chromosome and position, which is required for valid indexing and for tools expecting sorted input.

### Create an index for random access in a compressed VCF
**Args:** `index output.vcf.gz`
**Explanation:** This generates a .tbi (Tabix) index file enabling fast retrieval of specific genomic regions without scanning the entire file.