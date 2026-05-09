---
name: biopet-vcffilter
category: Variant Filtering
description: A flexible VCF filtering tool from the Biopet toolkit that applies various statistical and functional annotations to filter variants from VCF files. Supports filtering by read depth, genotype quality, allele frequency, SnpEff functional impact, and many other variant quality metrics.
tags: [vcffilter, variant-filtering, vcf, bioinformatics, snp, indel, biopet, ngs, genomics]
author: AI-generated
source_url: https://biopet.nl/
---

## Concepts

- **VCF I/O Format**: The tool operates on VCF (Variant Call Format) files, accepting either uncompressed (.vcf) or compressed (.vcf.gz) input files, and outputs filtered variants to an output VCF file while optionally writing a log file documenting all applied filters.
- **Multi-criteria Filtering**: Variants can be filtered using any combination of fields present in the VCF, including standard INFO fields (DP, QD, FS, MQ, MQRankSum, ReadPosRankSum, SOR, InbreedingCoeff), FORMAT fields (GT, GQ, AD, DP, PL), and SnpEff annotation fields (IMPACT, FUNCLASS, GENE, BIOTYPE).
- **Filter Logic and Evaluation**: Each filter is evaluated as a numeric or string comparison against the specified threshold; multiple filters are combined with AND logic (variant must pass ALL filters), and individual filter names and their pass/fail status are recorded in the FILTER column of the output VCF.
- **SnpEff Integration**: When VCF files contain SnpEff annotations, biopet-vcffilter can filter based on functional impact (HIGH, MODERATE, LOW, MODIFIER), functional class (SILENT, MISSENSE, NONSENSE, STOP_LOST), gene name, or protein domain affected.
- **Chromosome Restriction**: The `--restrictToChr` option restricts analysis to specific chromosomes or contigs listed in the VCF header, useful for sex chromosome analysis, mitochondrial analysis, or targeted chromosome studies.

## Pitfalls

- **Missing Annotation Fields**: Specifying a filter on a field that does not exist in the VCF (e.g., filtering by MQ when the field is absent) causes the tool to fail with an error, leaving no output file generated.
- **Duplicate Filter Names**: Using the same filter name multiple times in a single command overwrites previous filter definitions with the new criteria, which may silently produce unexpected results.
- **Compressed Input without Index**: Using a bgzip-compressed VCF input (.vcf.gz) without a corresponding .tbi index file will cause the tool to fail; always generate the index using `tabix` prior to filtering.
- **Floating-Point Comparison Precision**: When filtering on float fields like QD (Quality by Depth) or MQRankSum, using very small threshold differences may lead to unintended exclusion of borderline variants; always verify thresholds against your specific dataset distribution.
- **Multi-allelic Site Handling**: Filtering on AD (Allelic Depth) or allele-specific metrics at multi-allelic sites requires careful attention to which allele is being evaluated, as the tool may not distinguish between alternative alleles unless explicitly specified.

## Examples

### Filter variants by minimum read depth

**Args:** `--input variants.vcf.gz --output filtered_depth.vcf --minDP 10`

**Explanation:** Filters the input VCF to retain only variants where the total read depth (DP) in the INFO field is at least 10 reads, removing low-coverage calls that may have unreliable genotype calls.

### Filter on genotype quality threshold

**Args:** `--input variants.vcf.gz --output filtered_gq.vcf --minGq 30`

**Explanation:** Retains only variant calls with a genotype quality (GQ) score of at least 30, ensuring high-confidence genotype calls before downstream analysis.

### Restrict analysis to specific chromosomes

**Args:** `--input variants.vcf.gz --output chr_filtered.vcf --restrictToChr chr1,chr2,chr3`

**Explanation:** Limits the output to variants located on chromosomes 1, 2, and 3 only, useful for subsetting analysis to autosomes or specific genomic regions of interest.

### Filter by SnpEff high functional impact

**Args:** `--input annotated.vcf.gz --output high_impact.vcf --snpEffImpact HIGH`

**Explanation:** Retains only variants with a HIGH functional impact annotation from SnpEff, prioritizing potentially protein-altering variants such as nonsense or frameshift mutations.

### Remove common variants with minor allele frequency threshold

**Args:** `--input population.vcf.gz --output rare_variants.vcf --maxMAF 0.01`

**Explanation:** Filters out variants with a minor allele frequency greater than 1%, keeping only rare variants that may be more likely to have functional significance in disease studies.

### Combine multiple filtering criteria

**Args:** `--input variants.vcf.gz --output strict_filtered.vcf --minDP 20 --minGq 50 --minQual 100`

**Explanation:** Applies three simultaneous filters requiring at least 20x read depth, genotype quality of 50, and variant quality of 100, producing a highly stringent set of high-confidence variants.

### Filter based on strand bias

**Args:** `--input variants.vcf.gz --output bias_filtered.vcf --maxFS 30.0`

**Explanation:** Removes variants with a Fisher's exact test strand bias (FS) value greater than 30, eliminating variants that show extreme strand imbalance suggesting sequencing artifacts.