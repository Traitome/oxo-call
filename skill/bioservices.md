---
name: bioservices
category: bioinformatics-web-services
description: A Python library providing programmatic access to bioinformatics web services including UniProt, KEGG, Reactome, BioGRID, and others. Enables search, retrieval, and analysis of biological data through a unified Python API.
tags:
  - protein-database
  - pathway-analysis
  - web-services
  - KEGG
  - UniProt
  - Reactome
  - BioGRID
  - HMMER
  - biopython-wrappers
author: AI-generated
source_url: https://bioservices.readthedocs.io
---

## Concepts

- **Service object instantiation and API endpoints**: Each bioservices module (UniProt, KEGG, Reactome, BioGRID, etc.) is a Python class. Instantiate with the default constructor `ServiceName()` to connect to the production (live) web service endpoint. Pass `dev=True` to connect to the development/staging endpoint instead, which is useful for testing queries before running them against production data.

- **Search and retrieval methods**: Services provide typed methods for specific operations—for example, `UniProt().search()` returns a list of accession identifiers matching a text query, while `UniProt().getJson()` returns full record metadata as a Python dictionary for a given accession. KEGG provides `list()` for browsing databases, `get()` for fetching records, and `find()` for keyword searches. Reactome provides `query()` for searching entities and `get_step()` for reaction details. Understanding which method to call for each service is essential because calling the wrong method silently returns an empty or incorrect result.

- **HTTP lazy-loading and caching**: Bioservices uses lazy-loading: the HTTP connection to the web service is not established until the first method that makes a network request is called. This means a service object can be created without immediate network activity. Results are cached in memory by default within the session, reducing redundant API calls and improving performance on repeated queries against the same identifiers.

- **Data output formats**: Retrieval methods return different types depending on the service and method—`UniProt().search()` returns a list of identifier strings, `UniProt().getJson()` returns a dictionary with nested metadata, `KEGG().get()` returns a string in KEGG's native flat-file format, and `Reactome().query()` returns a JSON-compatible list of dictionaries. Always inspect the actual Python return type with `type()` or a sample call before writing parsing logic, because downstream code that assumes the wrong type will raise `AttributeError` or `TypeError`.

- **Web service rate limits and the dev parameter**: Production web services enforce rate limits and may block or throttle IP addresses that send excessive requests. Using `dev=True` in production environments wastes quota on the development endpoint and may bypass intended stability controls. Always use the production endpoint (`dev=False`, the default) in production pipelines, and reserve `dev=True` only for testing new queries.

## Pitfalls

- **Assuming the wrong return type**: Calling `UniProt().search()` and treating the result as a dictionary, or calling `KEGG().get()` and trying to index it with a key, causes `AttributeError` or `TypeError`. Always verify the actual return type by running a small test query and printing `type(result)` before writing parsing or aggregation code.

- **Passing lists where a single identifier string is expected**: Methods like `UniProt().getJson()` expect a single string identifier such as `"P053屠"` or `"CDC42_HUMAN"`. Passing a list of identifiers like `["P053屠", "CDC42_HUMAN"]` causes a type error or malformed API request. To look up multiple identifiers, loop over the list or use the dedicated batch method if the service provides one.

- **Not handling None or empty results**: Search methods return an empty list `[]` or `None` when no matching records are found. Code that iterates over results without checking `if results is None or len(results) == 0` first will raise `IndexError` on an empty list or silently skip all records without warning. Always guard search operations with explicit emptiness checks.

- **Using dev=True in production pipelines**: Production scripts that instantiate services with `dev=True` consume quota on the development endpoint, may encounter different data or behaviour compared to production, and will not benefit from the full stability and rate-limit controls of the live service. This results in inconsistent data and potential pipeline failures in production environments.

- **Overlooking HTTP timeout behaviour**: Bioservices uses the `requests` library internally with default timeouts. Long-running queries or large batch requests may exceed the timeout threshold and raise a `requests.exceptions.Timeout` exception. When processing large datasets, implement explicit retry logic and timeout handling rather than assuming all requests will complete successfully.

- **Assuming persistent state across service instances**: Each `Service()` instantiation creates a fresh connection with an empty cache. Queries executed on one service instance are not cached on another instance, even in the same script. Running the same query twice with two separate instances results in two HTTP requests, wasting quota and slowing execution.

## Examples

### Search UniProt for a human kinase by gene name
**Args:** `UniProt().search("CDC42_HUMAN")`
**Explanation:** The `search()` method sends a text query to the UniProt web service and returns a list of matching accession identifiers; here it resolves the gene-centric name to one or more protein accession strings.

### Retrieve full UniProt record metadata as a dictionary
**Args:** `UniProt().getJson("P60981")`
**Explanation:** `getJson()` fetches the complete metadata record for the specified UniProt accession and returns it as a nested Python dictionary, which can be indexed for fields like "sequence" or "organism".

### List all KEGG pathways for E. coli
**Args:** `KEGG().list("pathway:eco")`
**Explanation:** The `list()` method requests a directory listing of all pathway entries in the KEGG database for the specified organism code ("eco" for E. coli) and returns the result as a formatted string.

### Fetch a KEGG compound record by its identifier
**Args:** `KEGG().get("cpd:C00022")`
**Explanation:** `get()` retrieves the full record for a given KEGG compound identifier and returns it as a KEGG flat-file formatted string containing the compound's name, formula, and other chemical properties.

### Query Reactome for pathways involving a human protein
**Args:** `Reactome().query("TP53")`
**Explanation:** `query()` sends the supplied gene or protein name to the Reactome web service and returns a JSON-compatible list of dictionaries describing matching pathway entities, each containing fields such as "name" and "id".

### Get a specific reaction step from Reactome
**Args:** `Reactome().get_step("R-HSA-1402")`
**Explanation:** `get_step()` fetches the detailed record for a single Reactome reaction step identified by its stable identifier and returns the full step metadata as a Python dictionary, which includes substrate and product information.

### Query BioGRID for protein interactions by gene name
**Args:** `BioGRID().query("BRCA1")`
**Explanation:** `query()` submits a search to the BioGRID interaction database using the gene name and returns a list of interaction records as dictionaries, each describing a protein-protein interaction partner and the associated publication.