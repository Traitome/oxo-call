---
name: ampd-up
category: Data Management
description: A bioinformatics utility for managing and updating locally cached reference databases, index files, and annotation sets. Used to synchronize external data resources with local storage, verify integrity of downloaded files, and prepare data structures for downstream analysis pipelines.
tags:
- database-management
- reference-files
- data-sync
- index-files
- cache-updates
- bioinformatics-infrastructure
author: AI-generated
source_url: https://github.com/example/ampd-up
---

## Concepts

- **Local Cache Directory**: The tool operates on a designated cache directory (specified via `--cache-dir` or environment variable `AMPD_CACHE`) where reference databases, index files, and annotation sets are stored. It maintains a manifest file tracking all cached resources with checksums, version numbers, and timestamps.
- **Manifest System**: Each cache directory contains a `manifest.json` file that records metadata for all cached items including md5/sha256 checksums, source URLs, version tags, and last-update timestamps. The manifest enables integrity verification and update detection.
- **Remote Sync Protocol**: The tool supports synchronization with remote repositories via HTTP/HTTPS. It performs conditional downloads by comparing remote manifest timestamps and version numbers against local copies, skipping unchanged files to save bandwidth and time.
- **Integrity Verification**: All downloaded or locally modified files are verified against stored checksums before being marked as valid. Files failing verification are quarantined to a separate directory and must be manually re-downloaded or repaired.

## Pitfalls

- **Manifest Corruption**: If the manifest.json file becomes corrupted or deleted, the tool loses track of which files are valid and may re-download all resources unnecessarily. Always back up the manifest before manual cache directory manipulation.
- **Insufficient Disk Space**: The update process requires free space equal to the size of new or updated files plus temporary download files. Running on a nearly full filesystem can cause partial downloads that fail integrity verification.
- **Inconsistent Network Connections**: Interrupted network connections during synchronization result in incomplete downloads that fail checksum verification. The tool does not support resume by default for interrupted transfers.
- **Version Mismatches**: Downgrading cached resources to older versions is not directly supported. Clearing the cache and re-downloading specific versions is required when pipeline compatibility demands older reference datasets.

## Examples

### Download and update all cached references to latest versions
**Args:** `--sync --all`
**Explanation:** Connects to all configured remote repositories and downloads any resources that have newer versions than currently cached.

### Verify integrity of all cached files without downloading
**Args:** `--verify`
**Explanation:** Reads the manifest and computes checksums for all cached files, reporting any that fail verification without modifying the cache.

### Add a new reference database to the cache
**Args:** `--add https://example.com/refs/db.fasta --name myref --cache-dir /data/refs`
**Explanation:** Downloads the specified FASTA file from the URL, computes its checksum, updates the manifest, and stores it in the designated cache directory.

### List all cached resources with their metadata
**Args:** `--list --verbose`
**Explanation:** Prints each cached resource including name, version, size, checksum, and last-update timestamp in a human-readable format.

### Remove obsolete cached files no longer in the manifest
**Args:** `--cleanup --dry-run`
**Explanation:** Identifies files in the cache directory that are not listed in the current manifest. The dry-run flag shows what would be deleted without actually removing files.