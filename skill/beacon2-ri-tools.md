---
name: beacon2-ri-tools
category: genomics/beacon-api
description: Command-line tool suite for the Beacon v2 Reference Implementation, enabling querying, validation, and processing of genomic variant data through the GA4GH Beacon API framework.
tags: [beacon, genomics, variant-query, api-client, vcf, ga4gh]
author: AI-generated
source_url: https://github.com/ga4gh-beacon/beacon-v2/tree/main/tools
---

## Concepts

- **Modular Tool Architecture**: The beacon2-ri-tools suite comprises multiple companion binaries (`beacon2-ri-query`, `beacon2-ri-validate`, `beacon2-ri-export`, `beacon2-ri-index`) that each handle a specific operation within the Beacon workflow. Choose the appropriate binary based on your task rather than passing all arguments to a single entry point.
- **Beacon API Request-Response Model**: Queries are constructed using genomic coordinates (chromosome, start, end) or variant identifiers (rsID, HGVS notation) and sent as GET or POST requests to a Beacon endpoint. The tool parses JSON responses into structured output formats including JSON, TSV, or VCF depending on the `--response-format` flag.
- **Reference Genome Compatibility**: All genomic positions must reference a supported assembly (GRCh37/hg19 or GRCh38/hg38). Mismatched genome builds between your input data and the target Beacon endpoint will produce silent failures or coordinate-based errors. Use the `--assembly` flag to explicitly declare your reference genome.
- **Authentication and Security**: Some Beacon endpoints require API key authentication via the `--api-key` flag or Bearer token via the `Authorization` header. Public Beacon instances typically require no authentication but may enforce rate limits. Using authenticated endpoints without credentials results in HTTP 401 responses.
- **Batch Processing via stdin**: Multiple queries can be piped to the tool using `--batch-input` to process variant lists efficiently. Input must be in BED-like or Beacon request JSON format. Failing to provide correctly formatted batch input causes parsing errors that abort the entire batch job.

## Pitfalls

- **Mismatched Assembly Version**: Specifying `--assembly hg19` when querying an endpoint indexed on GRCh38 produces coordinate mismatches where the Beacon returns no results for valid variants. Always verify the target endpoint's indexed assembly before querying.

- **Omitting Required Endpoint URL**: Forgetting the `--url` flag when required by the operation causes the tool to default to an undefined or localhost endpoint, resulting in connection refused errors. The endpoint URL is mandatory for all query and validate operations.

- **Incorrect Response Format for Downstream Tools**: Selecting `--response-format vcf` when downstream tools expect JSON produces parsing failures in automated pipelines. Verify your pipeline's expected input format before choosing the output format.

- **Rate Limit Exceeded Without Backoff**: Running batch queries against public Beacon endpoints without implementing delay causes HTTP 429 responses that abort the job. Use the `--delay` flag to introduce throttling between requests in batch mode.

- **Using Obsolete Variant Notation**: Querying with legacy HGVS notation not compliant with Sequence Ontology standards causes the Beacon to reject the request with a 400 validation error. Ensure variant descriptions use the current HGVS specification supported by your endpoint version.

## Examples

### Query a genomic range for variants using chromosome coordinates
**Args:** `beacon2-ri-query --url https://beacon.example.org/v2g/genomicShort --chromosome 7 --start 117171215 --end 117171315 --assembly GRCh38 --response-format json`
**Explanation:** This queries the Beacon for any variants within chromosomal position 117,171,215 to 117,171,315 on chromosome 7 using the GRCh38 assembly, returning results in JSON format for programmatic parsing.

### Search for a specific variant by rsID
**Args:** `beacon2-ri-query --url https://beacon.example.org/v2g/snp --rsid rs699 --assembly GRCh38 --response-format tsv`
**Explanation:** This performs a direct lookup of the single nucleotide variant identified by dbSNP rs699 using its rsID, outputting tabular results suitable for spreadsheet analysis.

### Validate a VCF file against Beacon schema requirements
**Args:** `beacon2-ri-validate --input variants.vcf --schema v2.0 --output validation_report.json`
**Explanation:** This validates a local VCF file for conformance to Beacon v2.0 schema requirements, generating a JSON report detailing any schema violations encountered during the validation process.

### Export Beacon query results to a BED file for downstream genomic analysis
**Args:** `beacon2-ri-export --url https://beacon.example.org/v2g/genomicShort --chromosome 3 --start 12700000 --end 12800000 --assembly GRCh38 --response-format bed`
**Explanation:** This exports all variants overlapping the specified genomic interval to BED format, enabling direct use in genome browsers or overlap analysis tools like BEDTools.

### Batch query multiple variants from a text file with rate limiting
**Args:** `beacon2-ri-query --batch-input variant_list.txt --url https://beacon.example.org/v2g/snp --response-format json --delay 100 --assembly GRCh38`
**Explanation:** This processes a newline-delimited list of variant identifiers with a 100 millisecond delay between requests to avoid rate limiting, suitable for querying large variant collections against a public endpoint.

### Index a genomic dataset for local Beacon deployment
**Args:** `beacon2-ri-index --input dataset.vcf.gz --assembly GRCh38 --reference-name chr1 --name local-cohort-2024 --metadata cohort_metadata.json`
**Explanation:** This creates an indexed representation of a compressed VCF dataset for use with a local Beacon instance, associating it with the specified cohort metadata for discoverability.

### Query using HGVS notation for a structural variant
**Args:** `beacon2-ri-query --url https://beacon.example.org/v2g/structuralShort --hgvs NC_000001.11:g.207436119_207440000del --response-format json --assembly GRCh38`
**Explanation:** This submits a deletion structural variant using HGVS genomic notation to the Beacon structural variant endpoint, which is necessary when coordinate-based queries would miss complex events.