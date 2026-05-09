---
name: apybiomart
category: genomic-database-query
description: A Python CLI tool and library for querying BioMart databases, enabling retrieval of gene annotations, genomic features, orthologs, and protein-domain data from Ensembl and other compliant servers.
tags:
  - biomart
  - ensembl
  - gene-annotation
  - genomic-features
  - orthologs
  - python
  - data-mining
author: AI-generated
source_url: https://github.com/IGC-FAJAS/apybiomart
---

## Concepts

- BioMart organizes data into **datasets** (typically one per organism), each exposing a fixed schema of **attributes** (output columns) and **filters** (query constraints). You must always specify the `--dataset` before selecting any attributes or filters; omitting the dataset results in a query error or empty response.
- Attributes are identified by their **internal BioMart names** (e.g., `ensembl_gene_id`, `hgnc_symbol`, `uniprotswissprot`), not display labels. Attribute names are dataset-specific and differ between organisms, so a valid attribute for *hsapiens_gene_ensembl* will fail for *mmusculus_gene_ensembl*.
- The `apybiomart query` command sends a HTTP request to the BioMart web service, which returns results as **TSV by default**. Large queries can produce very large result sets; use the `--limit` flag to cap rows returned and the `--header` flag to include column names in the output for easier parsing.

## Pitfalls

- Using attribute names from the BioMart web interface (human-readable display names) instead of internal identifiers causes silent column drops in output. Always use `--list-attributes` to confirm the exact internal name before building a query.
- Omitting `--limit` on a broad query (e.g., requesting all genes across all chromosomes for a human dataset) can generate thousands to millions of rows, consuming significant memory and bandwidth, and may trigger BioMart server timeouts.
- Specifying a filter without a matching attribute in the `--attributes` list does not produce an error but yields zero rows because the server has nothing to return for the requested output schema. Ensure every filter you apply corresponds to at least one selected attribute.
- Attempting to query an unavailable or archived BioMart dataset (e.g., an older Ensembl version no longer hosted) produces a network-level HTTP 400 error. Verify the dataset name with `--list-datasets` before querying.
- Mixing filter values from different organisms (e.g., applying an *M. musculus* chromosome name like `14` to the *H. sapiens* dataset) silently returns empty results, as the server cannot match the foreign key.

## Examples

### List all available datasets for the default BioMart server
**Args:** `query --list-datasets`
**Explanation:** Prints a table of dataset names and descriptions available on the configured BioMart host, allowing you to verify the exact dataset identifier before building a real query.

### Retrieve gene IDs and HGNC symbols for human genes on chromosome 14
**Args:** `query --dataset hsapiens_gene_ensembl --attributes ensembl_gene_id,hgnc_symbol --filters chromosome_name=14 --limit 500 --header`
**Explanation:** Combines a dataset, two output attributes, a filter on chromosome, and a row cap to produce a manageable TSV file with column headers for downstream parsing.

### Fetch UniProt identifiers and gene names for mouse genes on chromosome 1
**Args:** `query --dataset mmusculus_gene_ensembl --attributes ensembl_gene_id,uniprotswissprot,mgi_symbol --filters chromosome_name=1 --limit 200 --header`
**Explanation:** Targets a different organism dataset with organism-specific attribute names to retrieve protein and gene identifiers filtered to chromosome 1.

### Export human gene types and descriptions filtered by gene name prefix
**Args:** `query --dataset hsapiens_gene_ensembl --attributes ensembl_gene_id,external_gene_name,gene_type,description --filters external_gene_name_pattern=BRCA* --header`
**Explanation:** Uses a wildcard filter pattern to retrieve genes whose external name starts with "BRCA", returning type and description alongside identifiers.

### Retrieve gene IDs and GO term identifiers for all protein-coding human genes on chromosome X
**Args:** `query --dataset hsapiens_gene_ensembl --attributes ensembl_gene_id,go_id --filters gene_type=protein_coding,chromosome_name=X --limit 1000 --header`
**Explanation:** Chains two filters together to narrow results to protein-coding genes on chromosome X, demonstrating multi-filter logic while limiting output size.

### Query the RNAcentral dataset to obtain URS identifiers and taxonomy IDs
**Args:** `query --dataset rnacentral --attributes urs_id,taxon_id --filters taxon_id=9606 --limit 300 --header`
**Explanation:** Targets a non-Ensembl dataset (RNAcentral) to retrieve non-coding RNA identifiers filtered by NCBI taxonomy ID, showing that apybiomart works with any compliant BioMart installation.