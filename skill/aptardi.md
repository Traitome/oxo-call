---
name: aptardi
category: Aptamer Analysis / Sequence Design
description: A command-line tool for analyzing aptamer selection experiments, computing binding enrichment scores, and generating candidate sequences for directed evolution workflows.
tags:
  - aptamer
  - selex
  - sequence-analysis
  - binding-enrichment
  - directed-evolution
  - bioinformatics
  - nucleotide-sequences
author: AI-generated
source_url: https://github.com/ aptardi/aptardi
---

## Concepts

- **Enrichment Scoring Model**: aptardi calculates enrichment scores by comparing read counts between selection rounds using log2 fold-change ratios. Positive scores indicate stronger binding candidates; the default scoring uses a pseudocount of 1 to avoid log(0) undefined errors.
- **Input Format**: The tool accepts multi-FASTA files where each sequence entry must contain a unique identifier line (prefixed with `>`) followed by nucleotide bases (A, T, G, C, U). Mixed DNA/RNA notation is allowed but all U residues are normalized to T internally.
- **Round Comparison**: The `-r1` and `-r2` flags specify which selection rounds to compare. aptardi requires that round files contain the same number of sequence entries in the same order when using direct comparison mode (`--mode direct`). Mismatched entries produce exit code 2.
- **Output Columns**: The default tabular output contains five columns: `ID`, `sequence`, `count_r1`, `count_r2`, `enrichment_score`. The `--json` flag reformats output into JSON lines (one JSON object per line) with identical data fields plus a `pvalue` field from a binomial test.
- **Background Correction**: When a control input is provided via `--control`, aptardi subtracts background enrichment using a z-score normalization across all candidates before computing final scores.

## Pitfalls

- **Mismatched Sequence Sets**: Providing round files with different sequence sets in direct comparison mode causes aptardi to fail with exit code 2 and produce no output. Always ensure both input files share identical entry order and count before running with `--mode direct`.
- **Insufficient Read Depth**: Sequences with fewer than 10 total reads across both rounds are excluded from enrichment calculation by default. This threshold is controlled by `--min-count` and silently drops low-coverage candidates from the output, potentially causing you to miss genuine binders if your sequencing depth is low.
- **Base Composition Normalization**: Not using `--normalize-bases` when input contains inosine (I) or other ambiguous bases causes aptardi to treat these as 'N' instead of the correct degenerate-base interpretation, leading to incorrect enrichment scores for modified aptamer libraries.
- **Floating-Point Score Interpretation**: Enrichment scores are stored as double-precision floats and printed with 6 decimal places. Rounding these scores for display without accounting for floating-point precision can produce apparent inconsistencies when comparing outputs across runs.

## Examples

### Compute enrichment scores between two selection rounds
**Args:** `round1_counts.fa round2_counts.fa -o results.tsv`
**Explanation:** Compares read counts from `round1_counts.fa` and `round2_counts.fa`, outputting a tab-delimited file with sequences, counts, and enrichment scores.

### Output results as JSON lines
**Args:** `round1_counts.fa round2_counts.fa --json -o results.jsonl`
**Explanation:** Produces JSON lines format with enrichment scores and p-values, which is easier to parse with tools like jq or Python pandas.

### Adjust minimum read count threshold
**Args:** `round1_counts.fa round2_counts.fa --min-count 50 -o results.tsv`
**Explanation:** Raises the inclusion threshold to 50 total reads, filtering out low-coverage candidates that may produce statistically unreliable enrichment values.

### Use iterative refinement mode
**Args:** `round1_counts.fa round2_counts.fa --mode iterative --iterations 3 -o refined.tsv`
**Explanation:** Runs three rounds of iterative refinement, reweighting sequences after each pass to account for cross-reactivity and producing more accurate final scores.

### Include background control normalization
**Args:** `round1_counts.fa round2_counts.fa --control background.fa --normalize-bases -o normalized.tsv`
**Explanation:** Subtracts background enrichment using the control file and normalizes degenerate bases, producing z-score-corrected enrichment values that account for non-specific binding.