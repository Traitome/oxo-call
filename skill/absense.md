---
name: absense
category: sequence-analysis
description: Detects absent or divergent sequences between query and reference genomic datasets, commonly used in metagenomic analysis to identify missing or highly divergent genes across samples.
tags: [sequence-comparison, metagenomics, divergent-genes, absence-detection]
author: AI-generated
source_url: https://www.nite.go.jp/centre/bsi-results/data.html
---

## Concepts
- absense compares a query sequence dataset against a reference dataset to identify sequences that are either completely absent or show high divergence (low similarity) from any reference sequence
- Input files must be in FASTA or multi-FASTA format, with query sequences provided first followed by the reference sequence file on the command line
- The tool outputs sequence identifiers of absent or divergent queries, along with statistical information such as divergence scores and query coverage percentages
- absense supports two detection modes: strict absence mode (no match found) and divergence mode (match exists but falls below identity/coverage thresholds)
- The tool can process both nucleotide and translated protein sequences depending on the specified sequence type flag

## Pitfalls
- Forgetting to specify the output file redirects all results to stdout, making programmatic parsing difficult and potentially losing output on large datasets
- Using default divergence thresholds without adjusting for the evolutionary distance of your data may produce false negatives for distantly related sequences
- Input files with mixed line endings or malformed FASTA headers cause parsing errors that silently skip affected sequences from the comparison
- Specifying the wrong sequence type mode (nucleotide vs protein) results in meaningless alignments since scoring matrices are incompatible with the wrong alphabet
- Memory consumption grows with reference file size; large reference datasets on systems with limited RAM may cause crashes or severe performance degradation

## Examples

### Detect completely absent query sequences in a reference genome database
**Args:** query_seqs.fa ref_genomes.fna -o absent_queries.txt -m absence
**Explanation:** This identifies and lists all query sequences that have no detectable match in the reference database, useful for finding novel sequences.

### Find highly divergent sequences below a custom divergence threshold
**Args:** metagenome.fna ardb_ref.fna --cutoff 0.30 --output divergent.list
**Explanation:** This flags sequences with less than 30% identity to any reference, allowing flexible threshold tuning for datasets with varying evolutionary distances.

### Compare multiple query files against a single reference
**Args:** sample1.fa sample2.fa sample3.fa refDB.fa --combined-output batch_comparison.tsv
**Explanation:** This processes three separate query files in a single run against one reference, producing a combined results table for batch comparison workflows.

### Generate a summary statistics report for alignments
**Args:** query.fna reference.fna -S stats_report.txt --format tabular
**Explanation:** This outputs detailed per-sequence alignment statistics including identity percentages and coverage values in a tab-delimited format for downstream R or Python analysis.

### Extract only high-confidence matching sequences
**Args:** all_queries.fa core_refs.fa --min-identity 0.95 --min-cov 0.90 -o confident_matches.fa
**Explanation:** This filters and outputs only sequences matching the reference with at least 95% identity and 90% coverage, useful for building high-confidence functional annotation sets.