---
name: breakdancer
category: structural variant detection
description: BreakDancer detects genomic structural variations (deletions, insertions, inversions, translocations) from paired-end sequencing data by analyzing abnormal read pair alignments. It identifies SVs based on anomalies in insert size, read orientation, and chromosomal positioning of mapped read pairs.
tags: sv-detection, structural-variants, paired-end, genomics, variation, bioinformatics
author: AI-generated
source_url: https://github.com/genome/breakdancer
---

## Concepts

- **Paired-end Read Analysis**: BreakDancer analyzes BAM/SAM files containing paired-end reads and detects structural variants by identifying read pairs with anomalous insert sizes, improper orientation, or unexpected chromosomal pairing. The tool requires aligned reads with complete mate information from aligners like BWA or Bowtie.

- **SV Type Detection**: BreakDancer identifies five major structural variant types: DEL (deletions), INS (insertions), INV (inversions), DUP (duplications), and CTX (inter-chromosomal translocations). Each type produces a distinct signature in the read pair alignment patterns based on read orientation and fragment length.

- **Statistical Scoring**: The tool assigns a quality score to each SV call based on the number of supporting read pairs and the degree of insert size deviation from the library mean. Higher scores indicate greater confidence, and the default threshold requires at least 3 supporting read pairs for calling a variant.

- **Index Building**: The breakdancer-build utility creates a binary index from the input BAM file, which breakdancer-max then uses for efficient variant detection. This pre-processing step significantly accelerates the main detection algorithm and reduces memory usage during analysis.

- **Output Format**: Results are written in a tab-separated text format with columns detailing chromosome coordinates, SV type, size estimate, confidence score, and supporting evidence including read pair counts and breakpoint positions.

## Pitfalls

- **Ignoring Library Insert Size Distribution**: Using default parameters without accounting for the actual library insert size distribution (mean and standard deviation) leads to missed true positives or excessive false calls. Always calculate library statistics from properly mapped read pairs before running BreakDancer and adjust the `-d` parameter accordingly.

- **Using Unsorted or Unindexed BAM Files**: BreakDancer requires BAM files sorted by coordinate and indexed with BAM indices (.bai files). Using unsorted files or missing indices causes runtime errors or produces incorrect results because the tool relies on efficient binary search for mate retrieval.

- **Accepting All Default Thresholds**: The default minimum of 3 supporting read pairs may be insufficient for high-confidence calls in low-coverage regions or too stringent for high-coverage repetitive areas. Adjusting the `-c` and `-g` parameters based on sequencing depth improves detection accuracy.

- **Misinterpreting Small Variants**: BreakDancer is optimized for detecting structural variants typically larger than the library insert size (30 bp to several kb). Variants below this threshold, especially small insertions and deletions, are often missed or poorly sized because the anomalous read pairs fall within normal insert size variance.

- **Not Filtering Results**: Raw BreakDancer output contains many false positives, particularly in repetitive regions and low-complexity areas. Applying post-processing with breakdancer-filter or manual filtering based on score, read depth, and repeat masking is essential for producing reliable call sets.

## Examples

### Detect structural variants from an aligned BAM file

**Args:** `-o output.txt -d 500 sample.bam`
**Explanation:** This runs breakdancer-max to detect structural variants using an estimated library insert size of 500 bp, outputting results to the specified text file.

### Build an index from a large BAM file for faster processing

**Args:** `sample.bam`
**Explanation:** This runs breakdancer-build to create a binary index from the input BAM file, which accelerates subsequent breakdancer-max runs and reduces memory consumption.

### Adjust sensitivity for low-coverage sequencing data

**Args:** `-o output.txt -c 2 -g 5 sample.bam`
**Explanation:** This lowers the minimum supporting read pair threshold to 2 and increases the minimum size filter to 5 bp, making the detector more sensitive for low-coverage datasets where fewer supporting reads are available.

### Generate high-confidence SV calls by increasing the score threshold

**Args:** `-o output.txt -s 30 sample.bam`
**Explanation:** This filters results to only include SV calls with a quality score of 30 or higher, reducing false positives while maintaining reliable variant detection in regions with sufficient coverage.

### Process multiple BAM files in a single run

**Args:** `-o combined_output.txt sample1.bam sample2.bam sample3.bam`
**Explanation:** This runs breakdancer-max on multiple BAM files simultaneously, combining the read pair evidence across all samples to improve SV detection sensitivity and generate a unified output file.

### Filter SV calls to retain only high-quality predictions

**Args:** `-f 30 sample.sv > filtered_sample.sv`
**Explanation:** This runs breakdancer-filter to remove SV calls with scores below 30, keeping only high-confidence structural variant predictions for downstream analysis.