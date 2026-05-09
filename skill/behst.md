---
name: behst
category: gene-set-analysis
description: Biomarker Exploration via Hypergraph Scoring Tool for performing hypergraph-based enrichment analysis on gene sets to identify statistically significant biological terms and potential biomarkers.
tags:
  - enrichment
  - biomarker
  - hypergraph
  - gene-set-analysis
  - functional-annotation
author: AI-generated
source_url: https://github.com/oxo-security/behst
---

## Concepts

- BEHST constructs hypergraphs from gene sets where nodes represent genes and hyperedges represent biological terms ( pathways, GO terms, diseases), enabling modeling of many-to-many gene-term relationships that standard graphs cannot capture.
- The tool accepts gene sets in GMT (Gene Matrix Transposed) format where each row contains a term identifier, description, and associated gene list separated by tabs; alternative inputs include BED files for genomic regions.
- BEHST calculates enrichment statistics using a hypergeometric distribution with empirical p-value correction via permutation testing (default 1000 permutations), producing q-values for multiple testing correction.
- Output is written as a tab-separated file with columns for term ID, description, observed count, background count, enrichment ratio, p-value, q-value, and associated genes, allowing downstream analysis in spreadsheet software or custom pipelines.

## Pitfalls

- Using gene sets with inconsistent identifiers (e.g., mixing Entrez IDs with gene symbols) causes silent parsing failures where genes fail to map to the background genome, resulting in zero enrichment calls without warning.
- Specifying an overly restrictive p-value threshold (e.g., 0.001 without permutation correction) may eliminate true positives because empirical p-values require sufficient permutations to achieve resolution at that granularity.
- Running BEHST without specifying a background genome leads to automatic use of all genes in the gene set database as the background, inflating enrichment scores for large gene sets and producing false positives for generic terms.
- Forgetting to sort the input gene set file disrupts row parsing and causes fields to shift, resulting in incorrect gene counts assigned to wrong term descriptions in the output.

## Examples

### Basic enrichment analysis with a single gene set file

**Args:** `-i my_geneset.gmt -o results.tsv`
**Explanation:** This runs BEHST with default parameters on the gene set file my_geneset.gmt, outputting enriched terms to results.tsv using all built-in background genomes and 1000 permutation tests.

### Performing enrichment analysis with Entrez gene identifiers

**Args:** `-i candidate_genes.gmt --id-type entrez -o entrez_results.tsv`
**Explanation:** Specifying the identifier type ensures correct mapping between input genes and the background genome database, preventing parsing failures when Entrez IDs are used.

### Adjusting the p-value threshold to filter results

**Args:** `-i biomarkers.gmt -o filtered_biomarkers.tsv --p-value 0.05 --q-value 0.1`
**Explanation:** The q-value threshold of 0.1 applies Benjamini-Hochberg correction for multiple testing, producing more conservative results suitable for downstream validation experiments.

### Using a custom gene set database instead of the default

**Args:** `-i input_genes.gmt -d custom_pathways.gmt -o pathway_results.tsv`
**Explanation:** Providing a custom database allows analysis against user-curated or organism-specific gene sets rather than generic annotations, improving relevance for non-model organisms or specialized research areas.

### Running analysis with increased permutation depth for high-resolution p-values

**Args:** `-i gene_set.gmt -o high_precision.tsv --permutations 10000`
**Explanation:** Increasing permutations to 10000 provides finer resolution for p-value calculation, which is necessary when using very strict significance thresholds such as 0.001 or when working with small gene sets.

### Specifying a background genome for focused analysis

**Args:** `-i upregulated.gmt -o brain_results.tsv --background hsapiens_gene_set.gmt`
**Explanation:** Using a species-specific background genome restricts analysis to protein-coding genes with known annotations, producing biologically meaningful results rather than spurious enrichments from poorly characterized genes.