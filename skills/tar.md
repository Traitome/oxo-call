---
name: tar
category: filesystem
description: Archive files together and optionally compress with gzip, bzip2, or xz
tags: [archive, compress, tar, gzip, bzip2, xz, backup, extract]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/tar/manual/tar.html"
---

## Concepts

- tar creates, lists, and extracts archives. Core operation flags: -c (create), -x (extract), -t (list contents), -u (update). Always combine with -f <archive> to specify the archive filename.
- Compression flags (optional): -z for gzip (.tar.gz / .tgz), -j for bzip2 (.tar.bz2), -J for xz (.tar.xz), --zstd for zstd (.tar.zst). Modern GNU tar auto-detects compression on extraction.
- Useful supplementary flags: -v (verbose, list processed files), -p (preserve permissions), -C <dir> (change to directory before extracting), --strip-components=N (strip N path components on extract).
- Archive paths: by default tar preserves the directory structure relative to where it was created. Use -C when creating to control the root path stored in the archive.
- GNU tar accepts operations as short flags (-czf) or long options (--create --gzip --file). Short flags can be combined without a leading dash: 'tar czf archive.tar.gz dir/'.
- tar does NOT encrypt archives. For encrypted backups combine tar with gpg: 'tar czf - dir/ | gpg -c > archive.tar.gz.gpg'.
- Flag ordering matters: -f must be the LAST flag in a combined group because it takes the next argument as the archive filename. 'tar -czf' is correct; 'tar -cfz' would treat 'z' as the filename.

## Pitfalls

- -f must immediately precede the archive filename because it takes the next argument as the filename. 'tar -cvzf archive.tar.gz dir' works; 'tar -cvfz archive.tar.gz dir' is wrong (z would be treated as the filename).
- On extraction without -C, tar extracts into the current directory. Malicious archives (zip-slip) may contain paths like '../../../etc/passwd'. Inspect with 'tar -tf' before extracting untrusted archives.
- tar -c without a compression flag creates an uncompressed .tar. If you need .tar.gz, add -z; forgetting -z results in a large uncompressed archive.
- Extracting as root without --no-same-owner can change file ownership to match archive metadata. Use --no-same-owner when extracting community archives.
- tar -u (update) only adds files newer than the archive copy, but it cannot remove files. For a clean archive, recreate it with -c.
- On macOS (BSD tar) some GNU tar options (-J for xz, --zstd) may not be available. Install GNU tar via Homebrew: 'brew install gnu-tar'.
- The flag order -czf is idiomatic and correct: operation (-c), compression (-z), file (-f). Reversing to -cfz is a common mistake.

## Examples

### create a gzip-compressed archive of a directory
**Args:** `-czf archive.tar.gz data/`
**Explanation:** tar command; -c create archive; -z gzip compression; -f archive.tar.gz output archive filename; data/ source directory

### extract a gzip archive into the current directory
**Args:** `-xzf archive.tar.gz`
**Explanation:** tar command; -x extract; -z decompress gzip; -f archive.tar.gz input archive; extracts to current directory

### extract an archive into a specific directory
**Args:** `-xf archive.tar.gz -C /opt/myapp/`
**Explanation:** tar command; -x extract; -f archive.tar.gz input archive; -C /opt/myapp/ target directory; GNU tar auto-detects compression

### list contents of an archive without extracting
**Args:** `-tf archive.tar.gz`
**Explanation:** tar command; -t list archive contents; -f archive.tar.gz input archive; add -v for detailed listing

### create a verbose bzip2-compressed archive
**Args:** `-cjvf backup.tar.bz2 /home/user/documents/`
**Explanation:** tar command; -c create; -j bzip2 compression; -v verbose shows each file; -f backup.tar.bz2 output archive; /home/user/documents/ source directory

### extract and strip the top-level directory from the archive
**Args:** `-xzf project-1.0.tar.gz --strip-components=1 -C /opt/project/`
**Explanation:** tar command; -x extract; -z decompress gzip; -f project-1.0.tar.gz input archive; --strip-components=1 removes leading directory prefix; -C /opt/project/ target directory

### create an archive excluding certain file patterns
**Args:** `-czf backup.tar.gz project/ --exclude='*.pyc' --exclude='.git'`
**Explanation:** tar command; -c create; -z gzip compression; -f backup.tar.gz output archive; project/ source directory; --exclude='*.pyc' exclude Python cache files; --exclude='.git' exclude Git directory

### add files to an existing uncompressed archive
**Args:** `-rf existing.tar newfile.txt`
**Explanation:** tar command; -r append to existing archive; -f existing.tar existing .tar archive; newfile.txt file to add; cannot append to compressed archives

### create a highly compressed archive using xz
**Args:** `-cJf archive.tar.xz largedir/`
**Explanation:** tar command; -c create; -J xz compression (slowest but smallest); -f archive.tar.xz output archive; largedir/ source directory; best for long-term storage

### extract a single file from an archive
**Args:** `-xzf archive.tar.gz path/inside/archive/file.txt`
**Explanation:** tar command; -x extract; -z decompress gzip; -f archive.tar.gz input archive; path/inside/archive/file.txt specific path to extract

### create a zstd-compressed archive
**Args:** `--zstd -cf archive.tar.zst data/`
**Explanation:** tar command; --zstd Zstandard compression (fast + good ratio); -c create; -f archive.tar.zst output archive; data/ source directory

### extract an archive preserving permissions
**Args:** `-xpzf archive.tar.gz -C /opt/app/`
**Explanation:** tar command; -x extract; -p preserves permissions and ownership; -z decompress gzip; -f archive.tar.gz input archive; -C /opt/app/ target directory

### compare archive contents with the filesystem
**Args:** `-dzf archive.tar.gz`
**Explanation:** tar command; -d diff compares archive contents with files on disk; -z decompress gzip; -f archive.tar.gz input archive; reports differences without extracting
