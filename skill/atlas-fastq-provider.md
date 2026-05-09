---
name: atlas-fastq-provider
category: Data Retrieval
description: Command-line tool for downloading FASTQ sequencing reads from the BV-BRC (Bacterial Viral Bioinformatics Resource Center) database. Retrieves raw Illumina sequencing data using sample or accession identifiers.
tags:
  - fastq
  - sequencing
  - data-download
  - bv-brc
  - patric
  - reads
author: AI-generated
source_url: https://www.bv-brc.org
---

## Concepts

- **Input Identification**: The tool uses BV-BRC sample IDs or accession numbers (e.g., SRR, ERR, SRX identifiers) to query the database and locate corresponding FASTQ files. Sample IDs typically map to complete experiments with metadata.
- **Output Format**: Downloaded files are written in standard FASTQ format, preserving quality scores. For paired-end data, two files are produced with `_1` and `_2` suffixes. Single-end data produces a single FASTQ file.
- **Authentication**: BV-BRC requires user authentication. The tool checks for configured credentials (API key or session) before attempting downloads. Unauthenticated requests return access denial errors.
- ** paired-end Detection**: The tool automatically detects whether a sample contains paired-end or single-end reads based on the BV-BRC metadata and downloads accordingly. Paired-end samples always produce two files.

## Pitfalls

- **Invalid Credentials**: Using an expired or invalid BV-BRC API key causes all download attempts to fail with authentication errors. This blocks access to both public and private datasets. Always verify credentials before running batch downloads.
- **Incorrect Accession Format**: Providing incorrectly formatted accession numbers (e.g., using SRA run IDs instead of sample IDs) results in no matching records found. The tool searches specifically by sample ID, not raw SRA run accession.
- **Incomplete Downloads**: Interrupting the download process (network loss, pressing Ctrl+C) leaves partial FASTQ files on disk. These partial files may not be recognized as incomplete and could be mistakenly used downstream.
- **Storage Overflow**: Downloading large datasets to a disk with insufficient space causes partial file writes and data corruption. Check available storage before initiating downloads.

## Examples

### Download FASTQ reads for a specific sample ID
**Args:** `--sample-id SRS123456 --output-dir ./downloads`
**Explanation:** Downloads all FASTQ files associated with the given BV-BRC sample ID into the specified output directory, automatically handling paired-end or single-end format.

### Download using an SRA accession number
**Args:** `--accession SRR1234567 --output-dir ./reads`
**Explanation:** Retrieves FASTQ files using an SRA run accession. The tool converts the run ID to the corresponding sample and downloads the associated read files.

### Provide BV-BRC credentials explicitly
**Args:** --sample-id SRS123456 --output-dir ./data --api-key YOUR_API_KEY
**Explanation:** Passes the API key directly to authenticate with BV-BRC. Useful in automated pipelines where environment variables are not set.

### Download with paired-end read separation
**Args:** --sample-id SRS123456 --output-dir ./paired_reads --paired
**Explanation:** Explicitly requests paired-end download, ensuring `_1` and `_2` suffixes are applied to the output files for downstream pipelines expecting this naming convention.

### Specify an output filename directly
**Args:** --sample-id SRS123456 --output-dir ./results --output-prefix experiment_A
**Explanation:** Overrides default naming and applies the custom prefix to all output FASTQ files, useful for organizing multiple samples during batch downloads.