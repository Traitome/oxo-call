---
name: cancerit-allelecount
category: variant-calling
description: Count reference and alternate alleles at specified genomic positions from tumor and/or normal BAM files, producing allele depth ratios for LOH detection and copy number analysis.
tags: [allelic-counting, tumor-normal, LOH-detection, copy-number, BAM]
author: AI-generated
source_url: https://github.com/cancerit/AlleleCount
---

## Concepts

- **Input data model**: The tool reads genomic positions from a BED file (chromosome, start, end, optional name columns) and extracts reads from one or two BAM files. For tumor-normal analysis, reads from the matched normal are used to establish germline allele frequencies, while reads from the tumor reveal somatic changes.
- **Allele counting mechanics**: At each position, reads are filtered by mapping quality (MQ) and base quality (BQ) thresholds, then classified as reference-forward, reference-reverse, alternate-forward, or alternate-reverse based on the called base. Reads with insertions, deletions, or soft-clips spanning the position are flagged but may be included if configured.
- **Output format**: Results are written tab-delimited with columns: chromosome, position, reference depth, alternate depth, total depth, allele frequency, and strand counts. For paired analysis, both tumor and normal counts appear on separate lines or as combined output depending on mode.
- **Haploid region handling**: Chromosome Y, mitochondrial DNA, and certain copy-number-aberrant regions require haploid allele frequency expectations. Specifying `--haploid` adjusts the statistical thresholds and reported heterozygosity assumptions accordingly.
- **Companion binary**: `cancerit-allelecount-build` constructs reference index files (FASTA registry) used by the main binary for base-calling alignment against the reference genome.

## Pitfalls

- **Swapping tumor and normal inputs**: Providing the normal BAM as the tumor file (or vice versa) inverts the allele frequency comparison, causing LOH calls to be reported for the wrong sample with inverted ratios.
- **BED file coordinate mismatch**: Using 0-based BED coordinates when the tool expects 1-based coordinates shifts every position by one base, corrupting all allele counts across the dataset.
- **Insufficient quality thresholds**: Setting `--min-mapping-quality 0` or `--min-base-quality 0` includes highly unreliable reads, inflating total depth and distorting allele frequency calculations toward noisy mid-range values.
- **Ignoring strand bias**: Without `--strand-counts` or post-hoc strand ratio checks, alleles supported primarily by forward or reverse reads appear biased due to sequencing artifacts, leading to false somatic mutation calls.
- **Haploid regions treated as diploid**: Analyzing chromosome Y without `--haploid` flags causes expected heterozygosity calculations to assume two alleles where only one exists, producing systematically elevated LOH predictions.
- **Mismatched reference genome**: Running with a BED file built against hg38 but providing a hg19 FASTA reference causes systematic base-calling errors, particularly at positions with known genome differences, rendering entire datasets unreliable.

## Examples

### Basic allele counting at genomic positions from a BED file
**Args:** `--threads 4 --inputtumour tumour.bam --input-normal none --file-ref-genes positions.bed --reference GRCh37.fa --output sample_counts.csv`
**Explanation:** Runs single-sample counting on a tumor BAM at all positions defined in the BED file, writing depth and allele frequency results to a CSV.

### Paired tumor-normal analysis for LOH detection
**Args:** `--inputtumour tumour.bam --input-normal normal.bam --file-ref-genes targets.bed --reference GRCh37.fa --output paired_counts.csv --min-base-quality 20`
**Explanation:** Performs paired analysis comparing tumor and normal allele frequencies at each position, enabling direct detection of loss-of-heterozygosity regions.

### High-confidence allele counting with strict quality filters
**Args:** `--inputtumour highcov.bam --file-ref-genes highconf_sites.bed --reference GRCh37.fa --min-mapping-quality 60 --min-base-quality 30 --min-coverage 20 --output filtered_counts.csv`
**Explanation:** Applies stringent mapping and base quality thresholds to minimize sequencing noise, producing reliable allele frequencies for downstreamCopy Number analysis.

### Strand-specific allele counting for artifact identification
**Args:** `--inputtumour sample.bam --input-normal normal.bam --file-ref-genes positions.bed --reference GRCh37.fa --output strand_counts.csv --strand-counts`
**Explanation:** Reports forward and reverse strand read counts separately, allowing downstream evaluation of strand bias and filtering of artifactual allele calls.

### Haploid chromosome Y analysis for male samples
**Args:** `--inputtumour male_tumor.bam --input-normal male_normal.bam --file-ref-genes chrY_targets.bed --reference GRCh37.fa --output chrY_counts.csv --haploid`
**Explanation:** Treats chromosome Y as haploid, adjusting allele frequency expectations to 0% or 100% for homozygous calls and preventing false LOH predictions in hemizygous regions.

### Multi-sample cohort analysis with parallel processing
**Args:** `--inputtumour cohort/*.bam --file-ref-genes panel.bed --reference GRCh37.fa --output cohort_counts.csv --min-mapping-quality 50 --min-coverage 10 --threads 8`
**Explanation:** Processes multiple tumor BAM files in parallel, producing a combined allele count table for cohort-wide copy number or driver mutation analysis.