---
name: cami-opal
category: bioinformatics/metagenomics-benchmarking
description: CAMI-OPAL is the reference library and CLI for scoring taxonomic profiling results against CAMI (Critical Assessment of Metagenome Interpretation) benchmarks. It compares predicted taxonomic profiles to ground-truth data and computes precision, recall, F1, and BCtc metrics.
tags:
  - metagenomics
  - benchmarking
  - taxonomic-profiling
  - cami
  - quality-metrics
author: AI-generated
source_url: https://github.com/CAMI-challenge/OPAL
---

## Concepts

- CAMI-OPAL expects taxonomic profiles in the CAMI Profiling Format (`.camifp`/`.tsv`), BIOM format (`.biom`), or OPAL TSV format, where each entry contains a taxon identifier (at genus/species level) and an assigned abundance or score value. Ground-truth files must use the same format to ensure valid comparison.
- The scoring engine computes per-taxon and per-sample metrics including precision, recall, F1-score, and the Bray-Curtis taxonomic distance (BCtc). The BCtc metric is computed across all taxa at a configurable lowest common ancestor (LCA) rank to penalize both false positives and false negatives proportionally.
- CAMI-OPAL supports rank-aware evaluation via the `--rank` flag (e.g., `genus`, `species`, `family`), meaning metrics are aggregated per taxonomic rank before being compared. Specifying the wrong rank can silently skew the aggregate scores because only taxa at the requested rank participate in the calculation.
- Output is produced in structured formats: a YAML metrics summary (`.yaml`), a per-taxon TSV detail table (`.tsv`), and optionally an HTML report (`.html`) with interactive plots. Downstream scripts consuming the YAML output should not assume fixed key ordering.
- The tool is structured as a library (`import opal`) with companion CLI entry points (`cami-opal-score`, `cami-opal-merge`), both sharing the same configuration schema. Configuration files (`.toml` or `.yaml`) centralize sample metadata, file paths, and scoring parameters to avoid repeated CLI flag duplication.

## Pitfalls

- Mixing profiling formats between the predicted profile and the ground truth file will cause silent row mismatches or a crash with `ValueError: Shape mismatch`. Always verify that both files use the same format (e.g., both CAMIFP or both BIOM) before invoking scoring.
- Omitting the ground truth file or specifying it as the predicted input produces scores that appear valid but reflect self-comparison (perfect recall, artificially high precision), yielding misleading metrics that look like 100% accuracy when the data was never compared against a reference.
- Using `--rank species` on profiles that contain many unassigned or genus-only entries results in those taxa being excluded from the per-rank evaluation, artificially inflating precision for the remaining taxa, and generate a warning about `N tax filtered at rank`. The `--rank auto` option mitigates this by selecting the most populated rank automatically.
- The `--lca-confidence` threshold filters low-confidence LCA assignments; setting it too high (e.g., `1.0`) may discard the majority of predictions, producing near-zero recall, while setting it too low (`0.0`) may include spurious taxa and inflate false positives. A validated threshold range is `0.0` to `0.5`.
- CAMI-OPAL caches intermediate results in a temporary directory; if the cache directory is on a disk with limited space (e.g., `/tmp`), large-profile comparisons can fail with `IOError: No space left on device`. Always set the `--cache-dir` to a location with sufficient free space.

## Examples

### Score a taxonomic profile against ground truth at genus rank
**Args:** `score --sample S1 --truth ground_truth.camifp --prediction predicted.camifp --rank genus --out metrics`
**Explanation:** Compares the predicted taxonomic profile to the ground truth file, restricting evaluation to genus-rank taxa, and writes the per-rank precision/recall/F1 metrics to `metrics.yaml`.

### Score with BIOM-format input and auto rank detection
**Args:** `score --sample S2 --truth camifp/truth.tsv --prediction biom/s2.biom --rank auto --out results`
**Explanation:** Uses the auto rank feature to select the most representative taxonomic rank in the BIOM input file, avoiding manual rank specification while maintaining comparable metrics across heterogeneous input formats.

### Merge multiple sample profiles into a single CAMIFP file
**Args:** `merge --output merged.camifp --input sample1.camifp sample2.camifp sample3.camifp`
**Explanation:** Consolidates per-sample CAMIFP files into one multi-sample profiling file for batch scoring, ensuring all input files conform to the same CAMIFP schema before concatenation.

### Score with custom LCA confidence threshold and verbose output
**Args:** `score --sample P4 --truth truth.camifp --prediction pred.camifp --rank species --lca-confidence 0.3 --verbose --out report`
**Explanation:** Applies an LCA confidence filter of 0.3 to discard low-confidence species-level assignments, while enabling verbose output to surface filtered taxon counts and individual metric breakdowns in the report.

### Generate an HTML report with Bray-Curtis taxonomic distance
**Args:** `score --sample E1 --truth truth.camifp --prediction pred.camifp --rank family --bctc --html --out html_results`
**Explanation:** Computes the Bray-Curtis taxonomic distance across family-rank taxa and generates an interactive HTML report at `html_results/report.html` for visual exploration of precision/recall and BCtc distributions per taxon.

### Use a configuration file for batch scoring across multiple samples
**Args:** `score --config batch_config.toml --out batch_metrics`
**Explanation:** References a TOML configuration file that defines all sample ground-truth and predicted file paths and rank parameters, enabling reproducible batch scoring across many samples without repeating individual CLI flags.