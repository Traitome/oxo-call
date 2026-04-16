---
name: gtdbtk
category: metagenomics
description: GTDB-Tk — taxonomic classification of prokaryotic genomes using the Genome Taxonomy Database
tags: [taxonomy, gtdb, phylogenomics, bacteria, archaea, mag, classification, metagenomics]
author: oxo-call built-in
source_url: "https://github.com/Ecogenomics/GTDBTk"
---

## Concepts

- GTDB-Tk classifies prokaryotic genomes (isolates and MAGs) using the Genome Taxonomy Database (GTDB).
- Main command: gtdbtk classify_wf for full workflow (identify markers + classify).
- Use --genome_dir for directory of genome FASTA files; --out_dir for results; --cpus for threads.
- GTDB-Tk requires a pre-downloaded database (GTDB reference data, ~100 GB) — set GTDBTK_DATA_PATH.
- Output: gtdbtk.bac120.summary.tsv (bacterial) and gtdbtk.ar53.summary.tsv (archaeal) with taxonomy.
- Use --extension fa or --extension fasta to specify genome file extension in --genome_dir.
- GTDB taxonomy (GTDB-Tk) differs from NCBI taxonomy — species names may differ significantly.
- classify_wf also produces phylogenetic trees and alignment files for reference.
- --batchfile provides an alternative to --genome_dir; tab-separated file with columns: FASTA path, genome ID, [translation table].
- --pplacer_cpus controls CPUs for the pplacer placement step separately from general --cpus; useful for memory-intensive steps.
- --scratch_dir reduces pplacer memory usage by writing to disk (slower but less RAM required).
- --genes flag indicates input contains predicted proteins (amino acids), skipping gene calling; useful for pre-annotated genomes.
- de_novo_wf creates phylogenetic trees without GTDB reference placement; requires --bacteria or --archaea and --outgroup_taxon.
- --skip_ani_screen skips the ANI screening step for faster processing; may reduce classification accuracy for novel genomes.
- --min_perc_aa filters genomes with insufficient marker gene coverage (default 10%); increase for higher quality requirements.

## Pitfalls

- GTDB database must be downloaded before running — set GTDBTK_DATA_PATH to the database directory.
- GTDB-Tk requires significant RAM (>300 GB for pplacer step with all reference genomes).
- Without --cpus flag, GTDB-Tk uses limited threads — always specify for faster processing.
- The --genome_dir must contain only genome FASTA files — remove any non-genome files first.
- GTDB and NCBI taxonomy do not map directly — report both when publishing.
- Low-quality MAGs (<50% completeness) may not classify accurately.
- --genes flag skips ANI screening and classification steps; only use with pre-annotated protein files, not for standard genome classification.
- --batchfile requires tab-separated format; spaces or commas will cause parsing errors.
- --pplacer_cpus does not reduce memory requirements; use --scratch_dir to trade speed for lower memory usage.
- --full_tree requires >320 GB RAM; only use on high-memory systems, otherwise use default split tree approach.
- --skip_ani_screen speeds up processing but may miss close relatives; not recommended for novel genome classification.
- Default extension is .fna; must specify --extension for .fa, .fasta, or .gz files.
- de_novo_wf requires explicit --bacteria or --archaea selection and a valid --outgroup_taxon; cannot run without these.

## Examples

### classify a directory of genome bins with GTDB-Tk
**Args:** `classify_wf --genome_dir bins/ --out_dir gtdbtk_output/ --cpus 32 --extension fa`
**Explanation:** --genome_dir directory with .fa files; --out_dir results directory; --extension fa file extension

### classify genomes with custom GTDB database path
**Args:** `classify_wf --genome_dir bins/ --out_dir gtdbtk_output/ --cpus 32 --extension fasta --skip_ani_screen`
**Explanation:** --skip_ani_screen skips slow ANI check; faster for large datasets but may reduce accuracy

### run only the identification step (marker gene identification)
**Args:** `identify --genome_dir bins/ --out_dir gtdbtk_identify/ --cpus 16 --extension fa`
**Explanation:** identify subcommand extracts marker genes; useful for intermediate workflow steps

### classify genomes using a batch file with custom genome IDs
**Args:** `classify_wf --batchfile genome_list.tsv --out_dir gtdbtk_output/ --cpus 32`
**Explanation:** --batchfile is tab-separated: column1=FASTA path, column2=genome ID, column3=translation table (optional); allows custom genome naming

### reduce memory usage during pplacer step
**Args:** `classify_wf --genome_dir bins/ --out_dir gtdbtk_output/ --cpus 32 --pplacer_cpus 4 --scratch_dir /scratch/gtdbtk`
**Explanation:** --scratch_dir writes to disk to reduce RAM; --pplacer_cpus limits parallel placement; useful for memory-constrained systems

### classify pre-annotated protein sequences
**Args:** `classify_wf --genome_dir proteins/ --out_dir gtdbtk_output/ --cpus 32 --genes --extension faa`
**Explanation:** --genes indicates input is amino acid sequences; skips gene calling; WARNING: also skips ANI screening

### run de novo tree inference for bacterial genomes
**Args:** `de_novo_wf --genome_dir bins/ --out_dir gtdbtk_denovo/ --cpus 32 --bacteria --outgroup_taxon p__Patescibacteria`
**Explanation:** de_novo_wf builds tree without GTDB reference; --bacteria specifies domain; --outgroup_taxon roots the tree

### filter low-quality genomes by marker gene coverage
**Args:** `classify_wf --genome_dir bins/ --out_dir gtdbtk_output/ --cpus 32 --min_perc_aa 50`
**Explanation:** --min_perc_aa 50 excludes genomes with <50% marker gene coverage; useful for removing low-quality MAGs before classification

### run align step separately after identify
**Args:** `align --identify_dir gtdbtk_identify/ --out_dir gtdbtk_align/ --cpus 16 --skip_gtdb_refs`
**Explanation:** align creates MSA from identify output; --skip_gtdb_refs excludes reference genomes for faster processing

### classify from existing alignment
**Args:** `classify --genome_dir bins/ --align_dir gtdbtk_align/ --out_dir gtdbtk_classify/ --cpus 32`
**Explanation:** classify runs final classification step using pre-computed alignment; useful for re-classification with different parameters
