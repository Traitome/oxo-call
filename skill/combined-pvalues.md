---
name: combined-pvalues
category: Statistics
description: Combines multiple statistical p-values from independent tests using meta-analytic methods including Fisher's combined probability, Stouffer's Z-score, and logit-based approaches. Commonly used in population genetics, GWAS meta-analysis, and multi-tissue differential expression studies.
tags:
- p-value
- meta-analysis
- statistics
- multiple-testing
- bioinformatics
- fisher-method
- stouffer
author: AI-generated
source_url: https://github.com/popoolation/poopulation
---

## Concepts

- **Input Format**: The tool reads p-values from a whitespace-separated or tab-separated text file where each line contains a single p-value (all values must be between 0 and 1). Alternatively, accepts multiple files as separate arguments to aggregate p-values across experimental conditions.
- **Combination Methods**: Three primary algorithms are supported—Fisher's combined probability (default), Stouffer's Z-score, and the logit method. Fisher's is most powerful when tests are independent and effects are in the same direction; Stouffer's is preferred when directional information (sign) is available.
- **Output**: Produces a single combined p-value to standard output, which can be redirected to a file or piped to other tools. The output format is typically a single numeric value representing the combined significance.
- **Statistical Independence Assumption**: All combined p-value methods assume the input p-values come from statistically independent tests. Violating this assumption (e.g., overlapping samples) leads to inflated false positive rates.
- **Weighting Support**: When using Stouffer's method, you can provide weights (e.g., sample sizes) for each p-value to weight contributions proportionally using the `-w/--weight` flag.

## Pitfalls

- **Non-numeric or out-of-range values**: Providing p-values outside the valid range (0, 1) or non-numeric text causes the tool to fail silently or produce nonsensical output. Always validate input files beforehand.
- **Correlated tests**: Combining p-values from overlapping samples or correlated tests using Fisher's method inflates the combined significance, yielding anti-conservative results. Use only independent test outcomes.
- **Missing input**: Running the tool without any p-value input files or with malformed files produces no output or an error, leaving downstream analyses without results.
- **Method mismatch**: Using Fisher's method when you have directional effect signs wastes information; Stouffer's method can incorporate weights for more accurate meta-analysis.
- **Insufficient sample size**: Combining very few p-values (e.g., 2–3) provides little statistical power; the combined p-value may be dominated by the smallest individual p-value rather than true signal.

## Examples

### Combine p-values from a single file using Fisher's method
**Args:** `-m fisher input_pvalues.txt`
**Explanation:** Applies Fisher's combined probability method to aggregate all p-values listed in the file, outputting a single combined p-value.

### Combine p-values from multiple files using Stouffer's Z-score
**Args:** `-m stouffer population1_pvals.txt population2_pvals.txt population3_pvals.txt`
**Explanation:** Uses Stouffer's Z-transform method across three separate p-value files to compute a weighted Z-score combined p-value.

### Use weights with Stouffer's method based on sample size
**Args:** `-m stouffer -w 10,25,15 pop1.txt pop2.txt pop3.txt`
**Explanation:** Applies custom weights (10, 25, 15) to each p-value file before combining, giving more influence to larger sample sizes.

### Output combined p-value to a specific file
**Args:** `-m fisher results.txt -o combined_result.txt`
**Explanation:** Reads p-values from results.txt, combines via Fisher's method, and writes the final p-value to combined_result.txt.

### Use logit method for combination
**Args:** `-m logit -i all_pvalues.tsv`
**Explanation:** Applies the logit-based combination method to p-values in the tab-separated input file, useful when effect sizes vary substantially.

### Combine p-values while suppressing log output
**Args:** `-m fisher --quiet input.txt`
**Explanation:** Runs Fisher's combination without verbose logging, outputting only the final combined p-value for scripting workflows.