---
name: find
category: filesystem
description: Search for files in a directory hierarchy by name, type, size, time, and permissions
tags: [search, filesystem, files, filter, locate, directories]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/findutils/manual/html_mono/find.html"
---

## Concepts

- find searches recursively from a starting directory: 'find <path> <expression>'. Use '.' for current directory. Expressions are evaluated left-to-right with implicit AND.
- Common tests: -name (case-sensitive glob), -iname (case-insensitive), -type f (files), -type d (directories), -type l (symlinks), -size +N[k/M/G] (larger than N), -mtime -N (modified within N days).
- Actions: -print (default, print paths), -delete (delete matched files in-place), -exec <cmd> {} \; (run command on each match), -exec <cmd> {} + (batch matches into one command call).
- Use -maxdepth N to limit search depth (e.g., -maxdepth 1 for current directory only). Place -maxdepth early in the expression for performance.
- Combine tests with logical operators: -and (default), -or (-o), -not (!). Parentheses group expressions but must be escaped: \( ... \).
- find -exec is slower than -delete or piping to xargs for large result sets. Use 'find ... -print0 | xargs -0 <cmd>' for handling filenames with spaces.

## Pitfalls

- 'find . -delete' deletes everything in the current directory tree. Always test with -print first: replace -delete with -print to preview matches before deleting.
- find -name uses shell globs, not regex — use -name '*.txt' (quoted to prevent shell expansion), not -name *.txt (the shell may expand the glob before find sees it).
- -mtime +N means 'more than N*24 hours ago'; -mtime -N means 'less than N*24 hours ago'. The sign is often confused: +7 means older than 7 days.
- find . -name 'file' searches the current directory AND all subdirectories. Add -maxdepth 1 to search only the current directory.
- Permissions test -perm /mode matches if ANY of the specified bits are set; -perm -mode requires ALL bits to be set. -perm 644 requires EXACTLY 644.
- The closing semicolon in -exec must be escaped or quoted: -exec rm {} \; or -exec rm {} ';'. Missing the escape causes a syntax error.

## Examples

### find all files larger than 100 MB in the current directory tree
**Args:** `. -type f -size +100M`
**Explanation:** -type f limits to regular files; -size +100M finds files strictly larger than 100 MB

### find all Python files modified in the last 7 days
**Args:** `. -name '*.py' -mtime -7`
**Explanation:** -name '*.py' matches Python files; -mtime -7 means modified within the last 7 days

### find and delete all .tmp files in a directory tree
**Args:** `/tmp -name '*.tmp' -type f -delete`
**Explanation:** preview with -print before using -delete; this permanently removes all matched files

### find files by name case-insensitively
**Args:** `. -iname 'readme*'`
**Explanation:** -iname is case-insensitive; matches README.md, readme.txt, Readme.rst, etc.

### find all directories in the current directory (depth 1 only)
**Args:** `. -maxdepth 1 -type d`
**Explanation:** -maxdepth 1 limits search to immediate children; -type d shows only directories

### find empty files and directories
**Args:** `. -empty`
**Explanation:** -empty matches both empty files and empty directories; useful for cleanup

### find files and execute a command on each match
**Args:** `. -name '*.log' -exec gzip {} \;`
**Explanation:** -exec runs gzip on each matched file; {} is replaced with the filename; \; terminates the -exec expression

### find files owned by a specific user
**Args:** `/home -user alice -type f`
**Explanation:** -user matches files owned by the specified username

### find recently modified files and sort by modification time
**Args:** `. -type f -newer reference_file.txt`
**Explanation:** -newer finds files modified more recently than reference_file.txt; combine with -ls for details

### find files with specific permissions
**Args:** `. -type f -perm /o+w`
**Explanation:** -perm /o+w finds world-writable files (any other-write bit set); useful for security audits
