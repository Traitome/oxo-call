---
name: biomaj
category: Data Management
description: A bioinformatics data management workflow engine that automates the download, processing, versioning, and maintenance of biological databases and datasets.
tags: [database, workflow, automation, bioinformatics, versioning, data-pipeline]
author: AI-generated
source_url: https://github.com/pierrick/mpiler
---

## Concepts

- **Bank-based data model**: biomaj organizes bioinformatics resources into "banks" (also called collections), which are self-contained datasets with their own configuration, download sources, and processing workflows. Each bank maintains a version history and can be updated independently.
- **YAML configuration files**: Banks are defined using YAML configuration files (`bank.yml`) that specify remote data sources (FTP/HTTP links), file patterns to download, processing commands to run, and dependencies on other banks. The configuration drives the entire workflow.
- **MongoDB state tracking**: biomaj uses a MongoDB database to track the state of each bank, including current version, file lists, checksums, and status. This enables efficient querying of bank metadata and supports incremental updates.
- **Companion binaries**: The biomaj suite includes `biomaj-build` for building/updating banks, `biomaj-stat` for displaying bank statistics, and `biomaj-clean` for removing old versions. These are invoked as separate commands rather than subcommands of the main `biomaj` binary.
- **Workflow phases**: Building a bank executes a sequence of phases — download, extract, transform, post-process — each defined in the bank's YAML configuration. The workflow can be customized with custom scripts and commands.

## Pitfalls

- **Missing MongoDB connection**: If the MongoDB instance is not running or the connection string is incorrect, all commands will fail with authentication or connection errors, leaving banks in an inconsistent state.
- **Incomplete bank.yml syntax**: A malformed YAML configuration file (e.g., incorrect indentation, missing required fields like `remote` or `files`) causes the build to fail silently or produce an empty bank with no downloadable data.
- **Disk space exhaustion**: Large bioinformatics databases (e.g., NCBI nt, RefSeq) can consume hundreds of gigabytes. If the designated data directory lacks sufficient space, the download or build process will terminate abruptly, leaving partial files that are difficult to clean up.
- **Version conflicts with `--force`**: Using `biomaj-build --force` to rebuild a bank overwrites the current version without creating a new version entry, losing the ability to roll back to the previous state if the new build is corrupted.
- **Incorrect file patterns**: Wildcard patterns in the `files` section of bank.yml that are too broad may download unintended files (e.g., `.md5` checksums alongside sequence files), while patterns that are too narrow may miss necessary files, causing downstream analysis failures.

## Examples

### List all available banks in the local storage
**Args:** `biomaj --list`
**Explanation:** Queries the MongoDB state database and displays all banks that have been downloaded or built locally, showing their current versions and status flags.

### Check the status of a specific bank
**Args:** `biomaj status helix`
**Explanation:** Retrieves the current state of the bank named "helix" from MongoDB, including its version number, last update timestamp, file count, and whether it is marked as dirty (modified).

### Force a complete rebuild of the "uniprot" bank
**Args:** `biomaj-build --force uniprot`
**Explanation:** Re-downloads all remote files and re-executes the processing workflow for the "uniprot" bank, overwriting the existing version without creating a new version entry.

### Build a new version of the "ensembl" bank without checksum verification
**Args:** `biomaj-build --no-check ensembl`
**Explanation:** Builds a new version of the "ensembl" bank, skipping the verification of file checksums after download, which speeds up the process but risks data corruption going undetected.

### Display disk usage and file statistics for the "pfam" bank
**Args:** `biomaj-stat pfam`
**Explanation:** Shows detailed statistics about the "pfam" bank, including total disk usage, number of files, file type distribution, and storage efficiency metrics.

### Remove old versions of the "blast" bank, keeping only the two most recent
**Args:** `biomaj-clean --keep 2 blast`
**Explanation:** Deletes all but the two most recent versions of the "blast" bank from the local storage and updates the MongoDB state to reflect the removed versions.

### Download a bank using a custom configuration file
**Args:** `biomaj --config /path/to/custom.yml download mydb`
**Explanation:** Uses a custom bank configuration file located at the specified path instead of the default bank.yml, allowing testing of modified configurations before applying them to the standard bank directory.