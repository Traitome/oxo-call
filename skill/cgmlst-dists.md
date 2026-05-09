---
name: cgmlst-dists
category: Genomics
description: Computes pairwise genetic distances between bacterial isolates based on core genome multilocus sequence typing (cgMLST) allele profiles. Takes an allele matrix as input and outputs a distance matrix reflecting the number of allele differences between each pair of isolates.
tags: [cgmlst, distance-matrix, bacterial-typing, allele-profile, genomics]
author: AI-generated
source_url: https://github.com/tseemann/cgmlst-dists
---

## Concepts

- **Input format**: cgmlst-dists accepts a tab-separated (TSV) or comma-separated (CSV) matrix where each row represents an isolate and each column represents a cgMLST locus. Cell values are allele numbers (integers), with missing alleles typically denoted by `0`, `*`, or `NA`.
- **Distance calculation**: The tool computes pairwise distances by counting the number of loci with different allele values between two isolates. A distance of 0 indicates identical profiles, while higher values indicate more allelic differences.
- **Output format**: By default, outputs a symmetric distance matrix in TSV format with isolate names as row and column headers. Each cell contains the pairwise allele difference count.

## Pitfalls

- **Invalid column structure**: If the input file lacks an isolate identifier column or has inconsistent column headers, the tool will fail to associate distances with the correct isolate names.
- **Mixed missing data representations**: Using different placeholders for missing alleles (e.g., `0` in some samples and `-1` in others) can lead to inflated distance calculations or parsing errors.
- **Non-numeric allele values**: Entering non-numeric values (such as allele names like "abc" instead of numeric alleles) in the allele matrix will cause calculation failures.

## Examples

### Calculate pairwise distances from a cgMLST profile file

**Args:** ` isolates.tsv -o distances.tsv`

**Explanation:** Reads the allele profile matrix from isolates.tsv and writes the pairwise distance matrix to distances.tsv.

### Output distances in CSV format

**Args:** ` isolates.tsv --format csv -o distances.csv`

**Explanation:** Produces the distance matrix in CSV format instead of the default TSV, which can be useful for downstream R or Python analysis.

### Exclude missing alleles from distance calculation

**Args:** ` isolates.tsv --ignore-missing -o distances.tsv`

**Explanation:** Treats missing allele calls (0, *, or NA) as excluded loci, preventing them from being counted as differences between isolates.

### Use only specific loci for distance calculation

**Args:** ` isolates.tsv --loci locus_list.txt -o distances.tsv`

**Explanation:** Computes distances using only the loci listed in locus_list.txt, allowing focused analysis on a subset of cgMLST targets.

### Set a minimum allele count threshold

**Args:** ` isolates.tsv --min-alleles 100 -o distances.tsv`

**Explanation:** Excludes isolates from the output that have fewer than 100 called alleles, filtering out low-quality or incomplete profiles.

### Verbose output for debugging

**Args:** ` isolates.tsv --verbose -o distances.tsv`

**Explanation:** Enables verbose logging to stderr, useful for diagnosing input parsing issues or unexpected distance values.