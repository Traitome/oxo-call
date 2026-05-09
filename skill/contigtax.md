---
name: contigtax
category: taxonomy_annotation
description: Assigns taxonomic labels to assembly contigs using marker gene alignment and a reference taxonomy database. Takes FASTA input and outputs per-contig taxonomy assignments in TSV or CSV format.
tags:
  - taxonomy
  - contigs
  - classification
  - metagenomics
  - assembly
  - binning
author: AI-generated
source_url: https://github.com/contigtax/contigtax
---

## Concepts

- **Input Format**: Accepts FASTA or multi-FASTA files containing assembled contigs. Each sequence must have a unique identifier in the header line (e.g., `>NODE_1_length_500_cov_42.5`).
- **Database Model**: Uses a pre-built taxonomy database containing marker gene sequences (e.g., ribosomal proteins, single-copy genes) mapped to taxonomic lineage. The database is created with the companion `contigtax-build` tool using GTDB, NCBI, or custom taxonomy files.
- **Output Modes**: Supports multiple output formats including TSV (default), CSV, and JSON. Output includes columns for contig ID, kingdom, phylum, class, order, family, genus, species, and confidence score.
- **Confidence Scoring**: Assigns a confidence score (0-100) based on alignment quality, sequence coverage, and uniqueness of the match. Low-confidence assignments are flagged with a `*` symbol in default output.

## Pitfalls

- **Database Not Built**: Running `contigtax` without first building or downloading a taxonomy database will cause immediate failure with a "database not found" error. Always ensure the `--db` path points to a valid, accessible database directory.
- **Sequence Identity Too Low**: Setting an overly strict minimum identity threshold (`--min-identity`) can result in zero classifications for divergent or novel sequences, causing downstream steps to fail. For metagenomes with unknown organisms, use 80-90% rather than 95%+.
- **Input File Format Errors**: Non-standard FASTA headers (e.g., missing `>` prefix, whitespace in sequence names) will cause parsing failures partway through the file. Validate input with `seqkit stats` before running full analyses.
- **Memory Limits with Large Assemblies**: Classifying assemblies with >1 million contigs may exhaust available RAM, especially with default database settings. Use the `--batch-size` flag to process in chunks or increase available memory.

## Examples

### Classify contigs from a metagenome assembly
**Args:** `--db /refdb/contigtax-db --input assembly.fa --output taxonomy.tsv`
**Explanation:** This runs the standard classification pipeline using the specified pre-built database and writes per-contig taxonomy assignments to a TSV file for downstream binning or profiling.

### Use a custom minimum confidence threshold
**Args:** `--db /refdb/contigtax-db --input assembly.fa --output high-conf.tsv --min-score 95`
**Explanation:** This filters output to include only high-confidence taxonomy assignments scoring 95 or above, reducing noise in datasets where many contigs are from unknown organisms.

### Output classifications in JSON format
**Args:** `--db /refdb/contigtax-db --input assembly.fa --output taxonomy.json --format json`
**Explanation:** This generates JSON output suitable for integration with scripting pipelines or web services, containing nested taxonomic lineage objects for each contig.

### Process a large assembly in batches
**Args:** `--db /refdb/contigtax-db --input huge_assembly.fa --output taxonomy.tsv --batch-size 50000`
**Explanation:** This processes the assembly in batches of 50,000 contigs to manage memory usage, writing results incrementally to the output file.

### Assign taxonomy only to bacterial sequences
**Args:** `--db /refdb/contigtax-db --input assembly.fa --output bac_tax.tsv --kingdom Bacteria`
**Explanation:** This filters classifications to only return assignments where the top-level kingdom is Bacteria, excluding viral, archaeal, and eukaryotic contigs from the output.