---
name: clinod
category: Bioinformatics / Clonality Analysis
description: A tool for analyzing clonal populations and immune repertoire diversity from V(D)J sequencing data. Computes clonal rankings, diversity indices (Shannon, Simpson, Chao1, Gini-Simpson), and generates ranked clonal profiles for distinguishing oligoclonal vs polyclonal immune responses.
tags:
  - clonality
  - immune-repertoire
  - vdj
  - diversity-index
  - cancer-immunology
  - bioinformatics
  - receptor-sequencing
author: AI-generated
source_url: https://github.com/f不下来/clinod
---

## Concepts

- `clinod rank` orders clonotypes by frequency or abundance and outputs a ranked clonal profile (RCP) table where each row represents a clone and columns include clone_id, frequency, proportion, and cumulative proportion. This ranked output is the primary input for all diversity calculations.
- Diversity metrics are computed from the ranked frequency distribution: Shannon entropy (H) is calculated from -sum(p_i * ln(p_i)), Simpson index (D) from 1 - sum(p_i^2), Gini-Simpson from 1 - D, and Chao1 from S + (n_1^2 / 2*n_2) where n_1 and n_2 are singletons and doubletons. These require the `--metric` flag to select which index to report.
- Input files must be in AIRR-TSV or 10x VD(J) CSV format; `clinod` auto-detects the format from column headers. Output is written to stdout in TSV format, making it pipeable to downstream tools. The `--out` flag redirects output to a named file.
- The `--groups` flag accepts a comma-separated list of column names from the input file header to stratify analysis; `clinod` computes per-group diversity statistics and outputs one row per group plus an aggregated row.
- Clonal threshold filtering via `--min-freq` removes low-abundance clonotypes (below the specified count) before ranking and metric computation, which is critical for reducing noise in diversity estimates from sequencing artifacts.

## Pitfalls

- Using `--min-freq 1` on datasets with high sequencing depth retains singletons (clones with exactly 1 read count), which inflates Chao1 estimates because the Chao1 formula relies on n_2 (doubletons) to correct for unobserved species. This can produce unreliable diversity estimates for shallow libraries.
- Mixing AIRR-TSV and 10x CSV inputs without specifying `--format` causes silent mis-parsing when column names differ; for example, the 10x column `frequency` means proportion (0–1 scale) while AIRR uses `clone_id` and `duplicate_count`. Diversity metrics will be wildly incorrect.
- Specifying `--groups` with a column name that does not exist in the input file header causes `clinod` to abort with a non-descriptive "column not found" error, and no output is produced. Always verify column names with `clinod validate` before running.
- Forgetting to sort output by rank (`--sort rank`) when generating RCPs means rows are in input file order rather than by clone size, which breaks the cumulative proportion calculation and makes the profile unusable for downstream diversity analysis.
- Using `--metric shannon --normalize` with zero-sum proportions (all clones equally abundant) produces NaN because ln(0) is undefined in Shannon entropy, and the tool does not warn about this edge case.

## Examples

### Compute Shannon diversity index from a ranked clonotype file
**Args:** `diversity --input clones.tsv --metric shannon`
**Explanation:** Reads the TSV file, orders clonotypes by frequency, calculates Shannon entropy H from the frequency distribution, and outputs a single TSV row with the H value. This is the most common single-sample diversity metric.

### Calculate Chao1 richness with clonal threshold filtering
**Args:** `diversity --input sample1.tsv --metric chao1 --min-freq 2 --out chao1_sample1.tsv`
**Explanation:** Filters out singletons before computing Chao1 richness, using doubletons to correct for unobserved clones. This produces more stable estimates for libraries with moderate sequencing depth.

### Generate a ranked clonal profile sorted by clone size
**Args:** `rank --input blood_sample.tsv --sort rank --out rcp_blood.tsv`
**Explanation:** Reads clonotypes, sorts rows descending by duplicate count, adds rank and cumulative_proportion columns, and writes the RCP table. This file is the required input for plotting rank-abundance curves.

### Compute per-group diversity for multiple samples
**Args:** `diversity --input pbmc_4samples.tsv --metric gini-simpson --groups sample_id --out group_div.tsv`
**Explanation:** Groups rows by the sample_id column, computes Gini-Simpson index independently for each group, and appends an aggregated row. Produces one diversity value per sample for comparative analysis.

### Validate input file format and column names before analysis
**Args:** `validate --input clones.tsv --format airr`
**Explanation:** Checks that all required columns (clone_id, duplicate_count, v_call, j_call) are present and that duplicate_count values are non-negative integers. Reports column names and row count without computing any metrics.