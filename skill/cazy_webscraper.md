---
name: cazy_webscraper
category: protein-database-retrieval
description: A command-line tool for automated retrieval of protein sequences and annotations from the CAZy (Carbohydrate-Active enZymes) database. Supports querying by CAZyme family, taxonomy, and EC number, outputting results in FASTA, JSON, or tabular formats.
tags:
  - CAZy
  - carbohydrate-active enzymes
  - protein sequence retrieval
  - webscraping
  - bioinformatics
  - enzyme annotation
  - FASTA
author: AI-generated
source_url: https://github.com/HobnobMancer/cazy_webscraper
---

## Concepts

- CAZy classifies enzymes into families based on amino acid sequence, structural, and mechanistic properties. Each family has a prefix (GH for Glycoside Hydrolases, GT for Glycosyl Transferases, PL for Polysaccharide Lyases, CE for Carbohydrate Esterases, AA for Auxiliary Activities, CBM for Carbohydrate-Binding Modules) followed by a number, e.g., GH1, GT2, CBM13. Passing an invalid or non-existent CAZyme family identifier results in an empty output and no error message, so users should verify family names on the official CAZy website before querying.
- The tool fetches data directly from the CAZy REST API (`https://api.cazy.org/`) and the web interface. Requests are rate-limited by default to avoid server overload, and the `--nodelay` flag is available to override this, but using it carelessly risks IP-level bans from the CAZy server, causing all subsequent queries to fail until the ban expires.
- Output formats are determined by `--format` (or `--f`, `--json`, `--csv`, `--tsv`): FASTA for sequences only, JSON for structured metadata (taxonomy, EC numbers, source organism), and tabular formats for bulk annotation downloads. When using `--fasta`, only the sequence data is written; annotation fields must be requested separately if needed.
- Filters for taxonomy (`--taxonomy`), EC number (`--ec`), and CAZyme family (`--families`) are applied as inclusive AND logic by default, meaning a protein must match ALL specified filters to be included. Using `--glyco` or other conditional flags changes this behavior to OR logic, which can produce unexpectedly large result sets if multiple broad categories are specified simultaneously.

## Pitfalls

- Specifying `--all` or `--kingdom` without a `--genera` or `--families` filter fetches the entire CAZy database, which can produce result sets of hundreds of thousands of entries, exhausting disk space and causing the process to hang indefinitely. Always scope queries with at least one filter unless the full database is explicitly required.
- Using `--json` alongside `--fasta` in a single command causes a parsing conflict because JSON and FASTA are mutually exclusive output modes; the tool will exit with a format-related error and produce no output. Run separate commands for each output format.
- The tool requires an API key (set via `--api-cazy` or the `CAZY_API_KEY` environment variable) for accessing the REST API beyond small queries. Running without an API key on a network that blocks requests to `api.cazy.org` silently falls back to web scraping, which is significantly slower and may return partial or stale data compared to API responses.
- CAZy family names are case-sensitive: `gh1` and `GH1` are treated as different queries. Passing lowercase family names results in empty results with no warning, making users think the family has no entries when it simply was not recognized.
- Timeout defaults are conservative. On slow or congested networks, queries may time out silently before completing, resulting in truncated output files that appear valid but contain only a subset of the expected sequences.

## Examples

### Retrieve all GH1 family protein sequences in FASTA format
**Args:** `--families GH1 --format fasta -o gh1_sequences.fasta`
**Explanation:** The `--families GH1` filter selects the Glycoside Hydrolase family 1, and `--format fasta` outputs only sequence data to the specified file.

### Retrieve proteins annotated with EC 3.2.1.4 from the PL8 family as JSON
**Args:** `--families PL8 --ec 3.2.1.4 --json -o pl8_ec3.2.1.4.json`
**Explanation:** Combining `--families PL8` and `--ec 3.2.1.4` applies an AND filter, returning only PL8 enzymes with the specified EC number, formatted as structured JSON metadata.

### Retrieve GT2 family proteins from genus Streptomyces, output as CSV
**Args:** `--families GT2 --genera Streptomyces --kingdom Bacteria --format csv -o streptomyces_gt2.csv`
**Explanation:** The `--kingdom Bacteria` combined with `--genera Streptomyces` restricts taxonomy to bacterial members of GT2, outputting annotation fields to CSV for downstream spreadsheet analysis.

### Retrieve all Auxiliary Activity family proteins with sequences and taxonomy, exclude obsolete entries
**Args:** `--families AA1,AA2,AA3 --fasta --kingdom Eukaryota --exclude_obsolete -o aa_eukaryotes.fasta`
**Explanation:** The comma-separated list `AA1,AA2,AA3` applies OR logic across multiple families, `--kingdom Eukaryota` filters to eukaryotic organisms, and `--exclude_obsolete` removes entries flagged as outdated in CAZy.

### Retrieve CAZy proteins by direct accession list from a text file
**Args:** `--accessions accessions.txt --format json --api-cazy "$CAZY_API_KEY" -o accession_results.json`
**Explanation:** The `--accessions` flag reads CAZy identifiers line-by-line from the specified file, `--format json` outputs full metadata, and the API key is supplied from an environment variable for authenticated REST API access.

### Retrieve CBM13 family proteins from fungi, limit to 500 results with verbose logging
**Args:** `--families CBM13 --kingdom Fungi --limit 500 --verbose -o cbm13_fungi.json`
**Explanation:** The `--kingdom Fungi` filter selects only fungal entries, `--limit 500` caps the result set at 500 proteins, and `--verbose` enables detailed logging to stderr for monitoring progress during long queries.

### Retrieve all CE10 family proteins with associated EC numbers, output as TSV
**Args:** `--families CE10 --ec --format tsv -o ce10_with_ec.tsv`
**Explanation:** The `--families CE10` selects carbohydrate esterase family 10, `--ec` annotates results with EC number data where available, and `--format tsv` writes a tab-separated file compatible with command-line text tools.