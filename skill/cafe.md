---
name: cafe
category: Phylogenetics / Evolutionary Genomics
description: CAFE (COmparative Analysis by Flexible evolutionary modeling) is a command-line tool that performs statistical inference of gene family evolution across a species phylogeny using a birth-death process. It models gene family size changes along a tree and reports which families have significantly expanded or contracted, estimating p-values for each change.
tags:
  - gene family evolution
  - birth-death process
  - phylogenetic modeling
  - ortholog analysis
  - comparative genomics
  - lambda estimation
  - gene expansion contraction
author: AI-generated
source_url: https://sourceforge.net/projects/cafe-software/
---

## Concepts

- **Input data model**: CAFE requires two core files — a Newick-format phylogenetic tree (`-t`) describing species relationships with branch lengths proportional to time or neutral substitutions, and a gene family count table (`-i`) in TAB-delimited format where rows are gene families (first column = family ID) and columns are species labels matching the tree tip names. The count file must contain only non-negative integer values; families with all zeros are silently skipped.
- **Lambda parameter and error model**: CAFE fits a single lambda (λ) value by default representing the per-gene per-unit-time birth-death rate, but `-e` loads an error model file (one lambda per tree branch) to account for branch-specific rate heterogeneity. When no error model is provided, a global λ is estimated via maximum likelihood across the entire tree, and users can override it with `-l`.
- **Output interpretation**: The main results file (`-o`) lists every gene family with its conditionaloot likelihood values per tip, estimated λ, p-value for expansion/contraction tests, and counts per species. Families with p-value ≤ the threshold (default 0.05, set via `-p`) are flagged as significantly changed. The `-x` flag prints an additional summary table with per-family delta values (observed minus expected gene count differences).
- **Companion tools**: `cafe-expand` preprocesses OrthoMCL/orthoDSM ortholog clustering output into the CAFE count format, taking a cluster file and a tree and outputting a `-i`-compatible count table. `cafe-foreach` runs CAFE iteratively over multiple error model files for sensitivity analysis.
- **Statistical testing**: CAFE uses likelihood ratio tests comparing a null model (constant gene family size) against an alternative model (size changes inferred from the tree). P-values in the output are computed from chi-squared distributions with degrees of freedom equal to the number of tips minus one. Families with very small count values or missing data may produce unreliable p-values and should be inspected manually.

## Pitfalls

- **Tree tip labels must exactly match the count table column headers**: If species names in the Newick tree do not match column names in the gene count file (case-sensitive, no trailing spaces), CAFE silently fails to map gene counts to tree branches and produces garbage output. Always verify label matching with a text diff or a dedicated validation script before running.
- **Using an integer count table with missing or non-integer values**: CAFE expects every cell in the input count table to be a non-negative integer. Empty cells, decimal values, or text strings cause the parser to halt with a cryptic error. When gene counts are derived from RNA-seq or functional annotation pipelines, round and impute missing values beforehand.
- **Confusing `-l` (lambda override) with `-e` (error model)**: Setting `-l` forces a fixed lambda value and skips maximum likelihood estimation entirely, which is useful for simulations but produces misleading p-values when applied to real data. Beginners often use `-l` unintentionally, discarding the statistical inference that CAFE exists to perform. Use `-l` only when you have a well-justified external lambda estimate.
- **Ignoring branch length units**: Lambda is scaled to the branch length units in the input tree. If branch lengths are in nucleotide substitutions per site but the true evolutionary time per substitution varies by lineage, the estimated lambda will be systematically biased. Always ensure tree branch lengths are in an appropriate proxy for evolutionary time for your clade.
- **Running CAFE without a物种-rich tree**: With very few species (fewer than ~10), likelihood estimation is poorly constrained and p-values are unreliable. CAFE may report statistically significant results for families that are simply noisy. Cross-validate significant families against independent data or subsample robustness testing.

## Examples

### Estimate gene family expansion and contraction significance across a phylogenetic tree
**Args:** `-i families.txt -t species_tree.newick -o cafe_results`
**Explanation:** This is the standard CAFE invocation, loading the gene family count table and species phylogeny, then estimating lambda and p-values for all families, writing results to `cafe_results`.

### Use an error model to account for branch-specific rate variation
**Args:** `-i families.txt -t species_tree.newick -e branch_errors.txt -o cafe_results_err`
**Explanation:** Providing an error model file with per-branch lambda estimates allows CAFE to model rate heterogeneity across lineages instead of assuming a single global lambda, yielding more accurate p-values for trees with variable substitution rates.

### Override lambda with a predefined value for simulation validation
**Args:** `-i mock_families.txt -t tree.newick -l 0.01 -o sim_results`
**Explanation:** Fixing lambda to 0.01 skips maximum likelihood estimation, which is appropriate when validating CAFE's behavior against simulated data where the true lambda is known.

### Set a stricter p-value threshold to focus on highly significant families
**Args:** `-i families.txt -t species_tree.newick -p 0.01 -o strict_results`
**Explanation:** Reducing the significance threshold from the default 0.05 to 0.01 filters the output to families with stronger statistical support, reducing false positives in genome-wide analyses.

### Generate an extended summary table with delta values per family
**Args:** `-i families.txt -t species_tree.newick -x -o extended_summary`
**Explanation:** The `-x` flag adds per-family delta columns to the output (observed minus expected gene counts at each tree tip), making it easier to identify which lineages drove a significant expansion or contraction.

### Preprocess OrthoMCL cluster output into CAFE-compatible format
**Args:** `-i orthoMCL_clusters.txt -t species_tree.newick -o family_counts.txt`
**Explanation:** Running `cafe-expand` with the OrthoMCL cluster file and tree converts the ortholog clustering output into a gene family count table that can be fed directly into CAFE with the `-i` flag.