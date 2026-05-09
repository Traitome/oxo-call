---
name: bakdrive
category: Data Management
description: Command-line tool for managing backup drives and archiving large biological datasets. Provides functionality to list, mount, unmount, verify, and sync backup storage volumes commonly used in high-throughput genomics workflows.
tags:
  - backup
  - storage
  - archive
  - data-management
  - genomics
author: AI-generated
source_url: https://github.com/biobakery/bakdrive
---

## Concepts

- **Backup Drive Identification**: Bakdrive uses UUIDs or device labels to uniquely identify backup volumes. Devices can be referenced by their mount point (`/mnt/backup1`), UUID string, or user-defined label.
- **Mount/Unmount States**: Backup drives must be in a mounted state before read/write operations. attempting operations on unmounted drives will result in errors. Always verify mount status before transferring data.
- **Data Verification**: The tool supports SHA-256 checksum verification to ensure data integrity during backup and restore operations. Verification can be performed on individual files or entire directory trees.
- **Incremental Sync**: Bakdrive uses rsync-like semantics for incremental backups, only transferring files that have changed since the last sync operation based on modification time and checksum.

## Pitfalls

- **Forgetting to Unmount**: If a backup drive is disconnected without proper unmounting, data corruption can occur. Always unmount before physical disconnection, even if no writes were performed.
- **Confusing Source and Destination**: In copy/sync commands, the argument order matters. The first path is the source, the second is the destination. Reversing them will overwrite your only copy of data.
- **Assuming Write Permissions**: Some backup drives may be mounted read-only. Attempting write operations will fail silently or produce cryptic permission denied errors without clear indication that the mount state is the issue.
- **Ignoring Space Constraints**: Bakdrive does not pre-check available space before transfer operations. Running out of space mid-transfer leaves partial data on the destination and may corrupt existing files.

## Examples

### List all available backup drives
**Args:** `list`
**Explanation:** Lists all detected backup drives with their UUID, mount point, capacity, and current mount status for easy identification before operations.

### Mount a backup drive by label
**Args:** `mount --label mygenomebackup`
**Explanation:** Mounts the backup drive with the user-defined label "mygenomebackup" to its default mount point, making it accessible for read/write operations.

### Unmount a specific backup drive
**Args:** `unmount /mnt/backup2`
**Explanation:** Safely unmounts the backup drive at the specified mount point, flushing any pending writes and ensuring data integrity before disconnection.

### Verify data integrity with checksum
**Args:** `verify --path /mnt/backup1/dataset --algorithm sha256`
**Explanation:** Computes SHA-256 checksums for all files in the specified path and compares them against stored metadata to detect any data corruption or bit-rot.

### Sync incremental backup from local to remote
**Args:** `sync /data/fastq --destination /mnt/backup1/fastq --incremental`
**Explanation:** Performs an incremental sync, only copying files that have changed since the last sync based on modification time, reducing transfer time and bandwidth usage.

### Copy a directory tree to backup drive
**Args:** `copy /data/projects --destination /mnt/backup1/projects --preserve`
**Explanation:** Copies the entire directory tree to the backup drive while preserving file permissions, timestamps, and symbolic links.

### Check available space on a backup drive
**Args:** `df /mnt/backup1`
**Explanation:** Displays the total, used, and available storage space on the specified backup drive to verify sufficient capacity before large transfers.

### Format a new backup drive with ext4
**Args:** `format --device /dev/sdd1 --label newbackup --filesystem ext4`
**Explanation:** Formats the specified device with ext4 filesystem and assigns a label for easy identification, preparing it for use as a backup drive.