---
name: biogridpy
category: Protein-Protein Interaction Database Client
description: Python client library for querying the BioGRID (Biological General Repository for Interaction Data) REST API to retrieve protein-protein interaction (PPI) data for genes and proteins across multiple organisms.
tags: [PPI, protein-protein interaction, bioinformatics, database, API, Biogrid, network-analysis]
author: AI-generated
source_url: https://thebiogrid.org/
---

## Concepts

- **Data Model**: BioGRID stores protein-protein interaction data as pairs of gene/protein identifiers with associated metadata including interaction type (genetic, physical), experimental evidence, publication references, and organism source. Each query returns interaction records with source and target gene identifiers.

- **I/O Formats**: The tool supports multiple output formats including tab-delimited text (default), PSI-MI XML for standardized interaction exchange, and JSON for programmatic parsing. Query results can also be retrieved as gene lists or interaction networks suitable for visualization tools like Cytoscape.

- **Key Behaviors**: Queries are performed against the BioGRID REST API using gene names, systematic IDs (e.g., SGD, FlyBase), or database identifiers (Entrez Gene ID, UniProt). Results can be filtered by organism, interaction type, and evidence method. The tool automatically handles rate limiting and pagination for large result sets.

- **Organism Filtering**: BioGRID supports queries for over 60 organisms including common model organisms (S. cerevisiae, D. melanogaster, C. elegans, M. musculus, H. sapiens). Proper organism taxon ID or name specification is required for targeted queries.

- **Evidence Codes**: Interactions are tagged with experimental evidence codes (e.g., MITRE, PCA, Co-IP) that indicate the method used to detect the interaction. Filtering by evidence increases result specificity and confidence.

## Pitfalls

- **Invalid Organism Names**: Using misspelled or unrecognized organism names (e.g., "human" instead of "Homo sapiens") returns empty results. BioGRID requires exact organism names or NCBI taxon IDs for filtering.

- **Large Queries Without Pagination**: Querying for highly connected hub proteins (e.g., p53 in humans) can return thousands of interactions, causing timeout or memory errors if pagination is not handled. Always use the result limit and offset parameters for large datasets.

- **Rate Limiting Without Handling**: Repeated rapid queries without respecting API rate limits results in temporary IP bans. Implement request throttling or use the tool's built-in rate limiting feature.

- **Inconsistent Gene Identifiers**: Mixing different identifier types (gene names vs. systematic IDs) in the same query leads to missed interactions. Ensure consistent identifier types or use the tool's identifier conversion feature.

- **Ignoring Deprecated Gene Names**: Querying with outdated gene aliases returns no results. BioGRID maintains a mapping but some legacy names may require manual lookup.

## Examples

### Query protein interactions for human TP53 gene
**Args:** `--gene TP53 --organism "Homo sapiens" --format tab`
**Explanation:** Retrieves all protein-protein interactions for the TP53 tumor suppressor gene in humans, returning results in tab-delimited format suitable for piping to other tools.

### Retrieve interactions for yeast CDC42
**Args:** --gene CDC42 --organism "Saccharomyces cerevisiae" --interactions-only
**Explanation:** Fetches interaction data for the CDC42 cell division cycle protein in budding yeast, filtering to include only physically or genetically validated interactions.

### Export human interactions in PSI-MI XML format
**Args:** --gene BRCA1 --organism "Homo sapiens" --format psi-mi --output brca1_interactions.xml
**Explanation:** Exports breast cancer gene BRCA1 interaction network in PSI-MI XML standard format for compatibility with other bioinformatics tools and databases.

### Get genetic interactions only for C. elegans gene
**Args:** --gene lin-39 --organism "Caenorhabditis elegans" --interaction-type genetic
**Explanation:** Filters query results to include only genetic interactions (synthetic lethality, enhancement, suppression) for the nematode lin-39 gene, useful for pathway analysis.

### Paginate through large interaction dataset
**Args:** --gene CDC19 --organism "Saccharomyces cerevisiae" --format json --limit 500 --offset 0
**Explanation:** Retrieves first 500 interactions for yeast CDC19 gene with JSON output, demonstrating pagination approach for handling large results.

### Filter by experimental evidence method
**Args:** --gene ACT1 --organism "Saccharomyces cerevisiae" --evidence "Affinity Capture-Western"
**Explanation:** Restricts results to interactions detected specifically by affinity capture followed by western blot, increasing confidence in physical interaction data.