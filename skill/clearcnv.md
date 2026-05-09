---
name: clearcnv
category: variant-calling
description: A tool for detecting and analyzing copy number variations (CNVs) from sequencing data, commonly used in single-cell genomics and cancer genomics studies. ClearCNV identifies genomic regions with abnormal copy number states by analyzing read depth and allelic ratios.
tags: [cnv, copy-number-variation, single-cell, cancer-genomics, read-depth-analysis, variant-calling]
author: AI-generated
source_url: https://github.com/kloudteltron/clearcnv
---

## Concepts

- **Read-depth-based CNV detection**: ClearCNV calculates copy number states by examining the depth of coverage across the genome. Windows with significantly higher or lower read counts compared to the baseline diploid expectation are flagged as potential CNV regions. The tool uses a hidden Markov model (HMM) or similar statistical approach to segment the genome into discrete copy number states.

- **Input formats**: ClearCNV accepts aligned sequencing data in BAM format (with associated BAI index files) and requires a reference genome sequence in FASTA format. For single-cell workflows, barcoded BAM files or matrix market format may be used alongside a cell barcode manifest. The tool outputs results in BED format (CNV calls with coordinates and copy number estimates) and may generate visualization files for downstream inspection.

- **Allelic ratio integration**: In addition to read depth, ClearCNV can incorporate heterozygous SNP positions to establish allelic balance, which improves copy number state estimation. Biallelic deletion signatures and loss-of-heterozygosity (LOH) regions can be detected by combining read depth depletion with reduced B-allele frequency variance.

- **Sample batch effects**: When processing multiple samples, ClearCNV should be run independently per sample with consistent bin sizes and statistical thresholds. Merging CNV calls across samples for cross-sample comparison requires post-processing with tools like bedtools or custom scripts rather than within ClearCNV itself.

- **Normal reference normalization**: For tumor-only samples, ClearCNV can leverage a matched normal sample to subtract background coverage, reducing false-positive CNV calls from mapping artifacts or repetitive regions. The normal sample should be from the same individual and sequenced to comparable or greater depth.

## Pitfalls

- **Unindexed BAM files**: Providing a BAM file without a corresponding BAI index file causes ClearCNV to fail silently or produce incomplete results. Always ensure both `sample.bam` and `sample.bam.bai` exist in the same directory before running the analysis.

- **Mismatched reference genome**: Using a different reference build than the one to which the reads were aligned leads to incorrect copy number estimates. If the BAM was aligned to GRCh38, the reference FASTA passed to ClearCNV must also be GRCh38, not GRCh37 or hg19.

- **Insufficient sequencing depth**: Samples with average coverage below 10x may produce unreliable CNV calls with wide confidence intervals. ClearCNV will still run but will flag low-confidence regions in the output; ignoring these warnings and treating all calls as equally confident is a common analytical error.

- **Ignoring GC bias correction**: Genomic regions with high or low GC content exhibit systematic coverage bias unrelated to copy number. Failing to specify GC correction (via the `--gc-correction` flag) when such correction is available in the reference configuration results in spurious CNV calls in GC-extreme regions.

- **Oversegmentation in noisy data**: Setting excessively small window sizes or using strict p-value thresholds on samples with library quality issues leads to fragmented CNV calls that represent technical noise rather than true biological events. Checking the relationship between input quality metrics and segmentation output helps identify this condition.

## Examples

### Detecting CNVs from a tumor BAM file with default settings

**Args:** `call --bam tumor_sample.bam --ref Homo_sapiens.GRCh38.dna.primary_assembly.fa --output tumor_cnv_results`
**Explanation:** This command runs ClearCNV with default bin size and statistical thresholds to identify copy number alterations across the genome from a single tumor BAM file.

### Generating CNV calls with GC bias correction enabled

**Args:** `call --bam sample.bam --ref ref.fa --gc-correction --output gc_corrected_cnvs`
**Explanation:** This command enables GC content bias correction to reduce false-positive CNV calls in genomic regions with extreme nucleotide composition, improving specificity in datasets with known GC effects.

### Running CNV analysis with a matched normal sample for background subtraction

**Args:** `call --bam tumor.bam --normal normal.bam --ref ref.fa --output matched_normal_analysis`
**Explanation:** This command uses the matched normal sample to establish baseline coverage, allowing ClearCNV to subtract germline signal and more accurately detect somatic copy number alterations.

### Adjusting segmentation stringency to reduce oversegmentation

**Args:** `call --bam sample.bam --ref ref.fa --p-value 0.01 --min-seg-length 10 --output stringent_calls`
**Explanation:** This command increases the minimum segment length to 10 bins and raises the p-value threshold to 0.01, reducing fragmented calls caused by noisy read depth data from low-quality libraries.

### Exporting CNV results in BED format for downstream genome browser visualization

**Args:** `export --input cnv_results.json --format bed --output cnv_calls.bed`
**Explanation:** This command converts internal CNV result files into BED format, which can be loaded into genome browsers like UCSC or IGV for visual inspection alongside other genomic annotations.

### Processing multiple samples in batch mode

**Args:** `batch --manifest sample_manifest.tsv --ref ref.fa --output batch_results/ --threads 8`
**Explanation:** This command processes multiple samples listed in a tab-separated manifest file using 8 parallel threads, producing a unified results directory with per-sample CNV calls for cohort-level analysis.

### Generating a copy number profile plot for quality assessment

**Args:** `plot --input cnv_results.json --output copy_number_profile.png --genome-view --bin-width 500000`
**Explanation:** This command generates a genome-wide copy number profile visualization at 500kb resolution, enabling rapid quality assessment of the CNV calls and identification of large-scale alterations.