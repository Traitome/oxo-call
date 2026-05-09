---
name: biobasehttptools
category: bioinformatics-databases
description: A suite of HTTP-based client tools for querying and retrieving data from BioBase biological databases. biobasehttptools provides command-line access to BioBase transcription factor binding site data, promoter analysis, and regulatory network information via REST API endpoints.
tags: [http, database-query, transcription-factors, promoters, biobase, api-client]
author: AI-generated
source_url: https://www.biobase.de/products/biobase-international
---

## Concepts

- **Authentication via API Key**: biobasehttptools requires a valid BioBase API key set via the `--api-key` flag or `BIOBASE_API_KEY` environment variable. Keys are issued per user and expire after the configured validity period; failing to provide a valid key results in HTTP 401 Unauthorized errors.

- **Data Output Formats**: Query results are returned in multiple formats controlled by `--format` (available: `json`, `xml`, `tsv`). JSON is the default and recommended format for programmatic parsing; TSV is suitable for direct import into spreadsheet applications.

- **Pagination Behavior**: Large result sets are paginated automatically. Use `--page` to specify the page number and `--page-size` to control results per page (default 50, maximum 500). Omitting pagination flags returns only the first page of results.

- **Rate Limiting**: The BioBase API enforces rate limits of 100 requests per minute for standard accounts and 500 for premium accounts. Exceeding the limit triggers HTTP 429 responses with a `Retry-After` header indicating the wait time in seconds.

- **Result Filtering**: Query results can be filtered at the API level using `--organism`, `--gene-id`, and `--region` flags. Server-side filtering reduces payload size and improves response times compared to client-side filtering.

## Pitfalls

- **Expired or Missing API Key**: Forgetting to set `--api-key` or using an expired key causes all requests to fail with a 401 error. The tool does not prompt for credentials interactively; the key must be provided on the command line or via the environment variable.

- **Exceeding Page Size Limits**: Setting `--page-size` to a value above 500 triggers a 400 Bad Request error. The API rejects excessively large page sizes to prevent server overload; split large queries into multiple pages instead.

- **Unquoted Special Characters in Query Strings**: Characters like `&`, `|`, or `:` in gene names or regions must be properly escaped or quoted when passed to `--gene-id` or `--region`. Unescaped special characters can corrupt the query string and return unexpected results or HTTP 400 errors.

- **Ignoring Rate Limit Headers**: Rapid consecutive queries without respecting rate limits causes temporary IP-based blocking after exceeding the limit. The tool outputs a warning but continues attempting requests; monitor the `Retry-After` header value when you encounter 429 responses.

- **Assuming JSON is Default for All Subcommands**: While JSON is the default output format for most subcommands, some specialized subcommands like `export-sequence` default to FASTA format. Always verify the format with `--help` for the specific subcommand to avoid parsing errors downstream.

## Examples

### Query transcription factor binding sites for a specific gene
**Args:** `query-tfs --gene-id "BRCA1" --organism "Homo sapiens" --format json`
**Explanation:** This queries the BioBase database for all transcription factor binding sites associated with the BRCA1 gene in human, returning results in JSON format for easy parsing.

### Export promoter sequences for multiple organisms in TSV format
**Args:** `export-sequence --organism "Mus musculus,Rattus norvegicus" --format tsv --page-size 200`
**Explanation:** The comma-separated organism list enables querying promoter sequences across two organisms, with TSV format suitable for import into spreadsheets.

### Retrieve regulatory network data with server-side filtering
**Args:** `query-regulatory --region "chr17:43000000-43100000" --organism "Homo sapiens" --format xml`
**Explanation:** Server-side region filtering reduces bandwidth by only fetching binding sites within the specified genomic coordinates on chromosome 17, using XML format for compatibility with XML-based analysis pipelines.

### Paginate through large result sets for a gene list
**Args:** `query-tfs --gene-list "genes.txt" --format json --page 3 --page-size 100`
**Explanation:** When processing a file of gene IDs, the third page of results with 100 entries per page is retrieved. This handles large gene lists by distributing queries across multiple pages.

### Check API rate limit status without running a query
**Args:** `status --api-key "your-api-key-here"`
**Explanation:** The status subcommand checks the current rate limit usage and account validity without consuming a query request, useful for monitoring before running batch operations.