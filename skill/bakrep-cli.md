---
name: bakrep-cli
category: Data Management
description: A command-line tool for managing backup repositories, replicating bioinformatics data across storage locations, and performing backup verification and restoration operations.
tags: [backup, repository, storage, replication, data-management, bioinformatic]
author: AI-generated
source_url: https://github.com/bioinformatics-tools/bakrep-cli
---

## Concepts

- bakrep-cli operates on a repository-based data model where each repository is defined by a unique identifier, storage path, and metadata configuration file stored in `.bakrep/config.yaml`.
- The tool supports three primary operations: `create` initializes a new repository with specified retention policies, `sync` replicates data between source and destination locations, and `verify` performs integrity checks using checksums.
- Input formats include plain text file lists (one path per line), glob patterns for directory operations, and JSON manifest files for batch operations. Output is typically JSON or tabular format depending on the `--output` flag.
- The tool maintains a manifest database (`manifest.db`) in each repository root that tracks file hashes, modification timestamps, and replication status for all managed files.

## Pitfalls

- Specifying the wrong source or destination path without trailing slashes can cause ambiguous path resolution, leading to files being placed in unexpected subdirectories rather than the intended root location.
- Running `sync` without first running `verify` on newly created repositories may result in incomplete replication if source files are still being written to, as the tool cannot detect ongoing file modifications.
- Omitting the `--retention-days` flag when creating repositories uses the default retention of 30 days, which may lead to unintended data deletion during automated cleanup operations.
- Using relative paths in manifests instead of absolute paths breaks batch operations when the working directory changes, causing files to be mislocated or not found.
- Overwriting existing repositories with `create --force` without adequate backup of the existing data results in permanent data loss, as the operation is not reversible.

## Examples

### Create a new backup repository with default settings
**Args:** create --path /data/archive --name experiment-2024
**Explanation:** Initializes a new repository named "experiment-2024" at the specified path with default retention policies and checksum verification enabled.

### Sync files from a source directory to a remote repository
**Args:** sync --source /data/raw/sequences --destination s3://bucket/archive --mode incremental
**Explanation:** Performs an incremental sync of all files from the local source to an S3 destination, only transferring files that have changed since the last sync.

### Verify integrity of all files in a repository
**Args:** verify --path /data/archive --algorithm sha256 --report-json
**Explanation:** Computes SHA-256 checksums for all files in the repository and compares them against stored manifest values, outputting results in JSON format.

### List all repositories with their status and file counts
**Args:** list --all --verbose
**Explanation:** Displays all known repositories along with detailed information including total file count, total size, last sync timestamp, and verification status.

### Restore files from a repository to a specified location
**Args:** restore --repository experiment-2024 --target /data/restore --date 2024-01-15
**Explanation:** Restores all files from the specified repository as they existed on January 15, 2024, to the target restoration directory.

### Delete expired backups older than 90 days
**Args:** prune --repository old-experiments --older-than 90 --dry-run
**Explanation:** Identifies and marks for deletion all files in the repository that are older than 90 days, with the dry-run flag displaying what would be deleted without performing actual removal.

### Add files to an existing repository manually
**Args:** add --repository experiment-2024 --files /data/new-files/*.fastq
**Explanation:** Adds all FASTQ files from the specified pattern to the existing repository, updating the manifest and computing checksums for each new file.