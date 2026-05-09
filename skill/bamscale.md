---
name: bamscale
category: Variant Analysis / Copy Number
description: Generates a scaling BAF (B-allele frequency) signal file from a BAM file for read-backed copy number variation analysis. Used to calculate per-window allele frequencies that help distinguish between homo- and heterozygous copy number states.
tags: [bam, copy-number-variation, cn-variants, b-allele-frequency, BAF, read-backed-haplotype, samtools]
author: AI-generated
source_url: https://github.com/samtools/samtools
---

## Concepts

- **BAF Signal Model**: bamscale calculates the B-allele frequency (BAF) for each genomic window by examining heterozygous SNP positions in reads. The output file contains two columns: genomic position and BAF signal value. Normal diploid heterozygosity produces a BAF of ~0.5, while copy number loss or gain shifts this value toward 0 or 1 respectively.

- **Window-based Processing**: The tool divides the reference genome into non-overlapping windows of configurable size (default 100 kb). Each window receives a single BAF score computed from all qualifying reads overlapping that region. Smaller windows increase resolution but reduce statistical confidence per window.

- **Quality-based Read Filtering**: Reads are filtered based on MAPQ (mapping quality) and base quality thresholds. Only reads with base quality ≥ threshold (default 10 at each examined base) and mapping quality ≥ threshold (default 10) contribute to BAF calculation. This prevents noisy or misaligned reads from corrupting the signal.

- **Input/Output Format**: Input must be a coordinate-sorted BAM file (indexed). Output is a text file with three columns: chromosome, position, and BAF value. This format integrates with downstream tools like CNV callers that consume B-allele frequency signals.

## Pitfalls

- **Unsorted or Unindexed BAM Files**: Feeding an unsorted BAM file produces nonsensical BAF values because positional calculations assume ordered reads; similarly, using an unindexed BAM causes the tool to fail on large files. Always ensure the BAM is coordinate-sorted and indexed.

- **Insufficient Read Depth in Windows**: Windows with few covering reads produce noisy BAF estimates that can mislead downstream CNV analysis. Using overly small windows combined with low-coverage data amplifies this effect, generating spurious copy number calls.

- **Confusing Heterozygous with Homozygous SNPs**: The tool cannot distinguish whether a position is truly heterozygous or homozygous; if the reference allele is fixed in the sample, BAF approaches 0 or 1 even though no copy number change exists. Downstream analysis must account for baseline allele frequencies.

- **Ignoring the -s/--smooth Option**: Unsmoothed BAF output contains high-frequency noise that complicates CNV segmentation. Enabling smoothing produces cleaner signals but reduces sharp boundaries between adjacent CNV regions.

## Examples

### Generate BAF signal with default window size
**Args:** input.bam
**Explanation:** Processes the BAM file using default 100 kb windows and quality thresholds (MAPQ ≥ 10, base quality ≥ 10), outputting BAF values per genomic window.

### Generate BAF signal with larger windows for high-coverage data
**Args:** -w 200000 input.bam
**Explanation:** Uses 200 kb non-overlapping windows, increasing statistical robustness when coverage is high and reducing noise at the cost of lower positional resolution.

### Generate BAF signal with increased base quality filter
**Args:** -t 20 input.bam
**Explanation:** Raises the minimum base quality threshold to 20, filtering out lower-quality bases to reduce noise in BAF calculation at the cost of fewer qualifying reads.

### Generate BAF signal with smoothing enabled
**Args:** -s input.bam
**Explanation:** Applies smoothing to the BAF signal output, reducing high-frequency noise and producing cleaner signals for downstream CNV segmentation algorithms.

### Generate BAF signal with combined custom parameters for high-coverage WGS
**Args:** -w 50000 -t 20 -s input.bam
**Explanation:** Uses 50 kb windows with base quality threshold 20 and smoothing enabled, balancing fine resolution with noise reduction appropriate for high-coverage whole-genome sequencing data.