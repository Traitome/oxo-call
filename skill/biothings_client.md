---
name: biothings_client
category: bioinformatics-api-client
description: A Python client library for querying BioThings APIs including MyGene.info (gene annotations) and MyVariant.info (variant annotations). Provides synchronous and asynchronous methods for single and batch queries, supporting fields filtering, species filtering, and paginated results.
tags: [api-client, gene-annotation, variant-annotation, rest-api, batch-query, bioinformatics]
author: AI-generated
source_url: https://github.com/biothings/biothings_client.py
---

## Concepts

- **Client Initialization and API Endpoints**: The `biothings_client.BiothingsClient` class connects to BioThings APIs (default: MyGene.info and MyVariant.info). Initialize with `BiothingsClient()` for the default gene service or specify `BiothingsClient("variant")` for variant queries. The client handles HTTP requests, JSON parsing, and response formatting internally, returning results as Python dictionaries or lists.

- **Query vs. Get Methods**: Use `query()` for searching indexed fields (returns list of matches), `getgene()` or `getvariant()` for fetching by specific ID (returns single record or list), and `querymany()` for batch queries with size limit handling. The `query()` method searches across indexed fields like `symbol`, `alias`, `taxid`, while `getgene()` requires a specific ID type (e.g., Entrez Gene ID, Ensembl Gene ID).

- **Fields Filtering and Projection**: The `fields` parameter specifies which annotation fields to return, and `species` filters results by taxonomy ID. Fields use dot notation for nested access (e.g., `go.BP`). Without field filtering, the API returns all available annotations which increases response size significantly, so always specify the minimum required fields for production queries.

- **Batch Size Limits and Paginated Results**: `querymany()` processes inputs in chunks (default 1000 for genes, 1000 for variants) to respect API limits. For large batches, enable `asynchronous=True` for parallel async requests or use `step=500` to reduce chunk size. The returned list aligns with input order, with `missing='skip'` to omit unfound entries or `missing='keep'` to include them with null values.

- **Rate Limiting and Async Queries**: Synchronous batch queries are throttled automatically, but large workloads benefit from `asyncio=False` parameter or the separate `BiothingsAsyncClient` class for concurrent requests. When querying thousands of IDs, consider splitting into multiple smaller batches with delays to avoid network timeouts and server-side rate limiting errors.

## Pitfalls

- **Assuming All Input IDs Are Found**: `querymany()` returns a record for every input even when the ID does not exist in the database (returned with status "error"). Forgetting to check `out['notfound']` after each query or silently propagating null values causes KeyError exceptions downstream when accessing missing keys like `symbol` or `exons`.

- **Specifying Wrong Field Names**: Field names are case-sensitive and must match the exact API schema (e.g., `uniprot.Swiss-Prot` not `uniprot.swiss-prot`). Using invalid field names in the `fields` parameter produces no error but returns empty field values, making it difficult to debug silently failing queries without comparing against known-good field specifications.

- **Forgetting Species Filtering for Ortholog Genes**: Without `species` parameter, queries search across all species and return the first match which may be unexpected (e.g., human `TP53` vs mouse `Tp53`). This is particularly problematic for batch queries where mixed-species results cause downstream analysis errors when fields like `chromosome` or `symbol` contain inconsistent values.

- **Not Handling Timeout for Large Batch Queries**: Synchronous `querymany()` requests timeout after ~60 seconds by default. Large batches of 10,000+ IDs without proper chunking (`step`) or without async mode can fail with `TimeoutError`, leaving partial results that are difficult to merge and reprocess reliably.

- **Confusing Query Scope**: Using `query(q="BRCA1")` searches all indexed fields including synonyms and descriptions, returning multiple matches. Using `getgene()` requires an exact ID (e.g., `"672"` for Entrez Gene ID). Mixing these two approaches leads to unexpected result counts and incorrect gene associations in downstream analysis.

## Examples

### Query a Single Gene by Symbol

**Args:** `geneclient.query(q="TP53", species="human")`
**Explanation:** Uses the `query()` method to search for the TP53 gene in human species, returning gene records with all default fields.

### Fetch Gene Details by Entrez Gene ID with Specific Fields

**Args:** `geneclient.getgene("672", fields=["symbol", "name", "summary", "go"])`
**Explanation:** Retrieves gene record by Entrez Gene ID with only the specified fields, reducing response payload size.

### Batch Query Multiple Variants by RSID

**Args:** `variantclient.querymany(["rs4964", "rs113488022", "rs28934574"], scopes="dbsnp.rsid", fields=["dbsnp.rsid", "cadd.gene"])`
**Explanation:** Batch queries variant database using the `dbsnp.rsid` scope to map RSIDs to CADD gene annotations.

### Asynchronous Batch Gene Query with Species Filter

**Args:** `geneclient.querymany(gene_ids, scopes="entrezgene", species=9606, fields=["symbol", "alias", "type_of_gene"], asynchrony=True)`
**Explanation:** Performs parallel async requests for a large list of Entrez Gene IDs in human (taxid 9606), significantly faster than synchronous batch queries.

### Query Variant with HGVS Notation

**Args:** `variantclient.query(q="chr7:g.55241671G>A", scopes="clinvar.hgvs")`
**Explanation:** Queries variant database using HGVS genomic notation within the clinvar.hgvs indexed field to retrieve variant clinical significance data.