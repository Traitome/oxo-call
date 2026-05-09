---
name: biom-format
category: Bioinformatics tools
description: Command-line tools for working with BIOM (Biological Observation Matrix) files, a standardized format for representing taxonomic or functional abundance data from metagenomic studies.
tags: [biom, metagenomics, abundance-matrix, otu-table, ecological-data, data-format-conversion]
author: AI-generated
source_url: https://biom-format.org/
---

## Concepts

- **BIOM file format**: A structured format for biological observation matrices representing counts of observations (e.g., OTUs, genera, or gene families) across samples. Files can use JSON, HDF5, or TSV backends, each with different compatibility and tooling support.
- **Observation vs sample axes**: BIOM tables are matrices where rows represent observations (typically taxonomic or functional units) and columns represent samples. Operations like filtering can target either axis depending on the command.
- **Conversion between formats**: The `convert` subcommand translates between BIOM backends (JSON, HDF5, TSV), enabling interoperability with QIIME, Mothur, and other ecological analysis pipelines.
- **Rich metadata**: Each BIOM record stores observation metadata (taxonomy, KO identifiers) and sample metadata (host, environment) embedded in the file, avoiding separate annotation files.

## Pitfalls

- **Using HDF5 format**: HDF5-backed BIOM files cannot be edited or filtered in place—you must convert to JSON, modify, then reconvert back to HDF5. Attempting direct filter operations fails silently.
- **Mismatched header columns**: When converting from TSV to BIOM, the first column must be named `#OTU ID` or `OTU ID`. Using alternative headers like `taxonomy` or `ID` causes parsing errors.
- **Losing metadata during conversion**: Converting from BIOM to TSV discards all nested observation/sample metadata stored in JSON or HDF5 formats—only the abundance matrix transfers.
- **Floating-point precision**: HDF5 format stores counts as floating-point numbers, while JSON uses integers. Converting between them can introduce rounding errors if counts exceed integer precision limits.

## Examples

### Summarize the contents of a BIOM table
**Args:** summarize-table -i table.biom
**Explanation:** Displays row/column counts, sample metadata fields, and observation diversity statistics for the BIOM file.

### Convert a JSON BIOM file to TSV format
**Args:** convert -i table.biom -o table.tsv --to-tsv
**Explanation:** Translates the internal BIOM JSON representation to tab-separated format readable by spreadsheet software or R scripts.

### Convert a TSV OTU table to BIOM JSON format
**Args:** convert -i otu_table.tsv -o otu_table.biom --table-type "OTU table" --header-key taxonomy
**Explanation:** Creates a binary BIOM file from a QIIME-style tab-separated table, specifying that observations represent OTUs and taxonomy metadata exists in that column.

### Validate a BIOM file for structural integrity
**Args:** validate-table -i table.biom
**Explanation:** Checks the BIOM file for format compliance, confirming that matrix dimensions are consistent and required fields are present.

### Filter observations present in fewer than 5 samples
**Args:** filter-table -i table.biom -o filtered.biom --min-count-samples 5
**Explanation:** Removes any observation (row) appearing in fewer than 5 samples, useful for removing rare OTUs before statistical analysis.

### Filter by sample metadata using a specific observation
**Args:** filter-table -i table.biom -o filtered.biom --sample-metadata-id "sample_id:healthy" --observation-metadata-filter "taxonomy:Proteobacteria"
**Explanation:** Retains only samples with metadata matching "healthy" and observations annotated with the taxonomy prefix "Proteobacteria".

### Convert to HDF5 format for QIIME1 compatibility
**Args:** convert -i table.biom -o table_hdf5.biom --hdf5-ftp
**Explanation:** Creates an HDF5-formatted BIOM file specifically configured for QIIME 1.9 compatibility, including necessary FTP metadata fields.