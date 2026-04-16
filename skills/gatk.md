---
name: gatk
category: variant-calling
description: Genome Analysis Toolkit — best-practice variant discovery pipeline for germline and somatic variants
tags: [variant-calling, snp, indel, germline, somatic, gatk, best-practices, gvcf, bqsr, vqsr]
author: oxo-call built-in
source_url: "https://gatk.broadinstitute.org/hc/en-us"
---

## Concepts

- GATK is invoked as 'gatk <tool>' (e.g., 'gatk HaplotypeCaller'); each tool has many options.
- GATK requires sorted, duplicate-marked BAM with a read group (@RG) — use MarkDuplicates before variant calling.
- The reference FASTA must be indexed with 'samtools faidx ref.fa' and have a sequence dictionary 'gatk CreateSequenceDictionary -R ref.fa'.
- HaplotypeCaller: germline variant calling; Mutect2: somatic variant calling; GenotypeGVCFs: joint genotyping.
- The GVCF workflow for cohort studies: HaplotypeCaller -ERC GVCF → GenomicsDBImport → GenotypeGVCFs.
- BQSR (Base Quality Score Recalibration) is a two-step process: (1) BaseRecalibrator generates a recalibration table, (2) ApplyBQSR applies it to the BAM.
- VQSR (VariantQualityScoreRecalibration) is preferred over hard filtering for large cohorts (>30 samples).
- For small cohorts, use hard filtering: SNPs: QD<2, FS>60, MQ<40; Indels: QD<2, FS>200.
- SelectVariants extracts a subset of variants from a VCF by type (SNP/INDEL), sample, region, or filter status.
- --alleles forces genotyping at specific sites regardless of evidence; useful for targeted re-sequencing.
- --max-alternate-alleles limits the number of alternate alleles considered (default 6); increase for highly polymorphic regions.
- --min-pruning controls haplotype pruning during local assembly; lower values increase sensitivity but may increase runtime.
- GenomicsDBImport creates a GenomicsDB workspace from multiple GVCFs; required before GenotypeGVCFs for large cohorts.
- AnalyzeCovariates evaluates BQSR tables and generates plots for quality assessment.
- --spark-runner enables distributed execution for Spark-enabled tools (MarkDuplicatesSpark, BQSRPipelineSpark).

## Pitfalls

- CRITICAL: GATK ARGS must start with a tool name subcommand (HaplotypeCaller, Mutect2, MarkDuplicates, BaseRecalibrator, ApplyBQSR, CreateSequenceDictionary, AddOrReplaceReadGroups, SelectVariants, GenotypeGVCFs, GenomicsDBImport, FilterMutectCalls, ValidateSamFile, SortSam, CollectAlignmentSummaryMetrics, CollectInsertSizeMetrics) — never with flags like -R, -I, -O. The tool name ALWAYS comes first.
- GATK requires a sequence dictionary (.dict file) alongside the reference FASTA — run CreateSequenceDictionary first.
- HaplotypeCaller needs read groups in the BAM; if missing, add with AddOrReplaceReadGroups first.
- Mutect2 somatic calling requires a matched normal sample with -I normal.bam -normal normal_sample_name.
- The --tmp-dir argument should point to a large temporary directory — GATK creates huge temp files.
- GATK4 uses Java 17+ by default; the gatk wrapper script handles this but the JAR must be executable.
- For WGS, always provide --intervals chr1 chr2 ... to parallelize; omitting it runs on the entire genome sequentially.
- BaseRecalibrator requires at least one --known-sites VCF (e.g., dbSNP, Mills indels) — omitting it causes an error.
- ApplyBQSR must receive the recalibration table from BaseRecalibrator via --bqsr-recal-file; do not skip this flag.
- GenomicsDBImport requires non-overlapping intervals; use -L with interval list for proper partitioning.
- --alleles forces genotyping but may produce low-quality calls if no evidence exists; filter post-calling.
- Spark tools require --spark-runner LOCAL (or SPARK/GCS); omitting causes errors on Spark-enabled tools.
- HaplotypeCaller -ERC GVCF mode requires GenotypeGVCFs for final VCF; do not use GVCF directly for analysis.

## Examples

### call germline variants from a BAM file using HaplotypeCaller
**Args:** `HaplotypeCaller -R reference.fa -I sorted_markdup.bam -O output.g.vcf.gz -ERC GVCF`
**Explanation:** -ERC GVCF creates a GVCF for later joint genotyping; needed for multi-sample workflows

### genotype a single sample directly (not GVCF mode)
**Args:** `HaplotypeCaller -R reference.fa -I sorted_markdup.bam -O variants.vcf.gz`
**Explanation:** without -ERC GVCF, outputs a standard VCF directly; suitable for single-sample projects

### mark PCR duplicates in a BAM file
**Args:** `MarkDuplicates -I input.bam -O markdup.bam -M metrics.txt`
**Explanation:** removes/marks PCR duplicates; -M writes duplicate metrics; required before HaplotypeCaller

