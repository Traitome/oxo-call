---
name: alphafetcher
category: protein-structure-retrieval
description: A command-line tool for fetching AlphaFold predicted protein structures and associated metadata from the AlphaFold DB and compatible sources. Supports batch retrieval, multiple output formats, and provenance tracking.
tags: [alphafold, protein-structure, database-query, uniprot, pdb, mmcif, bioinformatics, structure-prediction]
author: AI-generated
source_url: https://github.com/alphafetcher/alphafetcher
---

## Concepts

- **AlphaFold DB Integration**: alphafetcher queries the AlphaFold Protein Structure Database using UniProt accession numbers as primary identifiers. Each prediction is linked to a specific UniProt entry, enabling accurate retrieval of predicted 3D protein models in PDB or mmCIF format.

- **Output Format Modes**: The tool supports structured metadata export (JSON), raw structure file download (PDB/mmCIF), and combined summary reports. Metadata includes pLDDT confidence scores, per-residue quality metrics, model version, and prediction date.

- **Batch Processing with Resume Support**: When processing multiple accessions from an input file, alphafetcher tracks completed entries in a manifest file. Interrupted batch jobs can be resumed without re-downloading successfully fetched structures, avoiding redundant network requests and saving bandwidth.

- **Checksum Verification**: Downloaded structure files are verified against MD5/SHA256 checksums reported by the AlphaFold DB. Corrupted or truncated files are automatically re-fetched, ensuring data integrity for downstream structural analysis workflows.

## Pitfalls

- **Using Gene Names Instead of UniProt Accessions**: AlphaFold DB uses UniProt accession numbers as primary keys. Passing gene names (e.g., "BRCA1") directly will fail with a not-found error. Always convert gene names to UniProt accessions beforehand using UniProt's ID mapping service.

- **Overwriting Existing Structure Files**: By default, alphafetcher overwrites existing files with the same name. In batch mode, this can silently discard structures fetched in a previous partial run, leading to missing data in downstream analyses. Always use the `--manifest` flag to enable safe resume behavior.

- **Ignoring AlphaFold Version Mismatches**: Different AlphaFold DB releases use different model versions (e.g., AlphaFold2 vs AlphaFold-Multimer). Structures fetched at different times may use incompatible model parameters. Not specifying `--version` can result in inconsistent datasets when re-running analyses months later.

- **Insufficient Disk Space for Large Batch Jobs**: A single human proteome-scale batch job can easily exceed 50 GB of storage. Running alphafetcher without checking available disk space will cause partial downloads and corrupt manifest files, requiring a full restart.

- **Rate Limiting from Repeated Batch Requests**: The AlphaFold DB enforces request rate limits. Submitting large batch jobs without throttling triggers HTTP 429 errors, temporary IP bans, and failed retrievals. Always configure `--delay` between requests for jobs exceeding 50 accessions.

## Examples

### Fetch a single AlphaFold predicted structure by UniProt accession
**Args:** `fetch --accession P05067 --output-dir ./structures`
**Explanation:** Downloads the AlphaFold predicted structure for human amyloid-beta precursor protein (UniProt P05067) in mmCIF format to the specified directory, using the default AlphaFold DB version.

### Export metadata as JSON for quality assessment
**Args:** `fetch --accession Q9Y6K9 --format json --output nfkappa_metadata.json`
**Explanation:** Retrieves the complete metadata record for NF-kappa-B inhibitor alpha (UniProt Q9Y6K9), including pLDDT scores and residue-level confidence values, saved as a JSON file for downstream scripting or reporting.

### Batch fetch multiple structures from a UniProt accession list file
**Args:** `batch --input accessions.txt --output-dir ./batch_structures --manifest batch_manifest.tsv`
**Explanation:** Processes all UniProt accessions listed in the input file, downloads each structure, and records progress in a tab-delimited manifest file enabling safe resume if the job is interrupted.

### Fetch a specific AlphaFold model version for dataset reproducibility
**Args:** `fetch --accession P00558 --version 4 --output-dir ./af_version4`
**Explanation:** Explicitly requests the AlphaFold v4 predicted structure for phosphoglycerate dehydrogenase (UniProt P00558), ensuring version consistency for reproducible research or comparative studies across releases.

### Check cache before downloading to avoid redundant network requests
**Args:** `fetch --accession Q9H0H5 --cache-dir ./cache --output cached_structure.cif`
**Explanation:** First checks the local cache directory for the existing structure of interferon regulatory factor 2 (UniProt Q9H0H5); if found, copies to the output path without querying the AlphaFold DB, reducing bandwidth usage and improving speed.

### Resume an interrupted batch job using the manifest file
**Args:** `batch --manifest batch_manifest.tsv --resume --output-dir ./batch_structures`
**Explanation:** Reads the manifest file tracking previous progress and resumes downloading only the accessions marked as incomplete or failed, skipping already-successful entries to complete the batch job efficiently.

### Fetch structure with explicit checksum verification enabled
**Args:** `fetch --accession O75369 --verify-checksum --output-dir ./verified_structures`
**Explanation:** Downloads the predicted structure for filamin B (UniProt O75369) and automatically verifies the MD5 checksum against the AlphaFold DB record, re-fetching if mismatch is detected to ensure data integrity.