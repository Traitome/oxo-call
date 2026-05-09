---
name: blobtools
category: Bioinformatics - Taxonomic Profiling
description: A tool for taxonomic profiling and visualization of metagenomic assembly data using GC content, coverage depth, and BLAST-based sequence similarity. Ideal for identifying contaminations and analyzing metagenome composition.
tags: [metagenomics, taxonomy, assembly, visualization, GC-content, coverage, BLAST]
author: AI-generated
source_url: https://github.com/DRL/blobtools
---

## Concepts

- Blobtools classifies contigs by performing BLAST searches against a reference database (typically NT/NR) and assigns taxonomy based on the best hits. Each contig receives a taxid and a "hit" status based on configurable criteria (e.g., evalue threshold, minimum alignment length).
- The tool operates on three core metrics: (1) GC content (derived from the FASTA assembly), (2) coverage depth (from aligned BAM files), and (3) taxonomic assignment (from BLAST hits). These two numerical fields create the classic "blob" visualization where taxa cluster based on nucleotide composition.
- Blobtools uses a SQLite database (`.blobDB.json` + associated files) to store all analysis results, enabling incremental updates and fast subsetting. The database is constructed via the `blobtools add` subcommand and queried via `blobtools view` or `blobtools text`.
- The companion binary `blobtools build` creates taxonomic summary files and Hit-Status-Lists (HSL) from the assembly by running BLAST or Diamond searches against a specified database (e.g., nt, nr).
- Output visualizations include blob plots (scatter of GC vs. coverage, colored by taxonomy), taxonomic pie/bar charts, and cumulative coverage plots. These are generated via `blobtools plot` using the `--format` and `--param` flags.

## Pitfalls

- Using a low coverage BAM file or failing to provide a BAM file at all produces plots showing only GC content, severely limiting the ability to distinguish eukaryotic, bacterial, and viral clusters. Without coverage, the tool cannot identify outliers based on abnormal sequencing depth.
- Setting an overly permissive evalue threshold (e.g., `--evalue 1e-1`) causes spurious taxonomic assignments, leading to incorrect contamination identification and biased taxonomic profiles.
- Running `blobtools build` without specifying the correct database path or using an outdated BLAST/nt database results in poor classification rates and many contigs labeled as "unknown" or "no-hit".
- Inconsistent naming between the assembly FASTA headers and the species names in the BLAST database causes mismatches. Headers must contain unique identifiers that map correctly to the expected taxids.
- Attempting to plot large assemblies (>10,000 contigs) without filtering by length or coverage produces visually cluttered blob plots and significantly increases rendering time.

## Examples

### Generate a blob plot from an assembly with coverage data

**Args:** `plot --input assembly.blobDB.json --plot blob --format png --output blob_plot.png`

**Explanation:** This command creates a standard blob plot visualizing each contig as a point positioned by GC content (x-axis) and coverage depth (y-axis), colored by assigned taxonomy.

### View taxonomic assignment results as a text table

**Args:** `view --input assembly.blobDB.json --field taxid --field taxname --field gc --field cov --outfmt tsv`

**Explanation:** This command outputs a tab-separated table containing the taxid, taxonomy name, GC content, and coverage for each contig in the database, enabling downstream analysis in spreadsheet software.

### Filter contigs longer than 1000bp and export to FASTA

**Args:** `view --input assembly.blobDB.json --filtcontig "length>=1000" --format fasta --output filtered_contigs.fasta`

**Explanation:** This command applies a length filter to retain only contigs with at least 1000 bases and writes them to a new FASTA file, useful for downstream gene prediction or re-assembly.

### Create a taxonomicsummary file in JSON format

**Args:** `view --input assembly.blobDB.json --field taxname --summarize taxname --outfmt json --output tax_summary.json`

**Explanation:** This command generates a JSON summary showing the number of contigs and total bases assigned to each taxonomic name, ideal for automated reporting pipelines.

### Generate a Hit-Status-List from custom BLAST results

**Args:** `add --input assembly.blobDB.json --hits custom_blast.out --format blast --hsp evalue --hsp minlength 50`

**Explanation:** This command incorporates custom BLAST results into the blobtools database, using an evalue cutoff and requiring a minimum aligned length of 50 bases to assign hit status.

### Plot coverage distribution in PDF format for a specific taxonomic phylum

**Args:** `plot --input assembly.blobDB.json --plot cov --format pdf --taxonlevel phylum --phylum Proteobacteria --output proteo_coverage.pdf`

**Explanation:** This command generates a PDF plot showing the coverage distribution specifically for contigs assigned to the phylum Proteobacteria, useful for targeted analysis of bacterial populations.

### Assign taxonomic information using a hit-list with Diamond

**Args:** `build --fasta assembly.fasta --db /path/to/nr.dmr --out assembly.blobDB.json --threads 16 --type diamond`

**Explanation:** This is a companion command (uses the `blobtools build` binary) that runs Diamond BLASTX against the NR database to generate taxonomic hit information, storing results in a new database.