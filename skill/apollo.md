---
name: apollo
category: genome_annotation
description: Apollo is a genome annotation viewer and editor from the GMOD suite, designed for visualizing, browsing, and editing gene models and other genomic features in eukaryotic genomes. It provides a web-based interface for manual annotation refinement and supports collaborative annotation workflows.
tags: [genome-annotation, gene-model, gff3, visualization, eukaryotic-genomes, gmod]
author: AI-generated
source_url: https://gmod.github.io/Apollo/
---

## Concepts

- Apollo reads genomic annotations primarily in GFF3 format, which describes genomic features (genes, mRNAs, exons, CDS) with standardized columns (seqid, source, type, start, end, score, strand, phase). The tool constructs a hierarchical annotation graph from these records, linking parent-child relationships.
- The web-based Apollo client communicates with a backend server (typically running on JBoss/WildFly) that stores annotations in a Chado database schema or as flat files. Users interact via a modern browser interface to create, modify, or validate gene models without direct CLI manipulation.
- Apollo supports export of annotations in multiple formats including GFF3, FASTA (for sequences), and CHADO (SQL database). Batch operations enable bulk creation or modification of features, and the tool maintains audit trails for collaborative annotation changes.

## Pitfalls

- Uploading GFF3 files with malformed column counts or phase values causes silent feature misparsing, resulting in incorrect exon boundaries or CDS frameshifts that may propagate through downstream analysis pipelines.
- Attempting to edit annotations without appropriate user permissions on a shared Apollo server results in "Access Denied" errors; the server administrator must grant appropriate roles (administrator, editor, viewer) before modifications are possible.
- Overlapping gene models on the same strand are not automatically flagged as conflicts, leading to redundant annotations that complicate downstream gene set enrichment analyses or transcriptome assemblies.
- Failing to synchronize local annotation files with the server after making edits creates version divergence; always refresh the view or export the current state before closing a session.
- Using outdated Java versions or unsupported browsers causes interface rendering failures, with buttons or canvas elements appearing non-functional; verify compatibility requirements in the Apollo documentation.

## Examples

### Load and display a GFF3 annotation file in the web interface
**Args:** `-i example.gff3 -d postgres://user:pass@localhost/apollo_db`
**Explanation:** This command initializes a new Apollo session by loading the GFF3 file into the connected database, making annotations visible in the browser-based viewer.

### Display only gene and mRNA features, filtering out other types
**Args:** `--filter=type:gene,mRNA --url=http://genome-server:8080/apollo`
**Explanation:** Using the filter flag restricts the visible annotation tracks to gene and transcript levels, reducing visual clutter when navigating large genomes.

### Export current annotations to FASTA format
**Args:** `--export-format=fasta --output=sequences.fa`
**Explanation:** Exporting to FASTA extracts the nucleotide sequences of all annotated features, useful for creating custom sequence databases or primer design.

### Create a new gene model starting at position 10000 on chromosome 2
**Args:** `--action=create-feature --type=gene --seqid=chr2 --start=10000 --end=15000 --strand=+`
**Explanation:** This CLI operation creates a gene feature with defined coordinates, which can then be refined interactively in the web interface by adding child mRNA and exon features.

### Bulk import annotations from multiple GFF3 files
**Args:** `--batch-import /path/to/gffdir/*.gff3 --organism=Drosophila_melanogaster`
**Explanation:** Batch importing processes all GFF3 files in the directory, assigning them to the specified organism to populate the database with annotations for bulk editing sessions.

### Generate a report of all CDS features longer than 1000bp
**Args:** `--query="SELECT * FROM feature WHERE type='CDS' AND seq_length > 1000" --format=tsv`
**Explanation:** Querying the backend database directly extracts long coding sequences, outputting a tab-delimited file for downstream selection of candidate genes for experimental validation.