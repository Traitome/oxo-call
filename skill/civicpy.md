---
name: civicpy
category: variant_annotation
description: A Python library and CLI tool for accessing the CIViC (Clinical Interpretation of Variants in Cancer) database, providing clinical variant annotations, evidence scores, molecular profiles, and assertion data for cancer-related variants.
tags: bioinformatics, variant_annotation, clinical, cancer, api, database, evidence_score, molecular_profiles
author: AI-generated
source_url: https://github.com/griffithlab/civicpy
---

## Concepts

- **API-Based Data Model**: civicpy interacts with the CIViC database through a RESTful API, fetching data as JSON objects containing gene info, variant details, clinical evidence, molecular profiles, and assertions with associated evidence scores.
- **Gene-Centric Queries**: The tool organizes data around gene symbols (e.g., EGFR, BRAF), allowing users to retrieve all variants, evidence, and assertions associated with a specific gene.
- **Evidence Score System**: CIViC provides an evidence score for each variant based on clinical evidence types (diagnostic, prognostic, predictive, predisposition), with higher scores indicating stronger clinical significance.
- **CLI and Library Modes**: civicpy can be used both as a Python library for programmatic access and as a CLI tool for ad-hoc queries, with output options for JSON, CSV, and summary formats.
- **Molecular Profile Queries**: The tool supports querying molecular profiles which describe variant combinations and their associated clinical interpretations, useful for complex cancer genotypes.

## Pitfalls

- **Stale Cached Data**: Using cached or locally stored CIViC data without refreshing may lead to outdated clinical interpretations; the database is regularly updated with new evidence.
- **Network Dependency**: All API queries require internet access; offline usage will fail unless data has been explicitly downloaded and saved beforehand.
- **Rate Limiting**: Excessive API requests without appropriate throttling may result in temporary access restrictions; implement delays between bulk queries.
- **Variant Name Mismatches**: CIViC uses specific variant nomenclature conventions; using HGVS names that don't match CIViC's naming format will return no results or incorrect variants.
- **Incomplete Variant Annotations**: Not all variants in CIViC have complete clinical evidence; querying a variant with no submissions may return minimal information despite the variant existing in the database.

## Examples

### Fetch all variants for a specific gene
**Args:** `query gene -g EGFR`
**Explanation:** Retrieves all variants associated with the EGFR gene from the CIViC database, including their variant names, variant types, and current evidence scores.

### Get detailed evidence for a specific variant
**Args:** `query variant -g BRAF -v V600E`
**Explanation:** Fetches detailed clinical evidence entries for the BRAF V600E variant, including evidence type, clinical significance, and associated publications.

### Export variants as JSON for programmatic use
**Args:** `query gene -g TP53 -o json`
**Explanation:** Exports all TP53 variants in JSON format, suitable for downstream programmatic analysis or integration with other bioinformatics pipelines.

### Filter evidence by clinical significance
**Args:** `query evidence -g KRAS -s "Sensitivity/Response"`
**Explanation:** Filters KRAS-related evidence entries to only those marked with Sensitivity/Response significance, useful for understanding drug response associations.

### Query molecular profiles containing a specific variant
**Args:** `query molecular-profile -g EGFR -v "EGFRvIII"`
**Explanation:** Retrieves molecular profiles that include the EGFRvIII deletion mutation, showing which variant combinations have associated clinical interpretations.

### Get assertions with evidence scores above a threshold
**Args:** `query assertion -g ALK -m 5`
**Explanation:** Returns ALK assertions with evidence scores of 5 or higher, highlighting variants with strong clinical support for interpretation.

### List all evidence supporting a genes diagnostic significance
**Args:** `query evidence -g BRCA1 -t diagnostic`
**Explanation:** Retrieves all BRCA1 evidence entries tagged with diagnostic significance, useful for understanding variant utility in disease diagnosis.