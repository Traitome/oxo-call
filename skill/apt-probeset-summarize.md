---
name: apt-probeset-summarize
category: microarray expression analysis
description: Affymetrix Power Tools utility for summarizing probe set intensity values from multiple CEL files into gene-level expression estimates using various statistical methods (RMA, MAS5, PLIER).
tags:
  - affymetrix
  - microarray
  - expression
  - cel-files
  - summarization
  - apt
author: AI-generated
source_url: https://www.affymetrix.com/support/downloadsutility.affx
---

## Concepts

- The tool operates on Affymetrix CEL files as input, reading raw intensity values from each array and computing probe set-level expression summaries using statistical aggregation methods such as RMA (Robust Multi-chip Average), MAS5 (Microarray Suite 5), or PLIER (Probe Logarithmic Error Intensity Estimate).
- Output formats are configurable via `--output-file` and include text-based report files, CHP (chip) files for downstream tools, and optional normalized intensities using quantile or lowess normalization strategies specified through `--norm` or `--quant-norm`.
- The tool requires a library (CDF) file via `--celfile-path` or explicit CDF module name to map individual probes to probe sets, and a gene annotation file (`.csv` or `.annot.csv`) via `--annotation-file` can be provided to include gene symbols and descriptive names in output.
- Probe set filtering thresholds such as `--sab-percent-present` and `--sab-noise-sab` control which probe sets are included in final reports based on detection p-values and signal-to-noise ratios, reducing false positives in low-expressed genes.
- Multi-chip analysis is enabled by default when multiple CEL files are provided, where the algorithm jointly normalizes across all arrays before summarization; single-chip mode requires the `--one-chip` flag to process each array independently.

## Pitfalls

- Specifying an incorrect or mismatched CDF file version relative to the CEL file annotation will cause the tool to fail with probe mapping errors or silently produce meaningless expression values, because different chip types use distinct probe set definitions.
- Omitting the `--out-file` parameter when writing to standard output can make large expression matrices difficult to parse in downstream pipelines, especially when switching between tab-delimited and CSV output formats.
- Using MAS5 normalization (`--norm mas5`) with fewer than three replicates produces unreliable detection p-values and may exclude legitimate expressed genes, as the statistical model requires sufficient samples to estimate the background distribution.
- Forgetting to set the `--cel-files` argument with a proper file list (one CEL file per line) instead of space-separated paths will truncate the file list and process only the first CEL file, leading to incomplete analysis.
- Mixing chip types or array generations within a single analysis run will cause alignment failures, because probe set identifiers and probe counts differ between generations (e.g., GeneChip Human Genome U133 Plus 2.0 versus older U95 arrays).

## Examples

### Summarize multiple CEL files using RMA method with quantile normalization
**Args:** `--cel-files chip_list.txt --celfile-path /path/to/cdfs --norm quant-norm --out-file rma_results.txt rma`
**Explanation:** The RMA method applies quantile normalization across all CEL files listed, then applies RMA background correction and median polish summarization to produce robust expression estimates.

### Generate MAS5 expression values with detection p-values
**Args:** `--cel-files chip_list.txt --celfile-path /path/to/cdfs --norm mas5 --out-file mas5_results.txt mas5 --sab-percent-present 90 --sab-noise-sab 0.5`
**Explanation:** The MAS5 method estimates expression intensities withtau values and computes detection p-values, filtering probe sets that are present in fewer than 90 percent of the arrays.

### Export expression values with gene annotations included
**Args:** `--cel-files chip_list.txt --celfile-path /path/to/cdfs --annotation-file gene_annot.csv --out-file annotated_results.txt --export-function signal`
**Explanation:** The annotation file maps probe set identifiers to gene symbols, enabling the output report to display meaningful gene names alongside expression intensities.

### Perform single-chip summarization for one CEL file
**Args:** `--cel-files sample1.CEL --celfile-path /path/to/cdfs --one-chip --out-file single_chip.txt plier`
**Explanation:** The `--one-chip` flag disables multi-chip normalization, running PLIER summarization independently on a single CEL file to generate expression values without cross-array scaling.

### Write CHP output files for GeneChip Operating Software compatibility
**Args:** `--cel-files chip_list.txt --celfile-path /path/to/cdfs --write-chp --out-dir chp_output/ --cdf-module HuGene1_0-st`
**Explanation:** Writing CHP files creates binary output in the GeneChip format, allowing direct import into downstream tools such as Expression Console or Transcriptome Analysis Console for additional quality assessment.