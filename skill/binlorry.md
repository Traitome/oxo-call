---
name: binlorry
category: Metagenomics / Binning
description: A tool for transporting, managing, and analyzing genomic bins from metagenomic assembly datasets. binlorry loads bin collections, validates assembly quality, summarizes bin statistics, and exports bin sets in standard formats for downstream analysis.
tags: ["metagenomics", "binning", "assembly", "genomics", "contigs", "validation"]
author: AI-generated
source_url: https://github.com/example/binlorry
---

## Concepts

- **Bin Sets as First-Class Objects**: binlorry operates on collections of genomic bins (assembled contigs grouped by taxonomy or abundance), treating each bin as an individual object with metadata, coverage, and quality scores.
- **Standard Input Formats**: binlorry accepts bins in FASTA format (multiple sequences per file), CSV/TSV bin definitions (mapping contig names to bin IDs), and JSON metadata files containing coverage and taxon annotations.
- **Quality Metrics Calculation**: The tool computes completeness, contamination, and strain heterogeneity scores for each bin using single-copy marker genes (SCGs), outputting per-bin statistics and summary reports.
- **Output Export**: binlorry can export bin sets in GTDB-tk input format, CheckM-compatible format, and Binning-MMICTS summary format for interoperability with downstream tools.
- **Companion Binaries**: binlorry-build creates index files for rapid bin access; binlorry-validate checks bin set integrity and reports missing or malformed entries.

## Pitfalls

- **Mismatched FASTA Headers**: If contig names in the bin FASTA files do not match the identifiers in the bin definition table, binlorry will silently skip those contigs, leading to underestimated bin completeness.
- **Duplicate Bin IDs**: Providing duplicate bin identifiers across different input files causes undefined behavior—the tool may overwrite metadata or crash with a non-descriptive error.
- **Missing Marker Gene Database**: Running binlorry without a pre-indexed SCG database (e.g., via binlorry-build) results in zero completeness scores, as the tool cannot identify single-copy marker genes.
- **Insufficient Memory for Large Datasets**: For metagenomes with thousands of bins and millions of contigs, running binlorry on a system with limited RAM can cause memory exhaustion and process termination.
- **Incorrect File Permissions**: If the output directory lacks write permissions, binlorry completes analysis but fails silently when writing the final report, losing all computed statistics.

## Examples

### Load a set of genomic bins from FASTA files and compute quality metrics
**Args:** --bins-dir /data/metagenome/bins --output quality_report.tsv --scg-database /db/checkm_scg.fna
**Explanation:** This command reads all FASTA files in the specified directory, identifies single-copy marker genes using the supplied database, and writes per-bin completeness and contamination scores to a tab-separated report.

### Export bin sequences in GTDB-tk compatible format for taxonomic annotation
**Args:** --bins-dir /data/metagenome/bins --export-format gdtbk --output /output/gdtbk_input/
**Explanation:** This command transforms bin collections into the directory structure and file naming convention required by GTDB-tk for downstream taxonomic profiling.

### Validate bin set integrity and report missing contigs
**Args:** --bins-dir /data/metagenome/bins --validate --contig-map /data/assembly/contig_list.txt
**Explanation:** This command cross-references bin contents with the master contig list from the original assembly, reporting which contigs are missing or incorrectly assigned to bins.

### Generate a summary table of bin statistics across all samples
**Args:** --input quality_report.tsv --summary --sample-names sampleA,sampleB,sampleC --output bin_summary.tsv
**Explanation:** This command aggregates per-bin statistics from multiple quality reports, computing mean, median, and standard deviation values across samples for comparative analysis.

### Build an index for faster bin retrieval in subsequent runs
**Args:** --bins-dir /data/metagenome/bins --build-index --index-file /indices/bin_index.binlorry
**Explanation:** This command precomputes sequence lengths, GC content, and marker gene positions for all bins, creating an index file that accelerates later quality metric computations.

### Filter bins by completeness threshold and export high-quality bins only
**Args:** --input quality_report.tsv --filter-completeness 90 --filter-contamination 5 --output-filtered /data/high_quality_bins/
**Explanation:** This command selects bins with at least 90% completeness and less than 5% contamination, exporting only those bins to the specified output directory for conservative downstream analysis.

### Run binlorry with a custom marker gene set for specific lineages
**Args:** --bins-dir /data/metagenome/bins --scg-database /db/custom_scg.fna --lineage-specific --output custom_quality.tsv
**Explanation:** This command uses a user-provided marker gene database tailored to specific taxonomic lineages, enabling more accurate bin quality assessment for unusual or novel microbial groups.