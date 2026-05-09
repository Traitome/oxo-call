---
name: bicseq2-seg
category: copy number analysis
description: Binary segmentation tool for detecting copy number alterations and loss of heterozygosity from targeted or whole-exome sequencing data using BICSeq2 normalization framework.
tags:
  - copy number variation
  - CNV detection
  - segmentation
  - cancer genomics
  - whole-exome sequencing
  - targeted sequencing
  - bioinformatics
author: AI-generated
source_url: https://github.com/chrisamiller/bicseq2
---

## Concepts

- BICSeq2-seg performs binary segmentation on normalized read counts to identify discrete copy number alteration regions, working on the assumption that copy number states follow a piecewise constant model across the genome.
- Input files must contain genomic coordinates (chromosome, start, end) paired with normalized read counts or ratio values; the tool accepts BED-like tabular formats where each row represents a target region with its associated depth-of-coverage signal.
- The segmentation algorithm uses a penalized likelihood approach with a biological prior that prefers fewer, larger alterations rather than many small breakpoints, controlled by a penalty parameter that balances sensitivity versus specificity.
- Output is written in segment-level format listing each called region with its chromosome, start position, end position, and estimated copy number state or log2 ratio value.
- The companion tool `bicseq2-seg-build` precomputes the reference distribution and normalizer files required by bicseq2-seg; running seg-build first is mandatory before executing segmentation.

## Pitfalls

- Running bicseq2-seg without first generating the appropriate normalizer file using bicseq2-seg-build results in errors or meaningless output, as the tool relies on pre-computed gc-bias and mappability corrections.
- Specifying an incorrectly formatted input file (e.g., providing raw counts without proper normalization or using a chromosome naming convention that differs from the reference) produces silently incorrect segment calls with no coordinate matches.
- Setting the penalty parameter too low (e.g., below 5) generates excessively fragmented output with hundreds of tiny segments that represent noise rather than true alterations, while setting it too high (e.g., above 50) merges real alterations into fewer but larger calls that may miss boundaries.
- Using a non-human reference genome without updating the chromosome naming scheme causes all genomic intervals to fail matching, as the tool expects chromosomes formatted as "chr1" or "1" depending on the configuration.
- Failing to provide paired tumor-normal inputs when analyzing cancer samples results in inability to distinguish somatic copy number changes from germline variations or normal copy number diploid regions.

## Examples

### Segment copy number alterations from normalized tumor read counts
**Args:** tumor.norm.bed subj1 norm out.txt 10 0.1
**Explanation:** This command segments normalized tumor read counts using a normalizer file to detect copy number alterations, with a penalty of 10 that balances detection of real alterations against fragmentation.

### Analyze copy number with stringent segmentation for high-confidence calls
**Args:** sample.norm.bed reference.norm out.txt 25 0.05
**Explanation:** Using a higher penalty value (25) produces fewer but more robust segment calls suitable for downstream validation or clinical reporting.

### Detect focal amplifications with relaxed segmentation parameters
**Args:** tumor.norm.bed reference.norm focal_calls.txt 5 0.15
**Explanation:** A low penalty (5) combined with higher fold-change threshold (0.15) increases sensitivity for focal amplifications but may introduce false positives requiring manual review.

### Generate segments for matched tumor-normal analysis
**Args:** tumor_input.bed normal_input.bed paired_output.txt 15 0.1
**Explanation:** Providing both tumor and normal BED files enables direct comparison to identify somatic copy number changes specific to the tumor sample.

### Process multiple samples in batch mode
**Args:** sample_list.txt norm_dir output_dir 12 0.1
**Explanation:** When a list file is provided instead of single inputs, the tool processes each sample sequentially, writing results to separate output files in the specified directory.

### Output segments in BED format for genome browser visualization
**Args:** normalized_tumor.bed hg19_normals out_segments.txt 10 0.1
**Explanation:** The textual output can be converted to standard BED format by extracting chromosome, start, end columns, enabling direct upload to genome browsers for visual inspection alongside other annotations.