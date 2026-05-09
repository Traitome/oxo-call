I need to clarify that I don't have specific knowledge about a tool called "cnvetti." There's a well-known bioinformatics tool called **CNVkit** (Copy Number Variation Kit) used for copy number variation analysis from targeted sequencing data. 

Given the similarity in naming ("cnvetti" vs "cnvkit"), I'll assume this is the intended tool and generate the skill file based on CNVkit's actual capabilities.

---
name: cnvkit
category: variant-calling
description: A tool for detecting copy number variations (CNVs) from targeted capture sequencing data (e.g., hybrid capture panels). It processes BAM/CRAM files to generate copy number profiles, segments the genome into copy number states, and identifies focal CNVs.
tags:
  - copy-number-variation
  - cnv
  - sequencing
  - coverage
  - segmentation
author: AI-generated
source_url: https://cnvkit.readthedocs.io/
---

## Concepts

- **Target Regions and BED Files**: CNVkit requires a BED file defining the targeted genomic regions (capture probes). This is used to calculate coverage and expected copy number. Without proper target definitions, coverage calculations will be incorrect.
- **Reference Pool Creation**: The tool builds a reference by combining multiple "normal" samples to create a pooled reference profile. This reference is critical for baseline copy number estimation—using poor-quality or uneven coverage samples results in noisy copy number calls.
- **Binary .cnn Files**: CNVkit stores coverage and copy number data in binary `.cnn` files. These are the primary data model containing gene-level and bin-level coverage, GC content corrections, and copy number estimates. All subsequent commands consume or produce these files.
- **Antisense and Sense Strand Handling**: For targeted panels, CNVkit separately handles reads mapping to sense and antisense strands to reduce GC bias artifacts. This is crucial for hybrid capture data where strand bias can distort coverage estimates.

## Pitfalls

- **Using Unsorted or Misaligned BAM Files**: CNVkit requires queryname-sorted BAM files for `coverage` and `batch` commands. Using position-sorted BAMs will cause the tool to fail silently or produce garbage coverage values.
- **Insufficient or Inappropriate Reference Samples**: Creating a reference from fewer than 10 samples or from samples with different library preparations leads to poor copy number normalization. The resulting `.cnn` files will have systematic biases.
- **Forgetting to Run 'segment' After 'call'**: The `call` command produces discrete copy numberinteger values, but segmentation must be done separately. Skipping segmentation removes necessary smoothing and reduces detection accuracy for small CNVs.
- **Incorrect Mitochondrial or Low-Complexity Region Handling**: CNVkit does not automatically mask mitochondrial DNA or low-complexity regions. These areas often show extreme copy number artifacts and must be excluded via the `--omit-merged` flag or manually in the target BED file.
- **Ignoring Tumor/Population Frequency Assumptions**: The `--tumor` flag assumes a pure tumor sample; for mixed populations (e.g., cell-free DNA), copy number thresholds need adjustment. Misapplying this causes either under-calling or over-calling of events.

## Examples

### Generate coverage file from a BAM file and targets
**Args:** coverage Sample1.bam targets.bed -o Sample1.targetCoverage.cnn
**Explanation:** Computes read depth at each target region, applies GC bias correction, and writes a `.cnn` file for downstream analysis.

### Calculate anti-target (off-target) coverage
**Args:** coverage Sample1.bam antitargets.bed -o Sample1.antitargetCoverage.cnn --antitarget
**Explanation:** Calculates coverage in genomic bins between capture targets to improve backbone segmentation for low-coverage regions.

### Create a pooled reference from multiple normal samples
**Args:** reference Sample1.cnn Sample2.cnn Sample3.cnn Sample4.cnn -o PanelReference.cnn
**Explanation:** Generates a combined reference profile by averaging coverage across multiple normal samples, forming the baseline for copy number comparison.

### Batch process multiple tumor samples against a reference
**Args:** batch SampleTumor1.bam --reference PanelReference.cnn --targets targets.bed --output-dir batch_results
**Explanation:** Simultaneously computes coverage, applies reference correction, performs segmentation, and generates copy number calls for multiple tumor BAMs in a single run.

### Call copy number segments with specified thresholds
**Args:** call BatchResults.cnr -o Sample1_copyNumber.call --ploidy 2 --purity 0.8 -m threshold
**Explanation:** Converts continuous copy number values to discrete integer calls assuming diploid ploidy and 80% tumor purity, using default threshold-based calling.