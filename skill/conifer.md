---
name: conifer
category: RT-qPCR Data Analysis
description: Conifer (Conditional Inference of RT-qPCR) is a tool for analyzing and normalizing reverse transcription quantitative PCR data. It performs quality control, conditional inference-based normalization, and statistical hypothesis testing on Ct values from RT-qPCR experiments.
tags: [rt-qpcr, normalization, differential-expression, statistics, gene-expression, quantitative-pcr]
author: AI-generated
source_Url: https://github.com/hyeshik/conifer
---

## Concepts

- Conifer processes **Ct (cycle threshold) values** from RT-qPCR experiments, requiring at minimum an input file with sample identifiers, target genes, and corresponding Ct measurements. Output includes normalized expression values (relative to reference genes) and statistical inference results.
- The tool supports **multiple reference genes** for normalization using conditional inference methodology, which adaptively weights genes based on their stability across samples rather than applying fixed geometric mean assumptions.
- Conifer accepts **standard instrument export formats** including CSV, tab-delimited TXT, and supports direct import from Bio-Rad CFX Maestro, Roche LightCycler, and Applied Biosystems StepOne output files via format auto-detection.
- Analysis workflows in Conifer follow a **progressive pipeline**: import → quality filtering → normalization → statistical comparison → export, where each stage can be run independently or chained in a single command.
- Statistical output includes **differential expression tests** with multiple testing correction (Benjamini-Hochberg by default), confidence intervals for fold-changes, and diagnostic plots for assessing model assumptions.

## Pitfalls

- Using **unstable reference genes** causes systematic bias in normalized values. Conifer will run without explicit stability assessment, but normalized results become unreliable when housekeeping genes vary more across conditions than targets. Always verify reference gene stability scores (M-value) before interpreting results.
- **Missing Ct values** are handled by listwise deletion by default, which can severely reduce power in unbalanced designs. If missingness exceeds 10% of any gene, consider using the `--impute` flag or investigating why specific wells failed rather than forcing analysis with gaps.
- **Confusing technical and biological replicates** leads to underestimation of variance and inflated false positives. Conifer treats each input row as an independent observation; technical replicates must be averaged before analysis unless explicitly annotated with the `--tech-rep` grouping column.
- Specifying **incorrect sample groupings** in the `--groups` column produces meaningless statistical comparisons. The tool does not validate that group labels correspond to experimental design, so typos or mislabeled samples silently propagate through to results.
- **Insufficient replication** (fewer than 3 biological replicates per group) yields unreliable variance estimates in conditional inference. While Conifer will complete analysis with n=2 per group, confidence intervals become wide and fold-change estimates unreliable.

## Examples

### Analyze RT-qPCR data with default settings
**Args:** `analyze --input samples.csv --output results`
**Explanation:** Runs the complete analysis pipeline on the input file, applying default quality thresholds and normalization. Output directory will contain normalized values, statistics, and diagnostic plots.

### Specify stable reference genes for normalization
**Args:** `analyze --input samples.csv --refs GAPDH ACTB --output results`
**Explanation:** Explicitly designates GAPDH and ACTB as reference genes. Conifer calculates stability weights and uses their conditional average for normalization, overriding the default automatic reference selection.

### Filter samples by Ct threshold before analysis
**Args:** `analyze --input samples.csv --max-ct 35 --min-reps 2 --output results`
**Explanation:** Excludes any measurement with Ct values above 35 (indicating low expression) and removes targets with fewer than 2 valid replicates across all samples. This removes low-quality measurements that would otherwise distort normalization.

### Compare gene expression between two experimental groups
**Args:** `contrast --input results/normalized.csv --groups condition --control vehicle --case treatment --output contrast_results`
**Explanation:** Performs differential expression analysis comparing treatment group against vehicle control, outputting fold-changes, p-values, and confidence intervals for each target gene.

### Export results with multiple testing correction
**Args:** `export --input results/stats.csv --method fdr --alpha 0.05 --format xlsx --output final_results`
**Explanation:** Generates a spreadsheet with Benjamini-Hochberg FDR correction applied and significant hits (adjusted p