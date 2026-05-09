---
name: cluster_vcf_records
category: variant_manipulation
description: Groups VCF records that overlap or are within a specified genomic distance threshold, useful for consolidating redundant variant calls from multiple callers or批次数据. Produces a VCF with clustered records.
tags: [vcf, clustering, variant_consolidation, genomics]
author: AI-generated
source_url: https://github.com/vcftools/vcftools
---

## Concepts

- **Input Format**: Accepts standard VCF files (version 4.0+) as input, including compressed (.gz) or uncompressed files. Multi-sample VCFs are supported. The tool reads VCF records sequentially and applies clustering logic based on position and allele overlap.

- **Clustering Logic**: Records are clustered when they overlap at the same genomic position or when their start/end coordinates fall within a user-specified distance threshold (measured in base pairs). Overlapping alleles (identical REF/ALT) within a cluster are merged; distinct alleles may be retained depending on mode.

- **Output**: Produces a new VCF where clustered records are combined. The output retains INFO and FORMAT fields from the most representative record in each cluster, and adds a new `CLUSTER_ID` INFO field to indicate which records belong to the same cluster. The original record order is generally preserved.

- **Key Flags**: Use `--distance` or `-d` to set the clustering distance threshold in base pairs. Use `--sample` to restrict clustering to specific samples in multi-sample VCFs. Use `--output` to specify the output VCF filename.

## Pitfalls

- **Setting distance too large**: Specifying an overly large clustering distance (e.g., `-d 10000`) will merge variants that are actually far apart, collapsing distinct mutations into single records and losing important information about separate events. This corrupts downstream analyses like association studies.

- **Ignoring multiallelic sites**: When clustering records with different ALT alleles at the same position (multiallelic sites), naive clustering may incorrectly merge distinct variant types. The tool preserves alleles but downstream tools may misinterpret combined INFO fields, leading to false genotype calls.

- **Forgetting that ordering matters**: VCF processing assumes a coordinate-sorted input. If the input VCF is not sorted by chromosome and position, clustering will be inconsistent and may produce incorrect groupings. Always sort input with `bcftools sort` first.

- **Output filename collisions**: Writing output to a file that already exists will silently overwrite it without warning, potentially losing valuable intermediate data. Always verify output paths before running.

- **Memory with large files**: Very large VCF files (millions of records) may consume significant memory during clustering. Without specifying `--buffer-size`, extremely large inputs may cause out-of-memory errors on systems with limited RAM.

## Examples

### Cluster variants within 100bp of each other
**Args:** `-d 100 input.vcf.gz -o clustered_output.vcf.gz`
**Explanation:** Sets a 100-base-pair maximum distance for clustering; records within this window are grouped together, and the output VCF is written to the specified file.

### Consolidate overlapping variant calls from multiple callers
**Args:** `--merge input.vcf.gz -o merged.vcf.gz`
**Explanation:** Uses merge mode to combine variant calls from different callers that overlap at the same genomic position, reducing redundancy in downstream analysis.

### Cluster only records in a specific sample
**Args:** `--sample NA12878 -d 50 in.vcf.gz -o na12878_clustered.vcf.gz`
**Explanation:** Restricts clustering to variants present in sample NA12878 while preserving records from other samples unchanged, useful for sample-specific analyses.

### Preserve structural variant annotations during clustering
**Args:** `-d 200 --keep-sv input.vcf.gz -o sv_clustered.vcf.gz`
**Explanation:** Retains structural variant-specific INFO fields (like END, SVLEN, SVTYPE) when clustering large variants, ensuring annotations are not lost in the output.

### Save clustering statistics to a log file
**Args:** `-d 100 --stats clustered_stats.txt input.vcf.gz -o output.vcf.gz`
**Explanation:** Writes summary statistics about cluster sizes, number of merged records, and positions to a log file for downstream quality assessment and reporting.

### Exclude symbolic ALT alleles from clustering
**Args:** `-d 100 --no-symbolic input.vcf.gz -o clean_output.vcf.gz`
**Explanation:** Skips clustering for records with symbolic ALT alleles (like