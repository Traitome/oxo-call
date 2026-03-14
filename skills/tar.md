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

## Pitfalls

- -f must immediately precede the archive filename because it takes the next argument as the filename. 'tar -cvzf archive.tar.gz dir' works; 'tar -cvfz archive.tar.gz dir' is wrong (z would be treated as the filename).
- On extraction without -C, tar extracts into the current directory. Malicious archives (zip-slip) may contain paths like '../../../etc/passwd'. Inspect with 'tar -tf' before extracting untrusted archives.
- tar -c without a compression flag creates an uncompressed .tar. If you need .tar.gz, add -z; forgetting -z results in a large uncompressed archive.
- Extracting as root without --no-same-owner can change file ownership to match archive metadata. Use --no-same-owner when extracting community archives.
- tar -u (update) only adds files newer than the archive copy, but it cannot remove files. For a clean archive, recreate it with -c.
- On macOS (BSD tar) some GNU tar options (-J for xz, --zstd) may not be available. Install GNU tar via Homebrew: 'brew install gnu-tar'.

## Examples

### create a gzip-compressed archive of a directory
**Args:** `-czf archive.tar.gz data/`
**Explanation:** -c create; -z gzip compression; -f specifies archive filename; data/ is the source directory

### extract a gzip archive into the current directory
**Args:** `-xzf archive.tar.gz`
**Explanation:** -x extract; -z decompress gzip; -f specifies the archive; extracts to current directory

### extract an archive into a specific directory
**Args:** `-xf archive.tar.gz -C /opt/myapp/`
**Explanation:** -C changes to /opt/myapp/ before extracting; GNU tar auto-detects compression so -z is optional

### list contents of an archive without extracting
**Args:** `-tf archive.tar.gz`
**Explanation:** -t lists archive contents; -f specifies the file; add -v for detailed listing with sizes and permissions

### create a verbose bzip2-compressed archive
**Args:** `-cjvf backup.tar.bz2 /home/user/documents/`
**Explanation:** -j uses bzip2 compression (better ratio than gzip); -v shows each file being archived

### extract and strip the top-level directory from the archive
**Args:** `-xzf project-1.0.tar.gz --strip-components=1 -C /opt/project/`
**Explanation:** --strip-components=1 removes the leading 'project-1.0/' prefix from all extracted paths

### create an archive excluding certain file patterns
**Args:** `-czf backup.tar.gz project/ --exclude='*.pyc' --exclude='.git'`
**Explanation:** --exclude patterns prevent matched files from being archived; useful for clean source backups

### add files to an existing uncompressed archive
**Args:** `-rf existing.tar newfile.txt`
**Explanation:** -r appends files to an existing .tar archive; cannot append to compressed archives

### create a highly compressed archive using xz
**Args:** `-cJf archive.tar.xz largedir/`
**Explanation:** -J uses xz compression (slowest but smallest); best for long-term storage or distribution

### extract a single file from an archive
**Args:** `-xzf archive.tar.gz path/inside/archive/file.txt`
**Explanation:** specify the exact path inside the archive as the last argument to extract only that file
