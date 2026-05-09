---
name: anvio
category: Metagenomics & Microbial Genomics
description: A comprehensive bioinformatics platform for analysis of microbial genomes, metagenomes, and integrated omics data with interactive visualization capabilities.
tags: [metagenomics, pangenomics, phylogenomics, microbial, visualization, genome-assembly, anvio]
author: AI-generated
source_url: https://anvio.org
---

## Concepts

- **Core Databases**: Anvi'o operates on specialized database files (*.db) including contigs.db for assembled sequences, profiles.db for abundance profiles, and pan.db for pangenomic analyses; these databases are interconnected through unique self-contained identifiers that allow cross-referencing between datasets.

- **Interactive Interface**: The anvi-display command launches a web-based graphical interface (using Flask) to explore genomic data interactively, allowing users to visualize coverage and composition across samples, annotate functions, and manually curate bins with real-time updates saved back to the profile database.

- **Profile Construction**: The anvi-profile command takes mapped BAM files and generates coverage/GC-normalized abundance profiles from metagenomic reads, requiring both a contigs database reference and input BAM files as mandatory arguments to produce cross-sample visualizations.

- **Data Import/Export**: Anvi'o uses anvi-import to bring external data into its ecosystem in standard formats (FASTA, BAM, COG, KEGG, GFF3), and anvi-export to extract data for downstream applications; collections and bins are stored as JSON structures defining group membership for contigs or splits.

- **Split and Contig Hierarchy**: Anvi'o organizes sequences in a hierarchical structure where full sequences (contigs) can be divided into splits (configurable length, default 10000 bp) that serve as units for binning, visualization, and downstream analyses, enabling multi-resolution exploration of the same dataset.

## Pitfalls

- **Mismatched Database Versions**: Using contigs and profile databases created with different anvi'o versions leads to crashes or corrupted displays; always verify compatibility using anvi-db-info and recreate databases if version mismatch is detected.

- **Memory Limits with Large Datasets**: Profiling BAM files with millions of reads without setting appropriate --skip-SQL-functions or --no-blast flags causes excessive RAM consumption and system instability on compute clusters with limited resources.

- **Missing Auxiliary Data Files**: Referencing gene calls, functional annotations, or external classification files that have been moved or deleted after initial import breaks downstream analyses without producing clear error messages; always maintain absolute paths or symlinks.

- **Incorrect Split Names in Collections**: Importing collections where split names do not exactly match entries in the profile database (case-sensitive) silently fails to apply bin assignments, leaving data ungrouped in visualizations.

- **Insufficient Coverage Filtering**: Running anvi-profile without setting appropriate --min-coverage-threshold flags includes spurious contigs from low-abundance reads, inflating the dataset with noise that degrades clustering quality and visualization clarity.

## Examples

### Profile metagenomic reads mapped to assembled contigs
**Args:** -c contigs.db -i sample01.bam --profile-db-mode single
**Explanation:** Creates a profile database storing coverage statistics from BAM file alignments for interactive visualization and downstream analyses.

### Import a pre-existing binning collection into a profile
**Args:** -p profile.db -C my_collection --source夺冠_json collections.json
**Explanation:** Loads bin definitions stored in JSON format into an existing profile database for visualization in the anvi-display interface.

### Merge multiple sample profiles into a combined dataset
**Args:** -p profile_01.db profile_02.db profile_03.db -o merged.db
**Explanation:** Combines individual sample profiles to enable cross-sample comparative analyses and unified binning across conditions.

### Run COG functional annotation on gene calls
**Args:** -c contigs.db --cog-data_dir /path/to/cog_db --num-threads 8
**Explanation:** Executes parallel annotation of coding sequences against the COG database, populating the contigs database with functional assignments.

### Launch interactive interface to explore and curate bins
**Args:** -p profile.db -c contigs.db
**Explanation:** Opens a web browser interface allowing real-time exploration of coverage patterns and manual refinement of bin assignments.

### Export collection bins to external FASTA files
**Args:** -p profile.db -c contigs.db -C my_collection --output-dir ./exported_bins
**Explanation:** Splits binned sequences into individual FASTA files named by bin identifier, enabling downstream phylogenetic or functional studies.

### Compute genome rarity scores for pangenome analysis
**Args:** -p pan.db --compute-composition-tests --num-threads 4
**Explanation:** Calculates statistical rarity metrics across genomes to identify gene clusters with unusual distribution patterns for ecological interpretation.