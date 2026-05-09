---
name: coidb
category: Database Retrieval
description: Command-line tool for querying and retrieving data from biological databases. Supports search by accession, gene name, taxonomy, or sequence identifier. Outputs results in multiple formats including JSON, CSV, and tabular text.
tags: [bioinformatics, database, query, retrieval, genomics, reference-data]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/coidb
---

## Concepts

- **Database selection and scope**: coidb operates against configurable database targets (e.g., geneDb, proteinDb, taxonomyDb). The tool automatically resolves database aliases, but explicit target specification via `--db` ensures accurate query routing and prevents ambiguous matches across multiple database namespaces.

- **Output format flexibility**: Results can be rendered in JSON (`-o json`), CSV (`-o csv`), or plain text table (`-o table`). JSON output includes full metadata and is suitable for downstream scripting; CSV enables direct import into spreadsheet applications; table format is optimized for human readability with column alignment and optional headers.

- **Query resolution precedence**: When given an identifier, coidb resolves it in a specific order: exact accession match → gene symbol alias → protein name → taxonomy ID. This allows both precise queries (exact accession) and exploratory queries (common gene names), but users expecting exact matches must ensure query specificity to avoid unintended alias resolution.

- **Batch query processing**: Multiple queries can be passed via file input (`-i query.txt`, one entry per line) or stdin pipe, enabling bulk retrieval. The tool processes queries sequentially and reports failures individually, allowing partial results to succeed even when some queries fail due to missing entries.

## Pitfalls

- **Misaligned databases produce empty results**: Querying an identifier that exists in proteinDb against geneDb (or vice versa) returns no error—only an empty result set. Users assume the query failed entirely when the actual issue is database mismatch. Always verify which database contains your target entries before running queries.

- **Version drift between local and remote databases**: coidb caches database snapshots locally. Stale caches cause outdated or missing results, especially for rapidly changing databases. Using `--refresh` or `--update-cache` forces synchronization with the remote source, but some environments have restricted network access preventing this.

- **Unquoted special characters in shell queries**: Queries containing `*`, `?`, or `[` are interpreted by the shell as glob patterns, causing unexpected results or "no match" errors. Always quote queries: `"BRCA1*"` not `BRCA1*`.

- **Output format mismatch with downstream tools**: Specifying `-o json` but piping output to a tool expecting CSV causes parsing failures. Downstream scripts expecting specific fields (e.g., `accession`, `sequence`) may fail if output format lacks those fields—always verify compatibility between coidb output format and subsequent processing steps.

## Examples

### Query a single gene by exact accession identifier
**Args:** `--db geneDb --query NM_001126112`
**Explanation:** This queries the gene database for the specific RefSeq mRNA identifier NM_001126112, returning all associated metadata including gene symbol, chromosome location, and functional annotation.

### Retrieve protein sequences in FASTA format
**Args:** `--db proteinDb --query P05362 --format fasta`
**Explanation:** Requests the protein entry P05362 (the human p53 protein) and returns the full amino acid sequence in standard FASTA format suitable for alignment or motif analysis tools.

### Search taxonomy database by common organism name
**Args:** `--db taxonomyDb --query "Homo sapiens" --format json`
**Explanation:** Performs an alias lookup for "Homo sapiens" in the taxonomy database, returning the taxonomic ID (9606), lineage, and rank information in JSON structure for programmatic parsing.

### Batch query multiple accessions from a file
**Args:** `--db geneDb --input queries.txt --output batch_results.csv --format csv`
**Explanation:** Reads a list of gene accessions from queries.txt (one per line), queries each in geneDb, and writes all results to batch_results.csv in CSV format for downstream analysis in R or Python.

### Force cache refresh before critical queries
**Args:** `--db proteinDb --query Q9Y6X3 --refresh --format table`
**Explanation:** Forces synchronization of the local protein database cache before retrieving entry Q9Y6X3, ensuring the result reflects the current remote database state rather than potentially stale cached data.

### Export results with column headers to stdout
**Args:** `--db taxonomyDb --query 9606 --output table --headers`
**Explanation:** Queries taxonomy entry 9606 (human) and outputs formatted table results with column headers to stdout, suitable for inspection without writing to a file.

### Limit and paginate large query results
**Args:** `--db geneDb --query "TP53*" --limit 50 --offset 0 --format json`
**Explanation:** Uses wildcard matching to find all gene entries starting with "TP53", returns the first 50 results in JSON format, and enables pagination through large result sets via offset parameter.

### Suppress verbose logging for scripted pipelines
**Args:** `--db proteinDb --query P04637 --quiet --output json`
**Explanation:** Queries the p53 protein entry while suppressing stdout progress messages and warnings, outputting only the JSON result data—ideal for embedding coidb in automated pipelines where only the result matters.