### call somatic mutations with Mutect2 using matched normal
**Args:** `Mutect2 -R reference.fa -I tumor.bam -I normal.bam -normal normal_sample_name -O somatic.vcf.gz`
**Explanation:** for somatic calling; -normal identifies which BAM is the matched normal

### filter Mutect2 variants with FilterMutectCalls
**Args:** `FilterMutectCalls -R reference.fa -V somatic.vcf.gz -O filtered_somatic.vcf.gz`
**Explanation:** applies filters to Mutect2 raw calls; variants PASS the filter or are tagged with a filter name

### create a sequence dictionary for a reference FASTA
**Args:** `CreateSequenceDictionary -R reference.fa`
**Explanation:** creates reference.dict in the same directory; required by HaplotypeCaller and other tools

### add read group to a BAM file (required before GATK variant calling)
**Args:** `AddOrReplaceReadGroups -I input.bam -O output_rg.bam -RGID sample1 -RGLB lib1 -RGPL ILLUMINA -RGPU unit1 -RGSM sample1`
**Explanation:** all four -RG fields are required; RGSM is the sample name used in the VCF

### perform base quality score recalibration (BQSR step 1) on markdup BAM with hg38 reference and dbSNP known sites
**Args:** `BaseRecalibrator -R hg38.fa -I markdup.bam --known-sites dbsnp.vcf -O recal.table`
**Explanation:** step 1 of BQSR: generates a recalibration table; --known-sites can be repeated for multiple VCFs (e.g., dbSNP + Mills indels)

### apply base quality score recalibration (BQSR step 2) to produce recalibrated BAM
**Args:** `ApplyBQSR -R hg38.fa -I markdup.bam --bqsr-recal-file recal.table -O recal.bam`
**Explanation:** step 2 of BQSR: applies the recalibration table from BaseRecalibrator to produce a recalibrated BAM ready for variant calling

### select only SNPs from a variants VCF
**Args:** `SelectVariants -V variants.vcf -O SNPs.vcf --select-type-to-include SNP`
**Explanation:** extracts SNP-type variants only; use INDEL for indels; can be combined with -L for region filtering

### joint genotype multiple samples from GenomicsDB
**Args:** `GenotypeGVCFs -R reference.fa -V gendb://genomicsdb -O cohort.vcf.gz`
**Explanation:** final joint genotyping step for cohort GVCF workflow; input is a GenomicsDB workspace created by GenomicsDBImport

### force genotype specific alleles from a target list
**Args:** `HaplotypeCaller -R reference.fa -I input.bam -O forced.vcf.gz --alleles targets.vcf --force-call-filtered-alleles`
**Explanation:** --alleles forces genotyping at target sites; --force-call-filtered-alleles includes filtered sites; useful for targeted re-sequencing

### import multiple GVCFs into GenomicsDB for joint calling
**Args:** `GenomicsDBImport -V sample1.g.vcf.gz -V sample2.g.vcf.gz -V sample3.g.vcf.gz --genomicsdb-workspace-path genomicsdb -L intervals.list`
**Explanation:** creates GenomicsDB workspace from multiple GVCFs; -L specifies non-overlapping intervals for partitioning

### run BQSR with multiple known sites
**Args:** `BaseRecalibrator -R reference.fa -I input.bam --known-sites dbsnp.vcf.gz --known-sites Mills_indels.vcf.gz -O recal.table`
**Explanation:** multiple --known-sites arguments for comprehensive recalibration; recommended to include both SNP and indel databases

### analyze BQSR recalibration quality
**Args:** `AnalyzeCovariates -before recal.table -plots bqsr_plots.pdf -csv bqsr_metrics.csv`
**Explanation:** evaluates recalibration table and generates quality plots; useful for validating BQSR performance

### run Spark-enabled MarkDuplicates for large BAMs
**Args:** `MarkDuplicatesSpark -I input.bam -O markdup.bam -M metrics.txt --spark-runner LOCAL`
**Explanation:** MarkDuplicatesSpark uses Spark for parallelization; --spark-runner LOCAL runs on single machine; faster for large files

### call variants with increased sensitivity for indels
**Args:** `HaplotypeCaller -R reference.fa -I input.bam -O variants.vcf.gz --min-pruning 1 --max-alternate-alleles 10`
**Explanation:** --min-pruning 1 reduces haplotype pruning for better indel detection; --max-alternate-alleles 10 allows more alternates in complex regions

### validate SAM/BAM file before GATK analysis
**Args:** `ValidateSamFile -I input.bam -MODE SUMMARY`
**Explanation:** checks BAM validity and reports errors; MODE SUMMARY gives overview; use MODE VERBOSE for detailed error listing

### sort BAM by coordinate for GATK compatibility
**Args:** `SortSam -I unsorted.bam -O sorted.bam -SO coordinate`
**Explanation:** sorts BAM by coordinate; GATK requires coordinate-sorted input; -SO coordinate specifies sort order
