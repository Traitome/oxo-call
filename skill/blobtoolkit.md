---
name: blobtoolkit
category: Bioinformatics - Taxonomic Analysis
description: A toolkit for taxonomic analysis of metagenomic assemblies using coverage and composition data. Generates blobplots to visualize and identify contamination, to filter sequences by taxonomy, and to produce filtered assemblies.
tags: [assembly-analysis, blobplot, contamination-detection, coverage-analysis, taxonomic-filtering, metagenomics, GC-content]
author: AI-generated
source_url: https://github.com/blobtoolkit/blobtoolkit
---

## Concepts

- **Data Model**: Blobtoolkit operates on a database containing FASTA sequences (assembly), read mappings (BAM/SAM), and coverage values. It calculates GC content and read coverage for each sequence, then assigns taxonomy using a reference database. The output is a blob database (JSON) that can be queried, filtered, and visualized as scatter plots.
- **I/O Formats**: Input accepts FASTA files (assemblies), BAM/SAM files (read mappings), and coverage files (from coverage tools). Output includes JSON databases, CSV tables, and graphical plots (PNG/SVG). Taxonomic assignment requires a `taxdump` directory from NCBI.
- **Core Subcommands**: `blobtools create` initializes a database from an assembly plus optional hits/coverage files; `blobtools map` maps reads to generate mappings; `blobtools view` queries the database with filters; `blobtools plot` generates blobplots showing GC content vs coverage colored by taxonomy.
- **Key Behaviors**: Sequences are binned by taxonomic assignment at user-specified rank (species, genus, family). Filtering by taxonomy, coverage, or length produces new FASTA files. Blobplots show coverage on X-axis, GC content on Y-axis, with point size proportional to sequence length.

## Pitfalls

- **Running without a valid taxdump**: Omitting the `--taxdump` path when creating a database or when taxonomic hits are needed causes assignments to fail and the blobplot to lack taxonomy colours. The database will exist but all sequences remain unclassified.
- **Mismatched or unsorted BAM files**: Using BAM files not sorted by reference name, or not coordinate-sorted, will cause `blobtools map` to fail with mapping errors. Always ensure BAM files are properly sorted and indexed with `samtools sort` and `samtools index`.
- **Conflicting output filenames**: Reusing the same output database filename across multiple runs without overwriting will keep old data. Use `-- известный` or specify a fresh filename to avoid stale results mixing with new analyses.
- **Specifying the wrong taxonomic rank**: Using a rank that does not exist in the assigned taxonomy (e.g., "superkingdom" when assignments are at genus level) results in empty output or all sequences grouped as "uncaught".
- **Insufficient memory for large assemblies**: Processing hundreds of thousands of contigs with deep read coverage can exhaust RAM, causing crashes. Process in chunks or increase available memory.

## Examples

### Create a new blobtools database from an assembly with taxonomic hits
**Args:** `create --fasta assembly.fa --hits hits.csv --taxdump taxdump/ --db blob_db.json`
**Explanation:** Initializes a blob database using an assembly, a CSV of BLAST/DIAMOND hits to a reference database, and the NCBI taxdump. This creates the base database for all subsequent viewing and plotting.

### Map Illumina reads to an assembly and add to database
**Args:** `map --fasta assembly.fa --bam reads.bam --db blob_db.json`
**Explanation:** Maps reads from a BAM file to the assembly using the embedded mapper (usually BWA) and stores coverage data in the database for blobplot generation.

### View the database and list all taxonomic assignments at genus rank
**Args:** `view --db blob_db.json --rank genus --hitsfile`
**Explanation:** Queries the blob database and outputs a table showing each sequence with its assigned genus. Useful for quick taxonomic overview before plotting.

### Generate a blobplot showing coverage vs GC content, coloured by phylum
**Args:** `plot --db blob_db.json -- phylum --plotfile blobplot.png --prefix out_`
**Explanation:** Creates a PNG blobplot with coverage on the X-axis and GC content on the Y-axis, points colored by assigned phylum. The prefix modifies output filenames.

### Filter the database to keep only bacterial sequences at class rank
**Args:** `view --db blob_db.json --rank class --taxlist cellular organisms,Bacteria --fastaout filtered.fa`
**Explanation:** Filters sequences by taxonomy (keeping Bacteria) at the class rank and writes matching sequences to a new FASTA file. Enables extraction of specific taxonomic subsets.

### Generate a blobplot with custom axis ranges and point size scaling
**Args:** `plot --db blob_db.json --phylum --plotfile custom_blobplot.png --xmax 50 --ymin 0.3 --ymax 0.7 --size 500`
**Explanation:** Creates a blobplot with coverage capped at 50, GC content range 0.3-0.7, and figure size 500px. Allows focusing on specific regions of the plot.

### Export taxonomy summary counts at family rank to CSV
**Args:** `view --db blob_db.json --rank family --hitlist > family_counts.csv`
**Explanation:** Outputs a CSV table summarizing the number of sequences and total bases assigned to each family. Useful for downstream statistical analysis.

### Filter sequences by minimum coverage and write new assembly
**Args:** `view --db blob_db.json --cov 5 --length 1000 --fastaout high_cov.fa`
**Explanation:** Filters the database to keep sequences with coverage >=5 and length >=1000bp, then writes the filtered sequences to a FASTA file. Useful for removing low-coverage contigs.

### Add a second coverage library to existing database
**Args:** `map --add --bam second_run.bam --db blob_db.json`
**Explanation:** Adds additional read mapping data from a second BAM file to an existing blob database. Enables comparison of multiple datasets on the same blobplot.

### Generate a PDF blobplot with transparent points for publication
**Args:** `plot --db blob_db.json --phylum --plotfile publication.pdf --format pdf --alpha 0.6`
**Explanation:** Creates a vector PDF blobplot with 60% transparency, suitable for direct inclusion in manuscripts. Vector format ensures scalability.