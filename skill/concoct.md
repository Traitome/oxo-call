---
name: concoct
category: Metagenomics / Binning
description: Clusters metagenomic contigs into putative MAGs (Metagenome-Assembled Genomes) using coverage vectors across samples and tetranucleotide composition patterns. Uses Gaussian Mixture Models for probabilistic binning.
tags: [metagenomics, contig-binning, MAGs, coverage, tetranucleotide, GMM]
author: AI-generated
source_url: https://github.com/BinPro/CONCOCT
---

## Concepts

- CONCOCT expects two core input matrices: a coverage table (rows = contigs, columns = samples) and a tetranucleotide frequency (TNF) table (256 features per contig). Both matrices must have contig names as row identifiers and same-order contigs across files.

- The clustering algorithm uses Gaussian Mixture Models (GMM) with optional weighting of coverage versus composition via the `--use_coverage` and `--use_composition` flags. Both features are used by default, enabling hybrid clustering that leverages both abundance and sequence patterns.

- Output is a simple two-column file mapping each contig to a cluster ID (0-indexed), which can be directly used for downstream MAG quality assessment or taxonomic profiling. The order of contigs in output matches input order.

- CONCOCT can leverage linkage information from read-pair or BWA MEM connections via the `--links_file` option to provide hard constraints during clustering, improving binning accuracy for complex metagenomes with strain heterogeneity.

- The `-c/--clusters` parameter specifies the expected number of bins (population-level genomes). Without prior knowledge, a range of cluster counts can be tested and optimal K selected using tools like GTDB-Tk or CheckM for completion/binary评估.

## Pitfalls

- Mismatched contig names between the coverage table and TNF table causes silent failures or garbage clustering. The row identifiers in both files must exactly match the FASTA headers from the input contigs, including any whitespace or special characters.

- Underestimating the number of true genomes leads to oversplitting or merging of unrelated contigs into false bins. Using too many clusters (over-splitting) is generally less harmful than underestimating, but wastes downstream analysis time.

- Skipping data normalization before running CONCOCT can bias clustering toward samples with higher coverage magnitudes. Each sample's coverage vector should be normalized (e.g., divide by mean or per-sample median) before input if sample depths vary dramatically.

- Failing to sort the coverage matrix so contig rows are in the same order as the TNF matrix produces completely incorrect results. CONCOCT assumes parallel ordering; users must pre-sort or verify row alignment.

- Running CONCOCT without checking for chimeric contigs in the original assembly can propagate assembly errors into downstream bins. It's best to remove potential chimeras before clustering.

## Examples

### Cluster contigs using both coverage and composition features
**Args:** `--coverage_file coverage.tsv --composition_file tnf.tsv --output_file clusters.csv -c 50`
**Explanation:** Runs CONCOCT with both coverage and tetranucleotide composition (default behavior), clustering into 50 bins. The coverage file must have contigs as rows and samples as columns.

### Cluster using only composition (no coverage data available)
**Args:** `--composition_file tnf.tsv --output_file composition_clusters.csv -c 30 --no_use_coverage`
**Explanation:** Clusters based solely on tetranucleotide frequencies when multiple sample coverage data does not exist. Useful for single-sample metagenomes or assembly-only datasets.

### Cluster with explicit verbose output for debugging
**Args:** `--coverage_file cov.tsv --output_file verbose_clusters.csv -c 25 --verbose`
**Explanation:** Runs clustering with verbose logging enabled, helpful when diagnosing unexpected behavior or verifying that input matrices are correctly parsed.

### Generate linkage file for improved binning accuracy
**Args:** `--links_file contig_links.csv --coverage_file cov.tsv --output_file linked_clusters.csv -c 40`
**Explanation:** Incorporates linkage constraints derived from read-pair connections to improve clustering, especially for strains or closely related populations.

### Use custom delimiter for input files
**Args:** `--coverage_file cov.csv --delimiter , --output_file delim_clusters.csv -c 15`
**Explanation:** Parses coverage file with comma delimiter instead of default tab, for CSV-formatted input from some assembly pipelines.

### Set random seed for reproducible results
**Args:** `--coverage_file cov.tsv --composition_file tnf.tsv --output_file reproducible_clusters.csv -c 20 -r 42`
**Explanation:** Sets random seed to make clustering output deterministic, essential for reproducible workflows and benchmarking.

---