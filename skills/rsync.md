---
name: rsync
category: networking
description: Fast, versatile file copying and synchronization tool with delta-transfer algorithm
tags: [rsync, sync, copy, backup, remote, transfer, mirror]
author: oxo-call built-in
source_url: "https://rsync.samba.org/documentation.html"
---

## Concepts

- rsync copies files efficiently by only transferring differences (delta transfer). Syntax: 'rsync [options] SOURCE DEST'. For remote: 'rsync [options] user@host:remote/path local/path' (or vice versa).
- Key flags: -a (archive: recursive + preserve permissions/times/symlinks/owner/group), -v (verbose), -z (compress during transfer), -P (show progress + resume partial transfers = --partial --progress), -n / --dry-run (preview without making changes).
- Trailing slash on SOURCE: 'rsync src/ dest/' copies the CONTENTS of src into dest. 'rsync src dest/' copies the src DIRECTORY itself into dest (creating dest/src/). This is the most common source of confusion.
- The --delete flag removes files in DEST that don't exist in SOURCE, turning rsync into a true mirror. Always test with --dry-run before adding --delete.
- SSH transport: rsync uses ssh by default for remote transfers. Specify a non-standard port or key with -e 'ssh -p 2222 -i ~/.ssh/key'.
- Exclusions: --exclude='pattern' skips matching files; --exclude-from=file reads patterns from a file; --filter='- pattern' is the more powerful alternative. Patterns are matched against the path relative to the transfer root.

## Pitfalls

- DANGER: rsync with --delete will permanently remove destination files that are absent in the source. ALWAYS do a --dry-run first to preview deletions before adding --delete to a live sync.
- The trailing slash rule: 'rsync -a src/ dest/' and 'rsync -a src dest/' behave DIFFERENTLY. The first copies contents; the second copies the directory. Verify with --dry-run when unsure.
- rsync -a preserves ownership only when run as root. For non-root transfers that need to preserve permissions, use --no-owner and --no-group if the destination owner differs.
- Transfers over SSH to hosts not in known_hosts will prompt for host key confirmation or fail in non-interactive scripts. Add the host key first with 'ssh-keyscan host >> ~/.ssh/known_hosts'.
- rsync --progress shows progress per file, not total overall progress. For total progress use --info=progress2 (rsync 3.1+).
- --checksum (-c) forces comparison by checksum rather than size+mtime, which is accurate but much slower. Avoid it for routine syncs of large datasets.

## Examples

### sync a local directory to a remote server with verbose output and compression
**Args:** `-avz /local/data/ user@remote:/remote/data/`
**Explanation:** -a archive mode (recursive + preserve attrs); -v verbose; -z compress; trailing slash copies contents

### dry-run to preview what would be transferred
**Args:** `-avzn /source/ /dest/`
**Explanation:** -n (--dry-run) shows what WOULD happen without making changes; always use before --delete

### mirror source to destination, deleting removed files
**Args:** `-avz --delete /source/ /dest/`
**Explanation:** --delete removes files in /dest/ that no longer exist in /source/; test with -n first

### sync from remote server to local directory
**Args:** `-avz user@remote:/remote/data/ /local/backup/`
**Explanation:** remote source syntax is user@host:path; rsync uses ssh for transport by default

### resume a large interrupted transfer
**Args:** `-avzP user@remote:/path/large-file.tar.gz /local/`
**Explanation:** -P enables --partial (resume) and --progress; allows resuming interrupted transfers

### sync excluding specific directories and patterns
**Args:** `-avz --exclude='.git' --exclude='*.pyc' --exclude='__pycache__' /src/ user@remote:/dest/`
**Explanation:** --exclude patterns prevent matched files/dirs from being transferred

### sync using a non-standard SSH port
**Args:** `-avz -e 'ssh -p 2222' /local/data/ user@remote:/data/`
**Explanation:** -e specifies the remote shell; -p 2222 sets the SSH port

### show total transfer progress instead of per-file progress
**Args:** `-avz --info=progress2 /source/ /dest/`
**Explanation:** --info=progress2 shows cumulative transfer progress; requires rsync 3.1+

### copy files preserving hard links
**Args:** `-avzH /source/ /dest/`
**Explanation:** -H preserves hard links; important for backups where hard links represent identical files

### sync only files newer than a reference file
**Args:** `-avz --update /source/ /dest/`
**Explanation:** --update skips destination files that are newer than the source; safe for incremental updates
