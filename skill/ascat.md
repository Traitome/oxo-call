---
name: ascat
category: variant_calling
description: Allele-Specific Copy Number Analysis Tool for detecting copy number variations and calculating allele-specific copy numbers in tumor samples using B-allele frequency and log2 ratio data
tags: [copy_number_variation, cnvcall, cancer_genomics, allele_specific, snp_array, sequencing]
author: AI-generated
source_url: https://github.com/cancerit/ASCAT
---

## Concepts

- **Data Input Model**: ASCAT requires paired tumor/normal BAM files or pre-processed data (LogR and B-allele frequency values). It supports multiple reference genomes (hg18, hg19, hg38, mm9) and works with both sequencing data and SNP array outputs.
- **Allele-Specific Copy Number Calculation**: The algorithm simultaneously estimates tumor purity and ploidy by segmenting the genome and fitting copy number profiles. It calculates the raw copy number (LogR) and allelic imbalance (BAF) to determine the best-fit allele-specific copy number solution.
- **Output Formats**: ASCAT produces multiple output files including: `.copynumber.cvs` (segment-level copy numbers), `.subclones.csv` (subclonal architecture), `.BAFs.txt` (B-allele frequencies), and `.LogR.txt` (log2 ratios). The raw data files enable downstream visualization and interpretation.
- **Segmentation Methods**: ASCAT supports different segmentation algorithms including CBS (circular binary segmentation), HaarSeg (wavelet-based), and can process pre-segmented data. The choice affects sensitivity to break points and runtime.
- **Companion Binary ascat-build**: The ascat-build tool pre-processes aligned read counts at SNP positions from BAM files, generating the input matrices required for ASCAT's core algorithm. This step is required when starting from raw sequencing data.

## Pitfalls

- **Incorrect Ploidy Constraints**: Setting maxCN too low (e.g., 5) when analyzing high-grade tumors with copy number amplifications can cause ASCAT to reject valid solutions. The algorithm may then fall back to suboptimal purity/ploidy combinations, yielding inaccurate copy number estimates.
- **Mismatched Reference Genomes**: Using the wrong reference genome (e.g., hg19 when the BAM files are aligned to hg38) results in coordinate mismatches between SNP positions and read alignments, causing zero or near-zero BAF values and failed copy number calling.
- **Over-Segmentation from Aggressive Segmentation**: Applying HaarSeg with default parameters on noisy data produces excessive short segments. This creates fragmented copy number profiles that obscure driver events. Always review segment lengths and merge adjacent segments with similar values.
- **Gender Mismatch**: Specifying the wrong gender (male vs. female) leads to incorrect expected BAF values on the X chromosome. This causes systematic over- or under-copying of chrX, particularly problematic in female tumors with X-specific events.
- **Insufficient Memory for Large Genomes**: Processing whole-genome sequencing data without specifying chunked processing can cause memory overflow. ASCAT loads entire chromosomes into memory; consider running chromosome-by-chromosome for WGS datasets larger than 30x depth.

## Examples

### Calculate allele-specific copy numbers from paired tumor/normal BAM files using default settings
**Args:** --tumourFile tumor.bam --normalFile normal.bam --refGenome hg19 --sampleID sample1 --gender female --outputDir ./ascat_output
**Explanation:** This runs ASCAT on pre-aligned BAM files with default parameters for a female sample, outputting all copy number tables and plots to the specified directory.

### Run ASCAT with custom purity and ploidy penalties to constrain the solution space
**Args:** --tumourFile tumor.bam --normalFile normal.bam --refGenome hg19 --sampleID sample1 --gender male --outputDir ./ascat_output --rhoPenalty 0.1 --psiPenalty 0.2
**Explanation:** Lowering the penalties allows ASCAT to explore more extreme purity and ploidy combinations, useful when the sample has very high purity (>80%) or known near-diploid ploidy.

### Process pre-computed LogR and BAF data without re-running segmentation
**Args:** --ascatNgs --sampleFile sample1.txt --outputDir ./ascat_output
**Explanation:** This skips the segmentation step and uses pre-computed LogR and BAF values, significantly reducing runtime when iterating on downstream parameters like rho/psi grids.

### Use CBS segmentation with adjusted thresholds for high-resolution break point detection
**Args:** --tumourFile tumor.bam --normalFile normal.bam --refGenome hg19 --sampleID sample1 --gender female --outputDir ./ascat_output --segmentationMethod cbs --alpha 0.01
**Explanation:** Setting alpha to 0.01 increases statistical significance requirements for segmentation, producing fewer but more robust break points, ideal for identifying focal amplifications.

### Build input matrices from BAM files using companion tool ascat-build
**Args:** -b tumor.bam -n normal.bam -o output_file.txt -g hg19 -r 3
**Explanation:** The ascat-build companion generates SNP-level read counts required for ASCAT, using a matching reference genome (hg19) and specifying ploidy assumption of 3.

### Run ascat-build with specific SNP list for targeted panel analysis
**Args:** -b tumor.bam -n normal.bam -o output_file.txt -g hg19 -s snp_positions.bed -r 2
**Explanation:** Using a custom BED file of SNP positions limits analysis to targeted regions, reducing runtime and file sizes for panel or exome sequencing experiments.

### Process sample with known high copy number state requiring elevated maxCN
**Args:** --tumourFile tumor.bam --normalFile normal.bam --refGenome hg19 --sampleID sample1 --gender female --outputDir ./ascat_output --maxCN 8
**Explanation:** Setting maxCN to 8 accommodates samples with high-level amplifications common in triple-negative breast cancers or esophageal carcinomas, preventing solution rejection.

### Generate output with explicit log2 ratio bounds matching array CGH data
**Args:** --tumourFile tumor.bam --normalFile normal.bam --refGenome hg19 --sampleID sample1 --gender male --outputDir ./ascat_output --logRbot -2.5 --logRtop 2.5 --minBAF 0.05
**Explanation:** Constraining logR bounds matches the dynamic range of SNP array data, ensuring consistency when pooling array and sequencing data in multi-platform studies.