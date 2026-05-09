---
name: bctools
category: Bioinformatics / Sequence Analysis
description: A command-line toolkit for analyzing binary count data, such as those generated from shRNA screens, CRISPR guides, or amplicon sequencing experiments. Provides tools for counting, filtering, normal
izing, and analyzing guide-level or target-level count data.
tags:
  - bioinformatics
  - count-data
  - crispr
  - shrna
  - screening
  - functional-genomics
  - guide-rna-analysis
author: AI-generated
source_url: https://github.com/bctools-project/bctools
---

## Concepts

- **Input Format**: bctools accepts raw count matrices in tab-separated (TSV) or comma-separated (CSV) format, where rows represent guides/targets and columns represent samples. A required header row identifies sample names.
- **Output Format**: Processed results are emitted as TSV files with consistent column naming: `target_id`, `sample_name`, `count`, `normalized_count`, `log2fold`, and optional statistical (`pvalue`, `adjpvalue`) columns.
- **Guide Filtering**: Implements a minimum count threshold (default: 3 reads) and minimum presence across replicates (default: 2 samples) to remove low-abundance or inconsistent guide observations before downstream analysis.
- **Normalization Methods**: Supports four normalization strategies: total counts (DESeq2-style), upper quartile, median-of-ratios, and TPM-like scaling for cross-sample comparability.
- **Statistical Analysis**: Includes built-in differential expression testing using a negative binomial model (similar to edgeR) for comparing guide abundances between conditions.

## Pitfalls

- **Misaligned sample columns**: Counting reads from the wrong column in multi-sample input files leads to completely incorrect results, as the tool processes columns in header order without validation.
- **Ignoring replicate requirements**: Not specifying replicate identifiers (`-r` flag) when multiple biological replicates exist causes pooling of all samples, inflating false positives in differential analysis.
- **Applying wrong normalization**: Using TPM normalization for CRISPR pooled screening depletion data (which expects total count normalization) can invert biological interpretations since the library complexity differs.
- **Inconsistent guide naming**: Mismatch between reference library file and count input (e.g., using Entrez IDs in one file and sgRNA sequences in another) produces zero counts that appear as true biological depletion rather than annotation error.

## Examples

### Basic count summary for a single sample
**Args:** `-i sample_counts.tsv -s Sample1 --summary`
**Explanation:** Generates basic statistics (total reads, unique guides detected, mean count per guide) for the named sample column without additional processing.

### Filter low-abundance guides across multiple samples
**Args:** `-i guide_counts.tsv --min-count 5 --min-replicates 3 -o filtered_counts.tsv`
**Explanation:** Removes any guide with fewer than 5 reads in fewer than 3 samples, reducing noise in downstream differential expression analysis.

### Normalize using upper quartile method
**Args:** `-i raw_counts.tsv --norm upper quartile -o normalizedCounts.tsv`
**Explanation:** Scales each sample's counts by the upper quartile (75th percentile) to account for library size differences across sequencing runs.

### Differential analysis between two conditions
**Args:** `-i treated_vs_control.tsv -c Control -t Treated --replicates 3 --method nb -o diff_results.tsv`
**Explanation:** Performs negative binomial-based differential expression testing with 3 replicates per condition, outputting log2 fold changes and p-values.

### Combine multiple count files into single matrix
**Args:** `-i rep1.tsv rep2.tsv rep3.tsv --combine -o combined_matrix.tsv`
**Explanation:** Merges separate replicate count files into a single TSV matrix with consistent guide ordering, required before replicate-aware analysis.

### Export statistical results with multiple testing correction
**Args:** `-i diff_raw.tsv --method exact-test --fdr --alpha 0.05 -o sig_hits.tsv`
**Explanation:** Applies Benjamini-Hochberg FDR correction to p-values and filters output to only include significant hits below the 0.05 threshold.

### Generate depletion ranking plot data
**Args:** `-i depleting_guides.tsv --rank log2 --top 50 -o top50_depleted.tsv`
**Explanation:** Ranks guides by log2 fold change and outputs the top 50 most depleted targets, suitable for visualization or gene set enrichment.

### Cross-reference with library annotation file
**Args:** `-i counts.tsv --annot library_annotations.tsv --gene-col TargetGene -o annotated_counts.tsv`
**Explanation:** Joins count data with gene annotations using the designated gene identifier column, enabling gene-level aggregation afterward.