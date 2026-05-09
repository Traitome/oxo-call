---
name: clinker
category: gene-cluster-comparison
description: Gene cluster comparison and synteny visualization tool for microbial genomics. Clinker aligns multiple annotated genomes or biosynthetic gene clusters (BGCs) and generates interactive HTML reports and static SVG images showing homologous relationships and synteny blocks.
tags:
  - synteny
  - gene-cluster
  - visualization
  - microbial-genomics
  - comparative-genomics
  - orthologs
author: AI-generated
source_url: https://github.com/gempypi/clinker
---

## Concepts

- **Input formats:** Clinker accepts multiple GenBank (.gbk, .gbff) files containing annotated genomes or BGCs. It can also process antiSMASH JSON output directly when gene clusters are annotated with biosynthetic features. Each input file should represent one genome or gene cluster for accurate comparison.

- **Homology calculation:** Clinker computes pairwise sequence similarities between genes using either BLAST+ or DIAMOND, depending on the dataset size. Genes are grouped into ortholog families via hierarchical clustering with configurable identity and coverage thresholds. This determines which genes are considered homologous for synteny visualization.

- **Output generation:** The tool produces an HTML report with an interactive dotplot matrix and synteny links, static SVG images for publication-quality figures, and CSV files containing homology scores and cluster assignments. Reports can be filtered by minimum identity, coverage, and e-value cutoffs specified at runtime.

- **Visual encoding:** In generated figures, line thickness maps to amino acid identity percentage, while color indicates cluster membership. This allows rapid identification of conserved synteny blocks versus lineage-specific gene gains or losses across the compared genomes or gene clusters.

## Pitfalls

- **Mixed file format incompatibility:** Providing both GenBank and FASTA files in a single run causes Clinker to crash or produce empty homology results. All input files must be in the same format (GenBank recommended) for consistent feature parsing across the dataset.

- **Overwhelming memory with large batches:** Comparing more than 20 large genomes (>5 MB each) without increasing available RAM can cause the process to terminate with a memory error. Process genomes in batches of 10–15 or use the `--parallel` flag to distribute computation across cores.

- **Default identity threshold too permissive:** The default 30% amino acid identity threshold may link distantly related genes, creating spurious synteny connections that obscure true evolutionary relationships. Adjust using `--min-id` (e.g., 50 or 70) for tighter homology definitions appropriate to the taxonomic rank being compared.

- **Missing CDS features breaks comparison:** GenBank files lacking proper CDS annotations (e.g., assembled contigs without gene prediction) result in zero matches and an empty output. Ensure all input files have been annotated with a gene caller (e.g., Prodigal for prokaryotes) before running Clinker.

- **Duplicate gene cluster IDs cause data loss:** When multiple input files share identical gene cluster IDs, Clinker silently overwrites earlier entries with later ones in the output. Prefix or suffix cluster IDs with a unique genome identifier to preserve all data during multi-sample comparisons.

## Examples

### Compare two bacterial genomes in GenBank format
**Args:** genome1.gbk genome2.gbk -o comparison_report
**Explanation:** Aligns all gene clusters between two annotated genomes and generates an HTML report with interactive dotplot visualization of homology links.

### Compare five genomes simultaneously for pan-genome analysis
**Args:** *.gbk -o pan_genome_output --format html --identity-cutoff 40
**Explanation:** Processes all matching GenBank files in the current directory, filtering homology connections to genes with at least 40% amino acid identity for cleaner pan-genome visualization.

### Export results as CSV for downstream analysis
**Args:** cluster1.gbk cluster2.gbk cluster3.gbk -o csv_export --exportcsv
**Explanation:** Produces a tab-delimited file containing all pairwise homology scores, enabling import into R or Python for statistical analysis of gene cluster conservation.

### Use Diamond for faster homology search in large datasets
**Args:** genomes/*.gbk -o large_comparison --diamond --min-id 50 --mincov 80
**Explanation:** Enables the faster DIAMOND aligner and applies stringent filters requiring genes to share at least 50% identity and 80% coverage, suitable for closely related strains where high-confidence orthologs are expected.

### Generate publication-ready SVG image
**Args:** BGC1.gbk BGC2.gbk -o publication_figure --format svg --links
**Explanation:** Produces a static vector graphic with drawn synteny links between the two biosynthetic gene clusters, directly usable in manuscript figures without further editing.